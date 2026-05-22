//! Hermes Agent session/message database access
//!
//! Reads from Hermes state.db (~/.hermes/state.db)
//! Sessions table: id, source, model, title, started_at, ended_at, message_count, ...
//! Messages table: id, session_id, role, content, timestamp, tool_name, ...

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hermes session summary for list display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
}

/// Hermes message detail for conversation display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<String>, // JSON string, parse in frontend
    pub timestamp: f64,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Whether this message belongs to a child session (subagent)
    #[serde(default)]
    pub is_child: bool,
}

/// Get Hermes home directory (~/.hermes)
pub fn get_hermes_home() -> PathBuf {
    // Respect HERMES_HOME env var if set (for profile support)
    if let Ok(home) = std::env::var("HERMES_HOME") {
        PathBuf::from(home)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".hermes")
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
    // Only show parent sessions (parent_session_id IS NULL), child sessions are embedded in parent's dialog
    // Similar to hermes_state.py list_sessions_rich with include_children=False
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
            ) AS last_active,
            s.parent_session_id
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
                // Use chars() to properly handle UTF-8 characters
                if text.chars().count() > 60 {
                    format!("{}...", text.chars().take(60).collect::<String>())
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
                parent_session_id: row.get::<_, Option<String>>(9)?,
            })
        })
        .map_err(|e| format!("读取会话失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析会话失败: {}", e))?;

    Ok(sessions)
}

/// Count total Hermes sessions (excluding child sessions)
pub fn count_hermes_sessions() -> Result<i64, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Ok(0);
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE parent_session_id IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(count)
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
            s.parent_session_id,
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

    let result = stmt.query_row([session_id], |row| {
        let raw_preview: String = row.get(8)?;
        let preview = if raw_preview.is_empty() {
            String::new()
        } else {
            let text = raw_preview.trim();
            // Use chars() to properly handle UTF-8 characters
            if text.chars().count() > 60 {
                format!("{}...", text.chars().take(60).collect::<String>())
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
            parent_session_id: row.get::<_, Option<String>>(7)?,
            preview,
            last_active: row.get(9)?,
        })
    });

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("读取会话失败: {}", e)),
    }
}

/// List Hermes messages for a session (including child sessions)
///
/// Order by timestamp, include all roles: user, assistant, tool, system
/// For parent sessions, also include messages from all child sessions (subagent)
pub fn list_hermes_messages(session_id: &str) -> Result<Vec<HermesMessage>, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    // Unified query: include messages from this session and any child sessions
    // is_child is 1 if message belongs to a child session, 0 otherwise
    let query = r#"
        SELECT 
            m.id,
            m.session_id,
            m.role,
            m.content,
            m.tool_name,
            m.tool_call_id,
            m.tool_calls,
            m.timestamp,
            m.finish_reason,
            m.reasoning,
            m.reasoning_content,
            CASE WHEN m.session_id != ? THEN 1 ELSE 0 END as is_child
        FROM messages m
        WHERE m.session_id = ? OR m.session_id IN (
            SELECT id FROM sessions WHERE parent_session_id = ?
        )
        ORDER BY m.timestamp, m.id
    "#;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("查询消息失败: {}", e))?;

    let messages = stmt
        .query_map([session_id, session_id, session_id], |row| {
            Ok(HermesMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get::<_, Option<String>>(3)?,
                tool_name: row.get::<_, Option<String>>(4)?,
                tool_call_id: row.get::<_, Option<String>>(5)?,
                tool_calls: row.get::<_, Option<String>>(6)?,
                timestamp: row.get(7)?,
                finish_reason: row.get::<_, Option<String>>(8)?,
                reasoning: row.get::<_, Option<String>>(9)?,
                reasoning_content: row.get::<_, Option<String>>(10)?,
                is_child: row.get::<_, i32>(11)? != 0,
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
#[serde(rename_all = "camelCase")]
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

/// Rename a Hermes session (update title)
pub fn rename_hermes_session(session_id: &str, new_title: &str) -> Result<(), String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    conn.execute(
        "UPDATE sessions SET title = ? WHERE id = ?",
        [new_title, session_id],
    )
    .map_err(|e| format!("重命名会话失败: {}", e))?;

    Ok(())
}

/// Search Hermes sessions by keyword (title or preview)
pub fn search_hermes_sessions(keyword: &str, limit: i32) -> Result<Vec<HermesSession>, String> {
    let db_path = get_hermes_state_db_path();
    if !db_path.exists() {
        return Err("Hermes 未安装或 state.db 不存在".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("无法打开 Hermes state.db: {}", e))?;

    let pattern = format!("%{}%", keyword);
    // Only search parent sessions, child sessions are embedded in parent's dialog
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
            ) AS last_active,
            s.parent_session_id
        FROM sessions s
        WHERE s.parent_session_id IS NULL
          AND (s.title LIKE ?1 OR s.id IN (
            SELECT DISTINCT session_id FROM messages 
            WHERE content LIKE ?1
        ))
        ORDER BY s.started_at DESC
        LIMIT ?2
    "#;

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("查询会话失败: {}", e))?;

    let sessions = stmt
        .query_map([&pattern, &limit.to_string()], |row| {
            let raw_preview: String = row.get(7)?;
            let preview = if raw_preview.is_empty() {
                String::new()
            } else {
                let text = raw_preview.trim();
                if text.chars().count() > 60 {
                    format!("{}...", text.chars().take(60).collect::<String>())
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
                parent_session_id: row.get::<_, Option<String>>(9)?,
            })
        })
        .map_err(|e| format!("读取会话失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析会话失败: {}", e))?;

    Ok(sessions)
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

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a test database with the same schema as Hermes state.db
    /// Returns (Connection, TempDir) - keep TempDir alive to prevent directory deletion
    fn create_test_db() -> (Connection, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = dir.path().join("state.db");
        let conn = Connection::open(&db_path).expect("Failed to create test DB");

        // Create sessions table
        conn.execute(
            r#"CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                model TEXT NOT NULL,
                title TEXT,
                started_at REAL NOT NULL,
                ended_at REAL,
                message_count INTEGER DEFAULT 0,
                parent_session_id TEXT
            )"#,
            [],
        )
        .expect("Failed to create sessions table");

        // Create messages table
        conn.execute(
            r#"CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT,
                tool_name TEXT,
                tool_call_id TEXT,
                tool_calls TEXT,
                timestamp REAL NOT NULL,
                finish_reason TEXT,
                reasoning TEXT,
                reasoning_content TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )"#,
            [],
        )
        .expect("Failed to create messages table");

        (conn, dir)
    }

    /// Insert a test session
    fn insert_test_session(
        conn: &Connection,
        id: &str,
        source: &str,
        model: &str,
        title: Option<&str>,
        started_at: f64,
        message_count: i32,
        parent_session_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO sessions (id, source, model, title, started_at, message_count, parent_session_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![id, source, model, title, started_at, message_count, parent_session_id],
        )
        .expect("Failed to insert test session");
    }

    /// Insert a test message
    fn insert_test_message(
        conn: &Connection,
        session_id: &str,
        role: &str,
        content: Option<&str>,
        timestamp: f64,
    ) {
        conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?, ?, ?, ?)",
            rusqlite::params![session_id, role, content, timestamp],
        )
        .expect("Failed to insert test message");
    }

    #[test]
    fn test_format_hermes_timestamp() {
        use chrono::Utc;

        // Just now (< 1 minute)
        let now = Utc::now().timestamp() as f64;
        assert_eq!(format_hermes_timestamp(now), "刚刚");

        // 5 minutes ago
        let five_min_ago = (Utc::now().timestamp() - 300) as f64;
        assert!(format_hermes_timestamp(five_min_ago).contains("分钟前"));

        // 2 hours ago
        let two_hours_ago = (Utc::now().timestamp() - 7200) as f64;
        assert!(format_hermes_timestamp(two_hours_ago).contains("小时前"));

        // 3 days ago
        let three_days_ago = (Utc::now().timestamp() - 259200) as f64;
        assert!(format_hermes_timestamp(three_days_ago).contains("天前"));
    }

    #[test]
    fn test_hermes_session_struct() {
        let session = HermesSession {
            id: "test-session-123".to_string(),
            source: "cli".to_string(),
            model: "claude-3".to_string(),
            title: Some("Test Session".to_string()),
            started_at: 1700000000.0,
            ended_at: Some(1700003600.0),
            message_count: 5,
            preview: "Hello world...".to_string(),
            last_active: 1700003600.0,
        };

        assert_eq!(session.id, "test-session-123");
        assert_eq!(session.source, "cli");
        assert_eq!(session.model, "claude-3");
        assert_eq!(session.title, Some("Test Session".to_string()));
        assert_eq!(session.message_count, 5);
    }

    #[test]
    fn test_hermes_message_struct() {
        let message = HermesMessage {
            id: 1,
            session_id: "test-session".to_string(),
            role: "user".to_string(),
            content: Some("Hello".to_string()),
            tool_name: None,
            tool_call_id: None,
            tool_calls: None,
            timestamp: 1700000000.0,
            finish_reason: None,
            reasoning: None,
            reasoning_content: None,
        };

        assert_eq!(message.id, 1);
        assert_eq!(message.session_id, "test-session");
        assert_eq!(message.role, "user");
        assert_eq!(message.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_hermes_stats_struct() {
        let stats = HermesStats {
            total_sessions: 10,
            total_messages: 50,
            sources: vec![
                ("cli".to_string(), 5),
                ("telegram".to_string(), 3),
                ("web".to_string(), 2),
            ],
        };

        assert_eq!(stats.total_sessions, 10);
        assert_eq!(stats.total_messages, 50);
        assert_eq!(stats.sources.len(), 3);
    }

    #[test]
    fn test_list_sessions_mock() {
        let (conn, db_path) = create_test_db();

        // Insert test data
        insert_test_session(
            &conn,
            "session-1",
            "cli",
            "claude-3",
            Some("First Session"),
            1700000000.0,
            3,
            None,
        );
        insert_test_session(
            &conn,
            "session-2",
            "telegram",
            "gpt-4",
            None,
            1700100000.0,
            2,
            None,
        );
        insert_test_session(
            &conn,
            "child-session",
            "cli",
            "claude-3",
            None,
            1700200000.0,
            1,
            Some("session-1"),
        ); // Should be excluded

        insert_test_message(
            &conn,
            "session-1",
            "user",
            Some("Hello world this is a test message"),
            1700000100.0,
        );
        insert_test_message(
            &conn,
            "session-1",
            "assistant",
            Some("Hi there!"),
            1700000200.0,
        );
        insert_test_message(
            &conn,
            "session-2",
            "user",
            Some("Another message"),
            1700100100.0,
        );

        // Test the query logic directly
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
            LIMIT 50 OFFSET 0
        "#;

        let mut stmt = conn.prepare(query).expect("Failed to prepare query");
        let sessions: Vec<HermesSession> = stmt
            .query_map([], |row| {
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
            .expect("Failed to query sessions")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect sessions");

        // Should only have 2 sessions (child excluded)
        assert_eq!(sessions.len(), 2);

        // Most recent first (session-2 has higher started_at)
        assert_eq!(sessions[0].id, "session-2");
        assert_eq!(sessions[1].id, "session-1");

        // Check preview
        assert!(sessions[1].preview.contains("Hello world"));

        // conn and db_path will be auto-dropped at end of test
    }

    #[test]
    fn test_get_session_mock() {
        let (conn, _dir) = create_test_db();

        insert_test_session(
            &conn,
            "test-session-id",
            "web",
            "gpt-4o",
            Some("Test Title"),
            1700000000.0,
            1,
            None,
        );
        insert_test_message(
            &conn,
            "test-session-id",
            "user",
            Some("Test content"),
            1700000100.0,
        );

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

        let mut stmt = conn.prepare(query).expect("Failed to prepare");
        let result: Result<HermesSession, _> = stmt.query_row(["test-session-id"], |row| {
            let raw_preview: String = row.get(7)?;
            Ok(HermesSession {
                id: row.get(0)?,
                source: row.get(1)?,
                model: row.get(2)?,
                title: row.get::<_, Option<String>>(3)?,
                started_at: row.get(4)?,
                ended_at: row.get::<_, Option<f64>>(5)?,
                message_count: row.get(6)?,
                preview: raw_preview.trim().to_string(),
                last_active: row.get(8)?,
            })
        });

        let session = result.expect("Failed to get session");
        assert_eq!(session.id, "test-session-id");
        assert_eq!(session.source, "web");
        assert_eq!(session.model, "gpt-4o");
        assert_eq!(session.title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_list_messages_mock() {
        let (conn, _dir) = create_test_db();

        insert_test_session(
            &conn,
            "msg-test",
            "cli",
            "claude-3",
            None,
            1700000000.0,
            2,
            None,
        );
        insert_test_message(&conn, "msg-test", "user", Some("Hello"), 1700000100.0);
        insert_test_message(
            &conn,
            "msg-test",
            "assistant",
            Some("Hi there!"),
            1700000200.0,
        );
        insert_test_message(&conn, "msg-test", "tool", None, 1700000250.0);

        let query = r#"
            SELECT 
                id,
                session_id,
                role,
                content,
                tool_name,
                tool_call_id,
                tool_calls,
                timestamp,
                finish_reason,
                reasoning,
                reasoning_content
            FROM messages
            WHERE session_id = ?
            ORDER BY timestamp, id
        "#;

        let mut stmt = conn.prepare(query).expect("Failed to prepare");
        let messages: Vec<HermesMessage> = stmt
            .query_map(["msg-test"], |row| {
                Ok(HermesMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get::<_, Option<String>>(3)?,
                    tool_name: row.get::<_, Option<String>>(4)?,
                    tool_call_id: row.get::<_, Option<String>>(5)?,
                    tool_calls: row.get::<_, Option<String>>(6)?,
                    timestamp: row.get(7)?,
                    finish_reason: row.get::<_, Option<String>>(8)?,
                    reasoning: row.get::<_, Option<String>>(9)?,
                    reasoning_content: row.get::<_, Option<String>>(10)?,
                })
            })
            .expect("Failed to query messages")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect messages");

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, Some("Hello".to_string()));
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "tool");
        assert_eq!(messages[2].content, None);
    }

    #[test]
    fn test_get_stats_mock() {
        let (conn, _dir) = create_test_db();

        insert_test_session(&conn, "s1", "cli", "claude-3", None, 1700000000.0, 2, None);
        insert_test_session(&conn, "s2", "cli", "gpt-4", None, 1700100000.0, 1, None);
        insert_test_session(
            &conn,
            "s3",
            "telegram",
            "gpt-4",
            None,
            1700200000.0,
            1,
            None,
        );
        insert_test_session(&conn, "s4", "web", "claude-3", None, 1700300000.0, 1, None);

        insert_test_message(&conn, "s1", "user", Some("m1"), 1700000100.0);
        insert_test_message(&conn, "s1", "assistant", Some("m2"), 1700000200.0);
        insert_test_message(&conn, "s2", "user", Some("m3"), 1700100100.0);
        insert_test_message(&conn, "s3", "user", Some("m4"), 1700200100.0);
        insert_test_message(&conn, "s4", "user", Some("m5"), 1700300100.0);

        // Total sessions
        let total_sessions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE parent_session_id IS NULL",
                [],
                |row| row.get(0),
            )
            .expect("Failed to count sessions");
        assert_eq!(total_sessions, 4);

        // Total messages
        let total_messages: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .expect("Failed to count messages");
        assert_eq!(total_messages, 5);

        // Sources breakdown
        let sources: Vec<(String, i64)> = conn
            .prepare("SELECT source, COUNT(*) FROM sessions WHERE parent_session_id IS NULL GROUP BY source ORDER BY COUNT(*) DESC")
            .expect("Failed to prepare")
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
            .expect("Failed to query")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect");

        assert_eq!(sources.len(), 3); // cli, telegram, web
        assert!(sources.iter().any(|(s, c)| s == "cli" && *c == 2));
        assert!(sources.iter().any(|(s, c)| s == "telegram" && *c == 1));
        assert!(sources.iter().any(|(s, c)| s == "web" && *c == 1));
    }

    #[test]
    fn test_delete_session_mock() {
        let (conn, _dir) = create_test_db();

        insert_test_session(
            &conn,
            "to-delete",
            "cli",
            "claude-3",
            None,
            1700000000.0,
            1,
            None,
        );
        insert_test_message(
            &conn,
            "to-delete",
            "user",
            Some("Will be deleted"),
            1700000100.0,
        );

        // Verify exists
        let count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = 'to-delete'",
                [],
                |row| row.get(0),
            )
            .expect("Failed");
        assert_eq!(count_before, 1);

        let msg_count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = 'to-delete'",
                [],
                |row| row.get(0),
            )
            .expect("Failed");
        assert_eq!(msg_count_before, 1);

        // Delete
        conn.execute("DELETE FROM messages WHERE session_id = ?", ["to-delete"])
            .expect("Failed to delete messages");
        conn.execute("DELETE FROM sessions WHERE id = ?", ["to-delete"])
            .expect("Failed to delete session");

        // Verify deleted
        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = 'to-delete'",
                [],
                |row| row.get(0),
            )
            .expect("Failed");
        assert_eq!(count_after, 0);

        let msg_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = 'to-delete'",
                [],
                |row| row.get(0),
            )
            .expect("Failed");
        assert_eq!(msg_count_after, 0);
    }
}
