use rusqlite::params;
use serde::Serialize;

use super::Database;

#[derive(Debug, Serialize, Clone)]
pub struct DbConnectionConfig {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: String,
    pub db_name: Option<String>,
    pub db_index: Option<i64>,
    pub path: Option<String>,
}

pub fn get_all_db_connections(db: &mut Database) -> Result<Vec<DbConnectionConfig>, String> {
    let mut stmt = db.conn.prepare("SELECT id, name, type, host, port, username, password, dbName, dbIndex, path FROM db_connections ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(DbConnectionConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            db_type: row.get(2)?,
            host: row.get(3)?,
            port: row.get(4)?,
            username: row.get(5)?,
            password: row.get(6)?,
            db_name: row.get(7)?,
            db_index: row.get(8)?,
            path: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn add_db_connection(db: &mut Database, config: &DbConnectionConfig) -> Result<(), String> {
    db.conn.execute(
        "INSERT INTO db_connections (id, name, type, host, port, username, password, dbName, dbIndex, path, createdAt, updatedAt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)",
        params![config.id, config.name, config.db_type, config.host, config.port, config.username, config.password, config.db_name, config.db_index, config.path, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_db_connection(db: &mut Database, config: &DbConnectionConfig) -> Result<(), String> {
    db.conn.execute(
        "UPDATE db_connections SET name=?1, type=?2, host=?3, port=?4, username=?5, password=?6, dbName=?7, dbIndex=?8, path=?9, updatedAt=?10 WHERE id=?11",
        params![config.name, config.db_type, config.host, config.port, config.username, config.password, config.db_name, config.db_index, config.path, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), config.id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_db_connection(db: &mut Database, id: &str) -> Result<(), String> {
    db.conn.execute("DELETE FROM db_connections WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
