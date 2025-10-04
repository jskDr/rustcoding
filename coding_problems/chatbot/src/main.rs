// src/main.rs
use anyhow::{Context, Result};
use console::{style, Style};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};

#[derive(Serialize)]
struct ChatRequest {
    model: &'static str,
    messages: Vec<Message>,
    stream: bool,
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
    delta: Message,
}

const MODEL: &str = "llama3-8b-8192";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    let term = console::Term::stdout();
    let styled = Styled::new();

    term.write_line(&format!(
        "{} {}",
        style("RustAgent:").bold().blue(),
        "Your friendly Rust-based chat assistant."
    ))?;
    term.write_line(&format!(
        "{} {}",
        style("Model:").bold().blue(),
        style(MODEL).yellow()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type 'exit' or 'quit' to end the conversation.").italic()
    ))?;
    term.write_line("")?;

    let mut messages = vec![Message {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    }];

    loop {
        let user_input = get_user_input(&styled)?;
        if user_input.is_empty() {
            continue;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: user_input,
        });

        let assistant_response =
            stream_completion(&client, MODEL, messages.clone(), &styled).await?;
        messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_response,
        });
    }
}

fn get_user_input(styled: &Styled) -> Result<String> {
    let mut input = String::new();
    print!("{}", styled.user_prompt());
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
        std::process::exit(0);
    }

    Ok(input)
}

async fn stream_completion(
    client: &Client,
    model: &'static str,
    messages: Vec<Message>,
    styled: &Styled,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?;
    let request = ChatRequest {
        model,
        messages,
        stream: true,
    };

    let progress_bar = new_progress_bar();
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .context("Failed to send request")?;

    progress_bar.finish_and_clear();

    let mut stream = response.bytes_stream();
    let mut full_response = String::new();

    print!("{}", styled.assistant_prompt());
    io::stdout().flush()?;

    while let Some(item) = stream.next().await {
        let chunk = item.context("Failed to read chunk")?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.lines() {
            if line.starts_with("data:") {
                let data = &line[5..].trim();
                if *data == "[DONE]" {
                    break;
                }
                if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                    if let Some(choice) = chunk.choices.first() {
                        let content = &choice.delta.content;
                        print!("{}", content);
                        io::stdout().flush()?;
                        full_response.push_str(content);
                    }
                }
            }
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
            user: Style::new().bold().cyan(),
            assistant: Style::new().bold().green(),
        }
    }

    fn user_prompt(&self) -> console::StyledObject<&str> {
        self.user.apply_to("You: ")
    }

    fn assistant_prompt(&self) -> console::StyledObject<&str> {
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