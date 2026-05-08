use crate::core::CoreService;
use crate::db::ApiResponse;

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
        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let result = self
            .run_ssh_blocking(move |ssh| {
                ssh.exec_command(&sid, &format!("cat '{}' 2>&1", safe_path))
            })
            .await?;
        if !result.success {
            return Ok(ApiResponse::err(format!(
                "Failed to read config: {}",
                result.output.trim()
            )));
        }
        Ok(ApiResponse::ok(result.output))
    }

    /// Test nginx config on remote server (nginx -t -c <path>)
    pub async fn test_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
    ) -> Result<ApiResponse<NginxTestResult>, String> {
        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let result = self
            .run_ssh_blocking(move |ssh| {
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
            .run_ssh_blocking(move |ssh| {
                ssh.exec_command(&sid2, &format!("cp '{}' '{}' 2>&1", sp2, sb2))
            })
            .await?;
        if !backup_result.success {
            return Ok(ApiResponse::err(format!(
                "Backup failed: {}",
                backup_result.output.trim()
            )));
        }

        // 2. Write new config via base64 to avoid shell escaping issues
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
        let sid3 = sid.clone();
        let sp3 = safe_path.clone();
        let write_result = self
            .run_ssh_blocking(move |ssh| {
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
                "Write failed{}. Rollback: {}",
                write_result.output.trim(),
                if rb_result.success { "ok" } else { "ALSO FAILED" }
            )));
        }

        // 3. Test new config
        let sid4 = sid.clone();
        let sp4 = safe_path.clone();
        let test_result = self
            .run_ssh_blocking(move |ssh| {
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
                "nginx -t failed: {}. Rollback: {}",
                test_result.output.trim(),
                if rb_result.success { "ok" } else { "ALSO FAILED" }
            )));
        }

        // 4. Reload nginx (try systemctl first, fallback to nginx -s)
        let sid6 = sid.clone();
        let reload_result = self
            .run_ssh_blocking(move |ssh| {
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
                "Config deployed. Reload: {}",
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
        let sid = server_id.to_string();
        let safe_path = shell_escape_path(config_path);
        let safe_backup = shell_escape_path(backup_path);
        let cmd = format!(
            "cp '{}' '{}' 2>&1 && nginx -t -c '{}' 2>&1 && (systemctl reload nginx 2>&1 || nginx -s reload 2>&1)",
            safe_backup, safe_path, safe_path
        );
        let result = self
            .run_ssh_blocking(move |ssh| ssh.exec_command(&sid, &cmd))
            .await?;
        if result.output.contains("syntax is ok") || result.output.contains("test is successful") {
            Ok(ApiResponse::ok(result.output))
        } else {
            Ok(ApiResponse::err(format!("Rollback failed: {}", result.output.trim())))
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
}
