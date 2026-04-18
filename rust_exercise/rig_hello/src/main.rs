use std::error::Error;
use std::io::{Write, stdin, stdout};

use futures::StreamExt;
use rig::agent::{MultiTurnStreamItem::StreamAssistantItem, StreamingResult};
use rig::client::{CompletionClient, Nothing};
use rig::completion::Prompt;
use rig::message::Message;
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use serde_json::json;

// ANSI escape codes for colored text
const GRAY: &str = "\x1b[90m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

const MAX_RETRIES: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = ollama::Client::new(Nothing).unwrap();

    // Teacher agent: answers the student
    let teacher = client
        .agent("gemma4:e2b")
        .preamble(include_str!("prompts/system_prompt_en_teacher.md"))
        .additional_params(json!({"think": true}))
        .build();

    // Reviewer agent: checks the teacher's response for problems
    let reviewer = client
        .agent("gemma4:e2b")
        .preamble(include_str!("prompts/system_prompt_reviewer.md"))
        .build();

    let mut history: Vec<Message> = Vec::new();
    println!("English Teacher AI (with reviewer)");
    println!("Type /quit to exit.");
    println!();

    loop {
        print!("You: ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        } else if input == "/quit" {
            break;
        }

        // Get teacher's streaming response
        println!("{CYAN}[Teacher]{RESET}");
        let stream = teacher.stream_chat(&input, &history).await;
        let mut assistant_reply = stream_responses(stream).await?;

        // Review loop: let reviewer check, retry if problematic
        for retry in 0..MAX_RETRIES {
            let review_prompt = format!(
                "Student said: \"{input}\"\n\nTeacher replied: \"{assistant_reply}\"\n\nIs this response OK?"
            );

            println!("{YELLOW}[Reviewer checking...]{RESET}");
            let review_result = reviewer.prompt(&review_prompt).await?;
            let review_text = review_result.to_string();

            if review_text.trim().starts_with("OK") {
                println!("{YELLOW}[Reviewer: OK ✓]{RESET}");
                break;
            } else {
                println!("{YELLOW}[Reviewer: {review_text}]{RESET}");

                if retry < MAX_RETRIES - 1 {
                    // Feed the feedback back to the teacher for a corrected response
                    let correction_prompt = format!(
                        "A reviewer found a problem with your previous response. \
                         Feedback: {review_text}\n\n\
                         Please correct your response to the student who said: \"{input}\""
                    );

                    println!("{CYAN}[Teacher retrying...]{RESET}");
                    let mut retry_history = history.clone();
                    retry_history.push(Message::user(&input));
                    retry_history.push(Message::assistant(&assistant_reply));

                    let stream = teacher
                        .stream_chat(&correction_prompt, &retry_history)
                        .await;
                    assistant_reply = stream_responses(stream).await?;
                }
            }
        }

        // Append final user message and assistant reply to history
        history.push(Message::user(&input));
        history.push(Message::assistant(&assistant_reply));
    }

    Ok(())
}

/// Streams the response to stdout, returns the full accumulated text.
async fn stream_responses<R>(mut stream: StreamingResult<R>) -> Result<String, Box<dyn Error>> {
    let mut full_reply = String::new();
    let mut is_thinking = false;

    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                // End thinking section if we were in one
                if is_thinking {
                    is_thinking = false;
                    print!("{RESET}\n");
                }
                print!("{}", text.text);
                stdout().flush()?;
                full_reply.push_str(&text.text);
            }
            StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning)) => {
                if !is_thinking {
                    is_thinking = true;
                    print!("{GRAY}[thinking] ");
                }
                let text = reasoning.display_text();
                print!("{text}");
                stdout().flush()?;
            }
            StreamAssistantItem(StreamedAssistantContent::ReasoningDelta {
                reasoning, ..
            }) => {
                if !is_thinking {
                    is_thinking = true;
                    print!("{GRAY}[thinking] ");
                }
                print!("{reasoning}");
                stdout().flush()?;
            }
            StreamAssistantItem(StreamedAssistantContent::Final(_)) => {
                if is_thinking {
                    is_thinking = false;
                    print!("{RESET}");
                }
                println!();
            }
            _ => {}
        }
    }

    Ok(full_reply)
}
