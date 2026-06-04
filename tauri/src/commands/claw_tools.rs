//! Claw Tools — MCP servers and plugins from ~/.claw/

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub required: bool,
    pub timeout_ms: Option<u64>,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        test::{get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn build_test_app() -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![
                crate::commands::claw_tools::claw_list_mcp_servers,
                crate::commands::claw_tools::claw_list_plugins,
            ])
            .build(mock_context(noop_assets()))
            .expect("mock app should build");
        let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("webview window should build");
        (app, ww)
    }

    fn invoke_ok<R: serde::de::DeserializeOwned>(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: serde_json::Value,
    ) -> R {
        let res = get_ipc_response(
            webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        res.unwrap_or_else(|e| panic!("IPC '{cmd}' failed: {e:?}"))
            .deserialize::<R>()
            .unwrap()
    }

    #[test]
    fn test_list_mcp_servers_returns_array() {
        let servers = claw_list_mcp_servers();
        for srv in &servers {
            assert!(!srv.name.is_empty());
        }
        if let Some(srv) = servers.first() {
            let json = serde_json::to_value(srv).unwrap();
            assert!(json.get("timeoutMs").is_some(), "field should be 'timeoutMs' not 'timeout_ms'");
        }
    }

    #[test]
    fn test_list_plugins_returns_array() {
        let plugins = claw_list_plugins();
        for p in &plugins {
            assert!(!p.id.is_empty());
            assert!(!p.name.is_empty());
        }
        if let Some(p) = plugins.first() {
            let json = serde_json::to_value(p).unwrap();
            assert!(json.get("installPath").is_some(), "field should be 'installPath' not 'install_path'");
            assert!(json.get("name").is_some());
            assert!(json.get("version").is_some());
        }
    }

    // ── IPC 风格测试 ─────────────────────────────────────────────────

    #[test]
    fn test_ipc_list_mcp_servers() {
        let (_app, ww) = build_test_app();
        let result: Vec<serde_json::Value> = invoke_ok(&ww, "claw_list_mcp_servers", serde_json::json!({}));
        for srv in &result {
            assert!(srv.get("name").and_then(|v| v.as_str()).is_some(), "mcp server: name");
            assert!(srv.get("command").and_then(|v| v.as_str()).is_some(), "mcp server: command");
        }
    }

    #[test]
    fn test_ipc_list_plugins() {
        let (_app, ww) = build_test_app();
        let result: Vec<serde_json::Value> = invoke_ok(&ww, "claw_list_plugins", serde_json::json!({}));
        for p in &result {
            assert!(p.get("id").and_then(|v| v.as_str()).is_some(), "plugin: id");
            assert!(p.get("name").and_then(|v| v.as_str()).is_some(), "plugin: name");
            assert!(p.get("version").and_then(|v| v.as_str()).is_some(), "plugin: version");
        }
    }
}
