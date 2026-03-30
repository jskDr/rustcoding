use crate::config::{AppState, ReactTraceMode};
use crate::models::{ChatCompletionChunk, ChatCompletionResponse, ChatRequest, Message};
use crate::tools::{bash_tools, execute_tool_call};
use crate::ui::{new_progress_bar, print_iteration_phase, Styled};
use anyhow::{Context, Result};
use console::style;
use futures_util::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::env;
use std::io::{self, Write};

const CONTROL_OUTPUT_INSTRUCTIONS: &str = r#"Return exactly one JSON object with this schema:
{"status":"continue|final","assistant_message":"string","reflection":"string"}

Rules:
- Do not wrap the JSON in markdown.
- "assistant_message" must contain the best current user-facing answer draft.
- "reflection" must be a short internal sufficiency judgment.
- Use "continue" only if the current answer is still insufficient and another iteration would materially improve it.
- Use "final" when the answer is sufficient or no more useful progress is likely.
- Never reveal chain-of-thought.
"#;

const TOOL_RETRY_TEMPERATURES: [f32; 3] = [0.7, 0.4, 0.2];
const ERROR_BODY_SNIPPET_LEN: usize = 300;

#[derive(Deserialize, Debug)]
struct ReactControlDecision {
    status: String,
    assistant_message: String,
    reflection: String,
}

#[derive(Debug)]
struct ApiRequestError {
    status: Option<StatusCode>,
    body: String,
    allow_tools: bool,
}

impl ApiRequestError {
    fn is_retryable_tool_400(&self) -> bool {
        self.allow_tools && self.status == Some(StatusCode::BAD_REQUEST)
    }

    fn is_disallowed_tool_call_without_tools(&self) -> bool {
        !self.allow_tools
            && self.status == Some(StatusCode::BAD_REQUEST)
            && (self.body.contains("Tool choice is none, but model called a tool")
                || self.body.contains("tool_use_failed"))
    }

    fn display_message(&self) -> String {
        match self.status {
            Some(status) => format!(
                "HTTP {} from API{}",
                status.as_u16(),
                if self.body.trim().is_empty() {
                    String::new()
                } else {
                    format!(": {}", sanitize_error_body(&self.body))
                }
            ),
            None => "Request failed before receiving an HTTP response.".to_string(),
        }
    }
}

pub async fn complete_turn(
    client: &Client,
    model: &str,
    messages: &mut Vec<Message>,
    styled: &Styled,
    app_state: &AppState,
) -> Result<String> {
    if !app_state.react_enabled {
        return complete_with_tools_single_pass(
            client,
            model,
            messages,
            styled,
            app_state.reasoning_effort.clone(),
        )
        .await;
    }

    run_react_loop(client, model, messages, styled, app_state).await
}

async fn run_react_loop(
    client: &Client,
    model: &str,
    messages: &mut Vec<Message>,
    styled: &Styled,
    app_state: &AppState,
) -> Result<String> {
    let mut working_messages = messages.clone();
    let max_iterations = app_state.react_max_iterations;
    let mut final_draft = String::new();
    let mut tools_available_for_turn = true;

    for iteration in 1..=max_iterations {
        if matches!(app_state.react_trace_mode, ReactTraceMode::Summary) {
            print_iteration_phase(iteration, max_iterations, "reasoning")?;
        }

        let assistant_message = match request_control_message_with_retry(
            client,
            model,
            build_control_messages(&working_messages, iteration, max_iterations, false),
            app_state.reasoning_effort.clone(),
            tools_available_for_turn,
        )
        .await {
            Ok(message) => message,
            Err(error) if error.is_retryable_tool_400() && tools_available_for_turn => {
                eprintln!(
                    "{} {}",
                    style("[api]").yellow().bold(),
                    style(error.display_message()).yellow()
                );
                tools_available_for_turn = false;
                let fallback_note = Message::system(tool_fallback_note());
                working_messages.push(fallback_note.clone());
                messages.push(fallback_note);
                request_control_message(
                    client,
                    model,
                    build_control_messages(&working_messages, iteration, max_iterations, false),
                    app_state.reasoning_effort.clone(),
                    false,
                    0.7,
                )
                .await
                .map_err(|error| anyhow::anyhow!(error.display_message()))?
            }
            Err(error) => return Err(anyhow::Error::msg(error.display_message())),
        };

        if let Some(tool_calls) = assistant_message.tool_calls.clone() {
            working_messages.push(assistant_message.clone());
            messages.push(assistant_message);

            if matches!(app_state.react_trace_mode, ReactTraceMode::Summary) {
                print_iteration_phase(iteration, max_iterations, "action")?;
            }

            for tool_call in tool_calls {
                let tool_output = execute_tool_call(&tool_call).await?;
                let tool_message = Message::tool(tool_call.id, tool_call.function.name, tool_output);
                working_messages.push(tool_message.clone());
                messages.push(tool_message);
            }

            if matches!(app_state.react_trace_mode, ReactTraceMode::Summary) {
                print_iteration_phase(iteration, max_iterations, "reflection")?;
            }

            let decision = request_control_decision(
                client,
                model,
                build_control_messages(&working_messages, iteration, max_iterations, true),
                app_state.reasoning_effort.clone(),
            )
            .await?;
            final_draft = decision.assistant_message.clone();

            if should_finalize(&decision, iteration, max_iterations) {
                break;
            }

            working_messages.push(Message::system(build_continue_note(
                &decision.assistant_message,
                &decision.reflection,
                iteration,
                max_iterations,
            )));
            continue;
        }

        let raw_content = assistant_message.content.unwrap_or_default();
        let decision = parse_control_decision(&raw_content);
        final_draft = decision.assistant_message.clone();

        if matches!(app_state.react_trace_mode, ReactTraceMode::Summary) {
            print_iteration_phase(iteration, max_iterations, "reflection")?;
        }

        if should_finalize(&decision, iteration, max_iterations) {
            break;
        }

        working_messages.push(Message::system(build_continue_note(
            &decision.assistant_message,
            &decision.reflection,
            iteration,
            max_iterations,
        )));
    }

    stream_final_answer(
        client,
        model,
        working_messages,
        styled,
        app_state.reasoning_effort.clone(),
        final_draft,
    )
    .await
}

async fn complete_with_tools_single_pass(
    client: &Client,
    model: &str,
    messages: &mut Vec<Message>,
    styled: &Styled,
    reasoning_effort: Option<String>,
) -> Result<String> {
    let assistant_message = match request_control_message_with_retry(
        client,
        model,
        messages.clone(),
        reasoning_effort.clone(),
        true,
    )
    .await
    {
        Ok(message) => message,
        Err(error) if error.is_retryable_tool_400() => {
            eprintln!(
                "{} {}",
                style("[api]").yellow().bold(),
                style(error.display_message()).yellow()
            );
            let fallback_note = Message::system(tool_fallback_note());
            messages.push(fallback_note.clone());
            request_control_message(
                client,
                model,
                {
                    let mut fallback_messages = messages.clone();
                    fallback_messages.push(Message::system(
                        "Answer the request without local tools. Clearly note any limitations."
                            .to_string(),
                    ));
                    fallback_messages
                },
                reasoning_effort.clone(),
                false,
                0.7,
            )
            .await
            .map_err(|error| anyhow::anyhow!(error.display_message()))?
        }
        Err(error) => return Err(anyhow::Error::msg(error.display_message())),
    };

    if let Some(tool_calls) = assistant_message.tool_calls.clone() {
        messages.push(assistant_message);

        for tool_call in tool_calls {
            let tool_output = execute_tool_call(&tool_call).await?;
            messages.push(Message::tool(
                tool_call.id,
                tool_call.function.name,
                tool_output,
            ));
        }

        return stream_completion(
            client,
            model,
            messages.clone(),
            styled,
            reasoning_effort,
            false,
        )
        .await;
    }

    let content = assistant_message.content.unwrap_or_default();
    print!("{}", styled.assistant_prompt());
    io::stdout().flush()?;
    print!("{}", style(&content).green());
    io::stdout().flush()?;
    println!();
    Ok(content)
}

async fn request_control_message_with_retry(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    reasoning_effort: Option<String>,
    allow_tools: bool,
) -> std::result::Result<Message, ApiRequestError> {
    if !allow_tools {
        return request_control_message(
            client,
            model,
            messages,
            reasoning_effort,
            false,
            TOOL_RETRY_TEMPERATURES[0],
        )
        .await;
    }

    let mut last_error = None;
    for temperature in TOOL_RETRY_TEMPERATURES {
        match request_control_message(
            client,
            model,
            messages.clone(),
            reasoning_effort.clone(),
            true,
            temperature,
        )
        .await
        {
            Ok(message) => return Ok(message),
            Err(error) if error.is_retryable_tool_400() => last_error = Some(error),
            Err(error) => return Err(error),
        }
    }

    Err(last_error.expect("tool retry loop must capture an error"))
}

async fn request_control_message(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    reasoning_effort: Option<String>,
    allow_tools: bool,
    temperature: f32,
) -> std::result::Result<Message, ApiRequestError> {
    let completion = call_chat_completion(
        client,
        model,
        messages,
        reasoning_effort,
        allow_tools,
        false,
        temperature,
    )
    .await?;

    completion
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .ok_or(ApiRequestError {
            status: None,
            body: "API returned no choices".to_string(),
            allow_tools,
        })
}

async fn request_control_decision(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    reasoning_effort: Option<String>,
) -> Result<ReactControlDecision> {
    let assistant_message = request_control_message(
        client,
        model,
        messages,
        reasoning_effort,
        false,
        TOOL_RETRY_TEMPERATURES[0],
    )
    .await;

    let assistant_message = match assistant_message {
        Ok(message) => message,
        Err(error) if error.is_disallowed_tool_call_without_tools() => {
            eprintln!(
                "{} {}",
                style("[api]").yellow().bold(),
                style("Reflection step requested another tool; continuing to next iteration.")
                    .yellow()
            );
            return Ok(ReactControlDecision {
                status: "continue".to_string(),
                assistant_message: "Additional tool work is needed before finalizing the answer."
                    .to_string(),
                reflection: "The model attempted another tool call during reflection, so the controller will continue with another tool-enabled iteration.".to_string(),
            });
        }
        Err(error) => return Err(anyhow::anyhow!(error.display_message())),
    };

    Ok(parse_control_decision(
        assistant_message.content.as_deref().unwrap_or_default(),
    ))
}

async fn call_chat_completion(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    reasoning_effort: Option<String>,
    allow_tools: bool,
    stream: bool,
    temperature: f32,
) -> std::result::Result<ChatCompletionResponse, ApiRequestError> {
    let api_key = env::var("GROQ_API_KEY").map_err(|error| ApiRequestError {
        status: None,
        body: format!("GROQ_API_KEY not set: {}", error),
        allow_tools,
    })?;
    let request = ChatRequest {
        model,
        messages,
        tools: if allow_tools { Some(bash_tools()) } else { None },
        tool_choice: if allow_tools { Some("auto") } else { None },
        stream,
        temperature,
        reasoning_effort,
    };

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|error| ApiRequestError {
            status: None,
            body: format!("Failed to send request: {}", error),
            allow_tools,
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|error| ApiRequestError {
        status: Some(status),
        body: format!("Failed to read response body: {}", error),
        allow_tools,
    })?;

    if !status.is_success() {
        return Err(ApiRequestError {
            status: Some(status),
            body,
            allow_tools,
        });
    }

    serde_json::from_str::<ChatCompletionResponse>(&body).map_err(|error| ApiRequestError {
        status: Some(status),
        body: format!("Failed to parse response body: {}", error),
        allow_tools,
    })
}

fn build_control_messages(
    base_messages: &[Message],
    iteration: usize,
    max_iterations: usize,
    post_tool_reflection: bool,
) -> Vec<Message> {
    let mut messages = base_messages.to_vec();
    let phase_note = if post_tool_reflection {
        "Reflect on the tool results and decide whether another iteration is needed."
    } else {
        "Reason about the request, call tools if needed, otherwise decide whether the answer is sufficient."
    };
    messages.push(Message::system(format!(
        "ReAct controller.\nIteration: {iteration}/{max_iterations}.\n{phase_note}\n{CONTROL_OUTPUT_INSTRUCTIONS}"
    )));
    messages
}

fn build_continue_note(
    assistant_message: &str,
    reflection: &str,
    iteration: usize,
    max_iterations: usize,
) -> String {
    format!(
        "Continue ReAct refinement. Previous draft answer: {}. Previous reflection: {}. Remaining iterations: {}. Improve the answer materially or finalize if no further progress is likely. Return JSON only.",
        assistant_message,
        reflection,
        max_iterations.saturating_sub(iteration)
    )
}

fn tool_fallback_note() -> String {
    "Local tools are unavailable for this turn because the provider rejected the tool-enabled request. Continue without tools and clearly state any limitations in the answer.".to_string()
}

fn should_finalize(
    decision: &ReactControlDecision,
    iteration: usize,
    max_iterations: usize,
) -> bool {
    iteration >= max_iterations || decision.status.eq_ignore_ascii_case("final")
}

fn parse_control_decision(raw_content: &str) -> ReactControlDecision {
    serde_json::from_str::<ReactControlDecision>(raw_content).unwrap_or_else(|_| {
        ReactControlDecision {
            status: "final".to_string(),
            assistant_message: raw_content.trim().to_string(),
            reflection: "Malformed controller output; using final-answer fallback.".to_string(),
        }
    })
}

fn sanitize_error_body(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= ERROR_BODY_SNIPPET_LEN {
        normalized
    } else {
        normalized.chars().take(ERROR_BODY_SNIPPET_LEN).collect::<String>() + "..."
    }
}

async fn stream_final_answer(
    client: &Client,
    model: &str,
    mut messages: Vec<Message>,
    styled: &Styled,
    reasoning_effort: Option<String>,
    final_draft: String,
) -> Result<String> {
    messages.push(Message::system(format!(
        "Produce the final user-facing answer now. Do not reveal internal reasoning. Use the completed work above. Draft answer to refine:\n{}",
        final_draft
    )));
    stream_completion(client, model, messages, styled, reasoning_effort, false).await
}

async fn stream_completion(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    styled: &Styled,
    reasoning_effort: Option<String>,
    allow_tools: bool,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?;
    let request = ChatRequest {
        model,
        messages,
        tools: if allow_tools { Some(bash_tools()) } else { None },
        tool_choice: if allow_tools { Some("auto") } else { None },
        stream: true,
        temperature: TOOL_RETRY_TEMPERATURES[0],
        reasoning_effort,
    };

    let progress_bar = new_progress_bar();
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&request)
        .send()
        .await
        .context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body.".to_string());
        anyhow::bail!("HTTP {} from API: {}", status.as_u16(), sanitize_error_body(&body));
    }

    progress_bar.finish_and_clear();

    let stream = response.bytes_stream();
    process_sse_stream(stream, styled).await
}

async fn process_sse_stream(
    mut stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    styled: &Styled,
) -> Result<String> {
    let mut buffer = String::new();
    let mut full_response = String::new();
    let mut done = false;

    print!("{}", styled.assistant_prompt());
    io::stdout().flush()?;

    while let Some(item) = stream.next().await {
        let chunk = item.context("Failed to read chunk")?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(idx) = buffer.find("\n\n") {
            let frame = &buffer[..idx];

            for line in frame.lines() {
                if !line.starts_with("data:") {
                    continue;
                }
                let data = line.trim_start_matches("data:").trim();

                if data == "[DONE]" {
                    done = true;
                    break;
                }

                match serde_json::from_str::<ChatCompletionChunk>(data) {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", style(content).green());
                                io::stdout().flush()?;
                                full_response.push_str(content);
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!(
                            "\n{} {}",
                            style("[Error]").red().bold(),
                            style(format!("Failed to parse response chunk: {}", error)).red()
                        );
                    }
                }
            }

            buffer.drain(..idx + 2);

            if done {
                break;
            }
        }

        if done {
            break;
        }
    }

    println!();
    Ok(full_response)
}

#[cfg(test)]
mod tests {
    use super::{sanitize_error_body, ApiRequestError, ERROR_BODY_SNIPPET_LEN};
    use reqwest::StatusCode;

    #[test]
    fn sanitize_error_body_normalizes_and_truncates() {
        let input = "line1\n\n   line2\tline3 ".repeat(40);
        let output = sanitize_error_body(&input);
        assert!(output.contains("line1 line2 line3"));
        assert!(output.len() <= ERROR_BODY_SNIPPET_LEN + 3);
        assert!(output.ends_with("..."));
    }

    #[test]
    fn only_tool_enabled_400_is_retryable() {
        let retryable = ApiRequestError {
            status: Some(StatusCode::BAD_REQUEST),
            body: "bad request".to_string(),
            allow_tools: true,
        };
        let not_retryable_status = ApiRequestError {
            status: Some(StatusCode::UNAUTHORIZED),
            body: "unauthorized".to_string(),
            allow_tools: true,
        };
        let not_retryable_tools = ApiRequestError {
            status: Some(StatusCode::BAD_REQUEST),
            body: "bad request".to_string(),
            allow_tools: false,
        };

        assert!(retryable.is_retryable_tool_400());
        assert!(!not_retryable_status.is_retryable_tool_400());
        assert!(!not_retryable_tools.is_retryable_tool_400());
    }
}
