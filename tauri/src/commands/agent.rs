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