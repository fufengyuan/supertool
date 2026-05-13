use crate::logic::CoreService;
use crate::db::ApiResponse;
use crate::db::nginx::{
    NginxServer, NginxLocation, NginxUpstream, NginxUpstreamServer,
    NginxHttpParam, NginxStream, NginxCert, NginxTemplate, NginxBasicSetting,
};

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
        let passed =
            output.contains("syntax is ok") || output.contains("test is successful");
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
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
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
                if rb_result.success { "成功" } else { "也失败" }
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
                if rb_result.success { "成功" } else { "也失败" }
            )));
        }

        // 4. Reload nginx (try systemctl first, fallback to nginx -s)
        let sid6 = sid.clone();
        let reload_result = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.exec_command(
                    &sid6,
                    "systemctl reload nginx 2>&1 || nginx -s reload 2>&1",
                )
            })
            .await?;

        Ok(ApiResponse::ok(NginxDeployResult {
            success: true,
            backup_path,
            message: format!(
                "配置已部署。重载: {}",
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
            Ok(ApiResponse::err(format!("回滚失败: {}", result.output.trim())))
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
        Ok(self.with_db(move |db| {
            crate::db::nginx::set_current_version(db, &pid, &vid)
        }))
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

    pub async fn add_nginx_server(
        &self,
        server: &NginxServer,
    ) -> Result<ApiResponse<()>, String> {
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

    pub async fn add_nginx_stream(
        &self,
        stream: &NginxStream,
    ) -> Result<ApiResponse<()>, String> {
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

    pub async fn add_nginx_cert(
        &self,
        cert: &NginxCert,
    ) -> Result<ApiResponse<()>, String> {
        let c = cert.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::add_nginx_cert(conn, &c).map_err(|e| e.to_string())
        })??)
        .map(|_| ApiResponse::ok(()))
    }

    pub async fn update_nginx_cert(
        &self,
        cert: &NginxCert,
    ) -> Result<ApiResponse<()>, String> {
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

    // ============ NginxBasicSetting CRUD ============

    pub async fn get_basic_setting(
        &self,
        preset_id: &str,
    ) -> Result<ApiResponse<Option<NginxBasicSetting>>, String> {
        let pid = preset_id.to_string();
        let result = self.db_read(move |conn| -> Result<Option<NginxBasicSetting>, String> {
            crate::db::nginx::get_basic_setting(conn, &pid).map_err(|e| e.to_string())
        })??;
        Ok(ApiResponse::ok(result))
    }

    pub async fn upsert_basic_setting(
        &self,
        setting: &NginxBasicSetting,
    ) -> Result<ApiResponse<()>, String> {
        let s = setting.clone();
        Ok(self.db_write(move |conn| -> Result<(), String> {
            crate::db::nginx::upsert_nginx_basic_setting(conn, &s).map_err(|e| e.to_string())
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
}
