use rusqlite::params;
use serde_json::{Value, json};

/// Notes module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_all_notes(
        &self,
        query: Option<String>,
        group_id: Option<String>,
    ) -> Result<Value, String> {
        let q = query.unwrap_or_default();
        let gid = group_id.unwrap_or_default();
        let result = self.with_db(|db| {
            let mut sql = "SELECT * FROM notes WHERE 1=1".to_string();
            if !gid.is_empty() {
                sql.push_str(&format!(" AND groupId = '{}'", gid.replace('\'', "''")));
            }
            if !q.is_empty() {
                sql.push_str(&format!(
                    " AND (title LIKE '%{}%' OR content LIKE '%{}%')",
                    q.replace('\'', "''"),
                    q.replace('\'', "''")
                ));
            }
            sql.push_str(" ORDER BY updatedAt DESC");
            let mut stmt = db.conn().prepare(&sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "title": row.get::<_, String>("title")?,
                        "content": row.get::<_, String>("content")?,
                        "groupId": row.get::<_, Option<String>>("groupId")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let notes: Result<Vec<Value>, _> = rows.collect();
            notes.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_note(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let title = params["title"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        let group_id = params.get("groupId").and_then(|v| v.as_str());
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO notes (id, title, content, groupId, createdAt, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, title, content, group_id, now, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "title": title}))
    }

    pub async fn update_note(&self, id: &str, params: Value) -> Result<Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = id.to_string();

        let has_title = params.get("title").is_some();
        let has_content = params.get("content").is_some();
        let has_group_id = params.get("groupId").is_some();

        if !has_title && !has_content && !has_group_id {
            // Nothing updatable — read and return current state
            return self.with_db(|db| {
                let mut stmt = db
                    .conn()
                    .prepare(
                        "SELECT id, title, content, groupId, createdAt, updatedAt FROM notes WHERE id = ?1",
                    )
                    .map_err(|e| e.to_string())?;
                stmt.query_row(params![id], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "title": row.get::<_, String>("title")?,
                        "content": row.get::<_, String>("content")?,
                        "groupId": row.get::<_, Option<String>>("groupId")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())
            });
        }

        self.with_db(|db| {
            // 1. Read current values to merge with partial params
            let (cur_title, cur_content, cur_group_id): (String, String, Option<String>) = {
                let mut stmt = db
                    .conn()
                    .prepare("SELECT title, content, groupId FROM notes WHERE id = ?1")
                    .map_err(|e| e.to_string())?;
                stmt.query_row(params![id], |row| {
                    Ok((
                        row.get::<_, String>("title")?,
                        row.get::<_, String>("content")?,
                        row.get::<_, Option<String>>("groupId")?,
                    ))
                })
                .map_err(|e| e.to_string())?
            }; // drop stmt, release immutable borrow on db

            // 2. Merge: only overwrite fields present in params
            let new_title = if has_title {
                params["title"].as_str().unwrap_or("").to_string()
            } else {
                cur_title
            };
            let new_content = if has_content {
                params["content"].as_str().unwrap_or("").to_string()
            } else {
                cur_content
            };
            let new_group_id = if has_group_id {
                params["groupId"].as_str().map(|s| s.to_string())
            } else {
                cur_group_id
            };

            // 3. Write merged values
            db.conn_mut()
                .execute(
                    "UPDATE notes SET title=?2, content=?3, groupId=?4, updatedAt=?5 WHERE id=?1",
                    params![id, new_title, new_content, new_group_id, now],
                )
                .map_err(|e| e.to_string())?;

            // 4. Return the full note
            let mut stmt2 = db
                .conn()
                .prepare(
                    "SELECT id, title, content, groupId, createdAt, updatedAt FROM notes WHERE id = ?1",
                )
                .map_err(|e| e.to_string())?;
            let note = stmt2
                .query_row(params![id], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "title": row.get::<_, String>("title")?,
                        "content": row.get::<_, String>("content")?,
                        "groupId": row.get::<_, Option<String>>("groupId")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;

            Ok(note)
        })
    }

    pub async fn delete_note(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM notes WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Note Groups ============

    pub async fn get_all_note_groups(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM note_groups ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let groups: Result<Vec<Value>, _> = rows.collect();
            groups.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_note_group(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO note_groups (id, name) VALUES (?1, ?2)",
                    params![id, name],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_note_group(&self, id: &str, params: Value) -> Result<Value, String> {
        if params.get("name").is_some() {
            let name = params["name"].as_str().unwrap_or("").to_string();
            self.with_db(|db| {
                db.conn_mut()
                    .execute(
                        "UPDATE note_groups SET name=?2 WHERE id=?1",
                        params![id, name],
                    )
                    .map_err(|e| e.to_string())
            })
            .map_err(|e| e.to_string())?;
        }

        // Return full group data
        self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT id, name FROM note_groups WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            stmt.query_row(params![id], |row| {
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "name": row.get::<_, String>("name")?,
                }))
            })
            .map_err(|e| e.to_string())
        })
    }

    pub async fn delete_note_group(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM note_groups WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Weekly Reports ============
}
