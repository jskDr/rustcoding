mod client;
mod config;
mod models;
mod tools;
mod ui;

use anyhow::{Context, Result};
use client::complete_turn;
use config::{load_state, save_state, Config};
use console::style;
use reqwest::Client;
use std::fs;
use models::{build_system_prompt, Message};
use ui::{
    get_user_input, handle_help_command, handle_model_selection, handle_system_command,
    handle_tools_command, HelpCommandOutcome, Styled, SystemCommandOutcome,
    ToolsCommandOutcome,
};

fn build_http_client() -> Result<Client> {
    Client::builder()
        .no_proxy()
        .build()
        .context("Failed to create HTTP client")
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = build_http_client()?;
    let term = console::Term::stdout();
    let styled = Styled::new();

    let config_str = fs::read_to_string("src/conf.json").context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&config_str).context("Failed to parse config file")?;
    let mut app_state = load_state(&config);
    let mut model = app_state.model.clone();

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
        style("Type '/quit' to end the conversation.").italic()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type '/model' to see available models.").italic()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type '/system' to view or update ReAct settings.").italic()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type '/help' to see all available commands.").italic()
    ))?;
    term.write_line(&format!(
        "{}",
        style("Type '/tools' to see available tools.").italic()
    ))?;
    term.write_line("")?;

    let mut messages = vec![Message::system(build_system_prompt(
        app_state.react_enabled,
        app_state.react_max_iterations,
    ))];

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
            handle_model_selection(
                &term,
                &config,
                &mut app_state,
                &mut model,
            )?;
            continue;
        }

        match handle_help_command(&term, &user_input)? {
            HelpCommandOutcome::Handled => continue,
            HelpCommandOutcome::NotHelpCommand => {}
        }

        match handle_tools_command(&term, &user_input)? {
            ToolsCommandOutcome::Handled => continue,
            ToolsCommandOutcome::NotToolsCommand => {}
        }

        match handle_system_command(&term, &user_input, &mut app_state)? {
            SystemCommandOutcome::Handled => {
                messages[0] = Message::system(build_system_prompt(
                    app_state.react_enabled,
                    app_state.react_max_iterations,
                ));
                continue;
            }
            SystemCommandOutcome::NotSystemCommand => {}
        }

        messages[0] = Message::system(build_system_prompt(
            app_state.react_enabled,
            app_state.react_max_iterations,
        ));
        messages.push(Message::user(user_input));

        let assistant_response = complete_turn(
            &client,
            &model,
            &mut messages,
            &styled,
            &app_state,
        )
        .await?;
        if !assistant_response.is_empty() {
            messages.push(Message::assistant(assistant_response));
        }
        println!();
    }

    Ok(())
}
