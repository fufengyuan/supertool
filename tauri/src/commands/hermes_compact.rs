//! Hermes Session Compact — compress older messages via claw-runtime compact_session.
//!
//! Reads messages from state.db (SQLite), runs the runtime compaction algorithm,
//! writes compacted messages back to state.db, and returns a summary.
//! This is the Hermes equivalent of `claw_chat_compact` (which works with
//! Claw's individual JSON session files under ~/.claw/sessions/).

use runtime::{
    compact_session, CompactionConfig, ContentBlock, ConversationMessage, MessageRole, Session,
};
use serde::{Deserialize, Serialize};

/// Result of a compact operation returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompactResult {
    pub session_id: String,
    pub removed_message_count: usize,
    pub summary: String,
}

/// Tauri command: compact a Hermes session by summarizing older messages.
///
/// Loads messages from state.db, runs compact_session from claw-runtime,
/// deletes old messages and inserts the compacted messages back.
#[tauri::command(rename_all = "camelCase")]
pub fn hermes_chat_compact(session_id: String) -> Result<CompactResult, String> {
    // 1. Load Hermes messages from state.db
    let hermes_messages = supertool_core::db::agent::list_hermes_messages(&session_id)
        .map_err(|e| format!("加载会话失败: {e}"))?;

    if hermes_messages.is_empty() {
        return Err("会话没有消息".to_string());
    }

    log::info!(
        "[hermes_compact] Loaded {} messages for session {}",
        hermes_messages.len(),
        session_id
    );

    // 2. Convert HermesMessage -> claw-runtime ConversationMessage
    //    (compact_session works with the claw-runtime type)
    let conv_messages: Vec<ConversationMessage> = hermes_messages
        .iter()
        .map(|hm| {
            let role = match hm.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "tool" => MessageRole::Tool,
                _ => MessageRole::System,
            };
            let text = hm.content.clone().unwrap_or_default();
            ConversationMessage {
                role,
                blocks: vec![ContentBlock::Text { text }],
                usage: None,
            }
        })
        .collect();

    // 3. Build a Session for compact_session
    let mut session = Session::new();
    session.session_id = session_id.clone();
    session.messages = conv_messages;

    // 4. Run compaction with default config
    let config = CompactionConfig::default();
    let result = compact_session(&session, config);

    if result.removed_message_count == 0 {
        log::info!(
            "[hermes_compact] Session {} already compact or too small",
            session_id
        );
        return Ok(CompactResult {
            session_id,
            removed_message_count: 0,
            summary: String::new(),
        });
    }

    // 5. Write compacted session back to state.db
    //
    // compact_session output:
    //   compacted_session.messages = [System summary, ...preserved original messages]
    //   preserved = session.messages[keep_from..]
    //   keep_from = hermes_messages.len() - preserved_count
    let preserved_count = result.compacted_session.messages.len().saturating_sub(1);
    let keep_from = hermes_messages.len().saturating_sub(preserved_count);

    log::info!(
        "[hermes_compact] Removing {} messages, preserving {} of {}",
        result.removed_message_count,
        preserved_count,
        hermes_messages.len()
    );

    // Delete all old messages for this session
    supertool_core::db::agent::delete_hermes_messages(&session_id)
        .map_err(|e| format!("删除旧消息失败: {e}"))?;

    // Insert the new system summary (first message in compacted session)
    if let Some(summary_msg) = result.compacted_session.messages.first() {
        if let ContentBlock::Text { text } = &summary_msg.blocks[0] {
            // Use raw SQL to insert with all fields, including timestamp
            // so ordering is preserved correctly
            let db_path = supertool_core::db::agent::get_hermes_state_db_path();
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                let _ = conn.execute(
                    "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![&session_id, "system", text, now],
                );
            }
        }
    }

    // Insert preserved original messages (keep their original field values)
    // Use direct SQL to preserve all fields including timestamps, reasoning, etc.
    let db_path = supertool_core::db::agent::get_hermes_state_db_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        for hm in hermes_messages.iter().skip(keep_from) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            let _ = conn.execute(
                "INSERT INTO messages (session_id, role, content, tool_name, tool_call_id, tool_calls, timestamp, finish_reason, reasoning, reasoning_content) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    &session_id,
                    hm.role,
                    hm.content,
                    hm.tool_name,
                    hm.tool_call_id,
                    hm.tool_calls,
                    now, // new timestamp to maintain chronological order
                    hm.finish_reason,
                    hm.reasoning,
                    hm.reasoning_content,
                ],
            );
        }

        // Update message_count on the sessions table
        let new_count = 1 + preserved_count; // system summary + preserved
        let _ = conn.execute(
            "UPDATE sessions SET message_count = ?1 WHERE id = ?2",
            rusqlite::params![new_count as i64, &session_id],
        );
    }

    log::info!(
        "[hermes_compact] Session {} compacted — removed {} messages",
        session_id,
        result.removed_message_count
    );

    Ok(CompactResult {
        session_id,
        removed_message_count: result.removed_message_count,
        summary: result.summary,
    })
}
