use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BASE_SYSTEM_PROMPT: &str = "You are a helpful assistant. You may use the run_bash_command tool when shell output is needed. Before using it, prefer concise, relevant commands and avoid destructive operations unless the user explicitly asked for them.";

pub fn build_system_prompt(react_enabled: bool, max_iterations: usize) -> String {
    if !react_enabled {
        return BASE_SYSTEM_PROMPT.to_string();
    }

    format!(
        "{BASE_SYSTEM_PROMPT}\n\nReAct mode is enabled.\n- Think privately and do not reveal chain-of-thought.\n- Work in bounded reason/action/reflection iterations.\n- Use tools only when necessary.\n- Reflect on whether the answer is sufficient.\n- Continue only if the answer is still insufficient.\n- Stop after at most {max_iterations} iterations and produce the best possible final answer.\n- Keep any visible phase summaries concise."
    )
}

#[derive(Serialize)]
pub struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<&'a str>,
    pub stream: bool,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.into()),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.into()),
            tool_call_id: None,
            name: None,
            tool_calls: None,
        }
    }

    pub fn tool(tool_call_id: String, tool_name: String, tool_output: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(tool_output),
            tool_call_id: Some(tool_call_id),
            name: Some(tool_name),
            tool_calls: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: ToolFunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub function: ToolDefinition,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: ToolParameters,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub properties: Value,
    pub required: Vec<&'static str>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub description: &'static str,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatCompletionResponseChoice>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponseChoice {
    pub message: Message,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionChunk {
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionChoice {
    pub delta: Delta,
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    #[allow(dead_code)]
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BashCommandArgs {
    pub command: String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherArgs {
    pub city: Option<String>,
}
