// src/main.rs
use anyhow::{Context, Result};
use reqwest::Client;
use std::io::Write; // Add this import for flush
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, BufRead};

#[derive(Serialize)]
struct ChatRequest {
    model: &'static str,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("GROQ_API_KEY")
        .context("GROQ_API_KEY environment variable is required")?;
    let client = Client::new();
    let model = "llama-3.1-8b-instant"; // You can change this to another Groq model

    let mut messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
    ];

    println!("Groq Chat CLI (type 'exit' to quit)");

    loop {
        print!("You: ");
        io::stdout().flush()?; // Ensure prompt is printed immediately

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let input = line.trim().to_string();

        if input == "exit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: input,
        });

        let request = ChatRequest {
            model,
            messages: messages.clone(),
            stream: false,
        };

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        if let Some(choice) = chat_response.choices.first() {
            let assistant_message = &choice.message.content;
            println!("Assistant: {}", assistant_message);
            messages.push(Message {
                role: "assistant".to_string(),
                content: assistant_message.clone(),
            });
        } else {
            println!("No response from assistant.");
        }
    }

    Ok(())
}