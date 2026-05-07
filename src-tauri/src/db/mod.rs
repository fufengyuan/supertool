use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod database;
pub mod projects;
pub mod servers;
pub mod cicd;
pub mod cicd_tables;
pub mod openvpn;
pub mod wireguard;
pub mod lan;
pub use cicd::*;

/// Initialize SQLite database with all required tables
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '#6366f1',
            repoPath TEXT,
            branch TEXT,
            repoPath2 TEXT,
            branch2 TEXT,
            gitUrl1 TEXT,
            gitUrl2 TEXT,
            category TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            archived INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            host TEXT NOT NULL DEFAULT '',
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL DEFAULT '',
            sshKeyPath TEXT,
            password TEXT,
            description TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '',
            groupId TEXT,
            requiresApproval INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS server_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            parentId TEXT,
            color TEXT NOT NULL DEFAULT '#6c63ff',
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL DEFAULT '',
            completed INTEGER NOT NULL DEFAULT 0,
            priority TEXT NOT NULL DEFAULT 'medium',
            dueDate TEXT,
            description TEXT NOT NULL DEFAULT '',
            markdownDescription TEXT,
            tag TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            completedAt TEXT,
            assignedTo TEXT,
            assignedBy TEXT,
            assignedAt TEXT,
            owner TEXT,
            orderNum INTEGER NOT NULL DEFAULT 0,
            repeatType TEXT,
            repeatInterval INTEGER,
            repeatEndDate TEXT,
            repeatCount INTEGER NOT NULL DEFAULT 0,
            parentTodoId TEXT,
            projectId TEXT
        );

        CREATE TABLE IF NOT EXISTS subtasks (
            id TEXT PRIMARY KEY,
            todoId TEXT NOT NULL,
            text TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            completed INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS mfa_secrets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            secret TEXT NOT NULL DEFAULT '',
            issuer TEXT,
            digits INTEGER NOT NULL DEFAULT 6,
            period INTEGER NOT NULL DEFAULT 30,
            algorithm TEXT NOT NULL DEFAULT 'SHA1',
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            groupId TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS note_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS weekly_reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            weekStart TEXT NOT NULL,
            weekEnd TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS accounting_categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            type TEXT NOT NULL DEFAULT 'expense',
            icon TEXT NOT NULL DEFAULT '',
            sortOrder INTEGER DEFAULT 0,
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS accounting_records (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'expense',
            category TEXT NOT NULL DEFAULT '',
            amount REAL NOT NULL DEFAULT 0,
            description TEXT DEFAULT '',
            status TEXT DEFAULT 'completed',
            attachmentPath TEXT,
            createdBy TEXT DEFAULT '',
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL,
            voucher_number TEXT DEFAULT '',
            receipt_type TEXT DEFAULT '',
            receipt_path TEXT DEFAULT '',
            entity TEXT DEFAULT '',
            project TEXT DEFAULT '',
            supplier TEXT DEFAULT '',
            invoice_number TEXT DEFAULT '',
            tax_amount REAL,
            payment_method TEXT DEFAULT '',
            approver TEXT DEFAULT '',
            attachments_json TEXT DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS budgets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            "limit" REAL NOT NULL DEFAULT 0,
            period TEXT NOT NULL DEFAULT 'monthly'
        );

        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS log_presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            serverIds TEXT NOT NULL DEFAULT '[]',
            logPath TEXT NOT NULL DEFAULT '',
            logType TEXT NOT NULL DEFAULT 'file',
            maxLines INTEGER NOT NULL DEFAULT 100,
            presetGroup TEXT,
            keywords TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS openvpn_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            filePath TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS wireguard_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            privateKey TEXT NOT NULL DEFAULT '',
            publicKey TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '10.0.0.2/32',
            dns TEXT,
            mtu INTEGER DEFAULT 1420,
            peerPublicKey TEXT NOT NULL DEFAULT '',
            peerEndpoint TEXT NOT NULL DEFAULT '',
            peerAllowedIPs TEXT NOT NULL DEFAULT '0.0.0.0/0',
            peerPersistentKeepalive INTEGER DEFAULT 25,
            presharedKey TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            createdAt TEXT NOT NULL
        );

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
            content TEXT NOT NULL DEFAULT '',
            type TEXT NOT NULL DEFAULT 'text',
            createdAt TEXT NOT NULL,
            read INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            fromUserId TEXT NOT NULL,
            fromUserName TEXT NOT NULL,
            toUserId TEXT NOT NULL,
            toUserName TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            type TEXT NOT NULL DEFAULT 'text',
            fileName TEXT,
            fileSize INTEGER,
            filePath TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            progress REAL DEFAULT 0,
            createdAt TEXT NOT NULL,
            read INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS file_transfers (
            id TEXT PRIMARY KEY,
            fromUserId TEXT NOT NULL,
            fromUserName TEXT NOT NULL,
            toUserId TEXT NOT NULL,
            toUserName TEXT NOT NULL,
            fileName TEXT NOT NULL DEFAULT '',
            fileSize INTEGER,
            filePath TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            progress REAL DEFAULT 0,
            createdAt TEXT NOT NULL,
            completedAt TEXT
        );

        CREATE TABLE IF NOT EXISTS calculator_history (
            id TEXT PRIMARY KEY,
            expression TEXT NOT NULL DEFAULT '',
            result TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS api_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            method TEXT NOT NULL DEFAULT 'GET',
            url TEXT NOT NULL DEFAULT '',
            headers TEXT NOT NULL DEFAULT '{}',
            body TEXT,
            statusCode INTEGER,
            responseTime INTEGER,
            createdAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS git_repos (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL DEFAULT '',
            remote TEXT,
            branch TEXT,
            lastCommit TEXT,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        "#,
    )?;
    // Migration: add keywords column for databases created before v3.1.9
    let _ = conn.execute(
        "ALTER TABLE log_presets ADD COLUMN keywords TEXT NOT NULL DEFAULT '[]'",
        [],
    );
    cicd_tables::init_cicd_tables(conn)?;
    lan::init_lan_tables(conn)?;
    Ok(())
}

/// Database wrapper with a rusqlite Connection
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;
            "#,
        )?;
        init_db(&conn)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

// =================== Shared Types ===================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    #[serde(rename = "repoPath")]
    pub repo_path: Option<String>,
    pub branch: Option<String>,
    #[serde(rename = "repoPath2")]
    pub repo_path2: Option<String>,
    pub branch2: Option<String>,
    #[serde(rename = "gitUrl1")]
    pub git_url1: Option<String>,
    #[serde(rename = "gitUrl2")]
    pub git_url2: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub archived: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    #[serde(rename = "sshKeyPath")]
    pub ssh_key_path: Option<String>,
    pub password: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "requiresApproval")]
    pub requires_approval: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub color: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectStats {
    pub total: i64,
    pub completed: i64,
    pub progress: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}
