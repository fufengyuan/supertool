use super::ssh;
use crate::db::servers;
use serde_json::{Value, json};

impl super::CoreService {
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
        let (password, ssh_key_path) =
            if param_pw.is_none() && param_key.is_none() && !server_id.is_empty() {
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
        let (password, ssh_key_path) =
            if param_pw.is_none() && param_key.is_none() && !server_id.is_empty() {
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
        let result = self
            .run_ssh_with_retry(server_id, move |ssh| ssh.exec_command(&sid, &cmd))
            .await?;
        Ok(json!(result))
    }
    /// 使用独立 SSH 连接批量执行命令（不共享连接池，不影响终端）
    pub async fn exec_ssh_commands_independent(
        &self,
        params: Value,
        commands: Vec<String>,
    ) -> Result<Value, String> {
        let config = ssh::SshServerConfig {
            id: params["id"].as_str().unwrap_or("").to_string(),
            name: params["name"].as_str().unwrap_or("").to_string(),
            host: params["host"].as_str().unwrap_or("").to_string(),
            port: params["port"].as_u64().unwrap_or(22) as u32,
            username: params["username"].as_str().unwrap_or("").to_string(),
            password: params
                .get("password")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            ssh_key_path: params
                .get("sshKeyPath")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
        };
        let ssh = self.ssh.clone();
        let results =
            tokio::task::spawn_blocking(move || ssh.exec_commands_independent(&config, &commands))
                .await
                .map_err(|e| format!("SSH 批量命令执行失败: {}\n", e))??;
        let mut json_results = serde_json::Map::new();
        for (cmd, result) in results {
            json_results.insert(
                cmd,
                json!({
                    "output": result.output,
                    "success": result.success,
                    "exitCode": result.exit_code,
                }),
            );
        }
        Ok(json!({ "success": true, "results": json_results }))
    }
    pub async fn sftp_list_dir(&self, server_id: &str, remote_path: &str) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        let files = self
            .run_ssh_with_retry(server_id, move |ssh| ssh.list_remote_dir(&sid, &rp))
            .await?;
        Ok(json!({"success": true, "files": files}))
    }
    pub async fn sftp_download_file(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        let content = self
            .run_ssh_with_retry(server_id, move |ssh| ssh.download_file_base64(&sid, &rp))
            .await?;
        Ok(json!({"content": content}))
    }
    pub async fn sftp_create_dir(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        self.run_ssh_with_retry(server_id, move |ssh| ssh.create_remote_dir(&sid, &rp))
            .await?;
        Ok(json!({"success": true}))
    }
    pub async fn sftp_delete_file(
        &self,
        server_id: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let sid = server_id.to_string();
        let rp = remote_path.to_string();
        self.run_ssh_with_retry(server_id, move |ssh| ssh.delete_remote_file(&sid, &rp))
            .await?;
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
        let size = self
            .run_ssh_with_retry(server_id, move |ssh| ssh.download_file(&sid, &rp, &lp))
            .await?;
        Ok(json!({"success": true, "data": {"bytesDownloaded": size, "localPath": local_path}}))
    }
    /// SFTP: 上传文件到远程
    pub async fn sftp_upload_to_remote(
        &self,
        server_id: &str,
        local_path: &str,
        remote_path: &str,
    ) -> Result<Value, String> {
        let metadata = tokio::fs::metadata(local_path)
            .await
            .map_err(|e| format!("读取本地文件失败: {}", e))?;
        if metadata.is_dir() {
            // 目录 → 递归上传
            let sid = server_id.to_string();
            let lp = local_path.to_string();
            let rp = remote_path.to_string();
            let size = self
                .run_ssh_with_retry(server_id, move |ssh| {
                    ssh.upload_dir_recursive(&sid, &lp, &rp)
                })
                .await?;
            Ok(json!({"success": true, "data": {"bytesUploaded": size, "remotePath": remote_path}}))
        } else {
            // 文件 → 单文件上传
            let sid = server_id.to_string();
            let lp = local_path.to_string();
            let rp = remote_path.to_string();
            let size = self
                .run_ssh_with_retry(server_id, move |ssh| ssh.upload_file(&sid, &lp, &rp))
                .await?;
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
        let size = self
            .run_ssh_with_retry(server_id, move |ssh| {
                ssh.upload_dir_recursive(&sid, &lp, &rp)
            })
            .await?;
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
        self.run_ssh_blocking(move |ssh| ssh.create_terminal(&sid, &tid, rows, cols))
            .await?;
        Ok(json!({"success": true, "terminalId": terminal_id}))
    }
    pub async fn ssh_read_terminal(&self, terminal_id: &str) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        let data = self
            .run_ssh_blocking(move |ssh| ssh.read_terminal(&tid))
            .await?;
        Ok(json!({"success": true, "data": data}))
    }
    pub async fn ssh_write_to_terminal(
        &self,
        terminal_id: &str,
        data: &str,
    ) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        let d = data.to_string();
        self.run_ssh_blocking(move |ssh| ssh.write_to_terminal(&tid, &d))
            .await?;
        Ok(json!({"success": true}))
    }
    pub async fn ssh_resize_terminal(
        &self,
        terminal_id: &str,
        rows: u32,
        cols: u32,
    ) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        self.run_ssh_blocking(move |ssh| ssh.resize_terminal(&tid, rows, cols))
            .await?;
        Ok(json!({"success": true}))
    }
    pub async fn ssh_close_terminal(&self, terminal_id: &str) -> Result<Value, String> {
        let tid = terminal_id.to_string();
        self.run_ssh_blocking(move |ssh| ssh.close_terminal(&tid))
            .await?;
        Ok(json!({"success": true}))
    }
    pub async fn ssh_is_terminal_active(&self, terminal_id: &str) -> bool {
        self.ssh.is_terminal_active(terminal_id)
    }
    // ============ Git 操作包装方法 ============
}
