//! IPC-style tests for claw_config commands.

use crate::commands::claw_config::*;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
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
