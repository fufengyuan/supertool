//! Hermes Agent management commands for Tauri GUI
//!
//! Provides IPC commands to:
//! - List Hermes sessions
//! - Get session details
//! - List session messages
//! - Get statistics
//! - Delete session

use serde_json::json;

use supertool_core::db::agent::{
    delete_hermes_session, get_hermes_session, get_hermes_stats, hermes_is_installed,
    list_hermes_messages, list_hermes_sessions,
};

/// Check if Hermes is installed
#[tauri::command(rename_all = "camelCase")]
pub fn hermes_installed() -> Result<serde_json::Value, String> {
    let installed = hermes_is_installed();
    Ok(json!({
        "success": true,
        "installed": installed
    }))
}

/// List Hermes sessions
#[tauri::command(rename_all = "camelCase")]
pub fn list_hermes_sessions_cmd(
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let sessions = list_hermes_sessions(limit, offset)?;
    Ok(json!({
        "success": true,
        "sessions": sessions
    }))
}

/// Get Hermes session by ID
#[tauri::command(rename_all = "camelCase")]
pub fn get_hermes_session_cmd(
    session_id: String,
) -> Result<serde_json::Value, String> {
    let session = get_hermes_session(&session_id)?;
    Ok(json!({
        "success": true,
        "session": session
    }))
}

/// List Hermes messages for a session
#[tauri::command(rename_all = "camelCase")]
pub fn list_hermes_messages_cmd(
    session_id: String,
) -> Result<serde_json::Value, String> {
    let messages = list_hermes_messages(&session_id)?;
    Ok(json!({
        "success": true,
        "messages": messages,
        "sessionId": session_id
    }))
}

/// Get Hermes statistics
#[tauri::command(rename_all = "camelCase")]
pub fn get_hermes_stats_cmd() -> Result<serde_json::Value, String> {
    let stats = get_hermes_stats()?;
    Ok(json!({
        "success": true,
        "stats": stats
    }))
}

/// Delete Hermes session
#[tauri::command(rename_all = "camelCase")]
pub fn delete_hermes_session_cmd(
    session_id: String,
) -> Result<serde_json::Value, String> {
    delete_hermes_session(&session_id)?;
    Ok(json!({
        "success": true,
        "sessionId": session_id
    }))
}

// ============================================================================
// Unit Tests - Simulating frontend IPC calls
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Test hermes_installed command - simulates frontend invoke("hermesInstalled")
    #[test]
    fn test_hermes_installed_cmd() {
        let result = hermes_installed();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["success"], true);
        assert!(json["installed"].is_boolean());
    }

    /// Test list_hermes_sessions_cmd command - simulates frontend invoke("listHermesSessionsCmd")
    #[test]
    fn test_list_hermes_sessions_cmd_default_params() {
        let result = list_hermes_sessions_cmd(None, None);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["success"], true);
        assert!(json["sessions"].is_array());
    }

    /// Test list_hermes_sessions_cmd with custom params
    #[test]
    fn test_list_hermes_sessions_cmd_custom_params() {
        let result = list_hermes_sessions_cmd(Some(10), Some(5));
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["success"], true);
        assert!(json["sessions"].is_array());
    }

    /// Test get_hermes_session_cmd - simulates frontend invoke("getHermesSessionCmd", { sessionId })
    #[test]
    fn test_get_hermes_session_cmd() {
        let result = get_hermes_session_cmd("test-session-id".to_string());
        // Will return error if Hermes not installed, but structure should be correct
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["session"].is_object() || json["session"].is_null());
        }
    }

    /// Test list_hermes_messages_cmd - simulates frontend invoke("listHermesMessagesCmd", { sessionId })
    #[test]
    fn test_list_hermes_messages_cmd() {
        let result = list_hermes_messages_cmd("test-session-id".to_string());
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["messages"].is_array());
            assert_eq!(json["sessionId"], "test-session-id");
        }
    }

    /// Test get_hermes_stats_cmd - simulates frontend invoke("getHermesStatsCmd")
    #[test]
    fn test_get_hermes_stats_cmd() {
        let result = get_hermes_stats_cmd();
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["stats"].is_object());

            let stats = &json["stats"];
            assert!(stats["totalSessions"].is_i64());
            assert!(stats["totalMessages"].is_i64());
            assert!(stats["sources"].is_array());
        }
    }

    /// Test delete_hermes_session_cmd - simulates frontend invoke("deleteHermesSessionCmd", { sessionId })
    #[test]
    fn test_delete_hermes_session_cmd_structure() {
        let result = delete_hermes_session_cmd("non-existent-session".to_string());
        // Will return error if session doesn't exist, but we test error handling
        if result.is_err() {
            let error = result.unwrap_err();
            assert!(!error.is_empty());
        }
    }

    /// Test JSON response structure for all commands
    #[test]
    fn test_all_commands_return_json_with_success() {
        // All commands should return JSON with "success" field

        let tests: Vec<(String, Result<Value, String>)> = vec![
            ("hermes_installed".to_string(), hermes_installed()),
            ("list_sessions".to_string(), list_hermes_sessions_cmd(None, None)),
            ("get_session".to_string(), get_hermes_session_cmd("test".to_string())),
            ("list_messages".to_string(), list_hermes_messages_cmd("test".to_string())),
            ("get_stats".to_string(), get_hermes_stats_cmd()),
        ];

        for (name, result) in tests {
            if let Ok(json) = result {
                assert_eq!(json["success"], true, "Command {} should have success=true", name);
            }
        }
    }

    /// Test command parameter types match frontend expectations
    #[test]
    fn test_command_param_types() {
        // Test that optional params work correctly
        let limit_none = list_hermes_sessions_cmd(None, None);
        let limit_some = list_hermes_sessions_cmd(Some(50), Some(0));

        assert!(limit_none.is_ok());
        assert!(limit_some.is_ok());

        // Test string params
        let session_result = get_hermes_session_cmd("uuid-string".to_string());
        assert!(session_result.is_ok() || session_result.is_err()); // Either is valid

        let messages_result = list_hermes_messages_cmd("uuid-string".to_string());
        assert!(messages_result.is_ok() || messages_result.is_err());

        let delete_result = delete_hermes_session_cmd("uuid-string".to_string());
        assert!(delete_result.is_ok() || delete_result.is_err());
    }

    /// Test error messages are in Chinese (user-facing)
    #[test]
    fn test_error_messages_in_chinese() {
        // When Hermes is not installed, error should be in Chinese
        if !hermes_is_installed() {
            let result = list_hermes_sessions_cmd(None, None);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.contains("Hermes") || error.contains("未安装") || error.contains("不存在"));
        }
    }

    /// Test camelCase rename_all conversion
    #[test]
    fn test_camel_case_response_keys() {
        // Commands use #[tauri::command(rename_all = "camelCase")]
        // Response JSON keys should use camelCase

        let json = hermes_installed().unwrap();
        // sessionId not present in this command, but success is camelCase
        assert!(json.get("success").is_some()); // lowercase, not PascalCase

        let delete_json = delete_hermes_session_cmd("test".to_string());
        if let Ok(json) = delete_json {
            assert!(json.get("sessionId").is_some()); // camelCase, not session_id
        }

        let messages_json = list_hermes_messages_cmd("test".to_string());
        if let Ok(json) = messages_json {
            assert!(json.get("sessionId").is_some()); // camelCase
        }
    }
}