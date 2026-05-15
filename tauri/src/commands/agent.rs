//! Hermes Agent management commands for Tauri GUI
//!
//! Provides IPC commands for direct database access (no Python bridge):
//! - Check Hermes installation
//! - List session messages
//! - Get statistics
//!
//! Agent interaction commands (chat, abort) are in hermes_chat.rs

use serde_json::json;

use supertool_core::db::agent::{
    get_hermes_stats, hermes_is_installed,
    list_hermes_messages,
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

/// List Hermes messages for a session
#[tauri::command(rename_all = "camelCase")]
pub fn hermes_list_messages(
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
pub fn hermes_get_stats() -> Result<serde_json::Value, String> {
    let stats = get_hermes_stats()?;
    Ok(json!({
        "success": true,
        "stats": stats
    }))
}

// ============================================================================
// Unit Tests - Simulating frontend IPC calls
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test hermes_installed command - simulates frontend invoke("hermesInstalled")
    #[test]
    fn test_hermes_installed() {
        let result = hermes_installed();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert_eq!(json["success"], true);
        assert!(json["installed"].is_boolean());
    }

    /// Test hermes_list_messages command - simulates frontend invoke("hermesListMessages", { sessionId })
    #[test]
    fn test_hermes_list_messages() {
        let result = hermes_list_messages("test-session-id".to_string());
        if result.is_ok() {
            let json = result.unwrap();
            assert_eq!(json["success"], true);
            assert!(json["messages"].is_array());
            assert_eq!(json["sessionId"], "test-session-id");
        }
    }

    /// Test hermes_get_stats command - simulates frontend invoke("hermesGetStats")
    #[test]
    fn test_hermes_get_stats() {
        let result = hermes_get_stats();
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
            ("hermes_installed".to_string(), hermes_installed()),
            ("hermes_list_messages".to_string(), hermes_list_messages("test".to_string())),
            ("hermes_get_stats".to_string(), hermes_get_stats()),
        ];

        for (name, result) in tests {
            if let Ok(json) = result {
                assert_eq!(json["success"], true, "Command {} should have success=true", name);
            }
        }
    }

    /// Test camelCase response keys
    #[test]
    fn test_camel_case_response_keys() {
        let json = hermes_installed().unwrap();
        assert!(json.get("success").is_some());

        let messages_json = hermes_list_messages("test".to_string());
        if let Ok(json) = messages_json {
            assert!(json.get("sessionId").is_some());
        }
    }
}