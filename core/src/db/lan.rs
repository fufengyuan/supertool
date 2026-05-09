use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// =================== LAN Tables ===================

pub fn init_lan_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER NOT NULL,
            lastSeen TEXT NOT NULL,
            isOnline INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            fromUserId TEXT NOT NULL,
            fromUserName TEXT NOT NULL,
            toUserId TEXT NOT NULL,
            toUserName TEXT NOT NULL,
            content TEXT NOT NULL,
            type TEXT DEFAULT 'text',
            createdAt TEXT NOT NULL,
            read INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            fromUserId TEXT NOT NULL,
            fromUserName TEXT NOT NULL,
            toUserId TEXT NOT NULL,
            toUserName TEXT NOT NULL,
            content TEXT,
            type TEXT DEFAULT 'text',
            fileName TEXT,
            fileSize INTEGER,
            filePath TEXT,
            status TEXT DEFAULT 'sent',
            progress INTEGER DEFAULT 0,
            createdAt TEXT NOT NULL,
            read INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS file_transfers (
            id TEXT PRIMARY KEY,
            fromUserId TEXT NOT NULL,
            fromUserName TEXT NOT NULL,
            toUserId TEXT NOT NULL,
            toUserName TEXT NOT NULL,
            fileName TEXT NOT NULL,
            fileSize INTEGER NOT NULL,
            filePath TEXT,
            status TEXT DEFAULT 'pending',
            progress INTEGER DEFAULT 0,
            createdAt TEXT NOT NULL,
            completedAt TEXT,
            localUserId TEXT
        );
        "#,
    )
}

// =================== Types ===================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanUser {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: i64,
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
    #[serde(rename = "isOnline")]
    pub is_online: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanMessage {
    pub id: String,
    #[serde(rename = "fromUserId")]
    pub from_user_id: String,
    #[serde(rename = "fromUserName")]
    pub from_user_name: String,
    #[serde(rename = "toUserId")]
    pub to_user_id: String,
    #[serde(rename = "toUserName")]
    pub to_user_name: String,
    pub content: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    #[serde(rename = "fromUserId")]
    pub from_user_id: String,
    #[serde(rename = "fromUserName")]
    pub from_user_name: String,
    #[serde(rename = "toUserId")]
    pub to_user_id: String,
    #[serde(rename = "toUserName")]
    pub to_user_name: String,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<i64>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub status: String,
    pub progress: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileTransfer {
    pub id: String,
    #[serde(rename = "fromUserId")]
    pub from_user_id: String,
    #[serde(rename = "fromUserName")]
    pub from_user_name: String,
    #[serde(rename = "toUserId")]
    pub to_user_id: String,
    #[serde(rename = "toUserName")]
    pub to_user_name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub status: String,
    pub progress: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(rename = "localUserId")]
    pub local_user_id: Option<String>,
}

// =================== CRUD ===================

// --- Users ---
pub fn get_all_users(conn: &Connection) -> Result<Vec<LanUser>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY lastSeen DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(LanUser {
            id: row.get("id")?,
            name: row.get("name")?,
            ip: row.get("ip")?,
            port: row.get("port")?,
            last_seen: row.get("lastSeen")?,
            is_online: row.get::<_, i64>("isOnline")? == 1,
        })
    })?;
    rows.collect()
}

#[allow(dead_code)]
pub fn insert_user(conn: &Connection, user: &LanUser) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO users (id, name, ip, port, lastSeen, isOnline) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![user.id, user.name, user.ip, user.port, user.last_seen, if user.is_online { 1 } else { 0 }],
    )?;
    Ok(())
}

// --- Messages ---
pub fn get_all_messages(conn: &Connection) -> Result<Vec<LanMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM messages ORDER BY createdAt DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(LanMessage {
            id: row.get("id")?,
            from_user_id: row.get("fromUserId")?,
            from_user_name: row.get("fromUserName")?,
            to_user_id: row.get("toUserId")?,
            to_user_name: row.get("toUserName")?,
            content: row.get("content")?,
            msg_type: row.get("type")?,
            created_at: row.get("createdAt")?,
            read: row.get::<_, i64>("read")? == 1,
        })
    })?;
    rows.collect()
}

#[allow(dead_code)]
pub fn insert_message(conn: &Connection, msg: &LanMessage) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![msg.id, msg.from_user_id, msg.from_user_name, msg.to_user_id, msg.to_user_name, msg.content, msg.msg_type, msg.created_at, if msg.read { 1 } else { 0 }],
    )?;
    Ok(())
}

// --- Chat Messages ---
pub fn get_all_chat_messages(conn: &Connection) -> Result<Vec<ChatMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM chat_messages ORDER BY createdAt DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(ChatMessage {
            id: row.get("id")?,
            from_user_id: row.get("fromUserId")?,
            from_user_name: row.get("fromUserName")?,
            to_user_id: row.get("toUserId")?,
            to_user_name: row.get("toUserName")?,
            content: row.get("content")?,
            msg_type: row.get("type")?,
            file_name: row.get("fileName")?,
            file_size: row.get("fileSize")?,
            file_path: row.get("filePath")?,
            status: row.get("status")?,
            progress: row.get("progress")?,
            created_at: row.get("createdAt")?,
            read: row.get::<_, i64>("read")? == 1,
        })
    })?;
    rows.collect()
}

pub fn insert_chat_message(conn: &Connection, msg: &ChatMessage) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, fileName, fileSize, filePath, status, progress, createdAt, read) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![msg.id, msg.from_user_id, msg.from_user_name, msg.to_user_id, msg.to_user_name, msg.content, msg.msg_type, msg.file_name, msg.file_size, msg.file_path, msg.status, msg.progress, msg.created_at, if msg.read { 1 } else { 0 }],
    )?;
    Ok(())
}

// --- File Transfers ---
pub fn get_all_file_transfers(conn: &Connection) -> Result<Vec<FileTransfer>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM file_transfers ORDER BY createdAt DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(FileTransfer {
            id: row.get("id")?,
            from_user_id: row.get("fromUserId")?,
            from_user_name: row.get("fromUserName")?,
            to_user_id: row.get("toUserId")?,
            to_user_name: row.get("toUserName")?,
            file_name: row.get("fileName")?,
            file_size: row.get("fileSize")?,
            file_path: row.get("filePath")?,
            status: row.get("status")?,
            progress: row.get("progress")?,
            created_at: row.get("createdAt")?,
            completed_at: row.get("completedAt")?,
            local_user_id: row.get("localUserId")?,
        })
    })?;
    rows.collect()
}

pub fn insert_file_transfer(conn: &Connection, ft: &FileTransfer) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, completedAt, localUserId) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![ft.id, ft.from_user_id, ft.from_user_name, ft.to_user_id, ft.to_user_name, ft.file_name, ft.file_size, ft.file_path, ft.status, ft.progress, ft.created_at, ft.completed_at, ft.local_user_id],
    )?;
    Ok(())
}

pub fn update_file_transfer(
    conn: &Connection,
    file_id: &str,
    status: &str,
    progress: i64,
    file_path: Option<&str>,
    completed_at: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE file_transfers SET status = ?2, progress = ?3, filePath = ?4, completedAt = ?5 WHERE id = ?1",
        params![file_id, status, progress, file_path, completed_at],
    )?;
    Ok(())
}

// =================== Query Functions for Commands ===================

/// Get messages between two users (both directions), paginated
pub fn get_messages_between(
    conn: &Connection,
    user1: &str,
    user2: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<ChatMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT * FROM chat_messages \
         WHERE (fromUserId = ?1 AND toUserId = ?2) OR (fromUserId = ?2 AND toUserId = ?1) \
         ORDER BY createdAt DESC \
         LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt.query_map(
        params![user1, user2, limit as i64, offset as i64],
        |row| {
            Ok(ChatMessage {
                id: row.get("id")?,
                from_user_id: row.get("fromUserId")?,
                from_user_name: row.get("fromUserName")?,
                to_user_id: row.get("toUserId")?,
                to_user_name: row.get("toUserName")?,
                content: row.get("content")?,
                msg_type: row.get("type")?,
                file_name: row.get("fileName")?,
                file_size: row.get("fileSize")?,
                file_path: row.get("filePath")?,
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("createdAt")?,
                read: row.get::<_, i64>("read")? == 1,
            })
        },
    )?;
    rows.collect()
}

/// Get messages between two users with LEFT JOIN to file_transfers for file details
#[allow(dead_code)]
pub fn get_messages_with_file_transfers(
    conn: &Connection,
    user1: &str,
    user2: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<ChatMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT cm.* FROM chat_messages cm \
         LEFT JOIN file_transfers ft ON cm.fileName = ft.fileName AND cm.fromUserId = ft.fromUserId \
         WHERE (cm.fromUserId = ?1 AND cm.toUserId = ?2) OR (cm.fromUserId = ?2 AND cm.toUserId = ?1) \
         ORDER BY cm.createdAt DESC \
         LIMIT ?3 OFFSET ?4",
    )?;
    let rows = stmt.query_map(
        params![user1, user2, limit as i64, offset as i64],
        |row| {
            Ok(ChatMessage {
                id: row.get("id")?,
                from_user_id: row.get("fromUserId")?,
                from_user_name: row.get("fromUserName")?,
                to_user_id: row.get("toUserId")?,
                to_user_name: row.get("toUserName")?,
                content: row.get("content")?,
                msg_type: row.get("type")?,
                file_name: row.get("fileName")?,
                file_size: row.get("fileSize")?,
                file_path: row.get("filePath")?,
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("createdAt")?,
                read: row.get::<_, i64>("read")? == 1,
            })
        },
    )?;
    rows.collect()
}

/// Mark all messages from peerId to myUserId as read
pub fn mark_messages_read(
    conn: &Connection,
    my_user_id: &str,
    peer_id: &str,
) -> Result<usize, rusqlite::Error> {
    let changed = conn.execute(
        "UPDATE chat_messages SET read = 1 WHERE fromUserId = ?1 AND toUserId = ?2 AND read = 0",
        params![peer_id, my_user_id],
    )?;
    Ok(changed)
}

/// Get unread message count from a specific peer
pub fn get_unread_count(
    conn: &Connection,
    my_user_id: &str,
    peer_id: &str,
) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM chat_messages WHERE fromUserId = ?1 AND toUserId = ?2 AND read = 0",
        params![peer_id, my_user_id],
        |row| row.get(0),
    )
}

/// Get unread counts from all peers (returns Vec of (peer_id, peer_name, count))
pub fn get_all_unread_counts(
    conn: &Connection,
    my_user_id: &str,
) -> Result<Vec<(String, String, i64)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT fromUserId, fromUserName, COUNT(*) as cnt \
         FROM chat_messages \
         WHERE toUserId = ?1 AND read = 0 \
         GROUP BY fromUserId \
         ORDER BY cnt DESC",
    )?;
    let rows = stmt.query_map(params![my_user_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    rows.collect()
}

/// Get all chat messages for a user (sent or received), paginated
#[allow(dead_code)]
pub fn get_chat_messages_for_user(
    conn: &Connection,
    user_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<ChatMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT * FROM chat_messages \
         WHERE fromUserId = ?1 OR toUserId = ?1 \
         ORDER BY createdAt DESC \
         LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(
        params![user_id, limit as i64, offset as i64],
        |row| {
            Ok(ChatMessage {
                id: row.get("id")?,
                from_user_id: row.get("fromUserId")?,
                from_user_name: row.get("fromUserName")?,
                to_user_id: row.get("toUserId")?,
                to_user_name: row.get("toUserName")?,
                content: row.get("content")?,
                msg_type: row.get("type")?,
                file_name: row.get("fileName")?,
                file_size: row.get("fileSize")?,
                file_path: row.get("filePath")?,
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("createdAt")?,
                read: row.get::<_, i64>("read")? == 1,
            })
        },
    )?;
    rows.collect()
}

/// Get file transfers for a local user
#[allow(dead_code)]
pub fn get_file_transfers_for_user(
    conn: &Connection,
    local_user_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<FileTransfer>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT * FROM file_transfers \
         WHERE localUserId = ?1 \
         ORDER BY createdAt DESC \
         LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(
        params![local_user_id, limit as i64, offset as i64],
        |row| {
            Ok(FileTransfer {
                id: row.get("id")?,
                from_user_id: row.get("fromUserId")?,
                from_user_name: row.get("fromUserName")?,
                to_user_id: row.get("toUserId")?,
                to_user_name: row.get("toUserName")?,
                file_name: row.get("fileName")?,
                file_size: row.get("fileSize")?,
                file_path: row.get("filePath")?,
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("createdAt")?,
                completed_at: row.get("completedAt")?,
                local_user_id: row.get("localUserId")?,
            })
        },
    )?;
    rows.collect()
}
