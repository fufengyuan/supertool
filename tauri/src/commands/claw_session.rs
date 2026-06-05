//! Claw Session management — set model, fork session
//!
//! Uses the public `runtime::Session` API for session operations.

use runtime::Session;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn claw_sessions_dir() -> PathBuf {
    let config_home = std::env::var_os("CLAW_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".claw")))
        .unwrap_or_else(|| PathBuf::from("~/.claw"));
    config_home.join("sessions")
}

fn session_path(id: &str) -> PathBuf {
    claw_sessions_dir().join(format!("{id}.json"))
}

/// Result of a fork operation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForkResult {
    pub new_session_id: String,
    pub branch_name: Option<String>,
}

/// Tauri command: set the model on an existing session.
///
/// Loads the session from disk, updates its model field, saves back.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_chat_set_model(session_id: String, model: String) -> Result<(), String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err(format!("Session file not found: {}", path.display()));
    }

    let mut session =
        Session::load_from_path(&path).map_err(|e| format!("Failed to load session: {e}"))?;

    session.model = if model.is_empty() { None } else { Some(model.clone()) };
    session.updated_at_ms = current_time_millis();

    session
        .save_to_path(&path)
        .map_err(|e| format!("Failed to save session: {e}"))?;

    log::info!(
        "[claw_session] Set model for session {} to {:?}",
        session_id,
        session.model
    );

    Ok(())
}

/// Tauri command: fork a session into a new branch.
///
/// Loads the session, clones it with a new ID, optionally sets a branch name,
/// and saves both files.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_chat_fork(session_id: String, branch_name: Option<String>) -> Result<ForkResult, String> {
    let path = session_path(&session_id);
    if !path.exists() {
        return Err(format!("Session file not found: {}", path.display()));
    }

    let session =
        Session::load_from_path(&path).map_err(|e| format!("Failed to load session: {e}"))?;

    let new_id = uuid::Uuid::new_v4().to_string();
    let now_ms = current_time_millis();

    let mut forked = session.clone();
    forked.session_id = new_id.clone();
    forked.created_at_ms = now_ms;
    forked.updated_at_ms = now_ms;
    forked.fork = Some(runtime::SessionFork {
        parent_session_id: session_id.clone(),
        branch_name: branch_name.clone(),
    });

    let fork_path = session_path(&new_id);
    forked
        .save_to_path(&fork_path)
        .map_err(|e| format!("Failed to save forked session: {e}"))?;

    log::info!(
        "[claw_session] Forked session {} → {} (branch: {:?})",
        session_id,
        new_id,
        branch_name
    );

    Ok(ForkResult {
        new_session_id: new_id,
        branch_name,
    })
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
