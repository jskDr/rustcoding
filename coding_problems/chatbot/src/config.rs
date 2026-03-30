use anyhow::Result;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;

pub const DEFAULT_REACT_MAX_ITERATIONS: usize = 5;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub models: Vec<String>,
    pub reaoning_effor_support_models: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReactTraceMode {
    #[default]
    Summary,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub model: String,
    pub reasoning_effort: Option<String>,
    #[serde(default = "default_react_enabled")]
    pub react_enabled: bool,
    #[serde(default = "default_react_max_iterations")]
    pub react_max_iterations: usize,
    #[serde(default)]
    pub react_trace_mode: ReactTraceMode,
}

impl AppState {
    fn default(config: &Config) -> Self {
        Self {
            model: config.models[0].clone(),
            reasoning_effort: None,
            react_enabled: default_react_enabled(),
            react_max_iterations: default_react_max_iterations(),
            react_trace_mode: ReactTraceMode::Summary,
        }
    }
}

fn default_react_enabled() -> bool {
    true
}

fn default_react_max_iterations() -> usize {
    DEFAULT_REACT_MAX_ITERATIONS
}

pub fn load_state(config: &Config) -> AppState {
    let state_path = match home_dir() {
        Some(path) => path.join(".rustagent"),
        None => return AppState::default(config),
    };

    if let Ok(state_str) = fs::read_to_string(state_path) {
        if let Ok(state) = serde_json::from_str(&state_str) {
            return state;
        }
    }

    AppState::default(config)
}

pub fn save_state(state: &AppState) -> Result<()> {
    let state_path = match home_dir() {
        Some(path) => path.join(".rustagent"),
        None => return Ok(()),
    };
    let state_str = serde_json::to_string_pretty(state)?;
    fs::write(state_path, state_str)?;
    Ok(())
}
