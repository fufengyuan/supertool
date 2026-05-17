use rusqlite::params;
use serde_json::{Value, json};

/// Todo module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_all_todos(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM todos ORDER BY createdAt DESC")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    let completed: i64 = row.get("completed")?;
                    let todo: Value = json!({
                        "id": row.get::<_, String>("id")?,
                        "text": row.get::<_, String>("text")?,
                        "completed": completed == 1,
                        "priority": row.get::<_, String>("priority")?,
                        "dueDate": row.get::<_, Option<String>>("dueDate")?,
                        "description": row.get::<_, String>("description")?,
                        "markdownDescription": row.get::<_, Option<String>>("markdownDescription")?,
                        "tag": row.get::<_, String>("tag")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                        "completedAt": row.get::<_, Option<String>>("completedAt")?,
                        "assignedTo": row.get::<_, Option<String>>("assignedTo")?,
                        "assignedBy": row.get::<_, Option<String>>("assignedBy")?,
                        "assignedAt": row.get::<_, Option<String>>("assignedAt")?,
                        "owner": row.get::<_, Option<String>>("owner")?,
                        "orderNum": row.get::<_, i64>("orderNum")?,
                        "repeatType": row.get::<_, Option<String>>("repeatType")?,
                        "repeatInterval": row.get::<_, Option<i64>>("repeatInterval")?,
                        "repeatEndDate": row.get::<_, Option<String>>("repeatEndDate")?,
                        "repeatCount": row.get::<_, i64>("repeatCount")?,
                        "parentTodoId": row.get::<_, Option<String>>("parentTodoId")?,
                        "projectId": row.get::<_, Option<String>>("projectId")?,
                    });
                    Ok(todo)
                })
                .map_err(|e| e.to_string())?;
            let todos: Result<Vec<Value>, _> = rows.collect();
            todos.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_todo(&self, params: Value) -> Result<Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, orderNum, repeatCount, projectId) VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10)",
                    params![
                        id,
                        params["text"].as_str().unwrap_or(""),
                        params["priority"].as_str().unwrap_or("medium"),
                        params["dueDate"].as_str().filter(|s| !s.is_empty()).unwrap_or(""),
                        params["description"].as_str().unwrap_or(""),
                        params.get("markdownDescription").and_then(|v| v.as_str()),
                        params["tag"].as_str().unwrap_or(""),
                        now,
                        now,
                        params.get("projectId").and_then(|v| v.as_str()),
                    ],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "text": params["text"], "createdAt": now}))
    }

    pub async fn update_todo(&self, params: Value) -> Result<Value, String> {
        let id = params["id"].as_str().unwrap_or("").to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            let completed = params["completed"].as_bool().unwrap_or(false);
            let completed_at = if completed { Some(now.clone()) } else { None };
            db.conn_mut()
                .execute(
                    "UPDATE todos SET text=?2, completed=?3, priority=?4, dueDate=?5, description=?6, tag=?7, updatedAt=?8, completedAt=?9, projectId=?10 WHERE id=?1",
                    params![
                        id,
                        params["text"].as_str().unwrap_or(""),
                        if completed { 1 } else { 0 },
                        params["priority"].as_str().unwrap_or("medium"),
                        params["dueDate"].as_str().unwrap_or(""),
                        params["description"].as_str().unwrap_or(""),
                        params["tag"].as_str().unwrap_or(""),
                        now,
                        completed_at,
                        params.get("projectId").and_then(|v| v.as_str()),
                    ],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_todo(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM todos WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Subtasks ============

    pub async fn get_subtasks_for_todo(&self, todo_id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM subtasks WHERE todoId = ?1 ORDER BY createdAt")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![todo_id], |row| {
                    let completed: i64 = row.get("completed")?;
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "todoId": row.get::<_, String>("todoId")?,
                        "text": row.get::<_, String>("text")?,
                        "description": row.get::<_, String>("description")?,
                        "completed": completed == 1,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let subtasks: Result<Vec<Value>, _> = rows.collect();
            subtasks.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_subtask(&self, params: Value) -> Result<Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO subtasks (id, todoId, text, description, completed, createdAt) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                    params![
                        id,
                        params["todoId"].as_str().unwrap_or(""),
                        params["text"].as_str().unwrap_or(""),
                        params["description"].as_str().unwrap_or(""),
                        now,
                    ],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "text": params["text"]}))
    }

    pub async fn update_subtask(&self, params: Value) -> Result<Value, String> {
        let id = params["id"].as_str().unwrap_or("").to_string();
        let completed = params["completed"].as_bool().unwrap_or(false);
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE subtasks SET text=?2, description=?3, completed=?4 WHERE id=?1",
                    params![
                        id,
                        params["text"].as_str().unwrap_or(""),
                        params["description"].as_str().unwrap_or(""),
                        if completed { 1 } else { 0 },
                    ],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_subtask(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM subtasks WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Tags ============

    pub async fn get_all_tags(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT DISTINCT tag FROM todos WHERE tag != '' ORDER BY tag")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok(row.get::<_, String>("tag")?))
                .map_err(|e| e.to_string())?;
            let tags: Result<Vec<String>, _> = rows.collect();
            tags.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_tag(&self, name: &str) -> Result<Value, String> {
        Ok(json!({"name": name}))
    }

    pub async fn delete_tag(&self, name: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("UPDATE todos SET tag = '' WHERE tag = ?1", params![name])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"name": name}))
    }

    // ============ Settings ============
}
