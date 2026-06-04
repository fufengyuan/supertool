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

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        test::{get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    // ── Direct unit tests ───────────────────────────────────────────────

    #[test]
    fn test_get_profile_returns_config_home() {
        let profile = claw_get_profile();
        assert!(!profile.config_home.is_empty());
        assert!(profile.config_home.ends_with(".claw"));
        // Verify camelCase serialization
        let json = serde_json::to_value(&profile).unwrap();
        assert!(json.get("configHome").is_some(), "field should be 'configHome' not 'config_home'");
        assert!(json.get("mcpServerCount").is_some(), "field should be 'mcpServerCount' not 'mcp_server_count'");
        assert!(json.get("pluginCount").is_some(), "field should be 'pluginCount' not 'plugin_count'");
        assert!(json.get("hasPermissions").is_some(), "field should be 'hasPermissions' not 'has_permissions'");
        assert!(json.get("hasHooks").is_some(), "field should be 'hasHooks' not 'has_hooks'");
        assert!(json.get("rawSettings").is_some(), "field should be 'rawSettings' not 'raw_settings'");
    }

    #[test]
    fn test_get_profile_fields_are_correct_types() {
        let profile = claw_get_profile();
        assert!(profile.mcp_server_count == 0 || profile.mcp_server_count > 0);
        assert_eq!(profile.settings_exists, profile.raw_settings.is_some());
        assert!(profile.plugin_count == 0 || profile.plugin_count > 0);
        assert!(!profile.has_permissions || profile.settings_exists);
        assert!(!profile.has_hooks || profile.settings_exists);
        assert!(!profile.has_features || profile.settings_exists);
    }

    // ── IPC 风格测试 (simulating frontend invoke("clawGetProfile")) ────

    fn build_test_app() -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![
                crate::commands::claw_profiles::claw_get_profile,
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
    fn test_ipc_claw_get_profile_returns_all_fields() {
        let (_app, ww) = build_test_app();
        let profile: ClawProfileInfo = invoke_ok(&ww, "claw_get_profile", serde_json::json!({}));

        assert!(!profile.config_home.is_empty());
        assert!(profile.config_home.ends_with(".claw"));
    }

    #[test]
    fn test_ipc_claw_get_profile_camel_case_json() {
        let (_app, ww) = build_test_app();
        let json: serde_json::Value = invoke_ok(&ww, "claw_get_profile", serde_json::json!({}));

        assert!(!json["configHome"].as_str().unwrap().is_empty(), "configHome should be non-empty");
        assert!(json["configHome"].as_str().unwrap().ends_with(".claw"), "configHome should end with .claw");
        assert!(json.get("mcpServerCount").is_some(), "missing mcpServerCount");
        assert!(json.get("pluginCount").is_some(), "missing pluginCount");
        assert!(json.get("hasPermissions").is_some(), "missing hasPermissions");
        assert!(json.get("hasHooks").is_some(), "missing hasHooks");
        assert!(json.get("hasFeatures").is_some(), "missing hasFeatures");
        assert!(json.get("rawSettings").is_some(), "missing rawSettings");

        // Ensure snake_case keys are NOT present in the IPC response
        assert!(json.get("config_home").is_none(), "snake_case key 'config_home' should not appear in IPC response");
        assert!(json.get("mcp_server_count").is_none(), "snake_case key 'mcp_server_count' should not appear in IPC response");
        assert!(json.get("plugin_count").is_none(), "snake_case key 'plugin_count' should not appear in IPC response");
    }

    #[test]
    fn test_ipc_claw_get_profile_types() {
        let (_app, ww) = build_test_app();
        let json: serde_json::Value = invoke_ok(&ww, "claw_get_profile", serde_json::json!({}));

        assert!(json["configHome"].is_string(), "configHome should be a string");
        assert!(json["mcpServerCount"].is_number(), "mcpServerCount should be a number");
        assert!(json["pluginCount"].is_number(), "pluginCount should be a number");
        assert!(json["hasPermissions"].is_boolean(), "hasPermissions should be a boolean");
        assert!(json["hasHooks"].is_boolean(), "hasHooks should be a boolean");
        assert!(json["hasFeatures"].is_boolean(), "hasFeatures should be a boolean");
        assert!(
            json["rawSettings"].is_null() || json["rawSettings"].is_object(),
            "rawSettings should be null or an object"
        );
    }
}
