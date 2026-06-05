//! Claw Session Compact — compress older messages to reduce context size.
//!
//! Loads the session JSON file from `~/.claw/sessions/<session_id>.json`,
//! runs `runtime::compact::compact_session()` to compress older messages,
//! saves the compacted session, and returns a summary.

use runtime::{compact_session, CompactionConfig, Session};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of a compact operation returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompactResult {
    pub session_id: String,
    pub removed_message_count: usize,
    pub summary: String,
}

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

/// Tauri command: compact a Claw session by summarizing older messages.
///
/// Reads the session file from disk, runs the runtime compaction algorithm,
/// writes the compacted session back, and returns a summary.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_chat_compact(session_id: String) -> Result<CompactResult, String> {
    let path = session_path(&session_id);

    if !path.exists() {
        return Err(format!(
            "Session file not found: {}",
            path.display()
        ));
    }

    log::info!(
        "[claw_compact] Loading session {} from {}",
        session_id,
        path.display()
    );

    // Load the session from disk
    let session = Session::load_from_path(&path)
        .map_err(|e| format!("Failed to load session: {e}"))?;

    let message_count_before = session.messages.len();
    if message_count_before == 0 {
        return Err("Session has no messages to compact".to_string());
    }

    log::info!(
        "[claw_compact] Session {} has {} messages before compaction",
        session_id,
        message_count_before
    );

    // Run compaction with default settings
    let config = CompactionConfig::default();
    let result = compact_session(&session, config);

    if result.removed_message_count == 0 {
        log::info!(
            "[claw_compact] Session {} is already compacted or too small to compact",
            session_id
        );
        return Ok(CompactResult {
            session_id,
            removed_message_count: 0,
            summary: String::new(),
        });
    }

    // Save the compacted session back to disk
    result
        .compacted_session
        .save_to_path(&path)
        .map_err(|e| format!("Failed to save compacted session: {e}"))?;

    log::info!(
        "[claw_compact] Compacted session {} — removed {} messages",
        session_id,
        result.removed_message_count
    );

    Ok(CompactResult {
        session_id,
        removed_message_count: result.removed_message_count,
        summary: result.summary,
    })
}
