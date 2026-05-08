/// Core Service — 共享业务逻辑层
///
/// UDS handlers 和 Tauri commands 都通过这一层操作数据库和服务。
/// 保证 CLI 和 GUI 走同一套代码路径，行为 100% 一致。
pub mod data_dir;
pub mod git;
pub mod ssh;
pub mod cicd_deploy;
pub mod openvpn;
pub mod wireguard;
pub mod lan;
pub mod system_logger;
pub mod tray_notification;
pub mod log_sanitizer;
use crate::db::projects;
use crate::db::servers;
use crate::db::{Database, Project, Server, ServerGroup};
use crate::encryption::encrypt_password;
use hmac::{KeyInit, Mac};
use rusqlite::params;
use serde_json::{json, Value};
use sha1::Sha1;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// CoreService 持有数据库连接和加密密钥
pub struct CoreService {
    db: Arc<Mutex<Database>>,
    ssh: Arc<ssh::SshService>,
    app_dir: PathBuf,
}

impl Clone for CoreService {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            ssh: self.ssh.clone(),
            app_dir: self.app_dir.clone(),
        }
    }
}

impl CoreService {
    pub fn new(db: Database, app_dir: PathBuf) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            ssh: Arc::new(ssh::SshService::new()),
            app_dir,
        }
    }

    /// 获取 SSH 服务引用
    #[allow(dead_code)]
    pub fn ssh(&self) -> &ssh::SshService {
        &self.ssh
    }

    /// 克隆 SSH 服务 Arc（用于 spawn_blocking）
    #[allow(dead_code)]
    pub fn clone_ssh(&self) -> Arc<ssh::SshService> {
        Arc::clone(&self.ssh)
    }

    /// 获取数据库连接（只读）
    pub fn db_read<T>(&self, f: impl FnOnce(&rusqlite::Connection) -> T) -> Result<T, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = f(db.conn());
        log::debug!("[CoreService] db_read completed");
        Ok(result)
    }

    /// 获取数据库连接（可写）
    pub fn db_write<T>(&self, f: impl FnOnce(&rusqlite::Connection) -> T) -> Result<T, String> {
        let t0 = std::time::Instant::now();
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let result = f(db.conn());
        let elapsed = t0.elapsed().as_millis();
        log::debug!("[CoreService] db_write completed in {}ms", elapsed);
        Ok(result)
    }

    /// 获取数据库的可变引用（锁保护）
    fn with_db<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Database) -> T,
    {
        let mut db = self.db.lock().unwrap();
        f(&mut db)
    }

    // ============ Todos ============

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

    pub async fn get_all_servers(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let resp = servers::get_all_servers(db);
            if let Some(servers) = resp.data {
                let filtered: Vec<Value> = servers
                    .into_iter()
                    .map(|s| {
                        let mut map = serde_json::to_value(&s).unwrap_or_default();
                        if let Some(obj) = map.as_object_mut() {
                            obj.remove("password");
                        }
                        map
                    })
                    .collect();
                Ok(json!(filtered))
            } else {
                Err(resp.error.unwrap_or_default())
            }
        });
        result
    }

    pub async fn get_server_by_id(&self, id: &str) -> Result<Value, String> {
        let sid = id.to_string();
        let result: Result<Value, String> = self.with_db(|db| {
            let resp = servers::get_server_by_id(db, sid);
            if let Some(server) = resp.data {
                let mut map = serde_json::to_value(&server).unwrap_or_default();
                if let Some(obj) = map.as_object_mut() {
                    // 解密密码返回，与 Electron 版行为一致（server-handlers.ts 解密后返回）
                    if let Some(enc_pw) = obj.get("password").and_then(|v| v.as_str()) {
                        if !enc_pw.is_empty() {
                            let dec_pw = crate::encryption::try_decrypt_password(enc_pw);
                            obj.insert("password".to_string(), json!(dec_pw));
                        }
                    }
                }
                Ok(map)
            } else {
                Ok(json!(null))
            }
        });
        Ok(result?)
    }

    pub async fn add_server(&self, params: Value) -> Result<Value, String> {
        let mut params = params.clone();
        // Auto-generate id if not provided (for CLI/stool)
        if params.get("id").and_then(|v| v.as_str()).map_or(true, |s| s.is_empty()) {
            params["id"] = json!(uuid::Uuid::new_v4().to_string());
        }
        let now = chrono::Utc::now().to_rfc3339();
        if params.get("createdAt").is_none() { params["createdAt"] = json!(now); }
        if params.get("updatedAt").is_none() { params["updatedAt"] = json!(now); }
        if params.get("tags").is_none() { params["tags"] = json!([]); }
        if params.get("description").is_none() { params["description"] = json!(""); }
        if params.get("requiresApproval").is_none() { params["requiresApproval"] = json!(false); }
        let mut server =
            serde_json::from_value::<Server>(params.clone()).map_err(|e| e.to_string())?;
        // CLI 发送明文密码，需要加密后存储
        if let Some(ref pwd) = server.password {
            if !pwd.is_empty() && !pwd.starts_with("enc:") {
                server.password = Some(encrypt_password(pwd).map_err(|e| e.to_string())?);
            }
        }
        let result = self.with_db(|db| servers::add_server(db, server));
        if result.success {
            Ok(json!({"id": params["id"].as_str().unwrap_or("")}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    pub async fn update_server(&self, params: Value) -> Result<Value, String> {
        let mut server =
            serde_json::from_value::<Server>(params.clone()).map_err(|e| e.to_string())?;
        // 只在明文密码时加密，避免重复加密已加密的密码
        if let Some(ref pwd) = server.password {
            if !pwd.is_empty() && !pwd.starts_with("enc:") {
                server.password = Some(encrypt_password(pwd).map_err(|e| e.to_string())?);
            }
        }
        let result = self.with_db(|db| servers::update_server(db, server));
        if result.success {
            Ok(json!({"id": params["id"].as_str().unwrap_or("")}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    pub async fn delete_server(&self, id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| servers::delete_server(db, id.to_string()));
        if result.success {
            Ok(json!({"id": id}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    pub async fn test_server_connection(&self, params: Value) -> Result<Value, String> {
        let config = ssh::SshServerConfig {
            id: params["id"].as_str().unwrap_or("").to_string(),
            name: params["name"].as_str().unwrap_or("").to_string(),
            host: params["host"].as_str().unwrap_or("").to_string(),
            port: params["port"].as_u64().unwrap_or(22) as u32,
            username: params["username"].as_str().unwrap_or("").to_string(),
            password: params
                .get("password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ssh_key_path: params
                .get("sshKeyPath")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };
        self.ssh.test_connection(&config)?;
        Ok(json!({"success": true}))
    }

    pub async fn get_all_server_groups(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let resp = servers::get_all_server_groups(db);
            if let Some(groups) = resp.data {
                serde_json::to_value(&groups).map_err(|e| e.to_string())
            } else {
                Err(resp.error.unwrap_or_default())
            }
        });
        Ok(json!(result?))
    }

    pub async fn add_server_group(&self, params: Value) -> Result<Value, String> {
        let group = serde_json::from_value::<ServerGroup>(params).map_err(|e| e.to_string())?;
        let result = self.with_db(|db| servers::add_server_group(db, group));
        if result.success {
            Ok(json!(result.data))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    pub async fn update_server_group(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let description = params["description"].as_str().unwrap_or("").to_string();
        let parent_id = params
            .get("parentId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let color = params["color"].as_str().unwrap_or("#6c63ff").to_string();
        let result = self.with_db(|db| {
            servers::update_server_group(db, id.to_string(), name, description, parent_id, color)
        });
        if result.success {
            Ok(json!({"id": id}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    pub async fn delete_server_group(&self, id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| servers::delete_server_group(db, id.to_string()));
        if result.success {
            Ok(json!({"id": id}))
        } else {
            Err(result.error.unwrap_or_default())
        }
    }

    // ============ MFA/OTP ============

    pub async fn get_all_mfa_secrets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM mfa_secrets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "secret": row.get::<_, String>("secret")?,
                        "issuer": row.get::<_, Option<String>>("issuer")?,
                        "digits": row.get::<_, i64>("digits")?,
                        "period": row.get::<_, i64>("period")?,
                        "algorithm": row.get::<_, String>("algorithm")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let secrets: Result<Vec<Value>, _> = rows.collect();
            secrets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_mfa_secret(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let secret = params["secret"].as_str().unwrap_or("").to_string();
        let issuer = params.get("issuer").and_then(|v| v.as_str()).unwrap_or("");
        let digits = params["digits"].as_u64().unwrap_or(6);
        let period = params["period"].as_u64().unwrap_or(30);
        let algorithm = params["algorithm"].as_str().unwrap_or("SHA1");
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO mfa_secrets (id, name, secret, issuer, digits, period, algorithm, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![id, name, secret, issuer, digits as i64, period as i64, algorithm, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_mfa_secret(&self, id: &str, params: Value) -> Result<Value, String> {
        let mut updates = Vec::new();
        if let Some(name) = params["name"].as_str() {
            updates.push(format!("name='{}'", name.replace('\'', "''")));
        }
        if let Some(secret) = params["secret"].as_str() {
            updates.push(format!("secret='{}'", secret.replace('\'', "''")));
        }
        if updates.is_empty() {
            return Ok(json!({"id": id}));
        }
        let sql = format!(
            "UPDATE mfa_secrets SET {} WHERE id='{}'",
            updates.join(", "),
            id
        );
        self.with_db(|db| db.conn_mut().execute(&sql, []).map_err(|e| e.to_string()))
            .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_mfa_secret(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM mfa_secrets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn generate_totp(
        &self,
        secret: &str,
        digits: u32,
        period: u32,
        algorithm: &str,
    ) -> Result<Value, String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let time_step = epoch.as_secs() / period as u64;
        let remaining = period - (epoch.as_secs() % period as u64) as u32;

        type HmacSha1 = hmac::Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).map_err(|e| e.to_string())?;
        mac.update(&time_step.to_be_bytes());
        let result = mac.finalize().into_bytes();

        let offset = (result[19] & 0xf) as usize;
        let code = ((result[offset] as u32 & 0x7f) << 24)
            | ((result[offset + 1] as u32) << 16)
            | ((result[offset + 2] as u32) << 8)
            | (result[offset + 3] as u32);
        let modulo = 10u32.pow(digits);
        let otp = code % modulo;

        Ok(json!({
            "code": format!("{:0>width$}", otp, width = digits as usize),
            "remaining": remaining,
            "algorithm": algorithm,
        }))
    }

    // ============ Notes ============

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
        let title = params["title"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        let group_id = params.get("groupId").and_then(|v| v.as_str());
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE notes SET title=?2, content=?3, groupId=?4, updatedAt=?5 WHERE id=?1",
                    params![id, title, content, group_id, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
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
        Ok(json!({"id": id}))
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

    pub async fn get_accounting_categories(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM accounting_categories ORDER BY sortOrder, name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "type": row.get::<_, String>("type")?,
                        "icon": row.get::<_, Option<String>>("icon")?,
                        "sortOrder": row.get::<_, i64>("sortOrder")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let cats: Result<Vec<Value>, _> = rows.collect();
            cats.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_accounting_category(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let cat_type = params["type"].as_str().unwrap_or("expense").to_string();
        let icon = params.get("icon").and_then(|v| v.as_str());
        let sort_order: i64 = params["sortOrder"].as_i64().unwrap_or(0);
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO accounting_categories (id, name, type, icon, sortOrder, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, name, cat_type, icon, sort_order, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_accounting_category(
        &self,
        id: &str,
        params: Value,
    ) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let cat_type = params["type"].as_str().unwrap_or("expense").to_string();
        let icon = params.get("icon").and_then(|v| v.as_str());
        let sort_order: i64 = params["sortOrder"].as_i64().unwrap_or(0);
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE accounting_categories SET name=?2, type=?3, icon=?4, sortOrder=?5 WHERE id=?1",
                    params![id, name, cat_type, icon, sort_order],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_accounting_category(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "DELETE FROM accounting_categories WHERE id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_accounting_records(&self, params: Value) -> Result<Value, String> {
        let start_date = params["startDate"].as_str().unwrap_or("");
        let end_date = params["endDate"].as_str().unwrap_or("");
        let r#type = params["type"].as_str().unwrap_or("");
        let category = params["category"].as_str().unwrap_or("");
        let status = params["status"].as_str().unwrap_or("");
        let entity = params["entity"].as_str().unwrap_or("");
        let project = params["project"].as_str().unwrap_or("");
        let payment_method = params["payment_method"].as_str().unwrap_or("");
        let search = params["search"].as_str().unwrap_or("");
        let page = params["page"].as_u64().unwrap_or(1).max(1);
        let page_size = params["pageSize"].as_u64().unwrap_or(50).max(1);
        let offset = (page - 1) * page_size;

        let result = self.with_db(|db| {
            // Build WHERE clauses with positional params
            let mut conditions: Vec<String> = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            let mut idx = 1;

            if !start_date.is_empty() {
                conditions.push(format!("date >= ?{}", idx));
                param_values.push(Box::new(start_date.to_string()));
                idx += 1;
            }
            if !end_date.is_empty() {
                conditions.push(format!("date <= ?{}", idx));
                param_values.push(Box::new(end_date.to_string()));
                idx += 1;
            }
            if !r#type.is_empty() {
                conditions.push(format!("type = ?{}", idx));
                param_values.push(Box::new(r#type.to_string()));
                idx += 1;
            }
            if !category.is_empty() {
                conditions.push(format!("category = ?{}", idx));
                param_values.push(Box::new(category.to_string()));
                idx += 1;
            }
            if !status.is_empty() {
                conditions.push(format!("status = ?{}", idx));
                param_values.push(Box::new(status.to_string()));
                idx += 1;
            }
            if !entity.is_empty() {
                conditions.push(format!("entity = ?{}", idx));
                param_values.push(Box::new(entity.to_string()));
                idx += 1;
            }
            if !project.is_empty() {
                conditions.push(format!("project = ?{}", idx));
                param_values.push(Box::new(project.to_string()));
                idx += 1;
            }
            if !payment_method.is_empty() {
                conditions.push(format!("payment_method = ?{}", idx));
                param_values.push(Box::new(payment_method.to_string()));
                idx += 1;
            }
            if !search.is_empty() {
                conditions.push(format!("(description LIKE ?{} OR supplier LIKE ?{})", idx, idx + 1));
                let sp = format!("%{}%", search);
                param_values.push(Box::new(sp.clone()));
                param_values.push(Box::new(sp));
                idx += 2;
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            // Count total
            let count_sql = format!("SELECT COUNT(*) FROM accounting_records {}", where_clause);
            let total: i64 = {
                let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
                db.conn().query_row(&count_sql, params_ref.as_slice(), |row| row.get(0)).unwrap_or(0)
            };

            // Query records with pagination
            let limit_idx = idx;
            let offset_idx = idx + 1;
            let query_sql = format!(
                "SELECT * FROM accounting_records {} ORDER BY date DESC, createdAt DESC LIMIT ?{} OFFSET ?{}",
                where_clause, limit_idx, offset_idx
            );
            param_values.push(Box::new(page_size as i64));
            param_values.push(Box::new(offset as i64));

            let mut stmt = db.conn().prepare(&query_sql).map_err(|e| e.to_string())?;
            let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(params_ref.as_slice(), |row| {
                let attachments_json: Option<String> = row.get("attachments_json").unwrap_or(None);
                let attachments: serde_json::Value = match attachments_json {
                    Some(ref s) if !s.is_empty() && s != "[]" => {
                        serde_json::from_str(s).unwrap_or(serde_json::json!([]))
                    }
                    _ => serde_json::json!([])
                };
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "date": row.get::<_, String>("date")?,
                    "type": row.get::<_, String>("type")?,
                    "category": row.get::<_, String>("category")?,
                    "amount": row.get::<_, f64>("amount")?,
                    "description": row.get::<_, Option<String>>("description")?,
                    "status": row.get::<_, Option<String>>("status")?,
                    "attachmentPath": row.get::<_, Option<String>>("attachmentPath")?,
                    "createdBy": row.get::<_, Option<String>>("createdBy")?,
                    "createdAt": row.get::<_, String>("createdAt")?,
                    "updatedAt": row.get::<_, Option<String>>("updatedAt")?,
                    "voucher_number": row.get::<_, Option<String>>("voucher_number")?,
                    "receipt_type": row.get::<_, Option<String>>("receipt_type")?,
                    "receipt_path": row.get::<_, Option<String>>("receipt_path")?,
                    "entity": row.get::<_, Option<String>>("entity")?,
                    "project": row.get::<_, Option<String>>("project")?,
                    "supplier": row.get::<_, Option<String>>("supplier")?,
                    "invoice_number": row.get::<_, Option<String>>("invoice_number")?,
                    "tax_amount": row.get::<_, Option<f64>>("tax_amount")?,
                    "payment_method": row.get::<_, Option<String>>("payment_method")?,
                    "approver": row.get::<_, Option<String>>("approver")?,
                    "attachments_json": attachments,
                }))
            }).map_err(|e| e.to_string())?;
            let records: Result<Vec<Value>, _> = rows.collect();
            let records = records.map_err(|e| e.to_string())?;
            Ok::<(i64, Vec<Value>), String>((total, records))
        });
        let (total, records) = result?;
        Ok(json!({
            "records": records,
            "total": total,
        }))
    }

    pub async fn add_accounting_record(&self, params: Value) -> Result<Value, String> {
        let id = params["id"].as_str().unwrap_or(&uuid::Uuid::new_v4().to_string()).to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let date = params["date"].as_str().unwrap_or("").to_string();
        let r#type = params["type"].as_str().unwrap_or("expense").to_string();
        let category = params["category"].as_str().unwrap_or("").to_string();
        let amount = params["amount"].as_f64().unwrap_or(0.0);
        let description = params["description"].as_str().or_else(|| params["note"].as_str());
        let status = params["status"].as_str().unwrap_or("completed");
        let attachment_path = params["attachmentPath"].as_str();
        let created_by = params["createdBy"].as_str().unwrap_or("");
        let voucher_number = params["voucher_number"].as_str().unwrap_or("");
        let receipt_type = params["receipt_type"].as_str().unwrap_or("");
        let receipt_path = params["receipt_path"].as_str().unwrap_or("");
        let entity = params["entity"].as_str().unwrap_or("");
        let project = params["project"].as_str().unwrap_or("");
        let supplier = params["supplier"].as_str().unwrap_or("");
        let invoice_number = params["invoice_number"].as_str().unwrap_or("");
        let tax_amount = params["tax_amount"].as_f64();
        let payment_method = params["payment_method"].as_str().unwrap_or("");
        let approver = params["approver"].as_str().unwrap_or("");
        let attachments_json = params["attachments_json"].as_str().unwrap_or("[]");
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
                    params![id, date, r#type, category, amount, description, status, attachment_path, created_by, now, now, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn update_accounting_record(&self, id: &str, params: Value) -> Result<Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        // Build dynamic SET clauses
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        // ?1 will be id at the end

        let fields = [
            ("date", "date"), ("type", "type"), ("category", "category"),
            ("amount", "amount"), ("description", "description"), ("status", "status"),
            ("attachmentPath", "attachmentPath"), ("voucher_number", "voucher_number"),
            ("receipt_type", "receipt_type"), ("receipt_path", "receipt_path"),
            ("entity", "entity"), ("project", "project"), ("supplier", "supplier"),
            ("invoice_number", "invoice_number"), ("payment_method", "payment_method"),
            ("approver", "approver"), ("attachments_json", "attachments_json"),
        ];

        for (json_key, db_col) in &fields {
            if let Some(val) = params.get(*json_key) {
                if val.is_string() {
                    let idx = values.len() + 1;
                    sets.push(format!("{}=?{}", db_col, idx));
                    values.push(Box::new(val.as_str().unwrap_or("").to_string()));
                } else if val.is_number() {
                    let idx = values.len() + 1;
                    sets.push(format!("{}=?{}", db_col, idx));
                    values.push(Box::new(val.as_f64().unwrap_or(0.0)));
                }
            }
        }

        // Also accept "note" as alias for "description"
        if !sets.iter().any(|s| s.starts_with("description")) {
            if let Some(note) = params.get("note").and_then(|v| v.as_str()) {
                let idx = values.len() + 1;
                sets.push(format!("description=?{}", idx));
                values.push(Box::new(note.to_string()));
            }
        }

        // Always update updatedAt
        let idx = values.len() + 1;
        sets.push(format!("updatedAt=?{}", idx));
        values.push(Box::new(now));

        if sets.is_empty() {
            return Ok(json!({"id": id}));
        }

        // id is the last parameter
        let idx = values.len() + 1;
        let sql = format!("UPDATE accounting_records SET {} WHERE id=?{}", sets.join(", "), idx);
        values.push(Box::new(id.to_string()));

        self.with_db(|db| {
            let params_ref: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|p| p.as_ref()).collect();
            db.conn_mut()
                .execute(&sql, params_ref.as_slice())
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_accounting_record(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM accounting_records WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_accounting_stats(&self, params: Value) -> Result<Value, String> {
        let start_date = params["startDate"].as_str().unwrap_or("");
        let end_date = params["endDate"].as_str().unwrap_or("");

        let result = self.with_db(|db| {
            // Build date conditions
            let mut date_conds = Vec::new();
            let mut date_vals: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if !start_date.is_empty() {
                date_conds.push("date >= ?");
                date_vals.push(Box::new(start_date.to_string()));
            }
            if !end_date.is_empty() {
                date_conds.push("date <= ?");
                date_vals.push(Box::new(end_date.to_string()));
            }
            let date_where = if date_conds.is_empty() { String::new() } else { format!(" WHERE {}", date_conds.join(" AND ")) };

            let query_sum = |db: &mut crate::db::Database, extra: &str| -> f64 {
                let where_clause = if date_conds.is_empty() {
                    format!(" WHERE {}", extra)
                } else {
                    format!("{} AND {}", date_where, extra)
                };
                let sql = format!("SELECT COALESCE(SUM(amount), 0) FROM accounting_records{}", where_clause);
                let vals: Vec<&dyn rusqlite::types::ToSql> = date_vals.iter().map(|v| v.as_ref()).collect();
                db.conn().query_row(&sql, vals.as_slice(), |row| row.get(0)).unwrap_or(0.0)
            };

            let total_income = query_sum(db, "type = 'income'");
            let total_expense = query_sum(db, "type = 'expense'");
            let pending_amount = query_sum(db, "status = 'pending'");
            let reimbursed_amount = query_sum(db, "status = 'reimbursed'");

            // Category breakdown (expense only)
            let cat_where = if date_conds.is_empty() {
                " WHERE type = 'expense' AND category != ''".to_string()
            } else {
                format!("{} AND type = 'expense' AND category != ''", date_where)
            };
            let cat_sql = format!("SELECT category, SUM(amount) as amount FROM accounting_records{} GROUP BY category ORDER BY amount DESC", cat_where);
            let vals: Vec<&dyn rusqlite::types::ToSql> = date_vals.iter().map(|v| v.as_ref()).collect();
            let mut cat_stmt = db.conn().prepare(&cat_sql).map_err(|e| e.to_string())?;
            let by_category: Vec<Value> = cat_stmt.query_map(vals.as_slice(), |row| {
                Ok(json!({
                    "category": row.get::<_, String>(0)?,
                    "amount": row.get::<_, f64>(1)?,
                }))
            }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

            Ok::<_, String>(json!({
                "totalIncome": total_income,
                "totalExpense": total_expense,
                "balance": total_income - total_expense,
                "pendingAmount": pending_amount,
                "reimbursedAmount": reimbursed_amount,
                "byCategory": by_category,
            }))
        });
        Ok(result?)
    }

    pub async fn get_budgets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM budgets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "limit": row.get::<_, f64>("limit")?,
                        "period": row.get::<_, String>("period")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let budgets: Result<Vec<Value>, _> = rows.collect();
            budgets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_budget(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let limit = params["limit"].as_f64().unwrap_or(0.0);
        let period = params["period"].as_str().unwrap_or("monthly").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO budgets (id, name, limit, period) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, limit, period],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_budget(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let limit = params["limit"].as_f64().unwrap_or(0.0);
        let period = params["period"].as_str().unwrap_or("monthly").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE budgets SET name=?2, limit=?3, period=?4 WHERE id=?1",
                    params![id, name, limit, period],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_budget(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM budgets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_templates(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM templates ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "content": row.get::<_, String>("content")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let templates: Result<Vec<Value>, _> = rows.collect();
            templates.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_template(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO templates (id, name, content) VALUES (?1, ?2, ?3)",
                    params![id, name, content],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_template(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE templates SET name=?2, content=?3 WHERE id=?1",
                    params![id, name, content],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_template(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM templates WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn use_template(&self, id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM templates WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            stmt.query_row(params![id], |row| {
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "name": row.get::<_, String>("name")?,
                    "content": row.get::<_, String>("content")?,
                }))
            })
            .map_err(|e| e.to_string())
        });
        Ok(result?)
    }

    pub async fn get_accounting_trend(&self, months: usize) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db.conn().prepare(
                "SELECT strftime('%Y-%m', date) as month, type, SUM(amount) as total FROM accounting_records GROUP BY month, type ORDER BY month DESC LIMIT ?1"
            ).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![months as i64], |row| {
                    Ok(json!({
                        "month": row.get::<_, String>("month")?,
                        "type": row.get::<_, String>("type")?,
                        "total": row.get::<_, f64>("total")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let trends: Result<Vec<Value>, _> = rows.collect();
            trends.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    // ============ LAN ============

    pub async fn get_all_lan_users(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let users = crate::db::lan::get_all_users(db.conn()).map_err(|e| e.to_string())?;
            Ok::<_, String>(json!(users))
        });
        result
    }

    pub async fn get_all_lan_messages(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let msgs = crate::db::lan::get_all_messages(db.conn()).map_err(|e| e.to_string())?;
            let chat_msgs = crate::db::lan::get_all_chat_messages(db.conn()).map_err(|e| e.to_string())?;
            let transfers = crate::db::lan::get_all_file_transfers(db.conn()).map_err(|e| e.to_string())?;
            Ok::<_, String>(json!({
                "messages": msgs,
                "chatMessages": chat_msgs,
                "fileTransfers": transfers,
            }))
        });
        result
    }

    #[allow(dead_code)]
    pub async fn insert_lan_user(&self, user: crate::db::lan::LanUser) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_user(db.conn(), &user).map_err(|e| e.to_string())
        })
    }

    #[allow(dead_code)]
    pub async fn insert_lan_message(&self, msg: crate::db::lan::LanMessage) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_message(db.conn(), &msg).map_err(|e| e.to_string())
        })
    }

    #[allow(dead_code)]
    pub async fn insert_chat_message(&self, msg: crate::db::lan::ChatMessage) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_chat_message(db.conn(), &msg).map_err(|e| e.to_string())
        })
    }

    #[allow(dead_code)]
    pub async fn insert_file_transfer(&self, ft: crate::db::lan::FileTransfer) -> Result<(), String> {
        self.with_db(|db| {
            crate::db::lan::insert_file_transfer(db.conn(), &ft).map_err(|e| e.to_string())
        })
    }

    // ============ Backup ============

    pub async fn export_all_data(&self) -> Result<Value, String> {
        let todos = self.get_all_todos().await?;
        let projects = self.get_all_projects(true).await?;
        let servers = self.get_all_servers().await?;
        Ok(json!({
            "todos": todos,
            "projects": projects,
            "servers": servers,
        }))
    }

    pub async fn import_all_data(&self, data: Value, mode: &str) -> Result<Value, String> {
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let data_clone = data.clone();
        let mode_owned = mode.to_string();

        if let Some(obj) = data.as_object() {
            // Import todos
            if let Some(items) = obj.get("todos").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_todo(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("todos".into(), c);
            }
            // Import projects
            if let Some(items) = obj.get("projects").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_project(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("projects".into(), c);
            }
            // Import servers
            if let Some(items) = obj.get("servers").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("servers".into(), c);
            }
            // Import tags
            if let Some(items) = obj.get("tags").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        if self.add_tag(name).await.is_ok() { c += 1; }
                    }
                }
                counts.insert("tags".into(), c);
            }
            // Import subtasks
            if let Some(items) = obj.get("subtasks").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_subtask(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("subtasks".into(), c);
            }
            // Import serverGroups
            if let Some(items) = obj.get("serverGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server_group(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("serverGroups".into(), c);
            }
            // Import mfaSecrets
            if let Some(items) = obj.get("mfaSecrets").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_mfa_secret(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("mfaSecrets".into(), c);
            }
            // Import notes
            if let Some(items) = obj.get("notes").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("notes".into(), c);
            }
            // Import noteGroups
            if let Some(items) = obj.get("noteGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note_group(item.clone()).await.is_ok() { c += 1; }
                }
                counts.insert("noteGroups".into(), c);
            }
        }

        // CICD data handled separately
        let (cicd_c, cicd_s) = self.import_cicd_data(&data_clone, &mode_owned).await.unwrap_or((0, 0));
        counts.insert("cicdConfigs".into(), cicd_c as u32 + cicd_s as u32);

        Ok(json!(counts))
    }

    // ============ CICD Backup Helpers ============

    pub async fn get_all_cicd_data(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let conn = db.conn();
            // cicd_configs
            let mut stmt = conn.prepare("SELECT * FROM cicd_configs")
                .map_err(|e| e.to_string())?;
            let configs: Vec<Value> = stmt.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "projectId": row.get::<_, String>("projectId")?,
                    "name": row.get::<_, String>("name")?,
                    "deployBranch": row.get::<_, String>("deployBranch")?,
                    "mavenSettings": row.get::<_, Option<String>>("mavenSettings")?,
                    "mavenProfile": row.get::<_, String>("mavenProfile")?,
                    "deployPath": row.get::<_, String>("deployPath")?,
                    "libSeparate": row.get::<_, i64>("libSeparate")? == 1,
                    "restartScript": row.get::<_, String>("restartScript")?,
                    "healthCheckUrl": row.get::<_, Option<String>>("healthCheckUrl")?,
                    "healthCheckTimeout": row.get::<_, i64>("healthCheckTimeout")?,
                    "createdAt": row.get::<_, String>("createdAt")?,
                    "updatedAt": row.get::<_, String>("updatedAt")?,
                    "buildTool": row.get::<_, Option<String>>("buildTool")?,
                    "buildCommand": row.get::<_, Option<String>>("buildCommand")?,
                    "buildPath": row.get::<_, Option<String>>("buildPath")?,
                    "repoUrl": row.get::<_, Option<String>>("repoUrl")?,
                    "localPath": row.get::<_, Option<String>>("localPath")?,
                    "npmScript": row.get::<_, Option<String>>("npmScript")?,
                    "npmCustomScript": row.get::<_, Option<String>>("npmCustomScript")?,
                    "mavenHome": row.get::<_, Option<String>>("mavenHome")?,
                    "npmHome": row.get::<_, Option<String>>("npmHome")?,
                    "javaHome": row.get::<_, Option<String>>("javaHome")?,
                }))
            }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_modules
            let mut stmt2 = conn.prepare("SELECT * FROM deploy_modules")
                .map_err(|e| e.to_string())?;
            let modules: Vec<Value> = stmt2.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "configId": row.get::<_, String>("configId")?,
                    "name": row.get::<_, String>("name")?,
                    "order": row.get::<_, i64>("order")?,
                    "serverId": row.get::<_, String>("serverId")?,
                    "remotePath": row.get::<_, String>("remotePath")?,
                    "localPath": row.get::<_, Option<String>>("localPath")?,
                    "preDeployScript": row.get::<_, Option<String>>("preDeployScript")?,
                    "postDeployScript": row.get::<_, Option<String>>("postDeployScript")?,
                }))
            }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_logs
            let mut stmt3 = conn.prepare("SELECT * FROM deploy_logs ORDER BY id DESC LIMIT 500")
                .map_err(|e| e.to_string())?;
            let logs: Vec<Value> = stmt3.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>("id")?,
                    "configId": row.get::<_, String>("configId")?,
                    "status": row.get::<_, String>("status")?,
                    "output": row.get::<_, Option<String>>("output")?,
                    "error": row.get::<_, Option<String>>("error")?,
                    "startedAt": row.get::<_, String>("startedAt")?,
                    "completedAt": row.get::<_, Option<String>>("completedAt")?,
                }))
            }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_history
            let mut stmt4 = conn.prepare("SELECT * FROM deploy_history ORDER BY id DESC LIMIT 200")
                .map_err(|e| e.to_string())?;
            let history: Vec<Value> = stmt4.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>("id")?,
                    "configId": row.get::<_, String>("configId")?,
                    "trigger": row.get::<_, String>("trigger")?,
                    "status": row.get::<_, String>("status")?,
                    "startedAt": row.get::<_, String>("startedAt")?,
                    "completedAt": row.get::<_, Option<String>>("completedAt")?,
                }))
            }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            // deploy_step_logs
            let mut stmt5 = conn.prepare("SELECT * FROM deploy_step_logs ORDER BY id DESC LIMIT 1000")
                .map_err(|e| e.to_string())?;
            let steps: Vec<Value> = stmt5.query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>("id")?,
                    "deployLogId": row.get::<_, String>("deployLogId")?,
                    "step": row.get::<_, String>("step")?,
                    "status": row.get::<_, String>("status")?,
                    "output": row.get::<_, Option<String>>("output")?,
                    "error": row.get::<_, Option<String>>("error")?,
                    "startedAt": row.get::<_, String>("startedAt")?,
                    "completedAt": row.get::<_, Option<String>>("completedAt")?,
                }))
            }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            Ok(json!({
                "cicdConfigs": configs,
                "deployModules": modules,
                "deployLogs": logs,
                "deployHistory": history,
                "deployStepLogs": steps,
            }))
        });
        result
    }

    pub async fn import_cicd_data(&self, data: &Value, mode: &str) -> Result<(usize, usize), String> {
        let mut imported = 0;
        let mut skipped = 0;

        let _ = self.with_db(|db| {
            let conn = db.conn();
            if mode == "replace" {
                conn.execute_batch("DELETE FROM deploy_step_logs; DELETE FROM deploy_logs; DELETE FROM deploy_modules; DELETE FROM cicd_configs; DELETE FROM deploy_history;")
                    .map_err(|e| e.to_string())?;
            }

            // cicd_configs — 30 columns
            if let Some(configs) = data.get("cicdConfigs").and_then(|v| v.as_array()) {
                for c in configs {
                    let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if mode == "merge" {
                        let exists: i64 = conn.prepare("SELECT COUNT(*) FROM cicd_configs WHERE id = ?")
                            .ok().and_then(|mut s| s.query_row([id], |r| r.get(0)).ok()).unwrap_or(0);
                        if exists > 0 { skipped += 1; continue; }
                    }
                    let servers_val: Option<String> = c.get("servers").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .or_else(|| c.get("servers").map(|v| serde_json::to_string(v).unwrap_or_default()));
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO cicd_configs (id, projectId, name, deployBranch, mavenSettings, mavenProfile, deployPath, libSeparate, restartScript, healthCheckUrl, healthCheckTimeout, createdAt, updatedAt, groupName, parentBuildMode, parentBuildPath, requiresApproval, buildTool, buildCommand, buildPath, repoUrl, localPath, npmScript, npmCustomScript, mavenHome, npmHome, javaHome, nodeHome, servers, lastDeployedAt)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30)",
                        rusqlite::params![
                            id,
                            c.get("projectId").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("name").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("deployBranch").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("mavenSettings").and_then(|v|v.as_str()),
                            c.get("mavenProfile").and_then(|v|v.as_str()).unwrap_or("prod"),
                            c.get("deployPath").and_then(|v|v.as_str()).unwrap_or("/"),
                            if c.get("libSeparate").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("restartScript").and_then(|v|v.as_str()).unwrap_or("./restart.sh"),
                            c.get("healthCheckUrl").and_then(|v|v.as_str()),
                            c.get("healthCheckTimeout").and_then(|v|v.as_i64()).unwrap_or(30),
                            c.get("createdAt").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("updatedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("groupName").and_then(|v|v.as_str()).unwrap_or("未分组"),
                            if c.get("parentBuildMode").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("parentBuildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            if c.get("requiresApproval").and_then(|v|v.as_bool()).unwrap_or(false) { 1 } else { 0 },
                            c.get("buildTool").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("buildCommand").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("buildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("repoUrl").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("localPath").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmScript").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmCustomScript").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("mavenHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("npmHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("javaHome").and_then(|v|v.as_str()).unwrap_or(""),
                            c.get("nodeHome").and_then(|v|v.as_str()).unwrap_or(""),
                            servers_val.unwrap_or_default(),
                            c.get("lastDeployedAt").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_modules — 16 columns
            if let Some(modules) = data.get("deployModules").and_then(|v| v.as_array()) {
                for m in modules {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_modules (id, configId, moduleName, modulePath, artifactName, deployOrder, deployPath, enabled, createdAt, updatedAt, libFilterRules, buildCommand, buildPath, outputPath, buildTool, artifactType)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                        rusqlite::params![
                            m.get("id").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("moduleName").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("modulePath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("artifactName").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("deployOrder").and_then(|v|v.as_i64()).unwrap_or(0),
                            m.get("deployPath").and_then(|v|v.as_str()).unwrap_or(""),
                            if m.get("enabled").and_then(|v|v.as_bool()).unwrap_or(true) { 1 } else { 0 },
                            m.get("createdAt").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("updatedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("libFilterRules").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildCommand").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildPath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("outputPath").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("buildTool").and_then(|v|v.as_str()).unwrap_or(""),
                            m.get("artifactType").and_then(|v|v.as_str()).unwrap_or(""),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_logs
            if let Some(logs) = data.get("deployLogs").and_then(|v| v.as_array()) {
                for l in logs {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_logs (id, configId, status, output, error, startedAt, completedAt)
                         VALUES (?1,?2,?3,?4,?5,?6,?7)",
                        rusqlite::params![
                            l.get("id").and_then(|v|v.as_i64()).unwrap_or(0),
                            l.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("output").and_then(|v|v.as_str()),
                            l.get("error").and_then(|v|v.as_str()),
                            l.get("startedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            l.get("completedAt").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_history
            if let Some(history) = data.get("deployHistory").and_then(|v| v.as_array()) {
                for h in history {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_history (id, configId, trigger, status, startedAt, completedAt)
                         VALUES (?1,?2,?3,?4,?5,?6)",
                        rusqlite::params![
                            h.get("id").and_then(|v|v.as_i64()).unwrap_or(0),
                            h.get("configId").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("trigger").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("startedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            h.get("completedAt").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            // deploy_step_logs
            if let Some(steps) = data.get("deployStepLogs").and_then(|v| v.as_array()) {
                for s in steps {
                    let _ = conn.execute(
                        "INSERT OR REPLACE INTO deploy_step_logs (id, deployLogId, step, status, output, error, startedAt, completedAt)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                        rusqlite::params![
                            s.get("id").and_then(|v|v.as_i64()).unwrap_or(0),
                            s.get("deployLogId").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("step").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("status").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("output").and_then(|v|v.as_str()),
                            s.get("error").and_then(|v|v.as_str()),
                            s.get("startedAt").and_then(|v|v.as_str()).unwrap_or(""),
                            s.get("completedAt").and_then(|v|v.as_str()),
                        ]
                    );
                    imported += 1;
                }
            }

            Ok::<(), String>(())
        });

        Ok((imported, skipped))
    }

    // ============ Log Presets ============

    pub async fn get_log_presets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM log_presets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    let server_ids: String = row.get("serverIds")?;
                    let keywords_str: String = row.get("keywords").unwrap_or_else(|_| "[]".to_string());
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "serverIds": serde_json::from_str::<Value>(&server_ids)
                            .unwrap_or(Value::Array(vec![])),
                        "logPath": row.get::<_, String>("logPath")?,
                        "logType": row.get::<_, String>("logType")?,
                        "maxLines": row.get::<_, i64>("maxLines")?,
                        "presetGroup": row.get::<_, Option<String>>("presetGroup")?,
                        "keywords": serde_json::from_str::<Value>(&keywords_str)
                            .unwrap_or(Value::Array(vec![])),
                    }))
                })
                .map_err(|e| e.to_string())?;
            let presets: Result<Vec<Value>, _> = rows.collect();
            presets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_log_preset(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let server_ids =
            serde_json::to_string(&params["serverIds"]).unwrap_or_else(|_| "[]".to_string());
        let log_path = params["logPath"].as_str().unwrap_or("").to_string();
        let log_type = params["logType"].as_str().unwrap_or("file").to_string();
        let max_lines = params["maxLines"].as_i64().unwrap_or(100);
        let preset_group = params.get("presetGroup").and_then(|v| v.as_str());
        let keywords =
            serde_json::to_string(&params["keywords"]).unwrap_or_else(|_| "[]".to_string());
        self.with_db(|db| {
            db.conn_mut().execute(
                "INSERT INTO log_presets (id, name, serverIds, logPath, logType, maxLines, presetGroup, keywords) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, name, server_ids, log_path, log_type, max_lines, preset_group, keywords],
            ).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_log_preset(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let server_ids =
            serde_json::to_string(&params["serverIds"]).unwrap_or_else(|_| "[]".to_string());
        let log_path = params["logPath"].as_str().unwrap_or("").to_string();
        let log_type = params["logType"].as_str().unwrap_or("file").to_string();
        let max_lines = params["maxLines"].as_i64().unwrap_or(100);
        let preset_group = params.get("presetGroup").and_then(|v| v.as_str());
        let keywords =
            serde_json::to_string(&params["keywords"]).unwrap_or_else(|_| "[]".to_string());
        self.with_db(|db| {
            db.conn_mut().execute(
                "UPDATE log_presets SET name=?2, serverIds=?3, logPath=?4, logType=?5, maxLines=?6, presetGroup=?7, keywords=?8 WHERE id=?1",
                params![id, name, server_ids, log_path, log_type, max_lines, preset_group, keywords],
            ).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_log_preset(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM log_presets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Log Aggregator ============

    pub async fn log_tail(&self, preset_id: &str, lines: usize) -> Result<Value, String> {
        // Get preset from database
        let preset = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM log_presets WHERE id = ?1",
                    params![preset_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "name": row.get::<_, String>("name")?,
                            "serverIds": row.get::<_, String>("serverIds")?,
                            "logPath": row.get::<_, String>("logPath")?,
                            "logType": row.get::<_, String>("logType")?,
                            "maxLines": row.get::<_, i64>("maxLines")?,
                        }))
                    },
                )
                .map_err(|e| e.to_string())
        })?;

        let server_ids: Vec<String> = serde_json::from_str(preset["serverIds"].as_str().unwrap_or("[]"))
            .unwrap_or_default();

        if server_ids.is_empty() {
            return Ok(json!({"presetId": preset_id, "lines": lines, "results": [], "note": "No servers configured"}));
        }

        // Build tail command based on log type
        let cmd = build_tail_command(&preset, lines);
        let mut results = Vec::new();

        for server_id in &server_ids {
            // Get server info
            let server = self.with_db(|db| {
                db.conn()
                    .query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        params![server_id],
                        |row| {
                            Ok(json!({
                                "id": row.get::<_, String>("id")?,
                                "name": row.get::<_, String>("name")?,
                                "host": row.get::<_, String>("host")?,
                                "port": row.get::<_, i64>("port")?,
                                "username": row.get::<_, String>("username")?,
                                "password": row.get::<_, Option<String>>("password")?,
                                "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                            }))
                        },
                    )
                    .map_err(|e| e.to_string())
            });

            let Ok(s) = server else { continue; };

            let host = s["host"].as_str().unwrap_or("").to_string();
            let port = s["port"].as_u64().unwrap_or(22) as u32;
            let username = s["username"].as_str().unwrap_or("").to_string();
            let raw_password = s.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ssh_key_path = s.get("sshKeyPath").and_then(|v| v.as_str()).map(|s| s.to_string());

            // 解密密码
            let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

            let config = ssh::SshServerConfig {
                id: server_id.clone(),
                name: s["name"].as_str().unwrap_or("").to_string(),
                host,
                port,
                username,
                password,
                ssh_key_path,
            };

            // Connect if not already connected
            if !self.ssh.is_connected(server_id) {
                if let Err(e) = self.ssh.connect(&config) {
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "error": e,
                        "lines": []
                    }));
                    continue;
                }
            }

            // Execute tail command
            let output = self.ssh.exec_command(server_id, &cmd);
            match output {
                Ok(exec_result) => {
                    let line_list: Vec<String> = exec_result.output
                        .lines()
                        .map(|l| l.to_string())
                        .take(lines)
                        .collect();
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "lines": line_list,
                        "error": if !exec_result.success { Some(exec_result.error_output) } else { None }
                    }));
                }
                Err(e) => {
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "error": e,
                        "lines": []
                    }));
                }
            }
        }

        Ok(json!({"presetId": preset_id, "lines": results.len(), "results": results}))
    }

    pub async fn log_search(
        &self,
        preset_id: &str,
        keyword: &str,
        lines: usize,
    ) -> Result<Value, String> {
        if keyword.trim().is_empty() {
            return Ok(json!({"presetId": preset_id, "keyword": keyword, "matches": []}));
        }

        // Get preset from database
        let preset = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM log_presets WHERE id = ?1",
                    params![preset_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "name": row.get::<_, String>("name")?,
                            "serverIds": row.get::<_, String>("serverIds")?,
                            "logPath": row.get::<_, String>("logPath")?,
                            "logType": row.get::<_, String>("logType")?,
                        }))
                    },
                )
                .map_err(|e| e.to_string())
        })?;

        let server_ids: Vec<String> = serde_json::from_str(preset["serverIds"].as_str().unwrap_or("[]"))
            .unwrap_or_default();

        let cmd = build_grep_command(&preset, keyword, lines);
        let mut matches = Vec::new();

        for server_id in &server_ids {
            let server = self.with_db(|db| {
                db.conn()
                    .query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        params![server_id],
                        |row| {
                            Ok(json!({
                                "id": row.get::<_, String>("id")?,
                                "name": row.get::<_, String>("name")?,
                                "host": row.get::<_, String>("host")?,
                                "port": row.get::<_, i64>("port")?,
                                "username": row.get::<_, String>("username")?,
                                "password": row.get::<_, Option<String>>("password")?,
                                "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                            }))
                        },
                    )
                    .map_err(|e| e.to_string())
            });

            let Ok(s) = server else { continue; };

            let host = s["host"].as_str().unwrap_or("").to_string();
            let port = s["port"].as_u64().unwrap_or(22) as u32;
            let username = s["username"].as_str().unwrap_or("").to_string();
            let raw_password = s.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ssh_key_path = s.get("sshKeyPath").and_then(|v| v.as_str()).map(|s| s.to_string());

            // 解密密码（与 logs_start_stream 一致）
            let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

            let config = ssh::SshServerConfig {
                id: server_id.clone(),
                name: s["name"].as_str().unwrap_or("").to_string(),
                host,
                port,
                username,
                password,
                ssh_key_path,
            };

            if !self.ssh.is_connected(server_id) {
                if let Err(e) = self.ssh.connect(&config) {
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": server_id,
                        "matchCount": 0,
                        "lines": [],
                        "error": e
                    }));
                    continue;
                }
            }

            let output = self.ssh.exec_command(server_id, &cmd);
            match output {
                Ok(exec_result) => {
                    let lines = parse_grep_output(&exec_result.output, keyword);
                    let match_count = lines.iter().filter(|l| l["isMatch"].as_bool().unwrap_or(false)).count();
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "matchCount": match_count,
                        "lines": lines
                    }));
                }
                Err(e) => {
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": server_id,
                        "matchCount": 0,
                        "lines": [],
                        "error": e
                    }));
                }
            }
        }

        Ok(json!({"presetId": preset_id, "keyword": keyword, "matches": matches}))
    }

    // ============ CI/CD ============

    pub async fn cicd_deploy(&self, config_id: &str) -> Result<Value, String> {
        Ok(json!({
            "configId": config_id,
            "note": "CI/CD deploy 尚未实现"
        }))
    }

    // ============ Misc ============

    pub async fn get_app_path(&self) -> Result<Value, String> {
        Ok(json!(self.app_dir.to_string_lossy()))
    }

    pub async fn read_file_content(&self, path: &str) -> Result<Value, String> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(json!({"success": true, "content": content})),
            Err(e) => Ok(json!({"success": false, "error": e.to_string()})),
        }
    }

    pub async fn read_directory(&self, path: &str) -> Result<Value, String> {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        result.push(json!({"name": name, "isDirectory": is_dir}));
                    }
                }
                Ok(json!({"success": true, "entries": result}))
            }
            Err(e) => Ok(json!({"success": false, "error": e.to_string()})),
        }
    }

    // ============ SSH 包装方法 ============

    /// 在 spawn_blocking 线程池中执行 SSH 操作，避免阻塞 async 运行时
    async fn run_ssh_blocking<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&crate::core::ssh::SshService) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        let ssh = self.ssh.clone();
        tokio::task::spawn_blocking(move || f(&ssh))
            .await
            .map_err(|e| format!("SSH 操作失败: {}", e))?
    }

    pub async fn ssh_connect(&self, params: Value) -> Result<Value, String> {
        let server_id = params["id"].as_str().unwrap_or("").to_string();
        let param_pw = params
            .get("password")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let param_key = params
            .get("sshKeyPath")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 如果 params 没传密码也没传密钥，从 DB 查询凭据
        let (password, ssh_key_path) = if param_pw.is_none() && param_key.is_none() && !server_id.is_empty() {
            let sid = server_id.clone();
            let result: Result<(Option<String>, Option<String>), String> = self.with_db(|db| {
                let resp = servers::get_server_by_id(db, sid);
                match resp.data.flatten() {
                    Some(server) => Ok((server.password, server.ssh_key_path)),
                    None => Err("服务器不存在".to_string()),
                }
            });
            let (pw, key) = result?;
            // 解密密码（DB 存的是加密后的）
            let decrypted_pw = pw.map(|p| crate::encryption::try_decrypt_password(&p));
            (decrypted_pw, key)
        } else {
            (param_pw, param_key)
        };

        let config = ssh::SshServerConfig {
            id: server_id,
            name: params["name"].as_str().unwrap_or("").to_string(),
            host: params["host"].as_str().unwrap_or("").to_string(),
            port: params["port"].as_u64().unwrap_or(22) as u32,
            username: params["username"].as_str().unwrap_or("").to_string(),
            password,
            ssh_key_path,
        };

        // 把阻塞的 TCP 连接 + SSH 握手移到 spawn_blocking 避免饿死 async 线程池
        let ssh_ref = self.ssh.clone();
        let config_clone = config.clone();
        tokio::task::spawn_blocking(move || ssh_ref.connect(&config_clone))
            .await
            .map_err(|e| format!("SSH 连接任务失败: {}", e))??;

        Ok(json!({"success": true}))
    }

    pub async fn ssh_disconnect(&self, server_id: &str) -> Result<Value, String> {
        self.ssh.disconnect(server_id);
        Ok(json!({"success": true}))
    }

    pub async fn ssh_is_connected(&self, server_id: &str) -> Result<Value, String> {
        Ok(json!({"connected": self.ssh.is_connected(server_id)}))
    }

    /// 确保 SSH 已连接（如未连接则自动重连），供 SFTP 命令使用
    pub async fn ensure_ssh_connected(&self, server_id: &str) -> Result<(), String> {
        if self.ssh.is_connected(server_id) {
            return Ok(());
        }
        // 从 DB 获取服务器信息并重连
        let server = self.get_server_by_id(server_id).await?;
        let data = server.as_object().ok_or("服务器数据格式错误")?;
        let config = json!({
            "id": data.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "name": data.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "host": data.get("host").and_then(|v| v.as_str()).unwrap_or(""),
            "port": data.get("port").and_then(|v| v.as_u64()).unwrap_or(22),
            "username": data.get("username").and_then(|v| v.as_str()).unwrap_or(""),
            "password": data.get("password").and_then(|v| v.as_str()),
            "sshKeyPath": data.get("sshKeyPath").and_then(|v| v.as_str()),
        });
        self.ssh_connect(config).await?;
        Ok(())
    }

    pub async fn ssh_test_connection(&self, params: Value) -> Result<Value, String> {
        let server_id = params["id"].as_str().unwrap_or("").to_string();
        let param_pw = params
            .get("password")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let param_key = params
            .get("sshKeyPath")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 如果 params 没传密码也没传密钥，从 DB 查询凭据
        let (password, ssh_key_path) = if param_pw.is_none() && param_key.is_none() && !server_id.is_empty() {
            let sid = server_id.clone();
            let result: Result<(Option<String>, Option<String>), String> = self.with_db(|db| {
                let resp = servers::get_server_by_id(db, sid);
                match resp.data.flatten() {
                    Some(server) => Ok((server.password, server.ssh_key_path)),
                    None => Err("服务器不存在".to_string()),
                }
            });
            result?
        } else {
            (param_pw, param_key)
        };

        let config = ssh::SshServerConfig {
            id: server_id,
            name: params["name"].as_str().unwrap_or("").to_string(),
            host: params["host"].as_str().unwrap_or("").to_string(),
            port: params["port"].as_u64().unwrap_or(22) as u32,
            username: params["username"].as_str().unwrap_or("").to_string(),
            password,
            ssh_key_path,
        };
        self.ssh.test_connection(&config)?;
        Ok(json!({"success": true}))
    }

    pub async fn exec_ssh_command(&self, server_id: &str, command: &str) -> Result<Value, String> {
        let sid = server_id.to_string();
        let cmd = command.to_string();
        let result = self.run_ssh_blocking(move |ssh| ssh.exec_command(&sid, &cmd)).await?;
        Ok(json!(result))
    }

    pub async fn sftp_list_dir(&self, server_id: &str, remote_path: &str) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        let files = self.run_ssh_blocking(move |ssh| ssh.list_remote_dir(&sid, &rp)).await?;
        Ok(json!({"success": true, "files": files}))
    }

    pub async fn sftp_download_file(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        let content = self.run_ssh_blocking(move |ssh| ssh.download_file_base64(&sid, &rp)).await?;
        Ok(json!({"content": content}))
    }

    pub async fn sftp_create_dir(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        self.run_ssh_blocking(move |ssh| ssh.create_remote_dir(&sid, &rp)).await?;
        Ok(json!({"success": true}))
    }

    pub async fn sftp_delete_file(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        self.run_ssh_blocking(move |ssh| ssh.delete_remote_file(&sid, &rp)).await?;
        Ok(json!({"success": true}))
    }

    /// SFTP: 下载到本地路径
    pub async fn sftp_download_to_local(
        &self,
        server_id: &str,
        remote_path: &str,
        local_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        let lp = local_path.to_string();
        let size = self.run_ssh_blocking(move |ssh| ssh.download_file(&sid, &rp, &lp)).await?;
        Ok(json!({"success": true, "data": {"bytesDownloaded": size, "localPath": local_path}}))
    }

    /// SFTP: 上传文件到远程
    pub async fn sftp_upload_to_remote(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let metadata = tokio::fs::metadata(local_path).await
            .map_err(|e| format!("读取本地文件失败: {}", e))?;
        
        if metadata.is_dir() {
            // 目录 → 递归上传
            let sid = server_id.to_string();
            let lp = local_path.to_string();
            let rp = remote_path.to_string();
            let size = self.run_ssh_blocking(move |ssh| ssh.upload_dir_recursive(&sid, &lp, &rp)).await?;
            Ok(json!({"success": true, "data": {"bytesUploaded": size, "remotePath": remote_path}}))
        } else {
            // 文件 → 单文件上传
            let sid = server_id.to_string();
            let lp = local_path.to_string();
            let rp = remote_path.to_string();
            let size = self.run_ssh_blocking(move |ssh| ssh.upload_file(&sid, &lp, &rp)).await?;
            Ok(json!({"success": true, "data": {"bytesUploaded": size, "remotePath": remote_path}}))
        }
    }

    /// SFTP: 递归上传目录
    pub async fn sftp_upload_dir_recursive(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let lp = local_path.to_string();
        let rp = remote_path.to_string();
        let size = self.run_ssh_blocking(move |ssh| ssh.upload_dir_recursive(&sid, &lp, &rp)).await?;
        Ok(json!({"success": true, "data": {"bytesUploaded": size, "remotePath": remote_path}}))
    }

    // ============ PTY Terminal 包装方法 ============

    pub async fn ssh_create_terminal(
        &self,
        server_id: &str,
        terminal_id: &str,
        rows: u32,
        cols: u32,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let tid = terminal_id.to_string();
        self.run_ssh_blocking(move |ssh| ssh.create_terminal(&sid, &tid, rows, cols)).await?;
        Ok(json!({"success": true, "terminalId": terminal_id}))
    }

    pub async fn ssh_read_terminal(&self, terminal_id: &str) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        let data = self.run_ssh_blocking(move |ssh| ssh.read_terminal(&tid)).await?;
        Ok(json!({"success": true, "data": data}))
    }

    pub async fn ssh_write_to_terminal(
        &self,
        terminal_id: &str,
        data: &str,
    ) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        let d = data.to_string();
        self.run_ssh_blocking(move |ssh| ssh.write_to_terminal(&tid, &d)).await?;
        Ok(json!({"success": true}))
    }

    pub async fn ssh_resize_terminal(
        &self,
        terminal_id: &str,
        rows: u32,
        cols: u32,
    ) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        self.run_ssh_blocking(move |ssh| ssh.resize_terminal(&tid, rows, cols)).await?;
        Ok(json!({"success": true}))
    }

    pub async fn ssh_close_terminal(&self, terminal_id: &str) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        self.run_ssh_blocking(move |ssh| ssh.close_terminal(&tid)).await?;
        Ok(json!({"success": true}))
    }

    pub async fn ssh_is_terminal_active(&self, terminal_id: &str) -> bool {
        self.ssh.is_terminal_active(terminal_id)
    }

    // ============ Git 操作包装方法 ============

    pub async fn git_status(&self, repo_path: &str) -> Result<Value, String> {
        git::git_status(repo_path).await
    }
    pub async fn git_log(&self, repo_path: &str, limit: Option<usize>) -> Result<Value, String> {
        git::git_log(repo_path, limit).await
    }
    pub async fn git_branches(&self, repo_path: &str) -> Result<Value, String> {
        git::git_branches(repo_path).await
    }
    pub async fn git_current_branch(&self, repo_path: &str) -> Result<Value, String> {
        git::git_current_branch(repo_path).await
    }
    pub async fn git_diff(&self, repo_path: &str, file: Option<&str>) -> Result<Value, String> {
        git::git_diff(repo_path, file).await
    }
    pub async fn git_commit_diff(&self, repo_path: &str, hash: &str) -> Result<Value, String> {
        git::git_commit_diff(repo_path, hash).await
    }
    pub async fn git_commit(&self, repo_path: &str, msg: &str, files: Option<&[&str]>) -> Result<Value, String> {
        git::git_commit(repo_path, msg, files).await
    }
    pub async fn git_add(&self, repo_path: &str, files: &[&str]) -> Result<Value, String> {
        git::git_add(repo_path, files).await
    }
    pub async fn git_reset(&self, repo_path: &str, file: Option<&str>) -> Result<Value, String> {
        git::git_reset(repo_path, file).await
    }
    pub async fn git_checkout(&self, repo_path: &str, branch: &str) -> Result<Value, String> {
        git::git_checkout(repo_path, branch).await
    }
    pub async fn git_create_branch(&self, repo_path: &str, name: &str, from: Option<&str>) -> Result<Value, String> {
        git::git_create_branch(repo_path, name, from).await
    }
    pub async fn git_delete_branch(&self, repo_path: &str, name: &str, force: bool) -> Result<Value, String> {
        git::git_delete_branch(repo_path, name, force).await
    }
    pub async fn git_merge(&self, repo_path: &str, branch: &str) -> Result<Value, String> {
        git::git_merge(repo_path, branch).await
    }
    pub async fn git_pull(&self, repo_path: &str) -> Result<Value, String> {
        git::git_pull(repo_path).await
    }
    pub async fn git_push(&self, repo_path: &str) -> Result<Value, String> {
        git::git_push(repo_path).await
    }
    pub async fn git_discard_changes(&self, repo_path: &str, file: &str) -> Result<Value, String> {
        git::git_discard_changes(repo_path, file).await
    }
    pub async fn git_fetch(&self, repo_path: &str, remote: Option<&str>) -> Result<Value, String> {
        git::git_fetch(repo_path, remote).await
    }
    pub async fn git_force_push(&self, repo_path: &str) -> Result<Value, String> {
        git::git_force_push(repo_path).await
    }
    pub async fn git_push_tags(&self, repo_path: &str, remote: &str) -> Result<Value, String> {
        git::git_push_tags(repo_path, remote).await
    }
    pub async fn git_delete_remote_branch(&self, repo_path: &str, remote: &str, branch: &str) -> Result<Value, String> {
        git::git_delete_remote_branch(repo_path, remote, branch).await
    }
    pub async fn git_unpushed_commits(&self, repo_path: &str) -> Result<Value, String> {
        git::git_unpushed_commits(repo_path).await
    }
    pub async fn git_incoming_commits(&self, repo_path: &str) -> Result<Value, String> {
        git::git_incoming_commits(repo_path).await
    }
    pub async fn git_checkout_remote_branch(&self, repo_path: &str, remote: &str, branch: &str) -> Result<Value, String> {
        git::git_checkout_remote_branch(repo_path, remote, branch).await
    }
    pub async fn git_rename_branch(&self, repo_path: &str, old: &str, new: &str) -> Result<Value, String> {
        git::git_rename_branch(repo_path, old, new).await
    }
    pub async fn git_stash_save(&self, repo_path: &str, msg: Option<&str>, untracked: bool, keep: bool) -> Result<Value, String> {
        git::git_stash_save(repo_path, msg, untracked, keep).await
    }
    pub async fn git_stash_list(&self, repo_path: &str) -> Result<Value, String> {
        git::git_stash_list(repo_path).await
    }
    pub async fn git_stash_apply(&self, repo_path: &str, stash: Option<&str>) -> Result<Value, String> {
        git::git_stash_apply(repo_path, stash).await
    }
    pub async fn git_stash_pop(&self, repo_path: &str, stash: Option<&str>) -> Result<Value, String> {
        git::git_stash_pop(repo_path, stash).await
    }
    pub async fn git_stash_drop(&self, repo_path: &str, stash: Option<&str>) -> Result<Value, String> {
        git::git_stash_drop(repo_path, stash).await
    }
    pub async fn git_stash_show(&self, repo_path: &str, stash: Option<&str>) -> Result<Value, String> {
        git::git_stash_show(repo_path, stash).await
    }
    pub async fn git_cherry_pick(&self, repo_path: &str, hash: &str, no_commit: bool) -> Result<Value, String> {
        git::git_cherry_pick(repo_path, hash, no_commit).await
    }
    pub async fn git_revert(&self, repo_path: &str, hash: &str, no_commit: bool) -> Result<Value, String> {
        git::git_revert(repo_path, hash, no_commit).await
    }
    pub async fn git_list_tags(&self, repo_path: &str) -> Result<Value, String> {
        git::git_list_tags(repo_path).await
    }
    pub async fn git_create_tag(&self, repo_path: &str, name: &str, msg: Option<&str>, force: bool) -> Result<Value, String> {
        git::git_create_tag(repo_path, name, msg, force).await
    }
    pub async fn git_delete_tag(&self, repo_path: &str, name: &str) -> Result<Value, String> {
        git::git_delete_tag(repo_path, name).await
    }
    pub async fn git_file_history(&self, repo_path: &str, file: &str, limit: Option<usize>) -> Result<Value, String> {
        git::git_file_history(repo_path, file, limit).await
    }
    pub async fn git_compare_branches(&self, repo_path: &str, target: &str, source: Option<&str>) -> Result<Value, String> {
        git::git_compare_branches(repo_path, target, source).await
    }
    pub async fn git_rebase(&self, repo_path: &str, target: &str, onto: Option<&str>) -> Result<Value, String> {
        git::git_rebase(repo_path, target, onto).await
    }
    pub async fn git_rebase_abort(&self, repo_path: &str) -> Result<Value, String> {
        git::git_rebase_abort(repo_path).await
    }
    pub async fn git_rebase_continue(&self, repo_path: &str) -> Result<Value, String> {
        git::git_rebase_continue(repo_path).await
    }
    pub async fn git_file_blame(&self, repo_path: &str, file: &str) -> Result<Value, String> {
        git::git_file_blame(repo_path, file).await
    }
    pub async fn git_conflict_files(&self, repo_path: &str) -> Result<Value, String> {
        git::git_conflict_files(repo_path).await
    }
    pub async fn git_accept_conflict(&self, repo_path: &str, file: &str, strategy: &str) -> Result<Value, String> {
        git::git_accept_conflict(repo_path, file, strategy).await
    }
    pub async fn git_amend_commit(&self, repo_path: &str, msg: &str) -> Result<Value, String> {
        git::git_amend_commit(repo_path, msg).await
    }
    pub async fn git_reset_to_commit(&self, repo_path: &str, hash: &str, mode: &str) -> Result<Value, String> {
        git::git_reset_to_commit(repo_path, hash, mode).await
    }
    pub async fn git_changed_files(&self, repo_path: &str, c1: &str, c2: Option<&str>) -> Result<Value, String> {
        git::git_changed_files(repo_path, c1, c2).await
    }
    pub async fn git_file_at_revision(&self, repo_path: &str, file: &str, rev: &str) -> Result<Value, String> {
        git::git_file_at_revision(repo_path, file, rev).await
    }
    pub async fn git_clean(&self, repo_path: &str, dry: bool, force: bool) -> Result<Value, String> {
        git::git_clean(repo_path, dry, force).await
    }
    pub async fn git_remotes(&self, repo_path: &str) -> Result<Value, String> {
        git::git_remotes(repo_path).await
    }
    pub async fn git_add_remote(&self, repo_path: &str, name: &str, url: &str) -> Result<Value, String> {
        git::git_add_remote(repo_path, name, url).await
    }
    pub async fn git_remove_remote(&self, repo_path: &str, name: &str) -> Result<Value, String> {
        git::git_remove_remote(repo_path, name).await
    }
    pub async fn git_set_remote_url(&self, repo_path: &str, name: &str, url: &str) -> Result<Value, String> {
        git::git_set_remote_url(repo_path, name, url).await
    }
    pub async fn git_undo_last_commit(&self, repo_path: &str) -> Result<Value, String> {
        git::git_undo_last_commit(repo_path).await
    }
    pub async fn git_submodules(&self, repo_path: &str) -> Result<Value, String> {
        git::git_submodules(repo_path).await
    }
    pub async fn git_submodule_list(&self, repo_path: &str) -> Result<Value, String> {
        git::git_submodule_list(repo_path).await
    }
    pub async fn git_submodule_init(&self, repo_path: &str, recursive: bool) -> Result<Value, String> {
        git::git_submodule_init(repo_path, recursive).await
    }
    pub async fn git_exec(&self, repo_path: &str, args: &[&str]) -> Result<Value, String> {
        git::git_exec(repo_path, args).await
    }
}

// ============ Log Aggregator Helper Functions ============

fn build_tail_command(preset: &Value, lines: usize) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");

    match log_type {
        "docker" => {
            let containers: Vec<String> = log_path
                .split('\n')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();
            if containers.is_empty() {
                return "echo 'No containers configured'".to_string();
            }
            containers
                .iter()
                .map(|c| format!("docker logs --tail {} '{}'", lines, c))
                .collect::<Vec<_>>()
                .join(" ; ")
        }
        "journalctl" => {
            let units: Vec<String> = log_path
                .split('\n')
                .filter(|u| !u.trim().is_empty())
                .map(|u| u.trim().to_string())
                .collect();
            if units.is_empty() {
                format!("journalctl -n {} --no-pager 2>/dev/null", lines)
            } else {
                let unit_args: Vec<String> = units.iter().map(|u| format!("-u '{}'", u)).collect();
                format!(
                    "journalctl {} -n {} --no-pager 2>/dev/null",
                    unit_args.join(" "),
                    lines
                )
            }
        }
        _ => {
            let paths: Vec<String> = log_path
                .split('\n')
                .filter(|p| !p.trim().is_empty())
                .map(|p| p.trim().to_string())
                .collect();
            if paths.is_empty() {
                return "echo 'No log paths configured'".to_string();
            }
            let q = |p: &str| format!("'{}'", p.replace('\'', "'\\''"));
            format!("tail -n {} {} 2>/dev/null", lines, paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" "))
        }
    }
}

fn build_grep_command(preset: &Value, keyword: &str, context_lines: usize) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");
    let escaped_kw = keyword.replace('\'', "'\\''");
    let grep_ctx = if context_lines > 0 {
        format!(" -C {}", context_lines)
    } else {
        String::new()
    };
    let grep = format!("grep{} -i -n '{}'", grep_ctx, escaped_kw);

    match log_type {
        "docker" => {
            let containers: Vec<String> = log_path
                .split('\n')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();
            containers
                .iter()
                .map(|c| {
                    format!("docker logs '{}' 2>&1 | {} 2>/dev/null", c, format!("grep{} -i -n '{}'", grep_ctx, escaped_kw))
                })
                .collect::<Vec<_>>()
                .join(" ; ")
        }
        "journalctl" => {
            let units: Vec<String> = log_path
                .split('\n')
                .filter(|u| !u.trim().is_empty())
                .map(|u| u.trim().to_string())
                .collect();
            if units.is_empty() {
                format!("journalctl --grep='{}' --no-pager 2>/dev/null", escaped_kw)
            } else {
                let unit_args: Vec<String> = units
                    .iter()
                    .map(|u| format!("-u '{}'", u.replace('\'', "'\\''")))
                    .collect();
                format!(
                    "journalctl {} --grep='{}' --no-pager 2>/dev/null",
                    unit_args.join(" "),
                    escaped_kw
                )
            }
        }
        _ => {
            let paths: Vec<String> = log_path
                .split('\n')
                .filter(|p| !p.trim().is_empty())
                .map(|p| p.trim().to_string())
                .collect();
            if paths.is_empty() {
                return "echo 'No log paths configured'".to_string();
            }
            let q = |p: &str| format!("'{}'", p.replace('\'', "'\\''"));
            format!("{} {} 2>/dev/null", grep, paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" "))
        }
    }
}

fn parse_grep_output(output: &str, keyword: &str) -> Vec<Value> {
    let kw_lower = keyword.to_lowercase();
    output
        .lines()
        .filter(|l| l.trim().is_empty() || *l == "--")
        .count(); // consume filter

    output
        .lines()
        .filter(|l| !l.trim().is_empty() && *l != "--")
        .filter_map(|line| {
            // grep -n output: "filename:lineNum:content" or "lineNum:content"
            // match line: lineNum:content, context line: lineNum-content
            let match_line = regex_match(line, r"^(?:[^:]*:)?(\d+):(.*)$");
            let context_line = regex_match(line, r"^(?:[^:]*:)?(\d+)-(.*)$");
            let parsed = match_line.or(context_line);

            parsed.map(|(line_num, content)| {
                // Strip ANSI color codes
                let content = content.replace("\x1b[0m", "").replace("\x1b[31m", "").replace("\x1b[32m", "");
                let is_match = content.to_lowercase().contains(&kw_lower);
                json!({
                    "content": content,
                    "isMatch": is_match,
                    "lineNum": line_num
                })
            })
        })
        .collect()
}

fn regex_match(line: &str, pattern: &str) -> Option<(String, String)> {
    if pattern.contains(r"^(\d+):(.*)$") {
        if let Some(pos) = line.find(':') {
            let num_part = &line[..pos];
            if num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
    }
    if pattern.contains(r"^(\d+)-(.*)$") {
        if let Some(pos) = line.find('-') {
            let num_part = &line[..pos];
            if num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
    }
    if pattern.contains(r"^(?:[^:]*:)?(\d+):(.*)$") {
        // "filename:lineNum:content" or "lineNum:content"
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 2 {
            if parts.len() == 2 {
                // "lineNum:content"
                if parts[0].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            } else {
                // "filename:lineNum:content"
                if parts[1].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[1].to_string(), parts[2].to_string()));
                }
            }
        }
    }
    if pattern.contains(r"^(?:[^:]*:)?(\d+)-(.*)$") {
        // Context line with dash separator
        let parts: Vec<&str> = line.splitn(3, '-').collect();
        if parts.len() >= 2 {
            if parts.len() == 2 {
                if parts[0].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            } else {
                if parts[1].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[1].to_string(), parts[2].to_string()));
                }
            }
        }
    }
    None
}
