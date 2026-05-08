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

impl CoreService {
    /// Fetch nginx config content from remote server
    pub async fn fetch_nginx_config(
        &self,
        server_id: &str,
        config_path: &str,
    ) -> Result<ApiResponse<String>, String> {
        let sid = server_id.to_string();
        let path = config_path.to_string();
        let result = self
            .run_ssh_blocking(move |ssh| ssh.exec_command(&sid, &format!("cat {}", path)))
            .await?;
        Ok(ApiResponse::ok(result.output))
    }

    /// Test nginx config on remote server (nginx -t)
    pub async fn test_nginx_config(
        &self,
        server_id: &str,
    ) -> Result<ApiResponse<NginxTestResult>, String> {
        let sid = server_id.to_string();
        let result = self
            .run_ssh_blocking(move |ssh| ssh.exec_command(&sid, "nginx -t 2>&1"))
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
        _comment: &str,
    ) -> Result<ApiResponse<NginxDeployResult>, String> {
        let sid = server_id.to_string();
        let path = config_path.to_string();
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let backup_path = format!("{}.bak.{}", path, ts);

        // 1. Backup current config
        let sid2 = sid.clone();
        let backup_cmd = format!("cp {} {}", path, backup_path);
        self.run_ssh_blocking(move |ssh| ssh.exec_command(&sid2, &backup_cmd))
            .await?;

        // 2. Write new config via base64 to avoid shell escaping issues
        let encoded =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, content);
        let write_cmd = format!(
            "echo '{}' | base64 -d > {}",
            encoded, path
        );
        let sid3 = sid.clone();
        self.run_ssh_blocking(move |ssh| ssh.exec_command(&sid3, &write_cmd))
            .await?;

        // 3. Test new config
        let sid4 = sid.clone();
        let test_result = self
            .run_ssh_blocking(move |ssh| ssh.exec_command(&sid4, "nginx -t 2>&1"))
            .await?;
        if !test_result.output.contains("syntax is ok")
            && !test_result.output.contains("test is successful")
        {
            // Rollback: restore backup
            let sid5 = sid.clone();
            let rb_cmd = format!("cp {} {}", backup_path, path);
            self.run_ssh_blocking(move |ssh| ssh.exec_command(&sid5, &rb_cmd))
                .await?;
            return Ok(ApiResponse::err(format!(
                "nginx -t failed, rolled back: {}",
                test_result.output
            )));
        }

        // 4. Reload nginx
        let sid6 = sid.clone();
        self.run_ssh_blocking(move |ssh| {
            ssh.exec_command(&sid6, "nginx -s reload 2>&1")
        })
        .await?;

        Ok(ApiResponse::ok(NginxDeployResult {
            success: true,
            backup_path,
            message: "Config deployed and nginx reloaded".to_string(),
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
        let path = config_path.to_string();
        let bp = backup_path.to_string();
        let cmd = format!("cp {} {} && nginx -t 2>&1 && nginx -s reload", bp, path);
        let result = self
            .run_ssh_blocking(move |ssh| ssh.exec_command(&sid, &cmd))
            .await?;
        Ok(ApiResponse::ok(result.output))
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
