use super::ssh;
use crate::db::Server;
use crate::db::ServerGroup;
use crate::db::servers;
use crate::encryption::encrypt_password;
use serde_json::{Value, json};

impl super::CoreService {
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
        if params
            .get("id")
            .and_then(|v| v.as_str())
            .map_or(true, |s| s.is_empty())
        {
            params["id"] = json!(uuid::Uuid::new_v4().to_string());
        }
        let now = chrono::Utc::now().to_rfc3339();
        if params.get("createdAt").is_none() {
            params["createdAt"] = json!(now);
        }
        if params.get("updatedAt").is_none() {
            params["updatedAt"] = json!(now);
        }
        if params.get("tags").is_none() {
            params["tags"] = json!([]);
        }
        if params.get("description").is_none() {
            params["description"] = json!("");
        }
        if params.get("requiresApproval").is_none() {
            params["requiresApproval"] = json!(false);
        }
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
}
