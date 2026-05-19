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
    count_hermes_sessions, delete_hermes_session, get_hermes_stats, hermes_is_installed,
    list_hermes_messages, list_hermes_sessions, rename_hermes_session, search_hermes_sessions,
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
    keyword: String,
    limit: Option<i32>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(50);
    let sessions = search_hermes_sessions(&keyword, limit)?;
    Ok(json!({
        "success": true,
        "sessions": sessions,
        "keyword": keyword
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
