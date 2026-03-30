use crate::models::{
    BashCommandArgs, Tool, ToolCall, ToolDefinition, ToolParameters, ToolProperty, WeatherArgs,
};
use anyhow::{Context, Result};
use console::style;
use reqwest::Client;
use serde_json::json;
use std::io::{self, Write};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub fn bash_tools() -> Vec<Tool> {
    vec![
        Tool {
            kind: "function",
            function: ToolDefinition {
                name: "run_bash_command",
                description: "Execute a bash command in the current working directory on the local machine and return stdout, stderr, and exit status. Assume macOS/BSD userland, prefer portable shell commands, and avoid GNU-only flags such as `find -printf`.",
                parameters: ToolParameters {
                    kind: "object",
                    properties: json!({
                        "command": ToolProperty {
                            kind: "string",
                            description: "The bash command to execute. Keep all file access inside the current working directory and prefer macOS-compatible command options.",
                        }
                    }),
                    required: vec!["command"],
                },
            },
        },
        Tool {
            kind: "function",
            function: ToolDefinition {
                name: "get_current_date_time",
                description: "Get the current local date and time from the machine running the chatbot.",
                parameters: ToolParameters {
                    kind: "object",
                    properties: json!({}),
                    required: vec![],
                },
            },
        },
        Tool {
            kind: "function",
            function: ToolDefinition {
                name: "get_weather",
                description: "Get the current weather for a city. If no city is provided, use the current city based on the machine's network location.",
                parameters: ToolParameters {
                    kind: "object",
                    properties: json!({
                        "city": ToolProperty {
                            kind: "string",
                            description: "Optional city name. If omitted, use the current city."
                        }
                    }),
                    required: vec![],
                },
            },
        },
    ]
}

pub async fn execute_tool_call(tool_call: &ToolCall) -> Result<String> {
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
        "get_current_date_time" => {
            print!(
                "{} {}\n",
                style("[tool]").yellow().bold(),
                style("date").yellow()
            );
            io::stdout().flush()?;
            get_current_date_time().await
        }
        "get_weather" => {
            let args: WeatherArgs = serde_json::from_str(&tool_call.function.arguments)
                .unwrap_or(WeatherArgs { city: None });
            let target = args.city.as_deref().unwrap_or("current city");
            print!(
                "{} {}\n",
                style("[tool]").yellow().bold(),
                style(format!("weather for {}", target)).yellow()
            );
            io::stdout().flush()?;
            get_weather(args.city).await
        }
        other => Ok(format!("Unsupported tool call: {}", other)),
    }
}

async fn run_bash_command(command: &str) -> Result<String> {
    let cwd = std::env::current_dir().context("Failed to determine current working directory")?;
    validate_command_scope(command, &cwd)?;

    let timed = timeout(
        Duration::from_secs(15),
        Command::new("/bin/bash")
            .arg("-lc")
            .arg(command)
            .current_dir(&cwd)
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

fn validate_command_scope(command: &str, cwd: &Path) -> Result<()> {
    let cwd_str = cwd.to_string_lossy();
    for raw_token in tokenize_shell_like(command) {
        let token = raw_token.trim_matches(|c| matches!(c, '"' | '\'' | '`'));
        if token.is_empty() {
            continue;
        }

        if token == ".."
            || token.starts_with("../")
            || token.ends_with("/..")
            || token.contains("/../")
        {
            anyhow::bail!(
                "Command is not allowed to access parent directories above {}.",
                cwd.display()
            );
        }

        if token.starts_with('/') && !token.starts_with(cwd_str.as_ref()) {
            anyhow::bail!(
                "Absolute paths outside the current directory are not allowed: {}",
                token
            );
        }
    }

    Ok(())
}

fn tokenize_shell_like(command: &str) -> Vec<&str> {
    command
        .split(|c: char| c.is_whitespace() || matches!(c, '|' | '&' | ';' | '(' | ')' | '<' | '>'))
        .collect()
}

async fn get_current_date_time() -> Result<String> {
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S %Z")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to get current date and time")?;

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

    Ok(format!(
        "exit_code: {}\ncurrent_date_time: {}\nstderr:\n{}",
        exit_code,
        if stdout.is_empty() { "<empty>" } else { &stdout },
        if stderr.is_empty() { "<empty>" } else { &stderr }
    ))
}

async fn get_weather(city: Option<String>) -> Result<String> {
    let client = Client::builder()
        .no_proxy()
        .build()
        .context("Failed to create weather client")?;

    let location = city.unwrap_or_default();
    let encoded_location = location.replace(' ', "%20");
    let url = if encoded_location.is_empty() {
        "https://wttr.in/?format=%l:+%C+%t+%h+%w".to_string()
    } else {
        format!(
            "https://wttr.in/{}?format=%l:+%C+%t+%h+%w",
            encoded_location
        )
    };

    let response = client
        .get(url)
        .header("User-Agent", "rustcoding-chatbot")
        .send()
        .await
        .context("Failed to fetch weather")?
        .error_for_status()
        .context("Weather service returned a non-success status")?;

    let weather = response
        .text()
        .await
        .context("Failed to read weather response")?
        .trim()
        .to_string();

    if !weather.is_empty() {
        print!("{}\n", style(&weather).dim());
    }
    io::stdout().flush()?;

    Ok(format!(
        "current_weather: {}",
        if weather.is_empty() { "<empty>" } else { &weather }
    ))
}
