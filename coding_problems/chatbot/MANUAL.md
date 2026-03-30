# Chatbot Manual

## Overview

This document is a developer-focused manual for the `chatbot` crate. It explains how the current implementation works, how each source module participates in the runtime flow, how slash commands and tools are handled, and what operational limits or failure modes exist today.

`README.md` is the quick-start document. `MANUAL.md` is the code-logic and maintenance reference.

At a high level, the crate is an interactive CLI chatbot built on top of Groq's OpenAI-compatible chat completions API. It supports:

- persistent app state
- model selection
- a bounded ReAct loop
- local function tools
- streaming final answers

The current source layout is:

- `src/main.rs`
- `src/config.rs`
- `src/models.rs`
- `src/ui.rs`
- `src/tools.rs`
- `src/client.rs`

## Runtime Flow

### 1. Startup

The entrypoint is `main()` in `src/main.rs`.

Startup sequence:

1. Build the HTTP client with `build_http_client()`
2. Create terminal styling helpers with `Styled::new()`
3. Load `src/conf.json`
4. Load persisted app state from `~/.rustagent`
5. Print CLI startup hints
6. Build the initial system message with `build_system_prompt(...)`
7. Enter the input loop

The initial conversation history always starts with a system message. That message is regenerated whenever ReAct settings change.

### 2. Input Loop

For each turn:

1. `get_user_input(...)` reads one line from stdin
2. Empty input is ignored
3. Slash commands are handled before normal chat input
4. If the input is not a handled slash command, it is appended as a user message
5. `complete_turn(...)` in `client.rs` executes the turn
6. The final assistant response is appended to conversation history

The command dispatch order in `main.rs` is:

1. `/model`
2. `/help`
3. `/tools`
4. `/system`
5. normal chat input

This matters because slash commands do not go to the model. They are handled locally.

### 3. ReAct Turn Execution

The main turn executor is `complete_turn(...)` in `src/client.rs`.

Behavior:

- If `react_enabled` is `false`, the app uses a simpler single-pass tool flow.
- If `react_enabled` is `true`, the app runs a bounded ReAct loop with at most `react_max_iterations`.

The ReAct loop does not print raw reasoning. It only prints app-owned summaries such as:

- `Iteration 1/5: reasoning`
- `Iteration 1/5: action`
- `Iteration 1/5: reflection`

### 4. Tool Execution

The model may request one or more local tools through Groq-compatible function calling.

When tool calls are returned:

1. The assistant tool-call message is appended to history
2. Each tool is executed locally in `tools.rs`
3. Tool output is wrapped as `role: "tool"` messages
4. The ReAct controller performs a reflection step
5. The loop either finalizes or continues to another iteration

### 5. Final Answer Streaming

Internal control steps are non-streaming. Only the final user-facing answer is streamed to the terminal.

This separation is intentional:

- internal control messages stay machine-oriented
- user output stays readable
- the final answer avoids extra tool negotiation during the last stream

## Module Logic

### `src/main.rs`

`main.rs` is the orchestration layer.

Responsibilities:

- create the HTTP client
- load config and persisted state
- print startup instructions
- dispatch local slash commands
- refresh the system prompt when settings change
- pass normal user messages into `complete_turn(...)`

Important functions:

- `build_http_client()`
  Uses `reqwest::Client::builder().no_proxy()` to avoid the macOS system proxy path that previously caused startup issues.
- `main()`
  Owns the top-level app loop and command ordering.

`main.rs` does not contain tool logic or API request logic. Those are intentionally delegated to `tools.rs` and `client.rs`.

### `src/config.rs`

This module owns static config loading defaults and persisted runtime state.

Important types:

- `Config`
  Loaded from `src/conf.json`
- `AppState`
  Persisted to `~/.rustagent`

Persisted fields:

- `model`
- `reasoning_effort`
- `react_enabled`
- `react_max_iterations`
- `react_trace_mode`

Important functions:

- `load_state(...)`
  Reads `~/.rustagent` if it exists; otherwise falls back to defaults
- `save_state(...)`
  Writes pretty JSON to `~/.rustagent`

Default ReAct behavior:

- enabled: `true`
- max iterations: `5`
- trace mode: `summary`

### `src/models.rs`

This module contains shared request, response, message, and tool schema types.

Important responsibilities:

- build the runtime system prompt with `build_system_prompt(...)`
- define the serialized request shape sent to Groq
- define message constructors used throughout the app
- define tool call types returned by the provider

Important types:

- `ChatRequest`
- `Message`
- `Tool`
- `ToolDefinition`
- `ToolParameters`
- `ToolCall`
- `ToolFunctionCall`
- `ChatCompletionResponse`
- `ChatCompletionChunk`
- `BashCommandArgs`
- `WeatherArgs`

Important `Message` constructors:

- `Message::system(...)`
- `Message::user(...)`
- `Message::assistant(...)`
- `Message::tool(...)`

These helpers keep message construction consistent across `main.rs`, `client.rs`, and `tools.rs`.

### `src/ui.rs`

This module owns CLI interaction and local slash command handling.

Responsibilities:

- prompt rendering
- user input handling
- progress spinner creation
- slash command handlers
- formatted tool and system introspection output
- iteration phase summaries

Important handlers:

- `handle_model_selection(...)`
- `handle_system_command(...)`
- `handle_help_command(...)`
- `handle_tools_command(...)`

This module is the user-facing command layer. It does not perform API calls or tool execution itself.

### `src/tools.rs`

This module is the local tool registry and execution layer.

Responsibilities:

- define the live tool list returned to the model
- execute a tool call by name
- enforce shell safety restrictions
- expose tool metadata to `/tools`

The current source of truth for available tools is `bash_tools()`.

Registered tools:

- `run_bash_command`
- `get_current_date_time`
- `get_weather`

Important behavior:

- `run_bash_command(...)`
  Executes `/bin/bash -lc "<command>"` in the current working directory
- `validate_command_scope(...)`
  Rejects parent traversal and absolute paths outside the current directory
- `get_current_date_time(...)`
  Runs `date` and returns the current machine-local timestamp
- `get_weather(...)`
  Fetches weather via `wttr.in`, optionally by city

Tool-specific notes:

- `run_bash_command` expects a required `command` argument
- `get_current_date_time` takes no arguments
- `get_weather` accepts an optional `city` argument; if omitted, the weather service resolves location from network context

### `src/client.rs`

This module owns all Groq chat-completion logic.

Core responsibilities:

- decide between ReAct mode and single-pass mode
- build control messages
- make provider requests
- retry tool-enabled requests on recoverable 400s
- fall back to non-tool behavior when needed
- stream the final answer

Important functions:

- `complete_turn(...)`
  Top-level entry used by `main.rs`
- `run_react_loop(...)`
  Bounded ReAct controller
- `complete_with_tools_single_pass(...)`
  Simpler path used when ReAct is disabled
- `request_control_message_with_retry(...)`
  Retries tool-enabled requests at lower temperatures
- `call_chat_completion(...)`
  Low-level request helper that captures HTTP status and body
- `stream_final_answer(...)`
  Forces a final user-facing answer path
- `process_sse_stream(...)`
  Parses the provider's server-sent event stream

## Slash Commands

### `/help`

Handled in `ui.rs`.

Shows the currently supported command list. This is the top-level discovery command for CLI users.

### `/model`

Handled partly in `main.rs` and `ui.rs`.

Behavior:

- lists all configured models from `src/conf.json`
- prompts for a numeric selection
- updates `app_state.model`
- optionally updates `reasoning_effort` if the chosen model supports it

### `/tools`

Handled in `ui.rs`.

This command introspects the live tool registry by calling `bash_tools()`, so the output stays synchronized with the actual tool schema rather than a hand-maintained list.

It prints:

- tool name
- tool description
- argument list
- whether each argument is required or optional

### `/system`

Handled in `ui.rs`.

Current supported forms:

- `/system`
- `/system react on`
- `/system react off`
- `/system max_iterations <n>`
- `/system trace summary`

This command mutates `AppState` in memory. The state is written to disk when the app exits.

### `/quit`

Handled in `get_user_input(...)`.

This is the only built-in exit command. Plain `quit` and `exit` are no longer special commands and are treated as ordinary user input.

## Tool System

### Tool Registration

The tool list is built in `tools.rs` by `bash_tools()`.

The model sees each tool as a function schema with:

- name
- description
- JSON-like argument properties
- required field list

That schema is used in two places:

- sent to the provider for tool calling
- rendered back to the user via `/tools`

### `run_bash_command`

Purpose:

- run a shell command locally inside the current working directory

Required argument:

- `command`

Restrictions:

- current-directory only
- parent directory traversal blocked
- absolute paths outside the current directory blocked
- intended for macOS/BSD-compatible commands

Examples:

- `list files in this folder`
- `find the largest file here`

### `get_current_date_time`

Purpose:

- return the machine's current local date and time

Arguments:

- none

Example:

- `what time is it right now?`

### `get_weather`

Purpose:

- return current weather for a specific city or the inferred current city

Arguments:

- optional `city`

Examples:

- `what is the weather in Seattle?`
- `what is the weather here right now?`

### Tool Output Contract

Tool functions return plain text summaries that are inserted into the model conversation as `role: "tool"` messages.

For bash commands this usually includes:

- command
- exit code
- stdout
- stderr

For date/time and weather, the output is a smaller structured text summary.

## ReAct Controller

### Purpose

The ReAct controller allows the chatbot to:

- reason privately
- use tools when needed
- reflect on whether the answer is sufficient
- continue refining until the answer is good enough or the iteration budget is exhausted

### Iteration Model

Each ReAct turn can run up to `react_max_iterations`.

Within each iteration:

1. reasoning
2. optional action
3. reflection

The visible terminal output shows only the phase summaries, not the raw reasoning text.

### Control Output Contract

The internal control prompt requires the model to return JSON with:

- `status`
- `assistant_message`
- `reflection`

Expected `status` values:

- `continue`
- `final`

If the controller output is malformed, `parse_control_decision(...)` falls back to treating the content as a final answer draft.

### Reflection Behavior

After tool execution, the app runs a reflection step. If that step still tries to call another tool while tools are disabled for that reflection request, the client treats it as a recoverable “continue” signal instead of terminating the turn.

This is an important safeguard because some model generations try to call another tool even in a reflection-only step.

### Final Answer Behavior

The final user-facing answer is streamed only after the internal ReAct work is complete.

The final stream is intentionally non-tool:

- it avoids provider errors during finalization
- it keeps the last response purely user-facing

## API and Error Handling

### Normal Request Path

`call_chat_completion(...)` builds a `ChatRequest` and posts to:

- `https://api.groq.com/openai/v1/chat/completions`

The function manually reads:

- HTTP status
- response body

This is done instead of using `error_for_status()` directly so the client can inspect provider failures and recover intelligently.

### Retry Logic for Tool-Enabled 400s

If a tool-enabled request gets `400 Bad Request`, the client treats it as potentially recoverable and retries at:

- `0.7`
- `0.4`
- `0.2`

If all retries fail:

- ReAct mode downgrades to non-tool behavior for the rest of that turn
- single-pass mode downgrades to one non-tool answer for that turn

### Disallowed Tool Calls During Reflection

If the provider returns an error equivalent to:

- `Tool choice is none, but model called a tool`

during a no-tools reflection step, the client converts that error into a recoverable continuation instead of failing the turn.

### Error Message Rendering

Provider error bodies are sanitized with `sanitize_error_body(...)`:

- whitespace is normalized
- output is truncated to a short snippet

This keeps terminal diagnostics useful without dumping large raw error payloads.

## Practical Examples

### Example 1: Inspect Available Tools

Command:

```text
/tools
```

Expected result:

- prints all registered tools
- prints required vs optional arguments

### Example 2: Change ReAct Iteration Budget

Command:

```text
/system max_iterations 3
```

Expected result:

- updates in-memory state immediately
- future turns use at most 3 ReAct iterations
- persisted on exit

### Example 3: Ask for the Current Time

Prompt:

```text
what time is it right now?
```

Expected behavior:

- model may choose `get_current_date_time`
- tool result is inserted into conversation history
- final answer is streamed back to the user

### Example 4: Ask for Weather Without a City

Prompt:

```text
what is the weather now?
```

Expected behavior:

- model may choose `get_weather` without arguments
- weather service resolves the current city automatically

### Example 5: File-System Prompt

Prompt:

```text
list files and find the largest file
```

Expected behavior:

- model may call `run_bash_command`
- command must remain inside the current working directory
- if Groq rejects tool generation, the client retries and may fall back to a non-tool answer with limitations

## Troubleshooting and Limitations

### Current Working Directory Matters

`run_bash_command` is scoped to the process working directory.

If you want shell commands to operate inside the chatbot crate, run the binary from:

```bash
coding_problems/chatbot/
```

### Parent Traversal Is Blocked

Commands that attempt to access paths above the current directory are rejected. This includes obvious `..` traversal and absolute paths outside the current working directory.

### macOS/BSD Shell Assumptions

The tool description explicitly steers the model away from GNU-only flags, but the model can still produce non-portable commands. When that happens:

- the command may fail locally
- the ReAct loop can continue and try another approach

### Weather Tool Requires Network Access

`get_weather` depends on `wttr.in`.

Possible failure cases:

- no internet connection
- DNS or TLS problems
- service outage
- unexpected service response

### Provider 400 Recovery

Tool-enabled 400 responses do not always mean the entire turn is lost. The current client:

- retries with lower temperatures
- falls back to non-tool behavior for the current turn if needed

This improves resilience but does not guarantee a fully tool-backed answer.

### Persisted State Location

Runtime state is stored in:

```text
~/.rustagent
```

If the file is missing or invalid JSON, the app falls back to defaults.

### Config File Location

The model list comes from:

```text
src/conf.json
```

If this file is missing relative to the run directory, startup will fail.

## Maintenance Notes

- `README.md` should stay short and user-facing
- `MANUAL.md` should track internal behavior and architecture
- `/tools` output should remain derived from `bash_tools()` so tool docs stay synchronized with live code
- if a new slash command or tool is added, update both `README.md` and `MANUAL.md`
