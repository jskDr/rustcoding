use anyhow::{bail, Result};
use crate::tools::bash_tools;
use console::{style, Style};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::io::{self, Write};
use crate::config::{AppState, Config, ReactTraceMode};

pub enum SystemCommandOutcome {
    Handled,
    NotSystemCommand,
}

pub enum HelpCommandOutcome {
    Handled,
    NotHelpCommand,
}

pub enum ToolsCommandOutcome {
    Handled,
    NotToolsCommand,
}

pub struct Styled {
    user: Style,
    assistant: Style,
}

impl Styled {
    pub fn new() -> Self {
        Self {
            user: Style::new().cyan().bold(),
            assistant: Style::new().green().bold(),
        }
    }

    pub fn user_prompt(&self) -> console::StyledObject<&'static str> {
        self.user.apply_to("You: ")
    }

    pub fn assistant_prompt(&self) -> console::StyledObject<&'static str> {
        self.assistant.apply_to("RustAgent: ")
    }
}

pub fn get_user_input(styled: &Styled) -> Result<Option<String>> {
    let mut input = String::new();
    print!("{}", styled.user_prompt());
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    if input == "/quit" {
        println!("{}", style("Goodbye!").dim());
        return Ok(None);
    }

    println!();
    Ok(Some(input))
}

pub fn new_progress_bar() -> ProgressBar {
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

pub fn handle_model_selection(
    term: &console::Term,
    config: &Config,
    app_state: &mut AppState,
    model: &mut String,
) -> Result<()> {
    term.write_line("Available models:")?;
    for (i, available_model) in config.models.iter().enumerate() {
        term.write_line(&format!("  {}: {}", i + 1, available_model))?;
    }
    term.write_str("Select a model (number): ")?;
    term.flush()?;

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    if let Ok(selection) = selection.trim().parse::<usize>() {
        if selection > 0 && selection <= config.models.len() {
            app_state.model = config.models[selection - 1].clone();
            *model = app_state.model.clone();
            term.write_line(&format!(
                "{} {}",
                style("Model set to:").bold().blue(),
                style(model.as_str()).yellow()
            ))?;

            if config.reaoning_effor_support_models.contains(&*model) {
                term.write_line("This model supports reasoning effort.")?;
                term.write_str("Select reasoning effort (low, medium, high): ")?;
                term.flush()?;

                let mut effort_selection = String::new();
                io::stdin().read_line(&mut effort_selection)?;
                let effort_selection = effort_selection.trim().to_lowercase();

                if ["low", "medium", "high"].contains(&effort_selection.as_str()) {
                    app_state.reasoning_effort = Some(effort_selection.clone());
                    term.write_line(&format!(
                        "{} {}",
                        style("Reasoning effort set to:").bold().blue(),
                        style(app_state.reasoning_effort.as_ref().unwrap()).yellow()
                    ))?;
                } else {
                    term.write_line(
                        &style("Invalid selection. Defaulting to no reasoning effort.")
                            .red()
                            .to_string(),
                    )?;
                    app_state.reasoning_effort = None;
                }
            } else {
                app_state.reasoning_effort = None;
            }
        } else {
            term.write_line(&style("Invalid selection.").red().to_string())?;
        }
    } else {
        term.write_line(&style("Invalid input.").red().to_string())?;
    }

    println!();
    Ok(())
}

pub fn handle_system_command(
    term: &console::Term,
    input: &str,
    app_state: &mut AppState,
) -> Result<SystemCommandOutcome> {
    if !input.starts_with("/system") {
        return Ok(SystemCommandOutcome::NotSystemCommand);
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.as_slice() {
        ["/system"] => print_system_settings(term, app_state)?,
        ["/system", "react", "on"] => {
            app_state.react_enabled = true;
            term.write_line("ReAct mode enabled.")?;
        }
        ["/system", "react", "off"] => {
            app_state.react_enabled = false;
            term.write_line("ReAct mode disabled.")?;
        }
        ["/system", "max_iterations", value] => {
            let parsed = value
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("max_iterations must be a positive integer"))?;
            if parsed == 0 {
                bail!("max_iterations must be greater than 0");
            }
            app_state.react_max_iterations = parsed;
            term.write_line(&format!("ReAct max iterations set to {}.", parsed))?;
        }
        ["/system", "trace", "summary"] => {
            app_state.react_trace_mode = ReactTraceMode::Summary;
            term.write_line("ReAct trace mode set to summary.")?;
        }
        _ => {
            term.write_line("Invalid /system command.")?;
            term.write_line("Usage: /system")?;
            term.write_line("       /system react on")?;
            term.write_line("       /system react off")?;
            term.write_line("       /system max_iterations <n>")?;
            term.write_line("       /system trace summary")?;
        }
    }

    println!();
    Ok(SystemCommandOutcome::Handled)
}

pub fn handle_help_command(term: &console::Term, input: &str) -> Result<HelpCommandOutcome> {
    if input != "/help" {
        return Ok(HelpCommandOutcome::NotHelpCommand);
    }

    print_help(term)?;
    println!();
    Ok(HelpCommandOutcome::Handled)
}

pub fn handle_tools_command(term: &console::Term, input: &str) -> Result<ToolsCommandOutcome> {
    if input != "/tools" {
        return Ok(ToolsCommandOutcome::NotToolsCommand);
    }

    print_tools(term)?;
    println!();
    Ok(ToolsCommandOutcome::Handled)
}

pub fn print_iteration_phase(iteration: usize, max_iterations: usize, phase: &str) -> Result<()> {
    println!(
        "{} {}",
        style(format!("Iteration {}/{}:", iteration, max_iterations))
            .blue()
            .bold(),
        style(phase).italic()
    );
    io::stdout().flush()?;
    Ok(())
}

fn print_system_settings(term: &console::Term, app_state: &AppState) -> Result<()> {
    term.write_line("Current system settings:")?;
    term.write_line(&format!(
        "  react: {}",
        if app_state.react_enabled { "on" } else { "off" }
    ))?;
    term.write_line(&format!(
        "  max_iterations: {}",
        app_state.react_max_iterations
    ))?;
    term.write_line(&format!(
        "  trace: {}",
        match app_state.react_trace_mode {
            ReactTraceMode::Summary => "summary",
        }
    ))?;
    Ok(())
}

fn print_help(term: &console::Term) -> Result<()> {
    term.write_line("Available commands:")?;
    term.write_line("  /help: show all available commands")?;
    term.write_line("  /model: list models and choose a new one")?;
    term.write_line("  /tools: show available tools and their arguments")?;
    term.write_line("  /system: show current ReAct settings")?;
    term.write_line("  /system react on|off: enable or disable ReAct mode")?;
    term.write_line("  /system max_iterations <n>: set the maximum ReAct loop iterations")?;
    term.write_line("  /system trace summary: keep concise visible phase summaries enabled")?;
    term.write_line("  /quit: quit the app")?;
    Ok(())
}

fn print_tools(term: &console::Term) -> Result<()> {
    term.write_line("Available tools:")?;

    for tool in bash_tools() {
        term.write_line(&format!(
            "  {}: {}",
            tool.function.name, tool.function.description
        ))?;

        let properties = tool
            .function
            .parameters
            .properties
            .as_object()
            .cloned()
            .unwrap_or_default();

        if properties.is_empty() {
            term.write_line("    arguments: none")?;
            continue;
        }

        term.write_line("    arguments:")?;
        for (name, value) in properties {
            let description = extract_property_description(&value);
            let required = if tool.function.parameters.required.iter().any(|item| item == &name) {
                "required"
            } else {
                "optional"
            };
            term.write_line(&format!("      {} ({}): {}", name, required, description))?;
        }
    }

    Ok(())
}

fn extract_property_description(value: &Value) -> String {
    value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("No description.")
        .to_string()
}
