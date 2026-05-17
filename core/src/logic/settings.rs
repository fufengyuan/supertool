use rusqlite::params;
use serde_json::{Value, json};

/// Settings module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_setting(&self, key: &str) -> Result<Value, String> {
        let result = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![key],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or(String::new())
        });
        Ok(json!(result))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                    params![key, value],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"key": key, "value": value}))
    }

    // ============ Projects ============
}
