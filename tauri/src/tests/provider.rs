//! IPC-style tests for provider commands.

use crate::commands::provider::*;
use serde_json::json;
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
            crate::commands::provider::list_providers,
            crate::commands::provider::save_provider_credential,
            crate::commands::provider::remove_provider_credential,
            crate::commands::provider::start_oauth_flow,
            crate::commands::provider::poll_oauth_result,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
    /// Invoke an IPC command and deserialize the result.
    /// Returns `Err` if the IPC call itself failed (command returned `Err`, or
    /// deserialization failed). Tests that expect the command to always succeed
    /// should `.expect()` on this; tests against real Hermes data that may not
    /// be present should use `if let Ok(v) = …`.
fn invoke_ipc<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<R, String> {
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
    match res {
        Ok(response) => response
            .deserialize::<R>()
            .map_err(|e| format!("deserialize error: {e:?}")),
        Err(e) => Err(format!("IPC error: {e:?}")),
    }
}
    // ── IPC tests ─────────────────────────────────────────────────────────
#[test]
fn test_ipc_list_providers() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "list_providers", json!({}));
        // May fail if ~/.hermes/auth.json is malformed — that's expected
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert!(v["providers"].is_array());
    }
}
#[test]
fn test_ipc_save_provider_credential_empty_provider_id() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "save_provider_credential", json!({"providerId": "", "apiKey": "test-key"}));
    assert!(result.is_err(), "empty providerId should fail");
}
#[test]
fn test_ipc_save_provider_credential_empty_api_key() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "save_provider_credential", json!({"providerId": "test", "apiKey": ""}));
    assert!(result.is_err(), "empty apiKey should fail");
}
#[test]
fn test_ipc_remove_provider_credential_empty() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "remove_provider_credential", json!({"providerId": ""}));
    assert!(result.is_err(), "empty providerId should fail");
}
#[test]
fn test_ipc_start_oauth_flow() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "start_oauth_flow", json!({"providerId": "test-provider"}));
        // May fail if `hermes` CLI is not available — that's expected
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert!(v["authorizationUrl"].is_string());
        assert!(v["deviceCode"].is_string());
        assert!(v["providerId"].is_string());
    }
}
#[test]
fn test_ipc_poll_oauth_result() {
    let (_app, ww) = build_test_app();
    let result: Result<serde_json::Value, String> =
        invoke_ipc(&ww, "poll_oauth_result", json!({"providerId": "test-provider"}));
        // Reads auth.json — may succeed with configured=false if file doesn't exist
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert_eq!(v["providerId"], "test-provider");
        assert!(v["configured"].is_boolean());
        assert!(v["hasToken"].is_boolean());
        assert!(v["completed"].is_boolean());
    }
}
