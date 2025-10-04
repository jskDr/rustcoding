use anyhow::{Context, Result};
use console::{style, Style};
use futures_util::{Stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    stream: bool,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    content: String,
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
struct Config {
    models: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    let term = console::Term::stdout();
    let styled = Styled::new();

    let config_str = fs::read_to_string("src/conf.json").context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&config_str).context("Failed to parse config file")?;
    let mut model = config.models[0].clone();

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
        content: "You are a helpful assistant.".to_string(),
    }];

    loop {
        let user_input = match get_user_input(&styled)? {
            Some(input) => input,
            None => break,
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
                    model = config.models[selection - 1].clone();
                    term.write_line(&format!(
                        "{} {}",
                        style("Model set to:").bold().blue(),
                        style(&model).yellow()
                    ))?;
                } else {
                    term.write_line(&style("Invalid selection.").red().to_string())?;
                }
            } else {
                term.write_line(&style("Invalid input.").red().to_string())?;
            }
            continue;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: user_input,
        });

        let assistant_response =
            stream_completion(&client, &model, messages.clone(), &styled).await?;
        messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_response,
        });
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

    Ok(Some(input))
}

async fn stream_completion(
    client: &Client,
    model: &str,
    messages: Vec<Message>,
    styled: &Styled,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?;
    let request = ChatRequest {
        model,
        messages,
        stream: true,
        temperature: 0.7,
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
