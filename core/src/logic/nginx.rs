use crate::db::ApiResponse;
use crate::db::nginx::{
    NginxBasicSetting, NginxCert, NginxDenyAllow, NginxHttpParam, NginxLocation, NginxParam,
    NginxPassword, NginxServer, NginxStream, NginxTemplate, NginxUpstream, NginxUpstreamServer,
};
use crate::logic::CoreService;
use crate::logic::nginx_generator::{NginxConfigResult, NginxSubFile};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxTestResult {
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NginxDeployResult {
    pub success: bool,
    #[serde(rename = "backupPath")]
    pub backup_path: String,
    pub message: String,
}

/// Escape a path for safe use in shell single quotes
fn shell_escape_path(path: &str) -> String {
    path.replace('\'', "'\\''")
}

impl CoreService {
    /// Fetch nginx config content from remote server
    pub async fn fetch_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
    ) -> Result<ApiResponse<String>, String> {
        // Ensure SSH connection before operation
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid, &format!("cat '{}' 2>&1", safe_path))
            })
            .await?;
        if !result.success {
            return Ok(ApiResponse::err(format!(
                "读取配置失败: {}",
                result.output.trim()
            )));
        }
        if result.output.trim().is_empty() {
            return Ok(ApiResponse::err("配置文件为空".to_string()));
        }
        Ok(ApiResponse::ok(result.output))
    }

    /// Test nginx config on remote server (nginx -t -c <path>)
    pub async fn test_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
    ) -> Result<ApiResponse<NginxTestResult>, String> {
        // Ensure SSH connection before operation
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid, &format!("nginx -t -c '{}' 2>&1", safe_path))
            })
            .await?;
        let output = result.output.clone();
        let passed = output.contains("syntax is ok") || output.contains("test is successful");
        Ok(ApiResponse::ok(NginxTestResult {
            passed,
            message: output,
        }))
    }

    /// Test a new nginx config content by writing to a temp file and running nginx -t
    /// This tests the LOCAL config before deploying, not the server's existing config
    pub async fn test_nginx_config_content(
        &self,
        server_id: &str,
        config_path: &str,
        content: &str,
    ) -> Result<ApiResponse<NginxTestResult>, String> {
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        // Use a temp file path: original path + .test suffix
        let test_path = format!("{}.test", config_path);
        let safe_test_path = shell_escape_path(&test_path);

        // 1. Write config content to temp file via base64
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
        let sid1 = sid.clone();
        let stp1 = safe_test_path.clone();
        let write_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(
                    &sid1,
                    &format!("printf '%s' '{}' | base64 -d > '{}' 2>&1", encoded, stp1),
                )
            })
            .await?;

        if !write_result.success {
            return Ok(ApiResponse::err(format!(
                "写入临时文件失败: {}",
                write_result.output.trim()
            )));
        }

        // 2. Test the temp file
        let sid2 = sid.clone();
        let stp2 = safe_test_path.clone();
        let test_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid2, &format!("nginx -t -c '{}' 2>&1", stp2))
            })
            .await?;

        let output = test_result.output.clone();
        let passed = output.contains("syntax is ok") || output.contains("test is successful");

        // 3. Clean up temp file (ignore errors)
        let sid3 = sid.clone();
        let stp3 = safe_test_path.clone();
        let _ = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid3, &format!("rm -f '{}' 2>&1", stp3))
            })
            .await;

        Ok(ApiResponse::ok(NginxTestResult {
            passed,
            message: output,
        }))
    }

    /// Deploy nginx config: backup → write → test → reload (with auto-rollback)
    pub async fn deploy_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
        content: &str,
    ) -> Result<ApiResponse<NginxDeployResult>, String> {
        // Ensure SSH connection before operation
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S%3f").to_string();
        let backup_path = format!("{}.bak.{}", config_path, ts);
        let safe_backup = shell_escape_path(&backup_path);

        // 1. Backup current config
        let sid2 = sid.clone();
        let sp2 = safe_path.clone();
        let sb2 = safe_backup.clone();
        let backup_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid2, &format!("cp '{}' '{}' 2>&1", sp2, sb2))
            })
            .await?;
        if !backup_result.success {
            return Ok(ApiResponse::err(format!(
                "备份失败: {}",
                backup_result.output.trim()
            )));
        }

        // 2. Write new config via base64 to avoid shell escaping issues
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
        let sid3 = sid.clone();
        let sp3 = safe_path.clone();
        let write_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(
                    &sid3,
                    &format!("printf '%s' '{}' | base64 -d > '{}' 2>&1", encoded, sp3),
                )
            })
            .await?;
        if !write_result.success {
            // Rollback: restore backup
            let sid_rb = sid.clone();
            let sb_rb = safe_backup.clone();
            let sp_rb = safe_path.clone();
            let rb_result = self
                .run_ssh_blocking(move |ssh| {
                    ssh.exec_command(&sid_rb, &format!("cp '{}' '{}' 2>&1", sb_rb, sp_rb))
                })
                .await?;
            return Ok(ApiResponse::err(format!(
                "写入失败: {}. 回滚: {}",
                write_result.output.trim(),
                if rb_result.success {
                    "成功"
                } else {
                    "也失败"
                }
            )));
        }

        // 3. Test new config
        let sid4 = sid.clone();
        let sp4 = safe_path.clone();
        let test_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid4, &format!("nginx -t -c '{}' 2>&1", sp4))
            })
            .await?;
        if !test_result.output.contains("syntax is ok")
            && !test_result.output.contains("test is successful")
        {
            // Rollback: restore backup
            let sid5 = sid.clone();
            let sb5 = safe_backup.clone();
            let sp5 = safe_path.clone();
            let rb_result = self
                .run_ssh_blocking(move |ssh| {
                    ssh.exec_command(&sid5, &format!("cp '{}' '{}' 2>&1", sb5, sp5))
                })
                .await?;
            return Ok(ApiResponse::err(format!(
                "nginx -t 检测失败: {}. 回滚: {}",
                test_result.output.trim(),
                if rb_result.success {
                    "成功"
                } else {
                    "也失败"
                }
            )));
        }

        // 4. Reload nginx (try systemctl first, fallback to nginx -s)
        let sid6 = sid.clone();
        let reload_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid6, "systemctl reload nginx 2>&1 || nginx -s reload 2>&1")
            })
            .await?;

        Ok(ApiResponse::ok(NginxDeployResult {
            success: true,
            backup_path,
            message: format!("配置已部署。重载: {}", reload_result.output.trim()),
        }))
    }

    pub async fn deploy_nginx_config_decomposed(
        &self,
        server_id: &str,
        config_path: &str,
        main_content: &str,
        sub_files: Vec<NginxSubFile>,
    ) -> Result<ApiResponse<NginxDeployResult>, String> {
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S%3f").to_string();
        let backup_path = format!("{}.bak.{}", config_path, ts);
        let safe_backup = shell_escape_path(&backup_path);
        let conf_d_dir = {
            let idx = config_path.rfind('/').unwrap_or(0);
            if idx > 0 {
                format!("{}/conf.d", &config_path[..idx])
            } else {
                "/etc/nginx/conf.d".to_string()
            }
        };
        let safe_conf_d = shell_escape_path(&conf_d_dir);

        // 1. Backup current config
        let sid1 = sid.clone();
        let sp1 = safe_path.clone();
        let sb1 = safe_backup.clone();
        let backup_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid1, &format!("cp '{}' '{}' 2>&1", sp1, sb1))
            })
            .await?;
        if !backup_result.success {
            return Ok(ApiResponse::err(format!(
                "备份失败: {}",
                backup_result.output.trim()
            )));
        }

        // 2. Write main config via base64
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, main_content);
        let sid2 = sid.clone();
        let sp2 = safe_path.clone();
        let write_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(
                    &sid2,
                    &format!("printf '%s' '{}' | base64 -d > '{}' 2>&1", encoded, sp2),
                )
            })
            .await?;
        if !write_result.success {
            return Ok(ApiResponse::err(format!(
                "写入主配置失败: {}",
                write_result.output.trim()
            )));
        }

        // 3. Create conf.d/ directory and write sub-files
        let sid3 = sid.clone();
        let sc3 = safe_conf_d.clone();
        let mkdir_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid3, &format!("mkdir -p '{}' 2>&1", sc3))
            })
            .await?;
        if !mkdir_result.success {
            return Ok(ApiResponse::err(format!(
                "创建 conf.d 目录失败: {}",
                mkdir_result.output.trim()
            )));
        }

        for sub in &sub_files {
            let sid_sub = sid.clone();
            let encoded_sub =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &sub.content);
            let safe_sub_path = shell_escape_path(&format!("{}/{}", conf_d_dir, sub.filename));
            let write_sub = self
                .run_ssh_with_retry(server_id, move |ssh| {
                    ssh.exec_command(
                        &sid_sub,
                        &format!(
                            "printf '%s' '{}' | base64 -d > '{}' 2>&1",
                            encoded_sub, safe_sub_path
                        ),
                    )
                })
                .await?;
            if !write_sub.success {
                return Ok(ApiResponse::err(format!(
                    "写入子文件 {} 失败: {}",
                    sub.filename,
                    write_sub.output.trim()
                )));
            }
        }

        // 4. Test with nginx -t
        let sid4 = sid.clone();
        let sp4 = safe_path.clone();
        let test_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid4, &format!("nginx -t -c '{}' 2>&1", sp4))
            })
            .await?;
        if !test_result.output.contains("syntax is ok")
            && !test_result.output.contains("test is successful")
        {
            // Rollback
            let sid5 = sid.clone();
            let sb5 = safe_backup.clone();
            let sp5 = safe_path.clone();
            let _ = self
                .run_ssh_blocking(move |ssh| {
                    ssh.exec_command(&sid5, &format!("cp '{}' '{}' 2>&1", sb5, sp5))
                })
                .await;
            return Ok(ApiResponse::err(format!(
                "nginx -t 检测失败: {}. 已回滚.",
                test_result.output.trim()
            )));
        }

        // 5. Reload nginx
        let sid6 = sid.clone();
        let reload_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(&sid6, "systemctl reload nginx 2>&1 || nginx -s reload 2>&1")
            })
            .await?;

        Ok(ApiResponse::ok(NginxDeployResult {
            success: true,
            backup_path,
            message: format!(
                "配置已部署 ({}个子文件)。重载: {}",
                sub_files.len(),
                reload_result.output.trim()
            ),
        }))
    }

    /// Rollback nginx config from a backup path
    pub async fn rollback_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
        backup_path: &str,
    ) -> Result<ApiResponse<String>, String> {
        // Ensure SSH connection before operation
        self.ensure_ssh_connected(server_id).await?;

        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let safe_backup = shell_escape_path(backup_path);
        let cmd = format!(
            "cp '{}' '{}' 2>&1 && nginx -t -c '{}' 2>&1 && (systemctl reload nginx 2>&1 || nginx -s reload 2>&1)",
            safe_backup, safe_path, safe_path
        );
        let result = self
            .run_ssh_with_retry(server_id, move |ssh| ssh.exec_command(&sid, &cmd))
            .await?;
        if result.output.contains("syntax is ok") || result.output.contains("test is successful") {
            Ok(ApiResponse::ok(result.output))
        } else {
            Ok(ApiResponse::err(format!(
                "回滚失败: {}",
                result.output.trim()
            )))
        }
    }

    // ============ Nginx DB Operations ============

    pub async fn get_all_nginx_presets(
        &self,
    ) -> Result<ApiResponse<Vec<crate::db::nginx::NginxPreset>>, String> {
        Ok(self.with_db(|db| crate::db::nginx::get_all_nginx_presets(db)))
    }

    pub async fn add_nginx_preset(
        &self,
        preset: crate::db::nginx::NginxPreset,
    ) -> Result<ApiResponse<crate::db::nginx::NginxPreset>, String> {
        Ok(self.with_db(|db| crate::db::nginx::add_nginx_preset(db, preset)))
    }

    pub async fn update_nginx_preset(
        &self,
        preset: crate::db::nginx::NginxPreset,
    ) -> Result<ApiResponse<()>, String> {
        Ok(self.with_db(|db| crate::db::nginx::update_nginx_preset(db, preset)))
    }

    pub async fn delete_nginx_preset(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let id = id.to_string();
        Ok(self.with_db(move |db| crate::db::nginx::delete_nginx_preset(db, &id)))
    }

    pub async fn get_nginx_config_versions(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<crate::db::nginx::NginxConfigVersion>>, String> {
        let pid = preset_id.to_string();
        Ok(self.with_db(move |db| crate::db::nginx::get_config_versions(db, &pid)))
    }

    pub async fn add_nginx_config_version(
        &self,
        version: crate::db::nginx::NginxConfigVersion,
    ) -> Result<ApiResponse<crate::db::nginx::NginxConfigVersion>, String> {
        Ok(self.with_db(|db| crate::db::nginx::add_config_version(db, version)))
    }

    pub async fn set_current_nginx_version(
        &self,
        preset_id: &str,
        version_id: &str,
    ) -> Result<ApiResponse<()>, String> {
        let pid = preset_id.to_string();
        let vid = version_id.to_string();
        Ok(self.with_db(move |db| crate::db::nginx::set_current_version(db, &pid, &vid)))
    }

    // ============ NginxServer CRUD ============

    pub async fn get_servers_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxServer>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxServer>, String> {
            crate::db::nginx::get_servers_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn get_nginx_server_by_id(
        &self,
        id: &str,
    ) -> Result<ApiResponse<Option<NginxServer>>, String> {
        let sid = id.to_string();
        let result = self.db_read(move |conn| -> Result<Option<NginxServer>, String> {
            crate::db::nginx::get_server_by_id(conn, &sid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_server(&self, server: &NginxServer) -> Result<ApiResponse<()>, String> {
        let s = server.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_server(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_server(
        &self,
        server: &NginxServer,
    ) -> Result<ApiResponse<()>, String> {
        let s = server.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_server(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_server(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let sid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_server(conn, &sid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    /// Preview a single server block config (without saving to DB).
    /// Accepts the full server object + locations array as JSON,
    /// generates the config text using the same generator logic.
    pub async fn preview_nginx_server(
        &self,
        _preset_id: &str,
        server: serde_json::Value,
        locations: serde_json::Value,
    ) -> Result<ApiResponse<String>, String> {
        let s: NginxServer = serde_json::from_value(server).map_err(|e| e.to_string())?;
        let locs: Vec<NginxLocation> =
            serde_json::from_value(locations).map_err(|e| e.to_string())?;
        let result = self.db_read(move |conn| {
            crate::logic::nginx_generator::generate_server_block_preview(conn, &s, &locs)
        })??;
        Ok(ApiResponse::ok(result))
    }

    // ============ NginxLocation CRUD ============

    pub async fn get_locations_by_server(
        &self,
        server_id: &str,
    ) -> Result<ApiResponse<Vec<NginxLocation>>, String> {
        let sid = server_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxLocation>, String> {
            crate::db::nginx::get_locations_by_server(conn, &sid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_location(
        &self,
        location: &NginxLocation,
    ) -> Result<ApiResponse<()>, String> {
        let loc = location.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_location(conn, &loc).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_location(
        &self,
        location: &NginxLocation,
    ) -> Result<ApiResponse<()>, String> {
        let loc = location.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_location(conn, &loc).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_location(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let sid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_location(conn, &sid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxUpstream CRUD ============

    pub async fn get_upstreams_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxUpstream>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxUpstream>, String> {
            crate::db::nginx::get_upstreams_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn get_upstream_by_id(
        &self,
        id: &str,
    ) -> Result<ApiResponse<Option<NginxUpstream>>, String> {
        let uid = id.to_string();
        let result = self.db_read(move |conn| -> Result<Option<NginxUpstream>, String> {
            crate::db::nginx::get_upstream_by_id(conn, &uid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_upstream(
        &self,
        upstream: &NginxUpstream,
    ) -> Result<ApiResponse<()>, String> {
        let u = upstream.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_upstream(conn, &u).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_upstream(
        &self,
        upstream: &NginxUpstream,
    ) -> Result<ApiResponse<()>, String> {
        let u = upstream.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_upstream(conn, &u).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_upstream(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let uid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_upstream(conn, &uid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxUpstreamServer CRUD ============

    pub async fn get_upstream_servers(
        &self,
        upstream_id: &str,
    ) -> Result<ApiResponse<Vec<NginxUpstreamServer>>, String> {
        let uid = upstream_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxUpstreamServer>, String> {
            crate::db::nginx::get_upstream_servers(conn, &uid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_upstream_server(
        &self,
        upstream_server: &NginxUpstreamServer,
    ) -> Result<ApiResponse<()>, String> {
        let us = upstream_server.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_upstream_server(conn, &us).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_upstream_server(
        &self,
        upstream_server: &NginxUpstreamServer,
    ) -> Result<ApiResponse<()>, String> {
        let us = upstream_server.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_upstream_server(conn, &us).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_upstream_server(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let uid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_upstream_server(conn, &uid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxHttpParam CRUD ============

    pub async fn get_http_params_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxHttpParam>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxHttpParam>, String> {
            crate::db::nginx::get_http_params_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_http_param(
        &self,
        param: &NginxHttpParam,
    ) -> Result<ApiResponse<()>, String> {
        let p = param.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_http_param(conn, &p).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_http_param(
        &self,
        param: &NginxHttpParam,
    ) -> Result<ApiResponse<()>, String> {
        let p = param.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_http_param(conn, &p).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_http_param(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let pid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_http_param(conn, &pid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxStream CRUD ============

    pub async fn get_streams_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxStream>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxStream>, String> {
            crate::db::nginx::get_streams_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_stream(&self, stream: &NginxStream) -> Result<ApiResponse<()>, String> {
        let s = stream.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_stream(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_stream(
        &self,
        stream: &NginxStream,
    ) -> Result<ApiResponse<()>, String> {
        let s = stream.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_stream(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_stream(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let sid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_stream(conn, &sid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxCert CRUD ============

    pub async fn get_certs_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxCert>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxCert>, String> {
            crate::db::nginx::get_certs_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_cert(&self, cert: &NginxCert) -> Result<ApiResponse<()>, String> {
        let c = cert.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_cert(conn, &c).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_cert(&self, cert: &NginxCert) -> Result<ApiResponse<()>, String> {
        let c = cert.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_cert(conn, &c).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_cert(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let cid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_cert(conn, &cid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxTemplate CRUD ============

    pub async fn get_templates_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxTemplate>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxTemplate>, String> {
            crate::db::nginx::get_templates_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_template(
        &self,
        template: &NginxTemplate,
    ) -> Result<ApiResponse<()>, String> {
        let t = template.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_template(conn, &t).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_template(
        &self,
        template: &NginxTemplate,
    ) -> Result<ApiResponse<()>, String> {
        let t = template.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_template(conn, &t).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_template(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let tid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_template(conn, &tid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxBasicSetting CRUD (key-value) ============

    pub async fn get_basic_settings(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxBasicSetting>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxBasicSetting>, String> {
            crate::db::nginx::get_basic_settings_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_basic_setting(
        &self,
        setting: &NginxBasicSetting,
    ) -> Result<ApiResponse<()>, String> {
        let s = setting.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_basic_setting(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_basic_setting(
        &self,
        setting: &NginxBasicSetting,
    ) -> Result<ApiResponse<()>, String> {
        let s = setting.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_basic_setting(conn, &s).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_basic_setting(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let sid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_basic_setting(conn, &sid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn save_basic_settings(
        &self,
        preset_id: &str,
        settings: &[NginxBasicSetting],
    ) -> Result<ApiResponse<()>, String> {
        let pid = preset_id.to_string();
        let items = settings.to_vec();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_basic_settings_by_preset(conn, &pid)
                .map_err(|e| e.to_string())?;
            for s in &items {
                crate::db::nginx::add_nginx_basic_setting(conn, s).map_err(|e| e.to_string())?;
            }
            Ok(())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxParam CRUD ============

    pub async fn get_params_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxParam>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxParam>, String> {
            crate::db::nginx::get_params_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_param(&self, param: &NginxParam) -> Result<ApiResponse<()>, String> {
        let p = param.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_param(conn, &p).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_param(&self, param: &NginxParam) -> Result<ApiResponse<()>, String> {
        let p = param.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_param(conn, &p).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_param(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let pid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_param(conn, &pid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxDenyAllow CRUD ============

    pub async fn get_deny_allows_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxDenyAllow>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxDenyAllow>, String> {
            crate::db::nginx::get_deny_allows_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_deny_allow(
        &self,
        deny_allow: &NginxDenyAllow,
    ) -> Result<ApiResponse<()>, String> {
        let d = deny_allow.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_deny_allow(conn, &d).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_deny_allow(
        &self,
        deny_allow: &NginxDenyAllow,
    ) -> Result<ApiResponse<()>, String> {
        let d = deny_allow.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_deny_allow(conn, &d).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_deny_allow(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let did = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_deny_allow(conn, &did).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ NginxPassword CRUD ============

    pub async fn get_passwords_by_preset(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Vec<NginxPassword>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Vec<NginxPassword>, String> {
            crate::db::nginx::get_passwords_by_preset(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn add_nginx_password(
        &self,
        password: &NginxPassword,
    ) -> Result<ApiResponse<()>, String> {
        let pw = password.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_password(conn, &pw).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_password(
        &self,
        password: &NginxPassword,
    ) -> Result<ApiResponse<()>, String> {
        let pw = password.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::update_nginx_password(conn, &pw).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn delete_nginx_password(&self, id: &str) -> Result<ApiResponse<()>, String> {
        let pid = id.to_string();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::delete_nginx_password(conn, &pid).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    // ============ Config Generation ============

    pub async fn generate_nginx_config(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<String>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| {
            crate::logic::nginx_generator::generate_nginx_config(conn, &pid)
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn generate_nginx_config_decomposed(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<NginxConfigResult>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| {
            crate::logic::nginx_generator::generate_nginx_config_decomposed(conn, &pid)
        })??;
        Ok(ApiResponse::ok(result))
    }

    // ============ Config Import ============

    /// Parse nginx config text and import into database.
    /// Returns a summary of what was imported.
    pub async fn import_nginx_config(
        &self,
        preset_id: &str,
        config_text: &str,
    ) -> Result<ApiResponse<serde_json::Value>, String> {
        let pid = preset_id.to_string();
        let text = config_text.to_string();

        // Parse the config
        let parsed = crate::logic::nginx_parser::parse_nginx_config(&text)
            .map_err(|e| format!("解析失败: {}", e))?;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let summary = serde_json::json!({
            "basic_settings": parsed.basic_settings.len(),
            "http_params": parsed.http_params.len(),
            "upstreams": parsed.upstreams.len(),
            "servers": parsed.servers.len(),
            "streams": parsed.streams.len(),
        });

        // Import in a single transaction
        self.db_write(move |conn| -> Result<(), String> {
            // 1. Basic settings (replace all for this preset)
            crate::db::nginx::delete_basic_settings_by_preset(conn, &pid).map_err(|e| e.to_string())?;
            for bs in &parsed.basic_settings {
                let id = format!("bs_{}", uuid::Uuid::new_v4().simple());
                crate::db::nginx::add_nginx_basic_setting(conn, &NginxBasicSetting {
                    id,
                    preset_id: pid.clone(),
                    name: bs.name.clone(),
                    value: bs.value.clone(),
                    sort: 0,
                    created_at: now.clone(),
                }).map_err(|e| e.to_string())?;
            }

            // 2. HTTP params (replace all)
            // Delete existing http params for this preset
            {
                let existing = crate::db::nginx::get_http_params_by_preset(conn, &pid)
                    .map_err(|e| e.to_string())?;
                for p in &existing {
                    crate::db::nginx::delete_nginx_http_param(conn, &p.id)
                        .map_err(|e| e.to_string())?;
                }
            }
            for hp in &parsed.http_params {
                let id = format!("hp_{}", uuid::Uuid::new_v4().simple());
                crate::db::nginx::add_nginx_http_param(conn, &NginxHttpParam {
                    id,
                    preset_id: pid.clone(),
                    name: hp.name.clone(),
                    value: hp.value.clone(),
                    enabled: true,
                    sort: 0,
                    created_at: now.clone(),
                }).map_err(|e| e.to_string())?;
            }

            // 3. Delete existing entity data for this preset, then insert new
            // Must delete child rows first (locations before servers, upstream_servers before upstreams)
            conn.execute("DELETE FROM nginx_locations WHERE serverId IN (SELECT id FROM nginx_servers WHERE presetId = ?1)", rusqlite::params![&pid]).ok();
            conn.execute("DELETE FROM nginx_servers WHERE presetId = ?1", rusqlite::params![&pid]).ok();
            conn.execute("DELETE FROM nginx_upstream_servers WHERE upstreamId IN (SELECT id FROM nginx_upstreams WHERE presetId = ?1)", rusqlite::params![&pid]).ok();
            conn.execute("DELETE FROM nginx_upstreams WHERE presetId = ?1", rusqlite::params![&pid]).ok();
            conn.execute("DELETE FROM nginx_streams WHERE presetId = ?1", rusqlite::params![&pid]).ok();

            // 4. Upstreams + their servers (insert new)
            for up in &parsed.upstreams {
                let up_id = format!("up_{}", uuid::Uuid::new_v4().simple());
                crate::db::nginx::add_nginx_upstream(conn, &NginxUpstream {
                    id: up_id.clone(),
                    preset_id: pid.clone(),
                    name: up.name.clone(),
                    proxy_type: 0,
                    strategy: up.strategy.clone(),
                    descr: up.descr.clone(),
                    param_json: if up.extra_params.is_empty() { String::new() } else { serde_json::to_string(&up.extra_params).unwrap_or_default() },
                    sort: 0,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                }).map_err(|e| e.to_string())?;

                for us in &up.servers {
                    let us_id = format!("us_{}", uuid::Uuid::new_v4().simple());
                    crate::db::nginx::add_nginx_upstream_server(conn, &NginxUpstreamServer {
                        id: us_id,
                        upstream_id: up_id.clone(),
                        address: us.address.clone(),
                        port: us.port,
                        weight: us.weight,
                        max_fails: us.max_fails,
                        fail_timeout: if us.fail_timeout.is_empty() { "10s".to_string() } else { us.fail_timeout.clone() },
                        max_conns: us.max_conns,
                        backup: us.backup,
                        down: us.down,
                        sort: 0,
                        enabled: true,
                        param: us.param.clone(),
                    }).map_err(|e| e.to_string())?;
                }
            }

            // 5. Servers + locations
            // Collect unique certs from imported SSL servers, insert into nginx_certs
            let mut cert_map: Vec<(String, String, String)> = Vec::new(); // (pem|key key, pem, key)
            for srv in &parsed.servers {
                if srv.ssl != 0 && !srv.pem.is_empty() && !srv.key.is_empty() {
                    let key = format!("{}|{}", srv.pem, srv.key);
                    if !cert_map.iter().any(|(k, _, _)| k == &key) {
                        cert_map.push((key, srv.pem.clone(), srv.key.clone()));
                    }
                }
            }
            // Insert cert records — reuse imported_cert_ prefix for the original cert_id
            let mut cert_lookup: std::collections::HashMap<String, String> = std::collections::HashMap::new(); // pem|key -> cert_id
            for (idx, (pem_key, pem_path, key_path)) in cert_map.iter().enumerate() {
                let cert_id = format!("icert_{}", uuid::Uuid::new_v4().simple());
                let domain = format!("imported_{}", idx);
                crate::db::nginx::add_nginx_cert(conn, &NginxCert {
                    id: cert_id.clone(),
                    preset_id: pid.clone(),
                    name: format!("导入证书 #{}", idx + 1),
                    pem: pem_path.clone(),
                    key: key_path.clone(),
                    domain: domain.clone(),
                    created_at: now.clone(),
                }).map_err(|e| e.to_string())?;
                cert_lookup.insert(pem_key.clone(), cert_id);
            }

            for srv in &parsed.servers {
                let srv_id = format!("sv_{}", uuid::Uuid::new_v4().simple());
                // Resolve cert_id for imported SSL servers
                let resolved_cert_id = if srv.ssl != 0 && !srv.pem.is_empty() && !srv.key.is_empty() {
                    let key = format!("{}|{}", srv.pem, srv.key);
                    cert_lookup.get(&key).cloned().unwrap_or_else(|| srv.cert_id.clone())
                } else {
                    srv.cert_id.clone()
                };

                let _ = crate::db::nginx::add_nginx_server(conn, &NginxServer {
                    id: srv_id.clone(),
                    preset_id: pid.clone(),
                    proxy_type: srv.proxy_type,
                    listen: srv.listen.clone(),
                    ip: srv.ip.clone(),
                    def: srv.def,
                    ipv6: srv.ipv6,
                    proxy_protocol: srv.proxy_protocol,
                    server_name: srv.server_name.clone(),
                    ssl: srv.ssl != 0,
                    cert_id: resolved_cert_id,
                    rewrite: srv.rewrite,
                    rewrite_listen: srv.rewrite_listen.clone(),
                    http2: srv.http2,
                    protocols: srv.protocols.clone(),
                    password_id: srv.password_id.clone(),
                    deny_allow: srv.deny_allow,
                    deny_id: srv.deny_id.clone(),
                    allow_id: srv.allow_id.clone(),
                    proxy_upstream_id: srv.proxy_upstream_id.clone(),
                    descr: srv.descr.clone(),
                    enabled: true,
                    sort: 0,
                    param_json: if srv.extra_params.is_empty() { String::new() } else { serde_json::to_string(&srv.extra_params).unwrap_or_default() },
                    created_at: now.clone(),
                    updated_at: now.clone(),
                }).map_err(|e| e.to_string())?;

                for loc in &srv.locations {
                    let loc_id = format!("loc_{}", uuid::Uuid::new_v4().simple());
                    let loc_type_val: i64 = match loc.loc_type.as_str() {
                        "proxy_pass" => 0,
                        "root" => 1,
                        "upstream" => 2,
                        "blank" => 3,
                        "return" => 4,
                        // 空 loc_type（如 named location @router/@ops-coffee，只有 rewrite 无 proxy/root/return）
                        // → 映射为 blank(3)，生成器不输出 proxy_pass/root/return/proxy_redirect
                        "" => 3,
                        _ => 0,
                    };
                    crate::db::nginx::add_nginx_location(conn, &NginxLocation {
                        id: loc_id,
                        server_id: srv_id.clone(),
                        enabled: true,
                        path: loc.path.clone(),
                        loc_type: loc_type_val,
                        value: loc.value.clone(),
                        upstream_type: 0,
                        upstream_id: loc.upstream_id.clone(),
                        upstream_path: loc.upstream_path.clone(),
                        root_path: loc.root_path.clone(),
                        root_page: String::new(),
                        root_type: String::new(),
                        header: loc.header,
                        header_host: String::new(),
                        websocket: loc.websocket,
                        cros: loc.cros,
                        return_url: loc.return_url.clone(),
                        return_path: false,
                        param_json: if loc.extra_params.is_empty() { String::new() } else { serde_json::to_string(&loc.extra_params).unwrap_or_default() },
                        sort: 0,
                        descr: loc.descr.clone(),
                        created_at: now.clone(),
                    }).map_err(|e| e.to_string())?;
                }
            }

            // 5. Streams
            // Collect unique certs from imported SSL streams, insert into nginx_certs
            let mut stream_cert_map: Vec<(String, String, String)> = Vec::new();
            for st in &parsed.streams {
                if st.ssl != 0 && !st.pem.is_empty() {
                    let key = format!("{}|{}", st.pem, st.key);
                    if !stream_cert_map.iter().any(|(k, _, _)| k == &key) {
                        stream_cert_map.push((key, st.pem.clone(), st.key.clone()));
                    }
                }
            }
            let mut stream_cert_lookup: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for (idx, (pem_key, pem_path, key_path)) in stream_cert_map.iter().enumerate() {
                let cert_id = format!("sicert_{}", uuid::Uuid::new_v4().simple());
                crate::db::nginx::add_nginx_cert(conn, &NginxCert {
                    id: cert_id.clone(),
                    preset_id: pid.clone(),
                    name: format!("导入流证书 #{}", idx + 1),
                    pem: pem_path.clone(),
                    key: key_path.clone(),
                    domain: "stream".to_string(),
                    created_at: now.clone(),
                }).map_err(|e| e.to_string())?;
                stream_cert_lookup.insert(pem_key.clone(), cert_id);
            }

            for st in &parsed.streams {
                let st_id = format!("st_{}", uuid::Uuid::new_v4().simple());
                // Resolve cert_id for imported SSL streams
                let resolved_cert_id = if st.ssl != 0 && !st.pem.is_empty() {
                    let key = format!("{}|{}", st.pem, st.key);
                    stream_cert_lookup.get(&key).cloned().unwrap_or_else(|| st.cert_id.clone())
                } else {
                    st.cert_id.clone()
                };
                crate::db::nginx::add_nginx_stream(conn, &NginxStream {
                    id: st_id,
                    preset_id: pid.clone(),
                    listen: st.listen.clone(),
                    proxy_upstream_id: st.proxy_upstream_id.clone(),
                    proxy_pass: st.proxy_pass.clone(),
                    ssl: st.ssl != 0,
                    cert_id: resolved_cert_id,
                    protocol: st.protocol.clone(),
                    descr: st.descr.clone(),
                    enabled: true,
                    sort: 0,
                    param_json: String::new(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                }).map_err(|e| e.to_string())?;
            }

            Ok(())
        })??;

        Ok(ApiResponse::ok(summary))
    }

    /// Get existing data stats for a preset (for import dedup check).
    pub async fn get_nginx_preset_stats(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<serde_json::Value>, String> {
        let pid = preset_id.to_string();
        let stats = self.db_read(move |conn| -> Result<serde_json::Value, String> {
            let server_count = crate::db::nginx::get_servers_by_preset(conn, &pid)
                .map(|v| v.len()).unwrap_or(0);
            let upstream_count = crate::db::nginx::get_upstreams_by_preset(conn, &pid)
                .map(|v| v.len()).unwrap_or(0);
            let stream_count = crate::db::nginx::get_streams_by_preset(conn, &pid)
                .map(|v| v.len()).unwrap_or(0);
            let basic_count = crate::db::nginx::get_basic_settings_by_preset(conn, &pid)
                .map(|v| v.len()).unwrap_or(0);
            Ok(serde_json::json!({
                "hasData": server_count > 0 || upstream_count > 0 || stream_count > 0 || basic_count > 0,
                "servers": server_count,
                "upstreams": upstream_count,
                "streams": stream_count,
                "basicSettings": basic_count,
            }))
        })??;
        Ok(ApiResponse::ok(stats))
    }
}
