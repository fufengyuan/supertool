//! Centralized IPC-style tests for Tauri commands
//!
//! All tests use `tauri::test::get_ipc_response` to simulate frontend `invoke()`
//! calls through the Tauri 2 IPC pipeline with `MockRuntime`.

// Modules are added one by one as command files are extracted.
pub mod agent;
pub mod alert;
pub mod claw_config;
pub mod claw_cron;
pub mod claw_profiles;
pub mod claw_skills;
pub mod claw_tools;
pub mod database;
pub mod hermes_config;
pub mod hermes_cron;
pub mod hermes_ipc;
pub mod hermes_memory;
pub mod hermes_skills;
pub mod provider;

// ── Shared IPC helpers ──────────────────────────────────────────────────────

use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};

/// Build a mock Tauri app + webview window.
///
/// Each module file creates its own `build_test_app()` since `generate_handler!`
/// produces a non-copyable closure type that can't be passed generically.
pub fn build_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}

/// Invoke a Tauri IPC command via `get_ipc_response` and deserialize the result.
///
/// Returns `Ok(R)` on success, `Err(String)` if the IPC call failed (command
/// returned `Err`, serialization failure, or ACL rejection).
pub fn invoke_ipc<R: serde::de::DeserializeOwned>(
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
        Err(v) => Err(format!("IPC error: {v:?}")),
    }
}

/// Convenience: invoke via IPC and unwrap the result.
pub fn invoke_ok<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> R {
    invoke_ipc(webview, cmd, body)
        .unwrap_or_else(|e| panic!("IPC command '{cmd}' failed: {e}"))
}
