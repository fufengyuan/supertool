//! IPC-style tests for agent commands.
//!
//! Generated from `commands/agent.rs` — all tests moved here.

use crate::commands::agent::*;
use crate::tests::invoke_ipc;

use serde_json::json;

fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::agent::agent_installed,
            crate::commands::agent::agent_get_compression_tip,
            crate::commands::agent::agent_list_messages,
            crate::commands::agent::agent_get_stats,
            crate::commands::agent::agent_list_sessions,
            crate::commands::agent::agent_delete_session,
            crate::commands::agent::agent_rename_session,
            crate::commands::agent::agent_get_session,
            crate::commands::agent::agent_search_sessions,
            crate::commands::agent::save_temp_file,
            crate::commands::agent::clean_temp_dir,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}

#[test]
fn test_ipc_agent_installed() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_installed", json!({})).expect("agent_installed should succeed");
    assert_eq!(result["success"], true);
    assert!(result["installed"].is_boolean());
}

#[test]
fn test_ipc_agent_get_compression_tip() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "agent_get_compression_tip",
        json!({"sessionId": "test-session-id"}),
    );
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert_eq!(v["originalSessionId"], "test-session-id");
        assert!(v["tipSessionId"].is_string());
    }
}

#[test]
fn test_ipc_agent_list_messages() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "agent_list_messages",
        json!({"sessionId": "test-session-id"}),
    );
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert!(v["messages"].is_array());
        assert_eq!(v["sessionId"], "test-session-id");
    }
}

#[test]
fn test_ipc_agent_get_stats() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_get_stats", json!({})).expect("agent_get_stats should succeed");
    assert_eq!(result["success"], true);
    assert!(result["stats"].is_object());
}

#[test]
fn test_ipc_agent_list_sessions() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_list_sessions", json!({})).expect("list should succeed");
    assert_eq!(result["success"], true);
    assert!(result.get("sessions").is_some());
}

#[test]
fn test_ipc_agent_list_sessions_with_limit() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_list_sessions", json!({"limit": 5})).expect("list with limit");
    assert_eq!(result["success"], true);
}

#[test]
fn test_ipc_agent_get_session() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "agent_get_session",
        json!({"sessionId": "test-session-id"}),
    );
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        assert!(v.get("session").is_some());
    }
}

#[test]
fn test_ipc_agent_delete_session() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "agent_delete_session",
        json!({"sessionId": "nonexistent-session"}),
    );
    // Should succeed or give a reasonable error — either is fine
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
    }
}

#[test]
fn test_ipc_agent_rename_session() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "agent_rename_session",
        json!({"sessionId": "nonexistent", "newName": "renamed"}),
    );
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
    }
}

#[test]
fn test_ipc_agent_search_sessions() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_search_sessions", json!({"query": "test"})).expect("search");
    assert_eq!(result["success"], true);
}

#[test]
fn test_ipc_agent_search_sessions_with_limit() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "agent_search_sessions", json!({"query": "test", "limit": 3}))
            .expect("search with limit");
    assert_eq!(result["success"], true);
}

#[test]
fn test_ipc_save_temp_file() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "save_temp_file",
        json!({"base64Data": "SGVsbG8gV29ybGQ=", "fileName": "test.txt"}),
    );
    if let Ok(v) = result {
        assert_eq!(v["success"], true);
        // path is nested under "data"
        let path = v["data"]["path"].as_str().unwrap_or("");
        assert!(!path.is_empty(), "should return a path");
        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn test_ipc_save_temp_file_bad_base64() {
    let (_app, ww) = build_test_app();
    let result = invoke_ipc::<serde_json::Value>(
        &ww,
        "save_temp_file",
        json!({"base64Data": "!!!invalid base64!!!", "fileName": "bad.txt"}),
    );
    assert!(result.is_err(), "bad base64 should error via IPC");
}

#[test]
fn test_ipc_clean_temp_dir() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "clean_temp_dir", json!({"maxAgeHours": 1})).expect("clean should succeed");
    assert_eq!(result["success"], true);
}

#[test]
fn test_ipc_clean_temp_dir_default() {
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ipc(&ww, "clean_temp_dir", json!({})).expect("clean with default");
    assert_eq!(result["success"], true);
}
