//! Claw Profiles — runtime configuration summary
//!
//! Uses `ConfigLoader::inspect_collecting_warnings()` for health info,
//! `PluginManager` for plugin count, and `McpConfigCollection` for MCP count.

use plugins::{PluginManager, PluginManagerConfig};
use runtime::ConfigLoader;
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

fn resolve_config_home() -> PathBuf {
    if let Ok(config_home) = std::env::var("CLAW_CONFIG_HOME") {
        return PathBuf::from(config_home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_get_profile() -> ClawProfileInfo {
    let config_home = resolve_config_home();
    let settings_path = config_home.join("settings.json");

    // Read raw settings for display (JSON directly)
    let (settings_exists, raw_settings) = match std::fs::read_to_string(&settings_path) {
        Ok(content) => (
            true,
            serde_json::from_str::<serde_json::Value>(&content).ok(),
        ),
        Err(_) => (false, None),
    };

    // Use ConfigLoader for structured health info
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let loader = ConfigLoader::default_for(&cwd);
    let inspection = loader.inspect_collecting_warnings();

    let (mcp_count, has_permissions, has_hooks, has_features) =
        match &inspection.runtime_config {
            Some(rc) => {
                let mcp = rc.mcp();
                let mcp_count = mcp.valid_count();
                let merged = rc.merged();
                let perms = merged
                    .get("permissions")
                    .and_then(|v| v.as_object())
                    .map(|o| !o.is_empty())
                    .unwrap_or(false);
                let hooks = merged
                    .get("hooks")
                    .and_then(|v| v.as_object())
                    .map(|o| !o.is_empty())
                    .unwrap_or(false);
                let feats = merged
                    .get("features")
                    .and_then(|v| v.as_object())
                    .map(|o| !o.is_empty())
                    .unwrap_or(false);
                (mcp_count, perms, hooks, feats)
            }
            None => (0, false, false, false),
        };

    // Plugin count
    let plugin_count = PluginManager::new(PluginManagerConfig::new(&config_home))
        .plugin_registry_report()
        .ok()
        .map(|r| r.summaries().len())
        .unwrap_or(0);

    ClawProfileInfo {
        config_home: config_home.to_string_lossy().to_string(),
        settings_exists,
        mcp_server_count: mcp_count,
        plugin_count,
        has_permissions,
        has_hooks,
        has_features,
        raw_settings,
    }
}
