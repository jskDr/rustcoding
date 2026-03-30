use anyhow::{Context, Result};
use console::{style, Style};
use futures_util::{Stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Stdio;
use home;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    stream: bool,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: ToolFunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ToolFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize, Debug, Clone)]
struct Tool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: ToolDefinition,
}

#[derive(Serialize, Debug, Clone)]
struct ToolDefinition {
    name: &'static str,
    description: &'static str,
    parameters: ToolParameters,
}

#[derive(Serialize, Debug, Clone)]
struct ToolParameters {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: BashToolProperties,
    required: Vec<&'static str>,
}

#[derive(Serialize, Debug, Clone)]
struct BashToolProperties {
    command: ToolProperty,
}

#[derive(Serialize, Debug, Clone)]
struct ToolProperty {
    #[serde(rename = "type")]
    kind: &'static str,
    description: &'static str,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionResponseChoice>,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponseChoice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionChunk {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionChoice {
    delta: Delta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Delta {
    #[allow(dead_code)]
    role: Option<String>,
    content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct BashCommandArgs {
    command: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    models: Vec<String>,
    reaoning_effor_support_models: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppState {
    model: String,
    reasoning_effort: Option<String>,
}

impl AppState {
    fn default(config: &Config) -> Self {
        Self {
            model: config.models[0].clone(),
            reasoning_effort: None,
        }
    }
}

fn load_state(config: &Config) -> AppState {
    let state_path = match home::home_dir() {
        Some(path) => path.join(".rustagent"),
        None => return AppState::default(config),
    };

    if let Ok(state_str) = fs::read_to_string(state_path) {
        if let Ok(state) = serde_json::from_str(&state_str) {
            return state;
        }
    }

    AppState::default(config)
}

fn save_state(state: &AppState) -> Result<()> {
    let state_path = match home::home_dir() {
        Some(path) => path.join(".rustagent"),
        None => return Ok(()),
    };
    let state_str = serde_json::to_string_pretty(state)?;
    fs::write(state_path, state_str)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    let term = console::Term::stdout();
    let styled = Styled::new();

    let config_str = fs::read_to_string("src/conf.json").context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&config_str).context("Failed to parse config file")?;
    let mut app_state = load_state(&config);
    let mut model = app_state.model.clone();
    let mut reasoning_effort = app_state.reasoning_effort.clone();

    term.write_line(&format!(
        "{} {}",
        style("RustAgent:").bold().blue(),
        "Your friendly Rust-based chat assistant."
    ))?;
    term.write_line(&format!(
        "{} {}",
        style("Model:").bold().blue(),
        style(&model).yellow()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type 'exit' or 'quit' to end the conversation.").italic()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type '/model' to see available models.").italic()
    ))?;
    term.write_line("")?;

    let mut messages = vec![Message {
        role: "system".to_string(),
        content: Some(
            "You are a helpful assistant. You may use the run_bash_command tool when shell output is needed. Before using it, prefer concise, relevant commands and avoid destructive operations unless the user explicitly asked for them."
                .to_string(),
        ),
        tool_call_id: None,
        name: None,
        tool_calls: None,
    }];

    loop {
        let user_input = match get_user_input(&styled)? {
            Some(input) => input,
            None => {
                save_state(&app_state)?;
                break;
            }
        };

        if user_input.is_empty() {
            continue;
        }

        if user_input == "/model" {
            term.write_line("Available models:")?;
            for (i, m) in config.models.iter().enumerate() {
                term.write_line(&format!("  {}: {}", i + 1, m))?;
            }
            term.write_str("Select a model (number): ")?;
            term.flush()?;
            let mut selection = String::new();
            io::stdin().read_line(&mut selection)?;
            if let Ok(selection) = selection.trim().parse::<usize>() {
                if selection > 0 && selection <= config.models.len() {
                    app_state.model = config.models[selection - 1].clone();
                    model = app_state.model.clone();
                    term.write_line(&format!(
                        "{} {}",
                        style("Model set to:").bold().blue(),
                        style(&model).yellow()
                    ))?;

                    if config.reaoning_effor_support_models.contains(&model) {
                        term.write_line("This model supports reasoning effort.")?;
                        term.write_str("Select reasoning effort (low, medium, high): ")?;
                        term.flush()?;
                        let mut effort_selection = String::new();
                        io::stdin().read_line(&mut effort_selection)?;
                        let effort_selection = effort_selection.trim().to_lowercase();
                        if ["low", "medium", "high"].contains(&effort_selection.as_str()) {
                            app_state.reasoning_effort = Some(effort_selection.clone());
                            reasoning_effort = Some(effort_selection);
                            term.write_line(&format!(
                                "{} {}",
                                style("Reasoning effort set to:").bold().blue(),
                                style(reasoning_effort.as_ref().unwrap()).yellow()
                            ))?;
                        } else {
                            term.write_line(&style("Invalid selection. Defaulting to no reasoning effort.").red().to_string())?;
                            app_state.reasoning_effort = None;
                            reasoning_effort = None;
                        }
                    } else {
                        app_state.reasoning_effort = None;
                        reasoning_effort = None;
                    }
                } else {
                    term.write_line(&style("Invalid selection.").red().to_string())?;
                }
            } else {
                term.write_line(&style("Invalid input.").red().to_string())?;
            }
            println!();
            continue;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: Some(user_input),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        });

        let assistant_response = complete_with_tools(
            &client,
            &model,
            &mut messages,
            &styled,
            reasoning_effort.clone(),
        )
        .await?;
        if !assistant_response.is_empty() {
            messages.push(Message {
                role: "assistant".to_string(),
                content: Some(assistant_response),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            });
        }
        println!();
    }

    Ok(())
}

fn get_user_input(styled: &Styled) -> Result<Option<String>> {
    let mut input = String::new();
    print!("{}", styled.user_prompt());
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
        println!("{}", style("Goodbye!").dim());
        return Ok(None);
    }
    println!();
    Ok(Some(input))
}

async fn complete_with_tools(
    client: &Client,
    model: &str,
    messages: &mut Vec<Message>,
    styled: &Styled,
    reasoning_effort: Option<String>,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?;
    let tools = bash_tools();
    let request = ChatRequest {
        model,
        messages: messages.clone(),
        tools: Some(tools.clone()),
        tool_choice: Some("auto"),
        stream: false,
        temperature: 0.7,
        reasoning_effort: reasoning_effort.clone(),
    };

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send tool-aware request")?
        .error_for_status()
        .context("Non-success status from API")?;

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .context("Failed to parse tool-aware response")?;
    let assistant_message = completion
        .choices
        .into_iter()
        .next()
        .context("API returned no choices")?
        .message;

    if let Some(tool_calls) = assistant_message.tool_calls.clone() {
        messages.push(assistant_message);

        for tool_call in tool_calls {
            let tool_output = execute_tool_call(&tool_call).await?;
            messages.push(Message {
                role: "tool".to_string(),
                content: Some(tool_output),
                tool_call_id: Some(tool_call.id),
                name: Some(tool_call.function.name),
                tool_calls: None,
            });
        }

        return stream_completion(client, model, messages.clone(), styled, reasoning_effort).await;
    }

    let content = assistant_message.content.unwrap_or_default();
    print!("{}", styled.assistant_prompt());
    io::stdout().flush()?;
    print!("{}", style(&content).green());
    io::stdout().flush()?;
    println!();
    Ok(content)
}

async fn stream_completion(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    styled: &Styled,
    reasoning_effort: Option<String>,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?;
    let request = ChatRequest {
        model,
        messages,
        tools: Some(bash_tools()),
        tool_choice: Some("auto"),
        stream: true,
        temperature: 0.7,
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
        .context("Failed to send request")?
        .error_for_status()
        .context("Non-success status from API")?;

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
                    Err(e) => {
                        eprintln!(
                            "\n{} {}",
                            style("[Error]").red().bold(),
                            style(format!("Failed to parse response chunk: {}", e)).red()
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

fn bash_tools() -> Vec<Tool> {
    vec![Tool {
        kind: "function",
        function: ToolDefinition {
            name: "run_bash_command",
            description: "Execute a bash command on the local machine and return stdout, stderr, and exit status.",
            parameters: ToolParameters {
                kind: "object",
                properties: BashToolProperties {
                    command: ToolProperty {
                        kind: "string",
                        description: "The bash command to execute.",
                    },
                },
                required: vec!["command"],
            },
        },
    }]
}

async fn execute_tool_call(tool_call: &ToolCall) -> Result<String> {
    match tool_call.function.name.as_str() {
        "run_bash_command" => {
            let args: BashCommandArgs = serde_json::from_str(&tool_call.function.arguments)
                .context("Failed to parse run_bash_command arguments")?;
            print!(
                "{} {}\n",
                style("[tool]").yellow().bold(),
                style(format!("bash -lc {:?}", args.command)).yellow()
            );
            io::stdout().flush()?;
            run_bash_command(&args.command).await
        }
        other => Ok(format!("Unsupported tool call: {}", other)),
    }
}

async fn run_bash_command(command: &str) -> Result<String> {
    let timed = timeout(
        Duration::from_secs(15),
        Command::new("/bin/bash")
            .arg("-lc")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await;

    let output = match timed {
        Ok(result) => result.context("Failed to execute bash command")?,
        Err(_) => {
            return Ok("Command timed out after 15 seconds.".to_string());
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    if !stdout.is_empty() {
        print!("{}\n", style(&stdout).dim());
    }
    if !stderr.is_empty() {
        print!("{}\n", style(&stderr).red());
    }
    io::stdout().flush()?;

    let rendered = format!(
        "command: {}\nexit_code: {}\nstdout:\n{}\nstderr:\n{}",
        command,
        exit_code,
        if stdout.is_empty() { "<empty>" } else { &stdout },
        if stderr.is_empty() { "<empty>" } else { &stderr }
    );

    if exit_code == 0 {
        Ok(rendered)
    } else {
        Ok(format!(
            "{}\nresult: command failed with a non-zero exit code",
            rendered
        ))
    }
}


struct Styled {
    user: Style,
    assistant: Style,
}

impl Styled {
    fn new() -> Self {
        Self {
            user: Style::new().cyan().bold(),
            assistant: Style::new().green().bold(),
        }
    }

    fn user_prompt(&self) -> console::StyledObject<&'static str> {
        self.user.apply_to("You: ")
    }

    fn assistant_prompt(&self) -> console::StyledObject<&'static str> {
        self.assistant.apply_to("RustAgent: ")
    }
}

fn new_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]",
                "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
            ])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message("Thinking...");
    pb
}
