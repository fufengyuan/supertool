//! Claw Agent Configuration — list and inspect agent definitions.
//!
//! Delegates to `handle_agents_slash_command_json()` from the `claw-commands`
//! crate and maps the JSON response to structured Rust types for the frontend.

use commands::handle_agents_slash_command_json;
use serde::{Deserialize, Serialize};

/// Agent info returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub model: String,
    pub path: String,
    pub config: Option<serde_json::Value>,
}

/// Tauri command: list all Claw agent configurations.
///
/// Calls `handle_agents_slash_command_json(None, cwd)` to discover agents from
/// all definition roots, then maps the JSON response into a structured Vec.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_agents() -> Result<Vec<AgentInfo>, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;

    let result = handle_agents_slash_command_json(None, &cwd)
        .map_err(|e| format!("failed to list agents: {e}"))?;

    log::info!("[claw_agents] Raw response: {:?}", result);

    // The response has the shape:
    // { "kind": "agents", "action": "list", "agents": [...] }
    let agents = result
        .get("agents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            "unexpected agent list JSON structure: missing 'agents' array".to_string()
        })?;

    let mut output = Vec::with_capacity(agents.len());
    for agent in agents {
        let name = agent
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let description = agent
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let model = agent
            .get("config")
            .and_then(|c| c.as_object())
            .and_then(|c| c.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = agent
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let config = agent.get("config").cloned();

        output.push(AgentInfo {
            name,
            description,
            model,
            path,
            config,
        });
    }

    Ok(output)
}
