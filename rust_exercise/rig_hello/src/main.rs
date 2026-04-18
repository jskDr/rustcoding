use rig::client::{Nothing, CompletionClient};
use rig::completion::Prompt;
use rig::providers::ollama;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = ollama::Client::new(Nothing).unwrap();

    let comedian_agent = client
        .agent("gemma4:e2b")
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();

    let response = comedian_agent.prompt("Entertain me!").await?;
    println!("{response}");

    Ok(())
}

