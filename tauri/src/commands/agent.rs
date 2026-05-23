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
    count_hermes_sessions, delete_hermes_session, get_compression_tip, get_hermes_stats,
    hermes_is_installed, list_hermes_messages, list_hermes_sessions, rename_hermes_session,
    search_hermes_sessions,
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

/// Clean temp files older than max_age_hours (default: 24h) in ~/.supertool/tmp/
#[tauri::command(rename_all = "camelCase")]
pub fn clean_temp_dir(max_age_hours: Option<u64>) -> Result<serde_json::Value, String> {
    let temp_dir = supertool_core::logic::data_dir::tmp_dir();
    if !temp_dir.exists() {
        return Ok(json!({ "success": true, "deleted": 0 }));
    }

    let max_age = max_age_hours.unwrap_or(24);
    let now = std::time::SystemTime::now();
    let mut deleted = 0u64;

    for entry in fs::read_dir(&temp_dir).map_err(|e| format!("读取目录失败: {e}"))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = now.duration_since(modified) {
                        if duration.as_secs() > max_age * 3600 {
                            let _ = fs::remove_file(&path);
                            deleted += 1;
                            log::info!("[clean_temp_dir] deleted old temp file: {:?}", path);
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "success": true,
        "deleted": deleted,
        "maxAgeHours": max_age
    }))
}

// ============================================================================
// Unit Tests - Simulating frontend IPC calls
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test agent_installed command - simulates frontend invoke("agentInstalled")
    #[test]
    fn test_agent_installed() {
        let result = agent_installed();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["success"], true);
        assert!(json["installed"].is_boolean());
    }

    /// Test agent_list_messages command - simulates frontend invoke("agentListMessages", { sessionId })
    #[test]
    fn test_agent_list_messages() {
        let result = agent_list_messages("test-session-id".to_string());
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["messages"].is_array());
            assert_eq!(json["sessionId"], "test-session-id");
        }
    }

    /// Test agent_get_stats command - simulates frontend invoke("agentGetStats")
    #[test]
    fn test_agent_get_stats() {
        let result = agent_get_stats();
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["stats"].is_object());
        }
    }

    /// Test all commands return JSON with success field
    #[test]
    fn test_all_commands_return_json_with_success() {
        let tests: Vec<(String, Result<serde_json::Value, String>)> = vec![
            ("agent_installed".to_string(), agent_installed()),
            (
                "agent_list_messages".to_string(),
                agent_list_messages("test".to_string()),
            ),
            ("agent_get_stats".to_string(), agent_get_stats()),
        ];

        for (name, result) in tests {
            if let Ok(json) = result {
                assert_eq!(
                    json["success"], true,
                    "Command {} should have success=true",
                    name
                );
            }
        }
    }

    /// Test camelCase response keys
    #[test]
    fn test_camel_case_response_keys() {
        let json = agent_installed().unwrap();
        assert!(json.get("success").is_some());

        let messages_json = agent_list_messages("test".to_string());
        if let Ok(json) = messages_json {
            assert!(json.get("sessionId").is_some());
        }
    }
}
