/// JSON Router — Handler 注册与派发
///
/// 与 Electron 端 uds-api.ts 的 registerHandler() 1:1 对应。
/// 保证 handler 名称不变，CLI 零修改即可对接。
use crate::commands::database as db_module;
use crate::core::CoreService;
use crate::uds::protocol::{UdsRequest, UdsResponse};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;

/// 在 UDS router 中直接调用 git（用于 apply-patch / interactive-rebase 等需要特殊处理的操作）
async fn run_git(repo: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(crate::core::git::find_git())
        .args(args)
        .current_dir(repo)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(stdout)
}

/// 异步 handler 函数签名
type AsyncHandler = Box<dyn Fn(Arc<CoreService>, Value) -> HandlerFuture + Send + Sync>;
type HandlerFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, String>> + Send>>;

/// JSON Router — 将 handler 名称映射到处理函数
pub struct JsonRouter {
    handlers: HashMap<String, Arc<AsyncHandler>>,
}

impl JsonRouter {
    pub fn new() -> Self {
        let mut router = Self {
            handlers: HashMap::new(),
        };
        router.register_default_handlers();
        router
    }

    /// 注册一个 handler
    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(Arc<CoreService>, Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, String>> + Send + 'static,
    {
        self.handlers.insert(
            name.to_string(),
            Arc::new(Box::new(move |core, params| {
                Box::pin(handler(core, params))
            })),
        );
    }

    /// 派发请求到对应 handler
    pub async fn dispatch(&self, req: &UdsRequest, core: Arc<CoreService>) -> UdsResponse {
        log::debug!("[Router] Dispatching handler: {}", req.handler);
        let Some(handler) = self.handlers.get(&req.handler) else {
            return UdsResponse::err(format!("Handler '{}' not found", req.handler));
        };

        let params = req.params.clone().unwrap_or(Value::Null);
        match handler(core, params).await {
            Ok(data) => UdsResponse::ok(data),
            Err(msg) => UdsResponse::err(msg),
        }
    }

    /// 返回已注册的 handler 数量
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    // ============ 默认 Handler 注册 ============
    // 这里注册所有 CLI 需要的 handler，调用 core 层的业务逻辑

    fn register_default_handlers(&mut self) {
        let router = &mut *self;

        // ===== Todos =====
        router.register("todos:get-all", |core, _params| async move {
            core.get_all_todos().await
        });
        router.register("todos:add", |core, params| async move {
            core.add_todo(params).await
        });
        router.register("todos:update", |core, params| async move {
            core.update_todo(params).await
        });
        router.register("todos:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_todo(&id).await
        });

        // ===== Subtasks =====
        router.register("subtasks:get-for-todo", |core, params| async move {
            let todo_id = params["todoId"].as_str().unwrap_or("").to_string();
            core.get_subtasks_for_todo(&todo_id).await
        });
        router.register("subtasks:add", |core, params| async move {
            core.add_subtask(params).await
        });
        router.register("subtasks:update", |core, params| async move {
            core.update_subtask(params).await
        });
        router.register("subtasks:delete", |core, params| async move {
            let id = params["subtaskId"].as_str().unwrap_or("").to_string();
            core.delete_subtask(&id).await
        });

        // ===== Tags =====
        router.register("tags:get-all", |core, _params| async move {
            core.get_all_tags().await
        });
        router.register("tags:add", |core, params| async move {
            let name = params["name"].as_str().unwrap_or("").to_string();
            core.add_tag(&name).await
        });
        router.register("tags:delete", |core, params| async move {
            let name = params["name"].as_str().unwrap_or("").to_string();
            core.delete_tag(&name).await
        });

        // ===== Settings =====
        router.register("settings:get", |core, params| async move {
            let key = params["key"].as_str().unwrap_or("").to_string();
            core.get_setting(&key).await
        });
        router.register("settings:set", |core, params| async move {
            let key = params["key"].as_str().unwrap_or("").to_string();
            let value = params["value"].as_str().unwrap_or("").to_string();
            core.set_setting(&key, &value).await
        });

        // ===== Projects =====
        router.register("projects:get-all", |core, params| async move {
            let only_active = params
                .get("onlyActive")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            core.get_all_projects(only_active).await
        });
        router.register("projects:add", |core, params| async move {
            core.add_project(params).await
        });
        router.register("projects:update", |core, params| async move {
            core.update_project(params).await
        });
        router.register("projects:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_project(&id).await
        });
        router.register("projects:get-stats", |core, params| async move {
            let id = params["projectId"].as_str().unwrap_or("").to_string();
            core.get_project_stats(&id).await
        });
        router.register("projects:get-todos", |core, params| async move {
            let id = params["projectId"].as_str().unwrap_or("").to_string();
            core.get_project_todos(&id).await
        });

        // ===== Servers =====
        router.register("servers:get-all", |core, _params| async move {
            core.get_all_servers().await
        });
        router.register("servers:get-by-id", |core, params| async move {
            let id = params["serverId"].as_str().unwrap_or("").to_string();
            core.get_server_by_id(&id).await
        });
        router.register("servers:add", |core, params| async move {
            core.add_server(params).await
        });
        router.register("servers:update", |core, params| async move {
            core.update_server(params).await
        });
        router.register("servers:delete", |core, params| async move {
            let id = params["serverId"].as_str().unwrap_or("").to_string();
            core.delete_server(&id).await
        });
        router.register("servers:test-connection", |core, params| async move {
            core.test_server_connection(params).await
        });
        router.register("servers:groups:get-all", |core, _params| async move {
            core.get_all_server_groups().await
        });
        router.register("servers:groups:add", |core, params| async move {
            core.add_server_group(params).await
        });
        router.register("servers:groups:update", |core, params| async move {
            let id = params["groupId"].as_str().unwrap_or("").to_string();
            core.update_server_group(&id, params).await
        });
        router.register("servers:groups:delete", |core, params| async move {
            let id = params["groupId"].as_str().unwrap_or("").to_string();
            core.delete_server_group(&id).await
        });

        // ===== MFA/OTP =====
        router.register("mfa:get-secrets", |core, _params| async move {
            core.get_all_mfa_secrets().await
        });
        router.register("mfa:add-secret", |core, params| async move {
            core.add_mfa_secret(params).await
        });
        router.register("mfa:update-secret", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_mfa_secret(&id, params).await
        });
        router.register("mfa:delete-secret", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_mfa_secret(&id).await
        });
        router.register("mfa:generate-code", |core, params| async move {
            let secret = params["secret"].as_str().unwrap_or("").to_string();
            let digits = params["digits"].as_u64().unwrap_or(6) as u32;
            let period = params["period"].as_u64().unwrap_or(30) as u32;
            let algorithm = params["algorithm"].as_str().unwrap_or("SHA1").to_string();
            core.generate_totp(&secret, digits, period, &algorithm)
                .await
        });

        // ===== Notes =====
        router.register("notes:get-all", |core, params| async move {
            let query = params["query"].as_str().map(|s| s.to_string());
            let group_id = params["groupId"].as_str().map(|s| s.to_string());
            core.get_all_notes(query, group_id).await
        });
        router.register("notes:add", |core, params| async move {
            core.add_note(params).await
        });
        router.register("notes:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_note(&id, params).await
        });
        router.register("notes:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_note(&id).await
        });

        // ===== Notes Groups =====
        router.register("note-groups:get-all", |core, _params| async move {
            core.get_all_note_groups().await
        });
        router.register("note-groups:add", |core, params| async move {
            core.add_note_group(params).await
        });
        router.register("note-groups:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_note_group(&id, params).await
        });
        router.register("note-groups:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_note_group(&id).await
        });

        // ===== Weekly Reports =====
        router.register("weekly-report:get-all", |core, params| async move {
            let limit = params["limit"].as_u64().unwrap_or(20) as usize;
            core.get_weekly_reports(limit).await
        });
        router.register("weekly-report:get", |core, params| async move {
            let id = params["id"].as_i64().unwrap_or(0);
            core.get_weekly_report(id).await
        });
        router.register("weekly-report:save", |core, params| async move {
            core.save_weekly_report(params).await
        });

        // ===== Notification =====
        router.register("notification:get-settings", |core, _params| async move {
            core.get_notification_settings().await
        });
        router.register("notification:set-settings", |core, params| async move {
            core.set_notification_settings(params).await
        });

        // ===== App =====
        router.register("app:get-version", |_core, _params| async move {
            Ok(serde_json::json!(env!("CARGO_PKG_VERSION")))
        });

        // ===== OpenVPN =====
        router.register("openvpn:get-all", |core, _params| async move {
            let configs = core.db_read(|conn| {
                crate::db::openvpn::get_all(conn).map_err(|e| e.to_string())
            })?;
            Ok(serde_json::to_value(configs).map_err(|e| e.to_string())?)
        });
        router.register("openvpn:add", |core, params| async move {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file_path = params.get("filePath").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let id = core.db_write(|conn| {
                crate::db::openvpn::add(conn, &name, &file_path, &content).map_err(|e| e.to_string())
            })?;
            Ok(serde_json::json!({ "id": id }))
        });
        router.register("openvpn:delete", |core, params| async move {
            let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let _ = core.db_write(|conn| {
                crate::db::openvpn::delete(conn, &id).map_err(|e| e.to_string())
            })?;
            Ok(serde_json::json!({ "success": true }))
        });

        // ===== Accounting =====
        router.register("accounting:categories:get", |core, _params| async move {
            core.get_accounting_categories().await
        });
        router.register("accounting:categories:add", |core, params| async move {
            core.add_accounting_category(params).await
        });
        router.register("accounting:categories:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_accounting_category(&id, params).await
        });
        router.register("accounting:categories:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_accounting_category(&id).await
        });
        router.register("accounting:records:get", |core, params| async move {
            core.get_accounting_records(params).await
        });
        router.register("accounting:records:add", |core, params| async move {
            core.add_accounting_record(params).await
        });
        router.register("accounting:records:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_accounting_record(&id, params).await
        });
        router.register("accounting:records:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_accounting_record(&id).await
        });
        router.register("accounting:stats:get", |core, params| async move {
            core.get_accounting_stats(params).await
        });
        router.register("accounting:budgets:get", |core, _params| async move {
            core.get_budgets().await
        });
        router.register("accounting:budgets:add", |core, params| async move {
            core.add_budget(params).await
        });
        router.register("accounting:budgets:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_budget(&id, params).await
        });
        router.register("accounting:budgets:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_budget(&id).await
        });
        router.register("accounting:templates:get", |core, _params| async move {
            core.get_templates().await
        });
        router.register("accounting:templates:add", |core, params| async move {
            core.add_template(params).await
        });
        router.register("accounting:templates:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_template(&id, params).await
        });
        router.register("accounting:templates:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_template(&id).await
        });
        router.register("accounting:templates:use", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.use_template(&id).await
        });
        router.register("accounting:trend:get", |core, params| async move {
            let months = params["months"].as_u64().unwrap_or(12) as usize;
            core.get_accounting_trend(months).await
        });

        // ===== Backup =====
        router.register("backup:export-data", |core, _params| async move {
            core.export_all_data().await
        });
        router.register("backup:import-json", |core, params| async move {
            let data = params["data"].clone();
            let mode = params["mode"].as_str().unwrap_or("merge").to_string();
            core.import_all_data(data, &mode).await
        });

        // ===== Log Presets =====
        router.register("log-presets:get-all", |core, _params| async move {
            core.get_log_presets().await
        });
        router.register("log-presets:add", |core, params| async move {
            core.add_log_preset(params).await
        });
        router.register("log-presets:update", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.update_log_preset(&id, params).await
        });
        router.register("log-presets:delete", |core, params| async move {
            let id = params["id"].as_str().unwrap_or("").to_string();
            core.delete_log_preset(&id).await
        });

        // ===== Log Aggregator (non-streaming) =====
        router.register("log:tail", |core, params| async move {
            let preset_id = params["presetId"].as_str().unwrap_or("").to_string();
            let lines = params["lines"].as_u64().unwrap_or(100) as usize;
            core.log_tail(&preset_id, lines).await
        });
        router.register("log:search", |core, params| async move {
            let preset_id = params["presetId"].as_str().unwrap_or("").to_string();
            let keyword = params["keyword"].as_str().unwrap_or("").to_string();
            let lines = params["lines"].as_u64().unwrap_or(50) as usize;
            core.log_search(&preset_id, &keyword, lines).await
        });

        // ===== Misc =====
        router.register("get-app-path", |core, _params| async move {
            core.get_app_path().await
        });
        router.register("file:read-content", |core, params| async move {
            let path = params["filePath"].as_str().unwrap_or("").to_string();
            core.read_file_content(&path).await
        });
        router.register("fs:readdir", |core, params| async move {
            let path = params["dirPath"].as_str().unwrap_or("").to_string();
            core.read_directory(&path).await
        });

        // ===== Update =====
        router.register("update:get-version", |_core, _params| async move {
            Ok(Value::String("0.1.0-tauri".to_string()))
        });

        // ===== SSH / Server Operations =====
        router.register("servers:exec", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            let command = params["command"]
                .as_str()
                .ok_or("Missing command")?
                .to_string();
            let result = core.exec_ssh_command(&server_id, &command).await?;
            Ok(result)
        });
        router.register("servers:batch-exec", |core, params| async move {
            let command = params["command"]
                .as_str()
                .ok_or("Missing command")?
                .to_string();
            let server_ids: Vec<String> = params["serverIds"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let mut results = Vec::new();
            for sid in &server_ids {
                let result = core.exec_ssh_command(sid, &command).await;
                results.push(json!({"serverId": sid, "result": result}));
            }
            Ok(json!({"results": results}))
        });
        router.register("servers:connect", |core, params| async move {
            core.ssh_connect(params).await
        });
        router.register("servers:disconnect", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            core.ssh_disconnect(&server_id).await
        });
        router.register("servers:is-connected", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            core.ssh_is_connected(&server_id).await
        });

        // ===== SFTP =====
        router.register("servers:sftp:list", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            let remote_path = params["remotePath"]
                .as_str()
                .ok_or("Missing remotePath")?
                .to_string();
            let files = core.sftp_list_dir(&server_id, &remote_path).await?;
            Ok(files)
        });
        router.register("servers:sftp:read-file", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            let remote_path = params["remotePath"]
                .as_str()
                .ok_or("Missing remotePath")?
                .to_string();
            let content = core.sftp_download_file(&server_id, &remote_path).await?;
            Ok(json!({"success": true, "content": content}))
        });
        router.register("servers:sftp:mkdir", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            let remote_path = params["remotePath"]
                .as_str()
                .ok_or("Missing remotePath")?
                .to_string();
            core.sftp_create_dir(&server_id, &remote_path).await?;
            Ok(json!({"success": true}))
        });
        router.register("servers:sftp:delete", |core, params| async move {
            let server_id = params["serverId"]
                .as_str()
                .ok_or("Missing serverId")?
                .to_string();
            let remote_path = params["remotePath"]
                .as_str()
                .ok_or("Missing remotePath")?
                .to_string();
            core.sftp_delete_file(&server_id, &remote_path).await?;
            Ok(json!({"success": true}))
        });

        // ===== PTY Terminal =====
        router.register("servers:terminal:create", |core, params| async move {
            let server_id = params["serverId"].as_str().ok_or("Missing serverId")?.to_string();
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            let rows = params["rows"].as_u64().unwrap_or(24) as u32;
            let cols = params["cols"].as_u64().unwrap_or(80) as u32;
            core.ssh_create_terminal(&server_id, &terminal_id, rows, cols).await
        });
        router.register("servers:terminal:read", |core, params| async move {
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            core.ssh_read_terminal(&terminal_id).await
        });
        router.register("servers:terminal:write", |core, params| async move {
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            let data = params["data"].as_str().ok_or("Missing data")?.to_string();
            core.ssh_write_to_terminal(&terminal_id, &data).await
        });
        router.register("servers:terminal:resize", |core, params| async move {
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            let rows = params["rows"].as_u64().unwrap_or(24) as u32;
            let cols = params["cols"].as_u64().unwrap_or(80) as u32;
            core.ssh_resize_terminal(&terminal_id, rows, cols).await
        });
        router.register("servers:terminal:close", |core, params| async move {
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            core.ssh_close_terminal(&terminal_id).await
        });
        router.register("servers:terminal:is-active", |core, params| async move {
            let terminal_id = params["terminalId"].as_str().ok_or("Missing terminalId")?.to_string();
            let active = core.ssh_is_terminal_active(&terminal_id).await;
            Ok(json!({"success": true, "active": active}))
        });

        // ===== Git Repo Management =====
        router.register("git:repos:get-all", |core, _params| async move {
            let repos = core.db_read(|conn| {
                let mut stmt = match conn.prepare("SELECT * FROM git_repos ORDER BY createdAt DESC") {
                    Ok(s) => s,
                    Err(e) => return Err(e.to_string()),
                };
                let rows = match stmt.query_map([], |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>("id")?,
                        "path": row.get::<_, String>("path")?,
                        "remote": row.get::<_, Option<String>>("remote")?,
                        "branch": row.get::<_, Option<String>>("branch")?,
                        "lastCommit": row.get::<_, Option<String>>("lastCommit")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                }) {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string()),
                };
                rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
            })??;
            Ok(serde_json::json!(repos))
        });
        router.register("git:repos:add", |core, params| async move {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let remote = params.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let branch = params.get("branch").and_then(|v| v.as_str()).unwrap_or("main").to_string();
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let _ = core.db_write(|conn| {
                match conn.execute(
                    "INSERT INTO git_repos (id, path, remote, branch, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)",
                    &[&id as &dyn rusqlite::ToSql, &path, &remote, &branch, &now, &now]
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"success": true, "id": id}))
        });
        router.register("git:repos:delete", |core, params| async move {
            let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let _ = core.db_write(|conn| {
                match conn.execute("DELETE FROM git_repos WHERE id = ?", &[&id as &dyn rusqlite::ToSql]) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"success": true}))
        });
        router.register("git:repos:update", |core, params| async move {
            let id = params.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let remote = params.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let branch = params.get("branch").and_then(|v| v.as_str()).unwrap_or("main").to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let _ = core.db_write(|conn| {
                match conn.execute(
                    "UPDATE git_repos SET path=?, remote=?, branch=?, updatedAt=? WHERE id=?",
                    &[&path as &dyn rusqlite::ToSql, &remote, &branch, &now, &id]
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"success": true}))
        });

        // ===== Git Operations (70+ handlers via git CLI) =====
        router.register("git:status", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path"))
                .and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_status(&repo).await
        });
        router.register("git:log", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path"))
                .and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let limit = params.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            core.git_log(&repo, limit).await
        });
        router.register("git:log-graph", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            core.git_log(&repo, Some(limit)).await
        });
        router.register("git:exec", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let args: Vec<&str> = params["args"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            core.git_exec(&repo, &args).await
        });
        router.register("git:branches", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_branches(&repo).await
        });
        router.register("git:current-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_current_branch(&repo).await
        });
        router.register("git:diff", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params.get("file").and_then(|v| v.as_str());
            if let Some(f) = file {
                core.git_diff(&repo, Some(f)).await
            } else {
                core.git_diff(&repo, None).await
            }
        });
        router.register("git:commit-diff", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            core.git_commit_diff(&repo, &hash).await
        });
        router.register("git:commit", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let msg = params["message"].as_str().ok_or("Missing message")?.to_string();
            let files: Option<Vec<&str>> = params.get("files").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect());
            core.git_commit(&repo, &msg, files.as_deref()).await
        });
        router.register("git:add", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let files: Vec<&str> = match params.get("files") {
                Some(Value::Array(arr)) => arr.iter().filter_map(|v| v.as_str()).collect(),
                Some(Value::String(s)) => vec![s.as_str()],
                _ => vec!["."],
            };
            core.git_add(&repo, &files).await
        });
        router.register("git:reset", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params.get("file").and_then(|v| v.as_str());
            core.git_reset(&repo, file).await
        });
        router.register("git:checkout", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let branch = params["branch"].as_str().ok_or("Missing branch")?.to_string();
            core.git_checkout(&repo, &branch).await
        });
        router.register("git:create-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["branchName"].as_str().ok_or("Missing branchName")?.to_string();
            let from = params.get("from").and_then(|v| v.as_str());
            core.git_create_branch(&repo, &name, from).await
        });
        router.register("git:delete-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["branchName"].as_str().ok_or("Missing branchName")?.to_string();
            let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_delete_branch(&repo, &name, force).await
        });
        router.register("git:merge", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let branch = params["branch"].as_str().ok_or("Missing branch")?.to_string();
            core.git_merge(&repo, &branch).await
        });
        router.register("git:pull", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_pull(&repo).await
        });
        router.register("git:push", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_push(&repo).await
        });
        router.register("git:discard-changes", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params["file"].as_str().ok_or("Missing file")?.to_string();
            core.git_discard_changes(&repo, &file).await
        });
        router.register("git:fetch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let remote = params.get("remote").and_then(|v| v.as_str());
            core.git_fetch(&repo, remote).await
        });
        router.register("git:force-push", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_force_push(&repo).await
        });
        router.register("git:push-tags", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let remote = params.get("remote").and_then(|v| v.as_str()).unwrap_or("origin");
            core.git_push_tags(&repo, remote).await
        });
        router.register("git:delete-remote-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let remote = params.get("remote").and_then(|v| v.as_str()).unwrap_or("origin");
            let branch = params["branchName"].as_str().ok_or("Missing branchName")?.to_string();
            core.git_delete_remote_branch(&repo, remote, &branch).await
        });
        router.register("git:unpushed-commits", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_unpushed_commits(&repo).await
        });
        router.register("git:incoming-commits", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_incoming_commits(&repo).await
        });
        router.register("git:checkout-remote-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let remote = params["remote"].as_str().ok_or("Missing remote")?.to_string();
            let branch = params["branch"].as_str().ok_or("Missing branch")?.to_string();
            core.git_checkout_remote_branch(&repo, &remote, &branch).await
        });
        router.register("git:rename-branch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let old = params["oldName"].as_str().ok_or("Missing oldName")?.to_string();
            let new = params["newName"].as_str().ok_or("Missing newName")?.to_string();
            core.git_rename_branch(&repo, &old, &new).await
        });
        // Git Stash
        router.register("git:stash-save", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let msg = params.get("message").and_then(|v| v.as_str());
            let untracked = params.get("includeUntracked").and_then(|v| v.as_bool()).unwrap_or(false);
            let keep = params.get("keepIndex").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_stash_save(&repo, msg, untracked, keep).await
        });
        router.register("git:stash-list", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_stash_list(&repo).await
        });
        router.register("git:stash-apply", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let stash = params.get("stashRef").and_then(|v| v.as_str());
            core.git_stash_apply(&repo, stash).await
        });
        router.register("git:stash-pop", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let stash = params.get("stashRef").and_then(|v| v.as_str());
            core.git_stash_pop(&repo, stash).await
        });
        router.register("git:stash-drop", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let stash = params.get("stashRef").and_then(|v| v.as_str());
            core.git_stash_drop(&repo, stash).await
        });
        router.register("git:stash-show", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let stash = params.get("stashRef").and_then(|v| v.as_str());
            core.git_stash_show(&repo, stash).await
        });
        // Git Cherry-pick & Revert
        router.register("git:cherry-pick", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            let no_commit = params.get("noCommit").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_cherry_pick(&repo, &hash, no_commit).await
        });
        router.register("git:revert", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            let no_commit = params.get("noCommit").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_revert(&repo, &hash, no_commit).await
        });
        // Git Tag
        router.register("git:tag-list", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_list_tags(&repo).await
        });
        router.register("git:tag-create", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["tagName"].as_str().ok_or("Missing tagName")?.to_string();
            let msg = params.get("message").and_then(|v| v.as_str());
            let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_create_tag(&repo, &name, msg, force).await
        });
        router.register("git:tag-delete", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["tagName"].as_str().ok_or("Missing tagName")?.to_string();
            core.git_delete_tag(&repo, &name).await
        });
        // Git File History & Compare
        router.register("git:file-history", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params["filePath"].as_str().ok_or("Missing filePath")?.to_string();
            let limit = params.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            core.git_file_history(&repo, &file, limit).await
        });
        router.register("git:compare-branches", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let target = params["targetBranch"].as_str().ok_or("Missing targetBranch")?.to_string();
            let source = params.get("sourceBranch").and_then(|v| v.as_str());
            core.git_compare_branches(&repo, &target, source).await
        });
        // Git Rebase
        router.register("git:rebase", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let target = params["targetBranch"].as_str().ok_or("Missing targetBranch")?.to_string();
            let onto = params.get("onto").and_then(|v| v.as_str());
            core.git_rebase(&repo, &target, onto).await
        });
        router.register("git:rebase-abort", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_rebase_abort(&repo).await
        });
        router.register("git:rebase-continue", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_rebase_continue(&repo).await
        });
        // Git Blame & Conflict
        router.register("git:file-blame", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params["filePath"].as_str().ok_or("Missing filePath")?.to_string();
            core.git_file_blame(&repo, &file).await
        });
        router.register("git:conflict-files", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_conflict_files(&repo).await
        });
        router.register("git:accept-conflict", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params["file"].as_str().ok_or("Missing file")?.to_string();
            let strategy = params["strategy"].as_str().unwrap_or("ours");
            core.git_accept_conflict(&repo, &file, strategy).await
        });
        // Git Advanced
        router.register("git:amend-commit", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let msg = params.get("message").and_then(|v| v.as_str()).unwrap_or("");
            core.git_amend_commit(&repo, msg).await
        });
        router.register("git:reset-to-commit", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("mixed");
            core.git_reset_to_commit(&repo, &hash, mode).await
        });
        router.register("git:changed-files", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let c1 = params.get("commit1").and_then(|v| v.as_str()).unwrap_or("");
            let c2 = params.get("commit2").and_then(|v| v.as_str());
            core.git_changed_files(&repo, c1, c2).await
        });
        router.register("git:file-at-revision", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let file = params["filePath"].as_str().ok_or("Missing filePath")?.to_string();
            let rev = params["revision"].as_str().ok_or("Missing revision")?.to_string();
            core.git_file_at_revision(&repo, &file, &rev).await
        });
        router.register("git:diff-file-revision", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let _file = params["filePath"].as_str().ok_or("Missing filePath")?.to_string();
            let rev1 = params.get("rev1").and_then(|v| v.as_str()).unwrap_or("HEAD");
            if let Some(rev2) = params.get("rev2").and_then(|v| v.as_str()) {
                core.git_commit_diff(&repo, &format!("{}..{}", rev1, rev2)).await
            } else {
                core.git_commit_diff(&repo, rev1).await
            }
        });
        router.register("git:compare-commits", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let c1 = params["commit1"].as_str().ok_or("Missing commit1")?.to_string();
            let c2 = params["commit2"].as_str().ok_or("Missing commit2")?.to_string();
            core.git_commit_diff(&repo, &format!("{}..{}", c1, c2)).await
        });
        router.register("git:create-patch", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let range = params["commitRange"].as_str().ok_or("Missing commitRange")?.to_string();
            core.git_commit_diff(&repo, &range).await
        });
        router.register("git:apply-patch", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let content = params["patchContent"].as_str().ok_or("Missing patchContent")?.to_string();
            // Write patch content to temp file and apply
            use std::io::Write;
            let tmp = std::env::temp_dir().join("patch.diff");
            let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
            run_git(&repo, &["apply", tmp.to_str().unwrap()]).await?;
            let _ = std::fs::remove_file(&tmp);
            Ok(json!({"success": true}))
        });
        router.register("git:cherry-pick-multiple", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hashes: Vec<String> = params["hashes"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let no_commit = params.get("noCommit").and_then(|v| v.as_bool()).unwrap_or(false);
            for hash in &hashes {
                core.git_cherry_pick(&repo, hash, no_commit).await?;
            }
            Ok(json!({"success": true}))
        });
        router.register("git:branch-from-tag", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["branchName"].as_str().ok_or("Missing branchName")?.to_string();
            let tag = params["tagName"].as_str().ok_or("Missing tagName")?.to_string();
            core.git_create_branch(&repo, &name, Some(&tag)).await
        });
        router.register("git:submodules", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_submodules(&repo).await
        });
        router.register("git:submodule-list", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_submodule_list(&repo).await
        });
        router.register("git:submodule-init", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_submodule_init(&repo, recursive).await
        });
        router.register("git:commit-count", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let branch = params.get("branch").and_then(|v| v.as_str()).unwrap_or("HEAD");
            let output = run_git(&repo, &["rev-list", "--count", branch]).await.unwrap_or_default();
            Ok(json!({"count": output.trim().parse::<u64>().unwrap_or(0)}))
        });
        router.register("git:clean", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let dry = params.get("dryRun").and_then(|v| v.as_bool()).unwrap_or(false);
            let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            core.git_clean(&repo, dry, force).await
        });
        router.register("git:remotes", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_remotes(&repo).await
        });
        router.register("git:add-remote", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["name"].as_str().ok_or("Missing name")?.to_string();
            let url = params["url"].as_str().ok_or("Missing url")?.to_string();
            core.git_add_remote(&repo, &name, &url).await
        });
        router.register("git:remove-remote", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["name"].as_str().ok_or("Missing name")?.to_string();
            core.git_remove_remote(&repo, &name).await
        });
        router.register("git:set-remote-url", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let name = params["name"].as_str().ok_or("Missing name")?.to_string();
            let url = params["url"].as_str().ok_or("Missing url")?.to_string();
            core.git_set_remote_url(&repo, &name, &url).await
        });
        router.register("git:add-gitignore", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let pattern = params["pattern"].as_str().ok_or("Missing pattern")?.to_string();
            use std::io::Write;
            let gitignore = Path::new(&repo).join(".gitignore");
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&gitignore).map_err(|e| e.to_string())?;
            writeln!(f, "{}", pattern).map_err(|e| e.to_string())?;
            Ok(json!({"success": true}))
        });
        router.register("git:get-gitignore", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let gitignore = Path::new(&repo).join(".gitignore");
            if gitignore.exists() {
                let content = std::fs::read_to_string(&gitignore).map_err(|e| e.to_string())?;
                Ok(json!({"content": content}))
            } else {
                Ok(json!({"content": ""}))
            }
        });
        router.register("git:undo-last-commit", |core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            core.git_undo_last_commit(&repo).await
        });
        router.register("git:interactive-rebase-list", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            let output = run_git(&repo, &["log", "--format=%H|%ai|%s", &format!("{}..HEAD", hash)]).await?;
            let commits: Vec<Value> = output.lines().filter(|l| !l.is_empty()).filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() >= 3 { Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]})) } else { None }
            }).collect();
            Ok(json!({"commits": commits}))
        });
        router.register("git:interactive-rebase", |_core, params| async move {
            let repo = params.get("repoPath").or_else(|| params.get("path")).and_then(|v| v.as_str()).ok_or("Missing repoPath")?.to_string();
            let hash = params["commitHash"].as_str().ok_or("Missing commitHash")?.to_string();
            let actions: Vec<Value> = params.get("actions").and_then(|v| v.as_array())
                .map(|arr| arr.iter().cloned().collect()).unwrap_or_default();
            let mut todo = String::new();
            for action in actions.iter().rev() {
                let h = action["hash"].as_str().unwrap_or("");
                let a = action["action"].as_str().unwrap_or("pick");
                todo.push_str(&format!("{} {}\n", a, h));
            }
            let script = format!("#!/bin/bash\necho '{}' > \"$1\"", todo.trim().replace('\'', "'\\''"));
            let script_path = std::env::temp_dir().join("git-rebase-editor.sh");
            std::fs::write(&script_path, &script).map_err(|e| e.to_string())?;
            std::fs::set_permissions(&script_path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).map_err(|e| e.to_string())?;
            let result = Command::new(crate::core::git::find_git())
                .args(["-c", &format!("core.editor={}", script_path.to_str().unwrap()), "rebase", "-i", &hash])
                .current_dir(&repo)
                .output().await;
            let _ = std::fs::remove_file(&script_path);
            let output = result.map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }
            Ok(json!({"success": true}))
        });

        // ===== Database Operations (CLI → UDS → db module) =====
        router.register("db:list", |core, _params| async move {
            let setting = core.get_setting("db_connections").await
                .unwrap_or(serde_json::json!(null));
            let connections: Vec<serde_json::Value> = match setting {
                serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
                serde_json::Value::Array(arr) => arr,
                _ => vec![],
            };
            let connections: Vec<serde_json::Value> = connections.into_iter().map(|mut c| {
                if let Some(obj) = c.as_object_mut() {
                    obj.remove("password");
                    let ra = obj.get("requiresApproval").and_then(|v| v.as_bool()).unwrap_or(false);
                    obj.insert("requiresApproval".to_string(), serde_json::Value::Bool(ra));
                }
                c
            }).collect();
            Ok(serde_json::json!({"success": true, "connections": connections}))
        });
        router.register("db:connect", |_core, params| async move {
            let id = params["id"].as_str().ok_or("Missing id")?.to_string();
            let db_type = params["type"].as_str().ok_or("Missing type")?.to_string();
            let host = params["host"].as_str().ok_or("Missing host")?.to_string();
            let port = params["port"].as_u64().unwrap_or(0) as i64;
            let username = params.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let password = params.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
            let db_name = params.get("dbName").and_then(|v| v.as_str()).map(|s| s.to_string());
            let db_index = params.get("dbIndex").and_then(|v| v.as_i64());

            let config = db_module::DbConnectionConfig {
                id: id.clone(),
                name: params.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                db_type,
                host,
                port,
                username,
                password,
                db_name,
                db_index,
                path: params.get("path").and_then(|v| v.as_str()).map(String::from),
            };

            match config.db_type.as_str() {
                "mysql" => db_module::test_mysql(&config).await,
                "postgres" => db_module::test_postgres(&config).await,
                "redis" => db_module::test_redis(&config).await,
                other => Err(format!("Unsupported database type: {}", other)),
            }
        });
        router.register("db:disconnect", |_core, params| async move {
            let id = params["id"].as_str().ok_or("Missing id")?.to_string();
            db_module::db_disconnect(id).await
        });
        router.register("db:query", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let sql = params["sql"].as_str().ok_or("Missing sql")?.to_string();
            db_module::db_query(id, sql).await
        });
        router.register("db:get-tables", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let db_name = params.get("dbName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            db_module::db_get_tables(id, db_name).await
        });
        router.register("db:get-databases", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            db_module::db_get_databases(id).await
        });
        router.register("db:get-table-structure", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let db_name = params.get("dbName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let table = params["tableName"].as_str().ok_or("Missing tableName")?.to_string();
            db_module::db_get_table_structure(id, db_name, table).await
        });
        router.register("db:get-table-data", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let db_name = params.get("dbName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let table = params["tableName"].as_str().ok_or("Missing tableName")?.to_string();
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            db_module::db_get_table_data(id, db_name, table, limit, offset, params.get("orderBy").and_then(|v| v.as_str()).map(String::from), params.get("orderDir").and_then(|v| v.as_str()).map(String::from)).await
        });
        router.register("db:redis-scan", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let pattern = params.get("pattern").and_then(|v| v.as_str()).unwrap_or("*").to_string();
            let count = params.get("count").and_then(|v| v.as_u64()).unwrap_or(100);
            db_module::db_redis_scan(id, pattern, count as usize).await
        });
        router.register("db:redis-get", |_core, params| async move {
            let id = params["connectionId"].as_str().ok_or("Missing connectionId")?.to_string();
            let key = params["key"].as_str().ok_or("Missing key")?.to_string();
            db_module::db_redis_get(id, key).await
        });

        // =================== CI/CD ===================
        use crate::commands::cicd;
        use crate::db::{CicdConfig, DeployModule};

        router.register("cicd:detect-tools", |_core, _params| async move {
            Ok(serde_json::to_value(cicd::detect_tools_impl()).map_err(|e| e.to_string())?)
        });

        router.register("cicd:detect-tool-paths", |_core, _params| async move {
            Ok(serde_json::to_value(cicd::detect_tool_paths_impl()).map_err(|e| e.to_string())?)
        });

        router.register("cicd:scan-project", |_core, params| async move {
            let local_path = params["localPath"].as_str().ok_or("Missing localPath")?.to_string();
            Ok(serde_json::to_value(cicd::scan_project_impl(&local_path)).map_err(|e| e.to_string())?)
        });

        // CRUD: configs
        router.register("cicd:get-all-configs", |core, _params| async move {
            let configs = core.db_read(|conn| {
                crate::db::get_all_cicd_configs(conn)
                    .map_err(|e| e.to_string())
            })??;
            Ok(serde_json::to_value(&configs).map_err(|e| e.to_string())?)
        });

        router.register("cicd:get-groups", |core, _params| async move {
            let groups = core.db_read(|conn| crate::db::get_cicd_groups(conn).map_err(|e| e.to_string()))??;
            Ok(serde_json::to_value(&groups).map_err(|e| e.to_string())?)
        });

        router.register("cicd:get-config", |core, params| async move {
            let project_id = params["projectId"].as_str().ok_or("Missing projectId")?.to_string();
            let config = core.db_read(|conn| crate::db::get_cicd_config(conn, &project_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
        });

        router.register("cicd:get-config-by-id", |core, params| async move {
            let config_id = params["configId"].as_str().ok_or("Missing configId")?.to_string();
            let config = core.db_read(|conn| crate::db::get_cicd_config_by_config_id(conn, &config_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
        });

        router.register("cicd:add-config", |core, params| async move {
            let config: CicdConfig = serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            let result = core.db_write(|conn| crate::db::add_cicd_config(conn, &config).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        });

        router.register("cicd:update-config", |core, params| async move {
            let config: CicdConfig = serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            let result = core.db_write(|conn| crate::db::update_cicd_config(conn, &config).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        });

        router.register("cicd:delete-config", |core, params| async move {
            let config_id = params["configId"].as_str().ok_or("Missing configId")?.to_string();
            let _ = core.db_write(|conn| crate::db::delete_cicd_config(conn, &config_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(json!({"success": true}))
        });

        // CRUD: modules
        router.register("cicd:get-modules", |core, params| async move {
            let config_id = params["configId"].as_str().ok_or("Missing configId")?.to_string();
            let modules = core.db_read(|conn| crate::db::get_deploy_modules(conn, &config_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(modules).map_err(|e| e.to_string())?)
        });

        router.register("cicd:add-module", |core, params| async move {
            let module: DeployModule = serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            let result = core.db_write(|conn| crate::db::add_deploy_module(conn, &module).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        });

        router.register("cicd:update-module", |core, params| async move {
            let module: DeployModule = serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
            let result = core.db_write(|conn| crate::db::update_deploy_module(conn, &module).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
        });

        router.register("cicd:delete-module", |core, params| async move {
            let module_id = params["moduleId"].as_str().ok_or("Missing moduleId")?.to_string();
            let _ = core.db_write(|conn| crate::db::delete_deploy_module(conn, &module_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(json!({"success": true}))
        });

        // CRUD: logs
        router.register("cicd:get-logs", |core, params| async move {
            let project_id = params["projectId"].as_str().ok_or("Missing projectId")?.to_string();
            let limit = params["limit"].as_i64().unwrap_or(50);
            let logs = core.db_read(|conn| crate::db::get_deploy_logs(conn, &project_id, limit).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(logs).map_err(|e| e.to_string())?)
        });

        router.register("cicd:get-step-logs", |core, params| async move {
            let deploy_log_id = params["deployLogId"].as_str().ok_or("Missing deployLogId")?.to_string();
            let steps = core.db_read(|conn| crate::db::get_deploy_step_logs(conn, &deploy_log_id).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(steps).map_err(|e| e.to_string())?)
        });

        router.register("cicd:get-deploy-history", |core, params| async move {
            let project_id = params["projectId"].as_str().ok_or("Missing projectId")?.to_string();
            let limit = params["limit"].as_i64().unwrap_or(20);
            let history = core.db_read(|conn| crate::db::get_deploy_history(conn, &project_id, limit).map_err(|e| e.to_string())).map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(history).map_err(|e| e.to_string())?)
        });

        // CI/CD Deploy execution
        router.register("cicd:deploy", |core, params| async move {
            use crate::core::cicd_deploy;
            use crate::db::DeployLog;

            let config_id = params["configId"].as_str().ok_or("Missing configId")?.to_string();
            let data_dir = crate::core::data_dir::resolve_data_dir()
                .to_string_lossy()
                .to_string();

            // Load config from DB
            let config: Option<CicdConfig> = core.db_read(|conn| crate::db::get_cicd_config_by_config_id(conn, &config_id)).map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
            let config = config.ok_or("未找到CI/CD配置")?;

            // Load modules
            let modules: Vec<crate::db::DeployModule> = core.db_read(|conn| crate::db::get_deploy_modules(conn, &config_id)).map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

            // Build deploy config
            let deploy_config = cicd_deploy::DeployConfig {
                repo_url: config.repo_url.clone().unwrap_or_default(),
                branch: config.deploy_branch.clone(),
                local_path: config.local_path.clone(),
                build_tool: config.build_tool.clone(),
                build_command: config.build_command.clone(),
                build_path: config.build_path.clone(),
                npm_script: config.npm_script.clone(),
                npm_custom_script: config.npm_custom_script.clone(),
                maven_home: config.maven_home.clone(),
                java_home: config.java_home.clone(),
                npm_home: config.npm_home.clone(),
                node_home: config.node_home.clone(),
                maven_profile: if config.maven_profile.is_empty() { None } else { Some(config.maven_profile) },
                maven_settings: config.maven_settings.clone(),
                modules: modules.into_iter().map(|m| cicd_deploy::DeployModuleConfig {
                    name: Some(m.module_name),
                    path: if m.module_path.is_empty() { None } else { Some(m.module_path) },
                    build_path: m.build_path.filter(|s| !s.is_empty()),
                    build_command: m.build_command.filter(|s| !s.is_empty()),
                    build_tool: m.build_tool.filter(|s| !s.is_empty()),
                    output_path: m.output_path.filter(|s| !s.is_empty()),
                    artifact_name: if m.artifact_name.is_empty() { None } else { Some(m.artifact_name) },
                    artifact_type: m.artifact_type,
                    lib_filter_rules: m.lib_filter_rules.filter(|s| !s.is_empty()),
                    deploy_order: m.deploy_order,
                    deploy_path: m.deploy_path.filter(|s| !s.is_empty()),
                    enabled: m.enabled,
                }).collect(),
                skip_tests: true,
                parent_build_mode: config.parent_build_mode,
                parent_build_path: if config.parent_build_path.is_empty() { None } else { Some(config.parent_build_path) },
                servers: config.servers.as_ref()
                    .and_then(|s| serde_json::from_str::<Vec<cicd_deploy::DeployServerConfig>>(s).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|mut srv| {
                        if srv.deploy_dir.is_empty() {
                            srv.deploy_dir = config.deploy_path.clone();
                        }
                        srv
                    })
                    .collect(),
                deploy_dir: config.deploy_path.clone(),
                lib_dir: if config.lib_separate { Some(format!("{}/lib", config.deploy_path)) } else { None },
                restart_script: if config.restart_script.is_empty() { None } else { Some(config.restart_script.clone()) },
                lib_separate: config.lib_separate,
            };

            let deploy_id = chrono::Utc::now().timestamp_millis().to_string();

            // Touch deploy timestamp
            let _ = core.db_write(|conn| crate::db::touch_cicd_config_deploy(conn, &config_id));

            let result = cicd_deploy::execute_deploy(
                &deploy_config,
                &data_dir,
                &deploy_id,
                |_event| { /* progress events — can be wired to WebSocket later */ }
            ).await.map_err(|e| e.to_string())?;

            // Save deploy log to DB
            let status = if result.success { "success" } else { "failed" };
            let log = DeployLog {
                id: deploy_id.clone(),
                project_id: config.project_id.clone(),
                config_id: config_id.clone(),
                status: status.to_string(),
                start_time: chrono::Utc::now().to_rfc3339(),
                end_time: Some(chrono::Utc::now().to_rfc3339()),
                error_message: result.error.clone(),
                progress: if result.success { 100 } else { 0 },
                triggered_by: "manual".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                log_file_path: Some(result.log_file_path.clone()),
                artifact_paths: Some(serde_json::to_string(&result.artifact_paths).unwrap_or_default()),
            };

            let _ = core.db_write(|conn| crate::db::add_deploy_log(conn, &log));

            Ok(json!({
                "success": result.success,
                "deployId": deploy_id,
                "logFilePath": result.log_file_path,
                "artifactPaths": result.artifact_paths,
                "error": result.error,
            }))
        });
    }
}
