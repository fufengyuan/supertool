//! Claw Tools management — MCP servers + plugins
//!
//! MCP: delegates to `handle_mcp_slash_command_json` from `claw-commands` crate
//! Plugins: delegates to `PluginManager` from `claw-plugins` crate

use commands::handle_mcp_slash_command_json;
use plugins::{PluginManager, PluginManagerConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn claw_home() -> PathBuf {
    if let Ok(config_home) = std::env::var("CLAW_CONFIG_HOME") {
        return PathBuf::from(config_home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
}

// ── MCP Servers ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub required: bool,
    pub timeout_ms: Option<u64>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_mcp_servers() -> Result<Vec<McpServerInfo>, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    let result = handle_mcp_slash_command_json(Some("list"), &cwd)
        .map_err(|e| format!("mcp lookup: {e}"))?;

    let servers = result
        .get("servers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "unexpected MCP JSON structure: missing 'servers' array".to_string())?;

    let mut output = Vec::with_capacity(servers.len());
    for server in servers {
        let name = server
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let required = server
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let (command, args, timeout_ms) = server
            .get("details")
            .map(|details| {
                let cmd = details
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let arg_list = details
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let timeout = details
                    .get("tool_call_timeout_ms")
                    .and_then(|v| v.as_u64());
                (cmd, arg_list, timeout)
            })
            .unwrap_or_else(|| (String::new(), Vec::new(), None));

        // For non-Stdio transports, use URL as the display command
        let final_command = if command.is_empty() {
            server
                .get("details")
                .and_then(|d| d.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            command
        };

        output.push(McpServerInfo {
            name,
            command: final_command,
            args,
            required,
            timeout_ms,
        });
    }

    Ok(output)
}

// ── MCP Health Check ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpHealthStatus {
    pub name: String,
    pub status: String, // "ok", "error", "unknown"
    pub command: String,
    pub transport: String,
    pub details: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// Tauri command: check health of all MCP servers.
///
/// Lists all configured MCP servers, then queries each one via
/// `handle_mcp_slash_command_json(Some("show <name>"), cwd)` to surface
/// configuration details and health status.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_mcp_health() -> Result<BTreeMap<String, McpHealthStatus>, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;

    // Get list of all MCP servers
    let list_result = handle_mcp_slash_command_json(Some("list"), &cwd)
        .map_err(|e| format!("mcp list: {e}"))?;

    let servers = list_result
        .get("servers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "unexpected MCP JSON structure: missing 'servers' array".to_string())?;

    let mut health_map = BTreeMap::new();

    for server in servers {
        let name = server
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Get the transport/command info from the list data
        let (command, transport) = server
            .get("details")
            .map(|details| {
                let cmd = details
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // Determine transport type: URL-based or command-based
                let transport_type = if details.get("url").and_then(|v| v.as_str()).is_some() {
                    "http"
                } else if details.get("command").and_then(|v| v.as_str()).is_some() {
                    "stdio"
                } else {
                    "unknown"
                };
                (cmd, transport_type.to_string())
            })
            .unwrap_or_else(|| (String::new(), "unknown".to_string()));

        // Query individual server status
        let show_args = format!("show {name}");
        match handle_mcp_slash_command_json(Some(&show_args), &cwd) {
            Ok(show_result) => {
                health_map.insert(
                    name.clone(),
                    McpHealthStatus {
                        name,
                        status: "ok".to_string(),
                        command,
                        transport,
                        details: Some(show_result),
                        error_message: None,
                    },
                );
            }
            Err(e) => {
                health_map.insert(
                    name.clone(),
                    McpHealthStatus {
                        name,
                        status: "error".to_string(),
                        command,
                        transport,
                        details: None,
                        error_message: Some(e.to_string()),
                    },
                );
            }
        }
    }

    Ok(health_map)
}

// ── Plugins ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: String,
    pub install_path: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_plugins() -> Result<Vec<PluginInfo>, String> {
    let home = claw_home();
    let config = PluginManagerConfig::new(home);
    let manager = PluginManager::new(config);
    let report = manager
        .plugin_registry_report()
        .map_err(|e| format!("plugin registry: {e}"))?;

    Ok(report
        .summaries()
        .iter()
        .map(|s| PluginInfo {
            id: s.metadata.id.clone(),
            name: s.metadata.name.clone(),
            version: s.metadata.version.clone(),
            description: s.metadata.description.clone(),
            kind: serde_json::to_value(&s.metadata.kind)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "external".to_string()),
            install_path: s
                .metadata
                .root
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        })
        .collect())
}
