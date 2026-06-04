//! Agent management commands for Tauri GUI
//!
//! Provides IPC commands for direct database access (no Python bridge):
//! - Check Agent installation
//! - List session messages
//! - Get statistics
//!
//! Agent interaction commands (chat, abort) are in agent_chat.rs

use serde_json::json;

use supertool_core::db::agent::{
    count_hermes_sessions, delete_hermes_session, get_compression_tip, get_hermes_session,
    get_hermes_stats, hermes_is_installed, list_hermes_messages, list_hermes_sessions,
    rename_hermes_session, search_hermes_sessions,
};

/// Check if Agent is installed
#[tauri::command(rename_all = "camelCase")]
pub fn agent_installed() -> Result<serde_json::Value, String> {
    let installed = hermes_is_installed();
    Ok(json!({
        "success": true,
        "installed": installed
    }))
}

/// Get the compression tip (latest continuation session) for a session
/// This resolves the correct session_id to use for chat after compression splits
#[tauri::command(rename_all = "camelCase")]
pub fn agent_get_compression_tip(session_id: String) -> Result<serde_json::Value, String> {
    let tip_id = get_compression_tip(&session_id)?;
    Ok(json!({
        "success": true,
        "tipSessionId": tip_id,
        "originalSessionId": session_id
    }))
}

/// List Agent messages for a session
#[tauri::command(rename_all = "camelCase")]
pub fn agent_list_messages(session_id: String) -> Result<serde_json::Value, String> {
    let messages = list_hermes_messages(&session_id)?;
    Ok(json!({
        "success": true,
        "messages": messages,
        "sessionId": session_id
    }))
}

/// Get Agent statistics
#[tauri::command(rename_all = "camelCase")]
pub fn agent_get_stats() -> Result<serde_json::Value, String> {
    let stats = get_hermes_stats()?;
    Ok(json!({
        "success": true,
        "stats": stats
    }))
}

/// List Agent sessions (direct SQLite access, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub fn agent_list_sessions(limit: Option<i32>) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(50);
    let sessions = list_hermes_sessions(limit, 0)?;
    let total = count_hermes_sessions()?;
    Ok(json!({
        "success": true,
        "sessions": sessions,
        "total": total
    }))
}

/// Delete Agent session (direct SQLite access)
#[tauri::command(rename_all = "camelCase")]
pub fn agent_delete_session(session_id: String) -> Result<serde_json::Value, String> {
    delete_hermes_session(&session_id)?;
    Ok(json!({
        "success": true,
        "sessionId": session_id
    }))
}

/// Rename Agent session (direct SQLite access)
#[tauri::command(rename_all = "camelCase")]
pub fn agent_rename_session(
    session_id: String,
    new_title: String,
) -> Result<serde_json::Value, String> {
    rename_hermes_session(&session_id, &new_title)?;
    Ok(json!({
        "success": true,
        "sessionId": session_id,
        "newTitle": new_title
    }))
}

/// Get a single session with its messages (direct SQLite access)
#[tauri::command(rename_all = "camelCase")]
pub fn agent_get_session(session_id: String) -> Result<serde_json::Value, String> {
    let session = get_hermes_session(&session_id)?;
    let messages = list_hermes_messages(&session_id)?;
    Ok(json!({
        "success": true,
        "sessionId": session_id,
        "session": session,
        "messages": messages,
    }))
}

/// Search Agent sessions (direct SQLite access)
#[tauri::command(rename_all = "camelCase")]
pub fn agent_search_sessions(
    query: String,
    limit: Option<i32>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(50);
    let sessions = search_hermes_sessions(&query, limit)?;
    let total = sessions.len() as i64;
    Ok(json!({
        "success": true,
        "results": sessions,
        "total": total,
        "query": query
    }))
}

// ============================================================================
// Generic temp file utilities (used by Agent, LAN Chat, and any paste feature)
// ============================================================================

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::fs;

/// Save base64-encoded data as a temp file in ~/.supertool/tmp/
/// Returns the saved file path.
#[tauri::command(rename_all = "camelCase")]
pub fn save_temp_file(base64_data: String, file_name: String) -> Result<serde_json::Value, String> {
    let temp_dir = supertool_core::logic::data_dir::tmp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;

    let file_path = temp_dir.join(&file_name);
    let decoded = BASE64
        .decode(&base64_data)
        .map_err(|e| format!("Base64 解码失败: {e}"))?;
    fs::write(&file_path, decoded).map_err(|e| format!("写入文件失败: {e}"))?;

    Ok(json!({
        "success": true,
        "data": { "path": file_path.to_string_lossy().to_string() }
    }))
}

/// Recursively delete old files under a directory, respecting max_age.
/// Directories are traversed but never deleted.
fn clean_dir_recursive(dir: &std::path::Path, now: std::time::SystemTime, max_age_secs: u64) -> u64 {
    let mut deleted = 0u64;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            deleted += clean_dir_recursive(&path, now, max_age_secs);
        } else if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = now.duration_since(modified) {
                    if duration.as_secs() > max_age_secs {
                        let _ = fs::remove_file(&path);
                        deleted += 1;
                        log::info!("[clean_temp_dir] deleted old temp file: {:?}", path);
                    }
                }
            }
        }
    }
    deleted
}

/// Clean temp files older than max_age_hours (default: 24h) in ~/.supertool/tmp/
#[tauri::command(rename_all = "camelCase")]
pub fn clean_temp_dir(max_age_hours: Option<u64>) -> Result<serde_json::Value, String> {
    let temp_dir = supertool_core::logic::data_dir::tmp_dir();
    if !temp_dir.exists() {
        return Ok(json!({ "success": true, "deleted": 0 }));
    }

    let max_age = max_age_hours.unwrap_or(24);
    let now = std::time::SystemTime::now();
    let deleted = clean_dir_recursive(&temp_dir, now, max_age * 3600);

    Ok(json!({
        "success": true,
        "deleted": deleted,
        "maxAgeHours": max_age
    }))
}

// ============================================================================
// Unit Tests — IPC-style tests via tauri::test::get_ipc_response
// ============================================================================

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
        let result: Result<serde_json::Value, String> =
            invoke_ipc(&ww, "agent_get_stats", json!({}));
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert!(v["stats"].is_object());
        }
    }

    #[test]
    fn test_ipc_agent_list_sessions() {
        let (_app, ww) = build_test_app();
        let result: Result<serde_json::Value, String> =
            invoke_ipc(&ww, "agent_list_sessions", json!({}));
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert!(v["sessions"].is_array());
            assert!(v["total"].is_number());
        }
    }

    #[test]
    fn test_ipc_agent_list_sessions_with_limit() {
        let (_app, ww) = build_test_app();
        let result: Result<serde_json::Value, String> =
            invoke_ipc(&ww, "agent_list_sessions", json!({"limit": 5}));
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert!(v["sessions"].is_array());
            assert!(v["total"].is_number());
        }
    }

    #[test]
    fn test_ipc_agent_delete_session() {
        let (_app, ww) = build_test_app();
        let result = invoke_ipc::<serde_json::Value>(
            &ww,
            "agent_delete_session",
            json!({"sessionId": "test-session-id"}),
        );
        // May fail if session doesn't exist, but response should still parse
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert_eq!(v["sessionId"], "test-session-id");
        }
    }

    #[test]
    fn test_ipc_agent_rename_session() {
        let (_app, ww) = build_test_app();
        let result = invoke_ipc::<serde_json::Value>(
            &ww,
            "agent_rename_session",
            json!({"sessionId": "test-session-id", "newTitle": "Renamed Session"}),
        );
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert_eq!(v["sessionId"], "test-session-id");
            assert_eq!(v["newTitle"], "Renamed Session");
        }
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
            assert_eq!(v["sessionId"], "test-session-id");
        }
    }

    #[test]
    fn test_ipc_agent_search_sessions() {
        let (_app, ww) = build_test_app();
        let result = invoke_ipc::<serde_json::Value>(
            &ww,
            "agent_search_sessions",
            json!({"query": "test"}),
        );
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert!(v["results"].is_array());
            assert!(v["total"].is_number());
            assert_eq!(v["query"], "test");
        }
    }

    #[test]
    fn test_ipc_agent_search_sessions_with_limit() {
        let (_app, ww) = build_test_app();
        let result = invoke_ipc::<serde_json::Value>(
            &ww,
            "agent_search_sessions",
            json!({"query": "test", "limit": 3}),
        );
        if let Ok(v) = result {
            assert_eq!(v["success"], true);
            assert!(v["results"].is_array());
            assert!(v["total"].is_number());
            assert_eq!(v["query"], "test");
        }
    }

    #[test]
    fn test_ipc_save_temp_file() {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "save_temp_file",
            json!({"base64Data": "aGVsbG8gd29ybGQ=", "fileName": "test.txt"}),
        )
        .expect("save_temp_file should succeed");
        assert_eq!(result["success"], true);
        let data = result["data"]
            .as_object()
            .expect("data field should be an object");
        assert!(data.contains_key("path"), "data should contain path");
        let path = data["path"]
            .as_str()
            .expect("path should be a string");
        assert!(!path.is_empty(), "path should not be empty");
        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_ipc_save_temp_file_bad_base64() {
        let (_app, ww) = build_test_app();
        // Invalid base64 causes command to return Err(...), which Tauri
        // converts to an IPC error response (not a JSON with success=false)
        let result: Result<serde_json::Value, String> = invoke_ipc(
            &ww,
            "save_temp_file",
            json!({"base64Data": "!!!invalid!!!", "fileName": "bad.txt"}),
        );
        assert!(result.is_err(), "bad base64 should produce IPC error");
    }

    #[test]
    fn test_ipc_clean_temp_dir() {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "clean_temp_dir",
            json!({"maxAgeHours": 1}),
        )
        .expect("clean_temp_dir should succeed");
        assert_eq!(result["success"], true);
        assert!(result["deleted"].is_number());
        assert_eq!(result["maxAgeHours"], 1);
    }

    #[test]
    fn test_ipc_clean_temp_dir_default() {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value =
            invoke_ipc(&ww, "clean_temp_dir", json!({})).expect("clean_temp_dir should succeed");
        assert_eq!(result["success"], true);
        assert!(result["deleted"].is_number());
        assert_eq!(result["maxAgeHours"], 24);
    }
}
