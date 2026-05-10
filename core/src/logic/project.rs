/// Project module — extracted from mod.rs
use super::CoreService;
use crate::db::Database;
use serde_json::{json, Value};
use std::path::PathBuf;
use rusqlite::params;
use crate::db::projects;
use crate::db::Project;

impl super::CoreService {
    pub async fn get_all_projects(&self, only_active: bool) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let resp = projects::get_projects(db, only_active);
            if resp.success {
                serde_json::to_value(&resp.data).map_err(|e| e.to_string())
            } else {
                Err(resp.error.unwrap_or_default())
            }
        });
        Ok(result?)
    }
    pub async fn add_project(&self, params: Value) -> Result<Value, String> {
        let project = serde_json::from_value::<Project>(params).map_err(|e| e.to_string())?;
        let result = self.with_db(|db| projects::add_project(db, project));
        if result.success {
            Ok(json!(result.data))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }
    pub async fn update_project(&self, params: Value) -> Result<Value, String> {
        let project = serde_json::from_value::<Project>(params).map_err(|e| e.to_string())?;
        let result = self.with_db(|db| projects::update_project(db, project));
        if result.success {
            Ok(json!(result.data))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }
    pub async fn delete_project(&self, id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| projects::delete_project(db, id.to_string()));
        if result.success {
            Ok(json!({"id": id}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }
    pub async fn get_project_stats(&self, project_id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| projects::get_project_stats(db, project_id.to_string()));
        if result.success {
            Ok(json!(result.data))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }
    pub async fn get_project_todos(&self, project_id: &str) -> Result<Value, String> {
        let pid = project_id.to_string();
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM todos WHERE projectId = ?1 ORDER BY createdAt DESC")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![pid], |row| {
                    let completed: i64 = row.get("completed")?;
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "text": row.get::<_, String>("text")?,
                        "completed": completed == 1,
                        "priority": row.get::<_, String>("priority")?,
                        "tag": row.get::<_, String>("tag")?,
                        "projectId": row.get::<_, Option<String>>("projectId")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let todos: Result<Vec<Value>, _> = rows.collect();
            todos.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }
    // ============ Servers ============
}