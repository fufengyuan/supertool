//! Hermes Agent session/message database access
//!
//! Reads from Hermes state.db (~/.hermes/state.db)
//! Sessions table: id, source, model, title, started_at, ended_at, message_count, ...
//! Messages table: id, session_id, role, content, timestamp, tool_name, ...

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hermes session summary for list display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesSession {
    pub id: String,
    pub source: String,
    pub model: String,
    pub title: Option<String>,
    pub started_at: f64,
    pub ended_at: Option<f64>,
    pub message_count: i32,
    pub preview: String,
    pub last_active: f64,
}

/// Hermes message detail for conversation display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub timestamp: f64,
    pub finish_reason: Option<String>,
}

/// Get Hermes home directory (~/.hermes)
pub fn get_hermes_home() -> PathBuf {
    // Respect HERMES_HOME env var if set (for profile support)
    if let Ok(home) = std::env::var("HERMES_HOME") {
        PathBuf::from(home)
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".hermes")
    }
}

/// Get Hermes state.db path
pub fn get_hermes_state_db_path() -> PathBuf {
    get_hermes_home().join("state.db")
}

/// Check if Hermes is installed (state.db exists)
pub fn hermes_is_installed() -> bool {
    get_hermes_state_db_path().exists()
}

/// List Hermes sessions with preview
///
/// Query similar to Hermes's `list_sessions_rich`:
/// - Exclude child sessions (parent_session_id IS NULL)
/// - Include preview (first 60 chars of first user message)
/// - Include last_active (timestamp of last message)
/// - Order by started_at DESC
pub fn list_hermes_sessions(limit: i32, offset: i32) -> Result<Vec<HermesSession>, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    // Query sessions with preview and last_active
    // Similar to hermes_state.py list_sessions_rich
    let query = r#"
        SELECT 
            s.id,
            s.source,
            s.model,
            s.title,
            s.started_at,
            s.ended_at,
            s.message_count,
            COALESCE(
                (SELECT SUBSTR(REPLACE(REPLACE(m.content, X'0A', ' '), X'0D', ' '), 1, 63)
                 FROM messages m
                 WHERE m.session_id = s.id AND m.role = 'user' AND m.content IS NOT NULL
                 ORDER BY m.timestamp, m.id LIMIT 1),
                ''
            ) AS preview_raw,
            COALESCE(
                (SELECT MAX(m2.timestamp) FROM messages m2 WHERE m2.session_id = s.id),
                s.started_at
            ) AS last_active
        FROM sessions s
        WHERE s.parent_session_id IS NULL
        ORDER BY s.started_at DESC
        LIMIT ? OFFSET ?
    "#;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("查询会话失败: {}", e))?;

    let sessions = stmt
        .query_map([limit, offset], |row| {
            let raw_preview: String = row.get(7)?;
            let preview = if raw_preview.is_empty() {
                String::new()
            } else {
                let text = raw_preview.trim();
                if text.len() > 60 {
                    format!("{}...", &text[..60])
                } else {
                    text.to_string()
                }
            };

            Ok(HermesSession {
                id: row.get(0)?,
                source: row.get(1)?,
                model: row.get(2)?,
                title: row.get::<_, Option<String>>(3)?,
                started_at: row.get(4)?,
                ended_at: row.get::<_, Option<f64>>(5)?,
                message_count: row.get(6)?,
                preview,
                last_active: row.get(8)?,
            })
        })
        .map_err(|e| format!("读取会话失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析会话失败: {}", e))?;

    Ok(sessions)
}

/// Get Hermes session by ID
pub fn get_hermes_session(session_id: &str) -> Result<Option<HermesSession>, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    let query = r#"
        SELECT 
            s.id,
            s.source,
            s.model,
            s.title,
            s.started_at,
            s.ended_at,
            s.message_count,
            COALESCE(
                (SELECT SUBSTR(REPLACE(REPLACE(m.content, X'0A', ' '), X'0D', ' '), 1, 63)
                 FROM messages m
                 WHERE m.session_id = s.id AND m.role = 'user' AND m.content IS NOT NULL
                 ORDER BY m.timestamp, m.id LIMIT 1),
                ''
            ) AS preview_raw,
            COALESCE(
                (SELECT MAX(m2.timestamp) FROM messages m2 WHERE m2.session_id = s.id),
                s.started_at
            ) AS last_active
        FROM sessions s
        WHERE s.id = ?
    "#;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("查询会话失败: {}", e))?;

    let result = stmt
        .query_row([session_id], |row| {
            let raw_preview: String = row.get(7)?;
            let preview = if raw_preview.is_empty() {
                String::new()
            } else {
                let text = raw_preview.trim();
                if text.len() > 60 {
                    format!("{}...", &text[..60])
                } else {
                    text.to_string()
                }
            };

            Ok(HermesSession {
                id: row.get(0)?,
                source: row.get(1)?,
                model: row.get(2)?,
                title: row.get::<_, Option<String>>(3)?,
                started_at: row.get(4)?,
                ended_at: row.get::<_, Option<f64>>(5)?,
                message_count: row.get(6)?,
                preview,
                last_active: row.get(8)?,
            })
        });

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("读取会话失败: {}", e)),
    }
}

/// List Hermes messages for a session
///
/// Order by timestamp, include all roles: user, assistant, tool, system
pub fn list_hermes_messages(session_id: &str) -> Result<Vec<HermesMessage>, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    let query = r#"
        SELECT 
            id,
            session_id,
            role,
            content,
            tool_name,
            tool_call_id,
            timestamp,
            finish_reason
        FROM messages
        WHERE session_id = ?
        ORDER BY timestamp, id
    "#;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("查询消息失败: {}", e))?;

    let messages = stmt
        .query_map([session_id], |row| {
            Ok(HermesMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get::<_, Option<String>>(3)?,
                tool_name: row.get::<_, Option<String>>(4)?,
                tool_call_id: row.get::<_, Option<String>>(5)?,
                timestamp: row.get(6)?,
                finish_reason: row.get::<_, Option<String>>(7)?,
            })
        })
        .map_err(|e| format!("读取消息失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析消息失败: {}", e))?;

    Ok(messages)
}

/// Get Hermes session statistics
pub fn get_hermes_stats() -> Result<HermesStats, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    // Total sessions (excluding children)
    let total_sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE parent_session_id IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("统计会话失败: {}", e))?;

    // Total messages
    let total_messages: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .map_err(|e| format!("统计消息失败: {}", e))?;

    // Sources breakdown
    let sources: Vec<(String, i64)> = conn
        .prepare("SELECT source, COUNT(*) FROM sessions WHERE parent_session_id IS NULL GROUP BY source ORDER BY COUNT(*) DESC")
        .map_err(|e| format!("统计来源失败: {}", e))?
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| format!("读取来源失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析来源失败: {}", e))?;

    Ok(HermesStats {
        total_sessions,
        total_messages,
        sources,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesStats {
    pub total_sessions: i64,
    pub total_messages: i64,
    pub sources: Vec<(String, i64)>,
}

/// Delete a Hermes session
pub fn delete_hermes_session(session_id: &str) -> Result<(), String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    // Delete messages first
    conn.execute("DELETE FROM messages WHERE session_id = ?", [session_id])
        .map_err(|e| format!("删除消息失败: {}", e))?;

    // Delete session
    conn.execute("DELETE FROM sessions WHERE id = ?", [session_id])
        .map_err(|e| format!("删除会话失败: {}", e))?;

    Ok(())
}

/// Format timestamp (Unix epoch float) to human-readable string
pub fn format_hermes_timestamp(ts: f64) -> String {
    use chrono::{DateTime, Utc};

    // Hermes uses Unix epoch seconds (float)
    let secs = ts as i64;
    let nanos = ((ts - secs as f64) * 1_000_000_000.0) as u32;

    if let Some(dt) = DateTime::from_timestamp(secs, nanos) {
        let now = Utc::now();
        let diff = now.signed_duration_since(dt);

        if diff.num_minutes() < 1 {
            "刚刚".to_string()
        } else if diff.num_hours() < 1 {
            format!("{} 分钟前", diff.num_minutes())
        } else if diff.num_days() < 1 {
            format!("{} 小时前", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{} 天前", diff.num_days())
        } else if diff.num_weeks() < 4 {
            format!("{} 周前", diff.num_weeks())
        } else {
            dt.format("%Y-%m-%d %H:%M").to_string()
        }
    } else {
        "未知时间".to_string()
    }
}