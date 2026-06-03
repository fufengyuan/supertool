//! Claw Tools — MCP servers and plugins from ~/.claw/

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub required: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: String,
    pub install_path: String,
}

fn claw_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_mcp_servers() -> Vec<McpServerInfo> {
    let settings_path = claw_home().join("settings.json");
    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let servers = match json.get("mcpServers") {
        Some(serde_json::Value::Object(m)) => m,
        _ => return Vec::new(),
    };
    servers
        .iter()
        .map(|(name, cfg)| {
            let command = cfg
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args = cfg
                .get("args")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let required = cfg
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let timeout_ms = cfg
                .get("toolCallTimeoutMs")
                .and_then(|v| v.as_u64());
            McpServerInfo {
                name: name.clone(),
                command,
                args,
                required,
                timeout_ms,
            }
        })
        .collect()
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_plugins() -> Vec<PluginInfo> {
    let installed_path = claw_home().join("plugins").join("installed.json");
    let content = match std::fs::read_to_string(&installed_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let plugins = match json.get("plugins") {
        Some(serde_json::Value::Object(m)) => m,
        _ => return Vec::new(),
    };
    plugins
        .iter()
        .map(|(_, p)| PluginInfo {
            id: p
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: p
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: p
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            kind: p
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("external")
                .to_string(),
            install_path: p
                .get("install_path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}
