# Chatbot

This crate is an interactive Rust CLI chat assistant backed by Groq's OpenAI-compatible chat completions API. It supports model switching, optional reasoning effort for supported models, persisted local state, a bounded ReAct loop, and one local tool for running bash commands inside the current working directory.

## Prerequisites

- Rust toolchain
- `GROQ_API_KEY` set in your shell environment

## Run

```bash
cargo run -p chatbot
```

The app reads model configuration from `src/conf.json` and stores the last selected model and reasoning effort in `~/.rustagent`.

If you want the bash tool to operate on the `chatbot` crate itself, run the binary from `coding_problems/chatbot/` so the current working directory is scoped there.

## Commands

- `/help`: show all available commands
- `/model`: list models and choose a new one
- `/tools`: show available tools and their arguments
- `/system`: show current ReAct settings
- `/system react on|off`: enable or disable ReAct mode
- `/system max_iterations <n>`: set the maximum ReAct loop iterations
- `/system trace summary`: keep concise visible phase summaries enabled
- `/quit`: quit the app

## ReAct Loop

When ReAct mode is enabled, each user turn may run through bounded reasoning, action, and reflection phases before the app streams the final answer. Internal reasoning stays hidden; the CLI only shows concise phase summaries such as `Iteration 1/5: reasoning`.

If Groq rejects a tool-enabled turn with a `400 Bad Request`, the client retries with lower temperature. If tool calls still fail, the current turn falls back to a non-tool answer with limitations instead of aborting the conversation.

By default:

- ReAct is enabled
- maximum iterations is `5`
- trace mode is `summary`

These settings are persisted in `~/.rustagent`.

## Tool Use

The assistant can call three local tools:

- `run_bash_command`: executes `/bin/bash -lc "<command>"` locally
- `get_current_date_time`: returns the machine's current local date and time
- `get_weather`: returns the current weather for a city; if no city is provided, it defaults to the current city based on the machine's network location

Use `/tools` in the CLI to see the current tool list and which arguments are required or optional.

The tool returns:

- exit code
- stdout
- stderr

Command execution is limited to 15 seconds.

Tool restrictions:

- commands run in the app's current working directory
- parent-directory traversal such as `..` is blocked
- absolute paths outside the current working directory are blocked
- the tool description instructs the model to prefer macOS/BSD-compatible shell commands over GNU-only flags

The date/time tool does not accept any arguments and does not access the filesystem.

The weather tool accepts an optional city name and uses `wttr.in` over HTTP. If the city is omitted, the service resolves the current location automatically.

Tool output is fed back into the ReAct controller, which can continue refining, request another iteration, or finalize the answer within the configured iteration budget.

## Module Layout

- `src/main.rs`: app startup, chat loop, and module wiring
- `src/models.rs`: request, response, message, and tool-call types
- `src/config.rs`: config parsing and persisted app state
- `src/client.rs`: Groq request flow, bounded ReAct controller, and final-answer streaming
- `src/tools.rs`: tool schema and local bash execution
- `src/ui.rs`: terminal prompts and spinner styling
