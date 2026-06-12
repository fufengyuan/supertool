//! Hermes Session Compact — compress older messages to reduce context size.
//!
//! Reads messages from state.db (SQLite), runs local compaction,
//! writes compacted messages back to state.db, and returns a summary.
//!
//! Completely independent from Claw — no claw-runtime types used.

use serde::{Deserialize, Serialize};
use supertool_core::db::agent;

/// Threshold: compact when messages exceed this count.
const COMPACT_THRESHOLD: usize = 15;
/// Keep this many recent messages after compaction.
const KEEP_RECENT: usize = 8;

/// Result of a compact operation returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompactResult {
    pub session_id: String,
    pub removed_message_count: usize,
    pub summary: String,
}

/// Build a local summary describing which messages were compacted.
fn build_summary(messages: &[agent::HermesMessage]) -> String {
    use std::collections::BTreeMap;

    let user_count = messages.iter().filter(|m| m.role == "user").count();
    let assistant_count = messages.iter().filter(|m| m.role == "assistant").count();
    let tool_count = messages.iter().filter(|m| m.role == "tool").count();

    // Collect unique tool names used in the compacted portion
    let mut tool_names = BTreeMap::new();
    for msg in messages {
        if let Some(ref tc) = msg.tool_calls {
            if let Ok(calls) = serde_json::from_str::<Vec<serde_json::Value>>(tc) {
                for call in calls {
                    if let Some(name) = call.get("name").and_then(|n| n.as_str()) {
                        *tool_names.entry(name.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let mut summary = format!(
        "Compacted {} earlier messages ({} user, {} assistant, {} tool messages)",
        messages.len(),
        user_count,
        assistant_count,
        tool_count,
    );

    if !tool_names.is_empty() {
        let tools: Vec<String> = tool_names
            .iter()
            .map(|(name, count)| format!("{} (×{})", name, count))
            .collect();
        summary.push_str(&format!("\nTools used: {}", tools.join(", ")));
    }

    summary
}

/// Tauri command: compact a Hermes session by summarizing older messages.
///
/// Loads messages from state.db, removes older ones beyond the retention
/// threshold, inserts a system summary message, and preserves recent messages.
#[tauri::command(rename_all = "camelCase")]
pub fn hermes_chat_compact(session_id: String) -> Result<CompactResult, String> {
    // 1. Load messages from state.db
    let hermes_messages = agent::list_hermes_messages(&session_id)
        .map_err(|e| format!("加载会话失败: {e}"))?;
    if hermes_messages.is_empty() {
        return Err("会话没有消息".to_string());
    }

    // 2. Check if compaction is needed
    if hermes_messages.len() <= COMPACT_THRESHOLD {
        log::info!(
            "[hermes_compact] Session {} has {} messages (threshold: {}), skipping",
            session_id,
            hermes_messages.len(),
            COMPACT_THRESHOLD,
        );
        return Ok(CompactResult {
            session_id,
            removed_message_count: 0,
            summary: String::new(),
        });
    }

    let keep_from = hermes_messages.len().saturating_sub(KEEP_RECENT);
    let removed_count = keep_from;

    // Build summary of compacted messages
    let summary = build_summary(&hermes_messages[..keep_from]);

    log::info!(
        "[hermes_compact] Session {} — compacting {} messages, keeping last {}",
        session_id,
        removed_count,
        KEEP_RECENT,
    );

    // 3. Write compacted session back to state.db
    // Use direct raw SQL to preserve all original fields (tool_calls, reasoning, etc.)
    let db_path = agent::get_hermes_state_db_path();

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 state.db: {e}"))?;

    // Begin transaction for atomicity
    conn.execute("BEGIN", [])
        .map_err(|e| format!("事务开始失败: {e}"))?;

    // Delete all old messages for this session
    conn.execute(
        "DELETE FROM messages WHERE session_id = ?1",
        rusqlite::params![&session_id],
    )
    .map_err(|e| format!("删除旧消息失败: {e}"))?;

    // Insert the system summary as the first message
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    conn.execute(
        "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, 'system', ?2, ?3)",
        rusqlite::params![&session_id, &summary, now],
    )
    .map_err(|e| format!("插入摘要消息失败: {e}"))?;

    // Insert preserved messages (from keep_from onwards) with all original fields
    for msg in hermes_messages.iter().skip(keep_from) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_name, tool_call_id, tool_calls, timestamp, finish_reason, reasoning, reasoning_content) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &session_id,
                msg.role,
                msg.content,
                msg.tool_name,
                msg.tool_call_id,
                msg.tool_calls,
                ts,
                msg.finish_reason,
                msg.reasoning,
                msg.reasoning_content,
            ],
        )
        .map_err(|e| format!("插入保留消息失败: {e}"))?;
    }

    // Update message_count on the sessions table
    let new_count = 1 + hermes_messages.len().saturating_sub(keep_from);
    conn.execute(
        "UPDATE sessions SET message_count = ?1 WHERE id = ?2",
        rusqlite::params![new_count as i64, &session_id],
    )
    .map_err(|e| format!("更新消息计数失败: {e}"))?;

    conn.execute("COMMIT", [])
        .map_err(|e| format!("事务提交失败: {e}"))?;

    log::info!(
        "[hermes_compact] Session {} compacted — removed {} messages",
        session_id,
        removed_count,
    );

    Ok(CompactResult {
        session_id,
        removed_message_count: removed_count,
        summary,
    })
}
