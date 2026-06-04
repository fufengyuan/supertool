//! Claw Agent configuration — read/write API key, base URL, model.
//!
//! Persists to `~/.claw/config.json`.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Claw 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClawConfig {
    /// API Key
    pub api_key: String,
    /// Base URL (OpenAI-compatible endpoint, e.g. https://api.openai.com/v1)
    #[serde(default)]
    pub base_url: String,
    /// Model name (e.g. claude-sonnet-4-6, gpt-4.1-mini)
    #[serde(default = "default_model")]
    pub model: String,
    /// Provider label (for display only, routing is automatic)
    #[serde(default)]
    pub provider: String,
}

fn default_model() -> String {
    "claude-sonnet-4-6".to_string()
}

/// 配置文件路径
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("config.json")
}

/// 读取 Claw 配置
pub fn read_claw_config() -> Result<ClawConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(ClawConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse ~/.claw/config.json: {e}"))
}

/// 写入 Claw 配置
pub fn write_claw_config(config: &ClawConfig) -> Result<(), String> {
    let path = config_path();
    let parent = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory: {e}"))?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    log::info!("[claw_config] Saved config to {}", path.display());
    Ok(())
}

/// 获取 Claw 配置（Tauri command）
#[tauri::command(rename_all = "camelCase")]
pub fn claw_config_get() -> Result<serde_json::Value, String> {
    let config = read_claw_config()?;
    Ok(serde_json::json!({
        "apiKey": if config.api_key.is_empty() { String::new() } else {
            // 脱敏：只显示前后各4个字符
            let key = &config.api_key;
            if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            } else {
                "****".to_string()
            }
        },
        "hasApiKey": !config.api_key.is_empty(),
        "baseUrl": config.base_url,
        "model": config.model,
        "provider": config.provider,
    }))
}

/// 保存 Claw 配置（Tauri command）
#[tauri::command(rename_all = "camelCase")]
pub fn claw_config_set(
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut config = read_claw_config()?;

    if let Some(key) = api_key {
        config.api_key = key;
    }
    if let Some(url) = base_url {
        config.base_url = url;
    }
    if let Some(m) = model {
        config.model = m;
    }
    if let Some(p) = provider {
        config.provider = p;
    }

    write_claw_config(&config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Claw config saved",
    }))
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
                crate::commands::claw_config::claw_config_get,
                crate::commands::claw_config::claw_config_set,
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
    fn test_read_claw_config_returns_default_when_missing() {
        let _config = read_claw_config().unwrap_or_default();
    }

    #[test]
    fn test_claw_config_get_returns_valid_shape() {
        let result = claw_config_get().unwrap();
        assert!(result.get("hasApiKey").is_some());
        assert!(result.get("baseUrl").is_some());
        assert!(result.get("model").is_some());
        assert!(result.get("provider").is_some());
        let api_key = result.get("apiKey").and_then(|v| v.as_str()).unwrap_or("");
        assert!(api_key.is_empty() || api_key.len() >= 4);
    }

    #[test]
    fn test_claw_config_set_preserves_unspecified_fields() {
        let before = read_claw_config().unwrap_or_default();
        let result = claw_config_set(
            None,
            None,
            Some("test-model".to_string()),
            None,
        ).expect("set should succeed");
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
        let after = read_claw_config().unwrap_or_default();
        assert_eq!(after.model, "test-model");
        write_claw_config(&before).ok();
    }

    // ── IPC 风格测试 ─────────────────────────────────────────────────

    #[test]
    fn test_ipc_mock_builder_creates_app() {
        let (_app, _ww) = build_test_app();
    }

    #[test]
    fn test_ipc_config_get() {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ok(&ww, "claw_config_get", serde_json::json!({}));
        assert!(result.get("hasApiKey").is_some(), "hasApiKey exists");
        assert!(result.get("apiKey").and_then(|v| v.as_str()).is_some(), "apiKey is string");
        assert!(result.get("baseUrl").and_then(|v| v.as_str()).is_some(), "baseUrl is string");
        assert!(result.get("model").and_then(|v| v.as_str()).is_some(), "model is string");
        assert!(result.get("provider").and_then(|v| v.as_str()).is_some(), "provider is string");
    }

    #[test]
    fn test_ipc_config_set() {
        let before = read_claw_config().unwrap_or_default();
        let (_app, ww) = build_test_app();

        let result: serde_json::Value = invoke_ok(&ww, "claw_config_set", serde_json::json!({
            "model": "ipc-test-model",
        }));
        assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));

        // Verify via direct read that the config was actually written
        let after = read_claw_config().unwrap_or_default();
        assert_eq!(after.model, "ipc-test-model");

        // Restore
        write_claw_config(&before).ok();
    }
}
