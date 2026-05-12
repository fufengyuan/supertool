/// Core Service — 共享业务逻辑层
///
/// Tauri commands 和 CLI 都通过这一层操作数据库和服务。
/// 保证 CLI 和 GUI 走同一套代码路径，行为 100% 一致。

pub mod data_dir;
pub mod git;
pub mod ssh;
pub mod cicd_deploy;
pub mod openvpn;
pub mod wireguard;
pub mod nginx;
pub mod log_sanitizer;

// 拆分后的模块
pub mod todo;
pub mod settings;
pub mod project;
pub mod server;
pub mod mfa;
pub mod notes;
pub mod weekly;
pub mod accounting;
pub mod lan;
pub mod backup;
pub mod cicd_data;
pub mod log_presets;
pub mod cicd_sync;
pub mod file_ops;
pub mod ssh_ops;
pub mod alert;

use crate::db::Database;
use serde_json::{json, Value};
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



    // ============ SSH 包装方法 ============

    /// 在 spawn_blocking 线程池中执行 SSH 操作，避免阻塞 async 运行时
    async fn run_ssh_blocking<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&crate::logic::ssh::SshService) -> Result<T, String> + Send + 'static,
        T: Send + 'static,
    {
        let ssh = self.ssh.clone();
        tokio::task::spawn_blocking(move || f(&ssh))
            .await
            .map_err(|e| format!("SSH 操作失败: {}", e))?
    }

    /// 带重试的 SSH 操作（连接断开时自动重连一次）
    async fn run_ssh_with_retry<F, T>(&self, server_id: &str, f: F) -> Result<T, String>
    where
        F: Fn(&crate::logic::ssh::SshService) -> Result<T, String> + Send + Sync + 'static,
        T: Send + 'static,
    {
        let sid = server_id.to_string();
        let ssh = self.ssh.clone();
        let f = std::sync::Arc::new(f);
        let f2 = f.clone();
        let result = tokio::task::spawn_blocking(move || f(&ssh))
            .await
            .map_err(|e| format!("SSH 操作失败: {}", e))?;

        match result {
            Ok(v) => Ok(v),
            Err(e) if e.contains("连接") || e.contains("未连接") || e.contains("通道")
                || e.contains("broken") || e.contains("Broken")
                || e.contains("reset") || e.contains("Reset")
                || e.contains("socket") || e.contains("Socket")
                || e.contains("timeout") || e.contains("Timed")
                || e.contains("refused") || e.contains("eof")
                || e.contains("EOF") || e.contains("write")
                || e.contains("Write") || e.contains("closed")
                || e.contains("Closed") || e.contains("dead")
                || e.contains("Dead") => {
                log::warn!("[SSH] Operation failed for {}, retrying: {}", server_id, e);
                let ssh2 = self.ssh.clone();
                let sid2 = sid.clone();
                let server = self.with_db(|db| {
                    db.conn()
                        .query_row(
                            "SELECT * FROM servers WHERE id = ?1",
                            rusqlite::params![sid2],
                            |row| {
                                Ok(serde_json::json!({
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
                if let Ok(s) = server {
                    let raw_password = s.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));
                    let ssh_config = crate::logic::ssh::SshServerConfig {
                        id: sid.clone(),
                        name: s["name"].as_str().unwrap_or("").to_string(),
                        host: s["host"].as_str().unwrap_or("").to_string(),
                        port: s["port"].as_u64().unwrap_or(22) as u32,
                        username: s["username"].as_str().unwrap_or("").to_string(),
                        password,
                        ssh_key_path: s.get("sshKeyPath").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    };
                    ssh2.disconnect(&sid);
                    if let Err(re) = ssh2.connect(&ssh_config) {
                        return Err(format!("重连失败: {}", re));
                    }
                    let ssh3 = self.ssh.clone();
                    tokio::task::spawn_blocking(move || f2(&ssh3))
                        .await
                        .map_err(|e| format!("SSH 操作失败: {}", e))?
                } else {
                    Err(e)
                }
            }
            Err(e) => Err(e),
        }
    }

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
    pub async fn git_submodule_update(&self, repo_path: &str, path: &str, recursive: bool) -> Result<Value, String> {
        git::git_submodule_update(repo_path, path, recursive).await
    }
    pub async fn git_submodule_update_all(&self, repo_path: &str, recursive: bool) -> Result<Value, String> {
        git::git_submodule_update_all(repo_path, recursive).await
    }
    pub async fn git_compare_commits(&self, repo_path: &str, c1: &str, c2: &str) -> Result<Value, String> {
        git::git_compare_commits(repo_path, c1, c2).await
    }
    pub async fn git_create_patch(&self, repo_path: &str, c1: &str, c2: &str) -> Result<Value, String> {
        git::git_create_patch(repo_path, c1, c2).await
    }
    pub async fn git_apply_patch(&self, repo_path: &str, patch: &str) -> Result<Value, String> {
        git::git_apply_patch(repo_path, patch).await
    }
    pub async fn git_exec(&self, repo_path: &str, args: &[&str]) -> Result<Value, String> {
        git::git_exec(repo_path, args).await
    }

    pub async fn cicd_deploy(&self, config_id: &str) -> Result<Value, String> {
        // Get CICD config from DB
        let cicd_config = self.db_read(|conn| {
            crate::db::cicd::get_cicd_config_by_config_id(conn, config_id)
                .map_err(|e| e.to_string())
        })??
        .ok_or("CI/CD 配置不存在")?;

        // Check approval requirement
        if cicd_config.requires_approval {
            return Ok(json!({
                "success": false,
                "requiresApproval": true,
                "configName": cicd_config.name
            }));
        }

        // Get deploy modules
        let modules = self.db_read(|conn| {
            crate::db::cicd::get_deploy_modules(conn, config_id).expect("db error")
        })?;

        // Parse server references from config JSON
        let servers: Vec<crate::logic::cicd_deploy::DeployServerConfig> = if let Some(ref servers_str) = cicd_config.servers {
            #[derive(serde::Deserialize)]
            struct ServerRef {
                #[serde(rename = "serverId")]
                server_id: String,
                #[serde(rename = "deployDir")]
                deploy_dir: String,
            }
            let refs: Vec<ServerRef> =
                serde_json::from_str(servers_str).map_err(|e| format!("解析服务器引用失败: {}", e))?;

            refs.into_iter()
                .map(|r| {
                    let server = self.db_read(|conn| {
                        conn.query_row(
                            "SELECT * FROM servers WHERE id = ?",
                            rusqlite::params![r.server_id],
                            crate::db::servers::row_to_server,
                        )
                        .map_err(|e| e.to_string())
                    })??;
                    let password = server.password.map(|pw| crate::encryption::try_decrypt_password(&pw));
                    let base_deploy_dir = if r.deploy_dir.is_empty() {
                        cicd_config.deploy_path.clone()
                    } else {
                        r.deploy_dir
                    };
                    Ok(crate::logic::cicd_deploy::DeployServerConfig {
                        host: server.host,
                        port: server.port as u16,
                        username: server.username,
                        password,
                        private_key: server.ssh_key_path,
                        deploy_dir: base_deploy_dir.clone(),
                        lib_dir: if cicd_config.lib_separate {
                            Some(format!("{}/lib", base_deploy_dir))
                        } else {
                            None
                        },
                        label: Some(server.name),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        } else {
            vec![]
        };

        // Build module configs
        let module_configs: Vec<crate::logic::cicd_deploy::DeployModuleConfig> = modules
            .iter()
            .map(|m| crate::logic::cicd_deploy::DeployModuleConfig {
                name: Some(m.module_name.clone()),
                path: Some(m.module_path.clone()),
                build_path: m.build_path.clone(),
                build_command: m.build_command.clone(),
                build_tool: m.build_tool.clone(),
                output_path: m.output_path.clone(),
                artifact_name: Some(m.artifact_name.clone()),
                artifact_type: m.artifact_type.clone(),
                lib_filter_rules: m.lib_filter_rules.clone(),
                deploy_order: m.deploy_order,
                deploy_path: m.deploy_path.clone(),
                enabled: m.enabled,
            })
            .collect();

        // 从 gitRepoId 解析仓库信息
        let git_repo: Option<crate::db::git_repo::GitRepo> = cicd_config.git_repo_id.as_ref().and_then(|id| {
            self.db_read(|conn| {
                crate::db::git_repo::get_by_id(conn, id)
                    .ok()
                    .flatten()
            }).ok().flatten()
        });
        let deploy_config = crate::logic::cicd_deploy::DeployConfig {
            repo_url: git_repo.as_ref().and_then(|r| r.remote.clone().or(Some(r.path.clone()))).unwrap_or_default(),
            branch: cicd_config.deploy_branch.clone(),
            local_path: git_repo.as_ref().map(|r| r.path.clone()),
            build_tool: cicd_config.build_tool.clone(),
            build_command: cicd_config.build_command.clone(),
            build_path: cicd_config.build_path.clone(),
            npm_script: cicd_config.npm_script.clone(),
            npm_custom_script: cicd_config.npm_custom_script.clone(),
            maven_home: cicd_config.maven_home.clone(),
            java_home: cicd_config.java_home.clone(),
            npm_home: cicd_config.npm_home.clone(),
            node_home: cicd_config.node_home.clone(),
            maven_profile: Some(cicd_config.maven_profile.clone()),
            maven_settings: cicd_config.maven_settings.clone(),
            modules: module_configs,
            skip_tests: true,
            parent_build_mode: cicd_config.parent_build_mode,
            parent_build_path: if cicd_config.parent_build_path.is_empty() {
                None
            } else {
                Some(cicd_config.parent_build_path.clone())
            },
            servers,
            deploy_dir: cicd_config.deploy_path.clone(),
            lib_dir: if cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven") {
                Some(format!("{}/lib", cicd_config.deploy_path))
            } else {
                None
            },
            restart_script: if cicd_config.restart_script.is_empty() {
                None
            } else {
                Some(cicd_config.restart_script.clone())
            },
            lib_separate: cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven"),
        };

        // Create deploy_id and deploy log
        let deploy_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let deploy_log = crate::db::cicd::DeployLog {
            id: deploy_id.clone(),
            project_id: cicd_config.project_id.clone(),
            config_id: config_id.to_string(),
            status: "running".to_string(),
            start_time: now.clone(),
            end_time: None,
            error_message: None,
            progress: 0,
            triggered_by: "user".to_string(),
            created_at: now.clone(),
            log_file_path: None,
            artifact_paths: None,
        };

        // Save deploy log
        self.db_write(|conn| {
            crate::db::cicd::add_deploy_log(conn, &deploy_log).expect("db error");
            crate::db::cicd::touch_cicd_config_deploy(conn, config_id).expect("db error");
        })?;

        // Collect step logs in memory during deploy
        let step_logs_arc = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        // Execute deploy (async)
        let data_dir_str = self.app_dir.to_string_lossy().to_string();
        let deploy_id_for_cb = deploy_id.clone();
        let step_logs_for_cb = step_logs_arc.clone();

        let result = crate::logic::cicd_deploy::execute_deploy(
            &deploy_config,
            &data_dir_str,
            &deploy_id,
            move |event: crate::logic::cicd_deploy::ProgressEvent| {
                let mut logs = step_logs_for_cb.lock().unwrap();
                logs.push(crate::db::cicd::DeployStepLog {
                    id: 0, // auto-increment
                    deploy_log_id: deploy_id_for_cb.clone(),
                    stage: event.stage,
                    status: event.status,
                    message: Some(event.message),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            },
            || false, // CLI 部署不支持取消
        )
        .await;

        // Write accumulated step logs to DB
        {
            let logs = step_logs_arc.lock().unwrap();
            let _ = self.db_write(|conn| {
                for step in logs.iter() {
                    let _ = crate::db::cicd::add_deploy_step_log(conn, step);
                }
            });
        }

        // Update deploy log with result
        let (final_status, final_error, final_progress, final_log_path, final_artifact_paths) = match &result {
            Ok(r) => {
                let status = if r.success { "success".to_string() } else { "failed".to_string() };
                (
                    status,
                    r.error.clone(),
                    if r.success { 100 } else { 0 },
                    Some(r.log_file_path.clone()),
                    Some(serde_json::to_string(&r.artifact_paths).unwrap_or_default()),
                )
            }
            Err(e) => ("failed".to_string(), Some(e.clone()), 0, None, None),
        };

        let end_time = chrono::Utc::now().to_rfc3339();
        let updated_log = crate::db::cicd::DeployLog {
            id: deploy_id.clone(),
            project_id: cicd_config.project_id.clone(),
            config_id: config_id.to_string(),
            status: final_status.clone(),
            start_time: now.clone(),
            end_time: Some(end_time.clone()),
            error_message: final_error,
            progress: final_progress,
            triggered_by: "user".to_string(),
            created_at: now.clone(),
            log_file_path: final_log_path.clone(),
            artifact_paths: final_artifact_paths,
        };
        let _ = self.db_write(|conn| crate::db::cicd::update_deploy_log(conn, &updated_log));

        // Add to deploy history
        let history = crate::db::cicd::DeployHistory {
            id: uuid::Uuid::new_v4().to_string(),
            config_id: config_id.to_string(),
            project_id: cicd_config.project_id.clone(),
            status: final_status,
            deployed_at: end_time,
            rolled_back: false,
            rolled_back_at: None,
        };
        let _ = self.db_write(|conn| crate::db::cicd::add_deploy_history(conn, &history));

        match result {
            Ok(r) => Ok(json!({
                "success": true,
                "deployId": r.deploy_id,
                "logFilePath": r.log_file_path,
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e,
            })),
        }
    }

    pub async fn cicd_cancel_deploy(&self, deploy_log_id: &str) -> Result<Value, String> {
        let deploy_log = self.db_read(|conn| {
            crate::db::cicd::get_deploy_log_by_id(conn, deploy_log_id)
                .map_err(|e| e.to_string())
        })??
        .ok_or("部署记录不存在")?;

        if deploy_log.status != "running" && deploy_log.status != "pending" {
            return Ok(json!({
                "success": false,
                "error": format!("当前状态 ({}) 不允许取消", deploy_log.status)
            }));
        }

        let end_time = chrono::Utc::now().to_rfc3339();
        let updated_log = crate::db::cicd::DeployLog {
            id: deploy_log.id.clone(),
            project_id: deploy_log.project_id.clone(),
            config_id: deploy_log.config_id.clone(),
            status: "cancelled".to_string(),
            start_time: deploy_log.start_time.clone(),
            end_time: Some(end_time),
            error_message: deploy_log.error_message,
            progress: deploy_log.progress,
            triggered_by: deploy_log.triggered_by.clone(),
            created_at: deploy_log.created_at.clone(),
            log_file_path: deploy_log.log_file_path,
            artifact_paths: deploy_log.artifact_paths,
        };

        self.db_write(|conn| {
            crate::db::cicd::update_deploy_log(conn, &updated_log).expect("db error");
        })?;

        Ok(json!({
            "success": true,
            "message": "部署已取消"
        }))
    }

        pub async fn cicd_rollback(&self, config_id: &str, log_id: &str) -> Result<Value, String> {
    let deploy_log = self.db_read(|conn| {
        crate::db::cicd::get_deploy_log_by_id(conn, log_id)
            .map_err(|e| e.to_string())
    })??
    .ok_or("部署记录不存在")?;

    let cicd_config = self.db_read(|conn| {
        crate::db::cicd::get_cicd_config_by_config_id(conn, config_id)
            .map_err(|e| e.to_string())
    })??
    .ok_or("CI/CD 配置不存在")?;

    let rollback_id = uuid::Uuid::new_v4().to_string();

    // Parse servers from config JSON
    let mut rollback_errors: Vec<String> = Vec::new();
    if let Some(ref servers_str) = cicd_config.servers {
        if let Ok(servers) = serde_json::from_str::<Vec<serde_json::Value>>(servers_str) {
            for server_val in &servers {
                let host = server_val.get("host")
                    .or_else(|| server_val.get("serverId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if host.is_empty() {
                    continue;
                }

                // Query full server config from servers table
                let server = match self.db_read(|conn| {
                    conn.query_row(
                        "SELECT * FROM servers WHERE id = ? OR host = ?",
                        rusqlite::params![host, host],
                        crate::db::servers::row_to_server,
                    )
                }) {
                    Ok(Ok(s)) => s,
                    _ => {
                        rollback_errors.push(format!("{}: 服务器不存在", host));
                        continue;
                    }
                };

                let port = server.port as u16;
                let username = server.username.clone();
                let password = server.password.clone()
                    .map(|pw| crate::encryption::try_decrypt_password(&pw));
                let private_key = server.ssh_key_path.clone();

                // Execute restart script via SSH
                if let Err(e) = self.cicd_execute_remote_restart(
                    &host, port, &username, password.as_deref(), private_key.as_deref(),
                    &cicd_config.restart_script,
                ) {
                    log::error!("[rollback] {}:{} restart failed: {}", host, port, e);
                    rollback_errors.push(format!("{}:{} → {}", host, port, e));
                } else {
                    log::info!("[rollback] {}:{} restart successful", host, port);
                }
            }
        } else {
            rollback_errors.push("服务器配置解析失败".to_string());
        }
    } else {
        rollback_errors.push("未配置部署服务器".to_string());
    }

    // Record rollback in deploy history
    let now = chrono::Utc::now().to_rfc3339();
    let history = crate::db::cicd::DeployHistory {
        id: rollback_id,
        config_id: config_id.to_string(),
        project_id: deploy_log.project_id,
        status: if rollback_errors.is_empty() {
            "rollback-success".to_string()
        } else {
            "rollback-partial".to_string()
        },
        deployed_at: now.clone(),
        rolled_back: true,
        rolled_back_at: Some(now.clone()),
    };
    self.db_write(|conn| {
        crate::db::cicd::add_deploy_history(conn, &history).expect("db error");
    })?;

    Ok(json!({
        "success": rollback_errors.is_empty(),
        "status": if rollback_errors.is_empty() { "rollback-success" } else { "rollback-partial" },
        "errors": rollback_errors,
    }))
    }

        fn cicd_execute_remote_restart(
    &self,
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    private_key: Option<&str>,
    restart_script: &str,
    ) -> Result<(), String> {
    use ssh2::Session;
    use std::net::TcpStream;

    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| format!("连接 {} 失败: {}", addr, e))?;

    let mut sess = Session::new()
        .map_err(|e| format!("创建 SSH session 失败: {}", e))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH 握手失败: {}", e))?;

    if let Some(key_path) = private_key {
        sess.userauth_pubkey_file(
        username,
        None,
        std::path::Path::new(key_path),
        password,
        )
        .map_err(|e| format!("SSH 密钥认证失败: {}", e))?;
    } else if let Some(pw) = password {
        sess.userauth_password(username, pw)
        .map_err(|e| format!("SSH 密码认证失败: {}", e))?;
    } else {
        return Err("缺少认证信息".to_string());
    }

    let cmd = format!("cd / && nohup {} > /dev/null 2>&1 &", restart_script);
    let mut channel = sess
        .channel_session()
        .map_err(|e| format!("创建 SSH channel 失败: {}", e))?;
    channel
        .exec(&cmd)
        .map_err(|e| format!("执行重启命令失败: {}", e))?;
    channel.wait_close().ok();

    Ok(())
    }
}
