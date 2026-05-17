use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod agent;
pub mod alert;
pub mod cicd;
pub mod cicd_tables;
pub mod database;
pub mod git_repo;
pub mod lan;
pub mod nginx;
pub mod openvpn;
pub mod projects;
pub mod servers;
pub mod wireguard;
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
            name TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
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

        CREATE TABLE IF NOT EXISTS nginx_presets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT '',
            serverId TEXT NOT NULL,
            configPath TEXT NOT NULL DEFAULT '/etc/nginx/nginx.conf',
            description TEXT NOT NULL DEFAULT '',
            groupName TEXT NOT NULL DEFAULT '未分组',
            isActive INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL,
            updatedAt TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nginx_config_versions (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            checksum TEXT NOT NULL DEFAULT '',
            comment TEXT NOT NULL DEFAULT '',
            isCurrent INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL,
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_nginx_config_versions_preset ON nginx_config_versions(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_config_versions_current ON nginx_config_versions(presetId, isCurrent);
        CREATE INDEX IF NOT EXISTS idx_nginx_presets_server ON nginx_presets(serverId);

        -- Nginx structured management tables
        CREATE TABLE IF NOT EXISTS nginx_servers (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            proxyType INTEGER NOT NULL DEFAULT 0,
            listen TEXT NOT NULL DEFAULT '80',
            ip TEXT NOT NULL DEFAULT '',
            def INTEGER NOT NULL DEFAULT 0,
            ipv6 INTEGER NOT NULL DEFAULT 0,
            proxyProtocol INTEGER NOT NULL DEFAULT 0,
            serverName TEXT NOT NULL DEFAULT '',
            ssl INTEGER NOT NULL DEFAULT 0,
            certId TEXT NOT NULL DEFAULT '',
            rewrite INTEGER NOT NULL DEFAULT 0,
            rewriteListen TEXT NOT NULL DEFAULT '80',
            http2 INTEGER NOT NULL DEFAULT 0,
            protocols TEXT NOT NULL DEFAULT '',
            passwordId TEXT NOT NULL DEFAULT '',
            denyAllow INTEGER NOT NULL DEFAULT 0,
            denyId TEXT NOT NULL DEFAULT '',
            allowId TEXT NOT NULL DEFAULT '',
            proxyUpstreamId TEXT NOT NULL DEFAULT '',
            descr TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            sort INTEGER NOT NULL DEFAULT 0,
            paramJson TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            updatedAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_locations (
            id TEXT PRIMARY KEY,
            serverId TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            path TEXT NOT NULL DEFAULT '/',
            locType INTEGER NOT NULL DEFAULT 0,
            value TEXT NOT NULL DEFAULT '',
            upstreamType INTEGER NOT NULL DEFAULT 0,
            upstreamId TEXT NOT NULL DEFAULT '',
            upstreamPath TEXT NOT NULL DEFAULT '',
            rootPath TEXT NOT NULL DEFAULT '',
            rootPage TEXT NOT NULL DEFAULT '',
            rootType TEXT NOT NULL DEFAULT '',
            header INTEGER NOT NULL DEFAULT 0,
            websocket INTEGER NOT NULL DEFAULT 0,
            cros INTEGER NOT NULL DEFAULT 0,
            headerHost TEXT NOT NULL DEFAULT '',
            returnUrl TEXT NOT NULL DEFAULT '',
            returnPath INTEGER NOT NULL DEFAULT 0,
            paramJson TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            descr TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (serverId) REFERENCES nginx_servers(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_upstreams (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            name TEXT NOT NULL,
            proxyType INTEGER NOT NULL DEFAULT 0,
            strategy TEXT NOT NULL DEFAULT 'polling',
            descr TEXT NOT NULL DEFAULT '',
            paramJson TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            updatedAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_upstream_servers (
            id TEXT PRIMARY KEY,
            upstreamId TEXT NOT NULL,
            address TEXT NOT NULL,
            port INTEGER NOT NULL,
            weight INTEGER NOT NULL DEFAULT 1,
            maxFails INTEGER NOT NULL DEFAULT 3,
            failTimeout TEXT NOT NULL DEFAULT '10s',
            maxConns INTEGER NOT NULL DEFAULT 0,
            backup INTEGER NOT NULL DEFAULT 0,
            down INTEGER NOT NULL DEFAULT 0,
            sort INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            param TEXT NOT NULL DEFAULT '',
            FOREIGN KEY (upstreamId) REFERENCES nginx_upstreams(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_http_params (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_streams (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            listen TEXT NOT NULL DEFAULT '0.0.0.0:80',
            proxyUpstreamId TEXT NOT NULL DEFAULT '',
            proxyPass TEXT NOT NULL DEFAULT '',
            ssl INTEGER NOT NULL DEFAULT 0,
            certId TEXT NOT NULL DEFAULT '',
            protocol TEXT NOT NULL DEFAULT 'TCP',
            descr TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            paramJson TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            updatedAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_certs (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            name TEXT NOT NULL DEFAULT '',
            pem TEXT NOT NULL DEFAULT '',
            key TEXT NOT NULL DEFAULT '',
            domain TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_templates (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            name TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_basic_settings (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_nginx_servers_preset ON nginx_servers(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_locations_server ON nginx_locations(serverId);
        CREATE INDEX IF NOT EXISTS idx_nginx_upstreams_preset ON nginx_upstreams(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_upstream_servers_upstream ON nginx_upstream_servers(upstreamId);
        CREATE INDEX IF NOT EXISTS idx_nginx_http_params_preset ON nginx_http_params(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_streams_preset ON nginx_streams(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_certs_preset ON nginx_certs(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_templates_preset ON nginx_templates(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_basic_settings_preset ON nginx_basic_settings(presetId);

        CREATE TABLE IF NOT EXISTS nginx_params (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL,
            serverId TEXT NOT NULL DEFAULT '',
            locationId TEXT NOT NULL DEFAULT '',
            upstreamId TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL DEFAULT '',
            value TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL DEFAULT 0,
            templateValue TEXT NOT NULL DEFAULT '',
            sort INTEGER NOT NULL DEFAULT 0,
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_deny_allows (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            ip TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS nginx_passwords (
            id TEXT PRIMARY KEY,
            presetId TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            pass TEXT NOT NULL DEFAULT '',
            descr TEXT NOT NULL DEFAULT '',
            path TEXT NOT NULL DEFAULT '',
            createdAt TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (presetId) REFERENCES nginx_presets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_nginx_params_preset ON nginx_params(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_deny_allows_preset ON nginx_deny_allows(presetId);
        CREATE INDEX IF NOT EXISTS idx_nginx_passwords_preset ON nginx_passwords(presetId);

        CREATE TABLE IF NOT EXISTS alert_email_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            smtp_host TEXT,
            smtp_port INTEGER DEFAULT 465,
            smtp_username TEXT,
            smtp_password TEXT,
            smtp_encryption TEXT DEFAULT 'starttls',
            from_email TEXT,
            to_email TEXT,
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS alert_services (
            id TEXT PRIMARY KEY,
            name TEXT,
            host TEXT,
            port INTEGER,
            check_interval INTEGER DEFAULT 60,
            timeout_seconds INTEGER DEFAULT 5,
            max_retries INTEGER DEFAULT 3,
            enabled INTEGER DEFAULT 1,
            last_check_at TEXT,
            last_status INTEGER,
            consecutive_failures INTEGER DEFAULT 0,
            alert_sent_at TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS alert_resources (
            id TEXT PRIMARY KEY,
            name TEXT,
            category TEXT,
            remark TEXT,
            expire_at TEXT,
            alert_advance_days INTEGER DEFAULT 30,
            enabled INTEGER DEFAULT 1,
            last_alert_sent_at TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS alert_history (
            id TEXT PRIMARY KEY,
            type TEXT,
            ref_id TEXT,
            ref_name TEXT,
            message TEXT,
            sent_at TEXT DEFAULT (datetime('now'))
        );

        "#,
    )?;
    // Migration: add keywords column for databases created before v3.1.9
    let _ = conn.execute(
        "ALTER TABLE log_presets ADD COLUMN keywords TEXT NOT NULL DEFAULT '[]'",
        [],
    );
    // Migration: add smtp_encryption column for databases created before v4.1
    let _ = conn.execute(
        "ALTER TABLE alert_email_config ADD COLUMN smtp_encryption TEXT NOT NULL DEFAULT 'starttls'",
        [],
    );
    // Migration: add name column to git_repos for databases created before v4.1
    let _ = conn.execute(
        "ALTER TABLE git_repos ADD COLUMN name TEXT NOT NULL DEFAULT ''",
        [],
    );
    // Migration: add gitRepoId columns to projects
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN gitRepoId TEXT DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN gitRepoId2 TEXT DEFAULT ''",
        [],
    );
    // Migration: add gitRepoId column to cicd_configs
    let _ = conn.execute(
        "ALTER TABLE cicd_configs ADD COLUMN gitRepoId TEXT DEFAULT ''",
        [],
    );
    // Migration: add remark column to alert_resources for databases created before v4.1
    let _ = conn.execute("ALTER TABLE alert_resources ADD COLUMN remark TEXT", []);
    // Migration: add sort column to nginx_* tables for databases created before sort was added
    let _ = conn.execute(
        "ALTER TABLE nginx_basic_settings ADD COLUMN sort INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE nginx_upstreams ADD COLUMN sort INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE nginx_certs ADD COLUMN sort INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE nginx_templates ADD COLUMN sort INTEGER NOT NULL DEFAULT 0",
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
    #[serde(rename = "gitRepoId")]
    pub git_repo_id: Option<String>,
    #[serde(rename = "gitRepoId2")]
    pub git_repo_id2: Option<String>,
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
