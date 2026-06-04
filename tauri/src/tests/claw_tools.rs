//! IPC-style tests for claw_tools commands.
//!
//! Generated from `commands/claw_tools.rs` — all tests moved here.

use crate::commands::claw_tools::*;
use crate::tests::invoke_ipc;

use serde_json::json;

fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::claw_tools::claw_list_mcp_servers,
            crate::commands::claw_tools::claw_list_plugins,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
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
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::get_ipc_response;
    use tauri::webview::InvokeRequest;

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
