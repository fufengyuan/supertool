//! Claw Profiles — runtime configuration summary from ~/.claw/

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClawProfileInfo {
    pub config_home: String,
    pub settings_exists: bool,
    pub mcp_server_count: usize,
    pub plugin_count: usize,
    pub has_permissions: bool,
    pub has_hooks: bool,
    pub has_features: bool,
    pub raw_settings: Option<serde_json::Value>,
}

fn claw_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_get_profile() -> ClawProfileInfo {
    let home = claw_home();
    let settings_path = home.join("settings.json");
    let installed_path = home.join("plugins").join("installed.json");

    let (settings_exists, raw_settings, mcp_count, has_permissions, has_hooks, has_features) =
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mcp = json
                    .get("mcpServers")
                    .and_then(|v| v.as_object())
                    .map(|m| m.len())
                    .unwrap_or(0);
                let perms = json.get("permissions").is_some();
                let hooks = json.get("hooks").is_some();
                let feats = json.get("features").is_some();
                (true, Some(json), mcp, perms, hooks, feats)
            } else {
                (true, None, 0, false, false, false)
            }
        } else {
            (false, None, 0, false, false, false)
        };

    let plugin_count = if let Ok(content) = std::fs::read_to_string(&installed_path) {
        serde_json::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|v| v.get("plugins").cloned())
            .and_then(|v| v.as_object().map(|m| m.len()))
            .unwrap_or(0)
    } else {
        0
    };

    ClawProfileInfo {
        config_home: home.to_string_lossy().to_string(),
        settings_exists,
        mcp_server_count: mcp_count,
        plugin_count,
        has_permissions,
        has_hooks,
        has_features,
        raw_settings,
    }
}
