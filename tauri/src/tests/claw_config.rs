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
    // The key is the raw value from disk — it can be any string (empty, '***', or a real key)
    // The old assertion `len >= 4` was from the backend masking era (4 asterisks).
    // Now that we return the raw key, just verify it's a valid string.
    assert!(result.get("apiKey").and_then(|v| v.as_str()).is_some(), "apiKey is a string");
}
#[test]
fn test_claw_config_set_preserves_unspecified_fields() {
    let before = read_claw_config().unwrap_or_default();
    let result = claw_config_set(
        None,
        None,
        Some("test-model".to_string()),
        None,
        None, None, None, None, None, None, None, None,
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

// ── API key masking regression tests ──────────────────────────────────

#[test]
fn test_claw_config_get_returns_raw_key_not_masked() {
    let before = read_claw_config().unwrap_or_default();

    // Save a known key
    claw_config_set(
        Some("sk-tes...cdef".to_string()),
        None, None, None,
        None, None, None, None, None, None, None, None,
    ).expect("set should succeed");

    // Read back — MUST be the raw key, NOT masked as "sk-t...cdef"
    let result = claw_config_get().unwrap();
    let api_key = result.get("apiKey").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(api_key, "sk-tes...cdef",
        "CRITICAL: claw_config_get returned masked key '{}' instead of raw key. \
         This will cause the frontend to save the masked key back to disk, \
         corrupting the user's real API key.",
        api_key,
    );

    // Cleanup
    write_claw_config(&before).ok();
}

#[test]
fn test_claw_config_get_returns_raw_key_when_only_asterisks() {
    let before = read_claw_config().unwrap_or_default();

    // Simulate the corrupted state: file has "***" literally
    // This happened when the previous backend masking returned "***" and the
    // frontend saved it back
    claw_config_set(
        Some("***".to_string()),
        None, None, None,
        None, None, None, None, None, None, None, None,
    ).expect("set should succeed");

    let result = claw_config_get().unwrap();
    let api_key = result.get("apiKey").and_then(|v| v.as_str()).unwrap_or("");
    // Must return "***" as-is, not "****" (4 asterisks from old masking logic)
    assert_eq!(api_key, "***",
        "claw_config_get must return raw '***' not '{}'", api_key,
    );

    // Cleanup
    write_claw_config(&before).ok();
}

#[test]
fn test_setup_env_from_claw_config_sets_raw_not_masked() {
    let before = read_claw_config().unwrap_or_default();

    // Save a known key
    claw_config_set(
        Some("sk-env...7890".to_string()),
        Some("https://test.api.com/v1".to_string()),
        Some("test-model".to_string()),
        None,
        None, None, None, None, None, None, None, None,
    ).expect("set should succeed");

    // This should set env vars with the RAW key, not a masked version
    crate::commands::claw_chat::setup_env_from_claw_config()
        .expect("setup_env should succeed");

    let env_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    assert_eq!(env_key, "sk-env...7890",
        "CRITICAL: setup_env set masked key '{}' instead of raw key",
        env_key,
    );

    let env_url = std::env::var("OPENAI_BASE_URL").unwrap_or_default();
    assert_eq!(env_url, "https://test.api.com/v1", "base URL should match");

    // Cleanup
    write_claw_config(&before).ok();
}
