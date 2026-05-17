use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitRepo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote: Option<String>,
    pub branch: Option<String>,
    #[serde(rename = "lastOpened")]
    pub last_opened: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<GitRepo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, remote, branch, lastOpened, createdAt, updatedAt FROM git_repos ORDER BY updatedAt DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GitRepo {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            remote: row.get(3)?,
            branch: row.get(4)?,
            last_opened: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    let mut repos = Vec::new();
    for row in rows {
        repos.push(row?);
    }
    Ok(repos)
}

pub fn get_by_id(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Option<GitRepo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, remote, branch, lastOpened, createdAt, updatedAt FROM git_repos WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(GitRepo {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            remote: row.get(3)?,
            branch: row.get(4)?,
            last_opened: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    match rows.next() {
        Some(Ok(repo)) => Ok(Some(repo)),
        _ => Ok(None),
    }
}

pub fn add(
    conn: &rusqlite::Connection,
    id: &str,
    name: &str,
    path: &str,
    remote: Option<&str>,
    branch: Option<&str>,
) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO git_repos (id, name, path, remote, branch, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![id, name, path, remote, branch, now, now],
    )?;
    Ok(())
}

pub fn update(
    conn: &rusqlite::Connection,
    id: &str,
    name: &str,
    path: &str,
    remote: Option<&str>,
    branch: Option<&str>,
) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE git_repos SET name=?, path=?, remote=?, branch=?, updatedAt=? WHERE id=?",
        params![name, path, remote, branch, now, id],
    )?;
    Ok(())
}

pub fn delete(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM git_repos WHERE id=?", params![id])?;
    Ok(())
}
