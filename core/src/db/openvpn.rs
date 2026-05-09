use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenVPNConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<OpenVPNConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, filePath, content, createdAt, updatedAt FROM openvpn_configs ORDER BY name COLLATE NOCASE"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(OpenVPNConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            file_path: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    let mut configs = Vec::new();
    for row in rows {
        configs.push(row?);
    }
    Ok(configs)
}

pub fn add(conn: &rusqlite::Connection, name: &str, file_path: &str, content: &str) -> rusqlite::Result<String> {
    let id = format!("ovpn_{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO openvpn_configs (id, name, filePath, content, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)",
        params![id, name, file_path, content, now, now],
    )?;
    Ok(id)
}

pub fn delete(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM openvpn_configs WHERE id = ?", params![id])?;
    Ok(())
}
