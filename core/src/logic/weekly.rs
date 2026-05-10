use serde_json::{json, Value};
use rusqlite::params;

/// Weekly module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_weekly_reports(&self, limit: usize) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM weekly_reports ORDER BY id DESC LIMIT ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    Ok(json!({
                        "id": row.get::<_, i64>("id")?,
                        "weekStart": row.get::<_, String>("weekStart")?,
                        "weekEnd": row.get::<_, String>("weekEnd")?,
                        "content": row.get::<_, String>("content")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let reports: Result<Vec<Value>, _> = rows.collect();
            reports.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn get_weekly_report(&self, id: i64) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM weekly_reports WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            stmt.query_row(params![id], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>("id")?,
                    "weekStart": row.get::<_, String>("weekStart")?,
                    "weekEnd": row.get::<_, String>("weekEnd")?,
                    "content": row.get::<_, String>("content")?,
                    "createdAt": row.get::<_, String>("createdAt")?,
                }))
            })
            .map_err(|e| e.to_string())
        });
        Ok(result?)
    }

    pub async fn save_weekly_report(&self, params: Value) -> Result<Value, String> {
        let week_start = params["weekStart"].as_str().unwrap_or("").to_string();
        let week_end = params["weekEnd"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let id = self
            .with_db(|db| {
                db.conn_mut()
                    .execute(
                        "INSERT INTO weekly_reports (weekStart, weekEnd, content, createdAt) VALUES (?1, ?2, ?3, ?4)",
                        params![week_start, week_end, content, now],
                    )
                    .map_err(|e| e.to_string())
            })
            .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Notification ============

    pub async fn get_notification_settings(&self) -> Result<Value, String> {
        let reminder_time: String = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT value FROM settings WHERE key = 'reminder_time'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or("15".to_string())
        });
        Ok(json!({"reminderTime": reminder_time.parse::<i64>().unwrap_or(15)}))
    }

    pub async fn set_notification_settings(&self, params: Value) -> Result<Value, String> {
        let reminder_time = params["reminderTime"].as_i64().unwrap_or(15);
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT OR REPLACE INTO settings (key, value) VALUES ('reminder_time', ?1)",
                    params![reminder_time.to_string()],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"reminderTime": reminder_time}))
    }

    // ============ Accounting ============


}
