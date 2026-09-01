use super::ssh;
use crate::db::Server;
use crate::db::ServerGroup;
use crate::db::servers;
use crate::encryption::encrypt_password;
use serde_json::{Value, json};

/// 规范化服务器的认证字段：密钥与密码**二选一**
///
/// ## 为什么需要
/// GUI 表单对「未配置密钥」提交的是空字符串 `''`（不是 NULL）。
/// 原样入库后 `sshKeyPath` 既不是 NULL 也不是有效路径，下游 ssh2 会去打开空路径并报
/// `Unable to open private key file`；又因密钥认证先于密码认证，
/// 配了密码的服务器会永远走不到密码分支（表现为 GUI 能连、CLI 却报错）。
///
/// ## 规则
/// - 用密钥认证：写入 trim 后的密钥路径，并把密码置为空串（db 层据此清成 NULL）
/// - 用密码认证：密钥路径一律写 NULL（绝不留空串），密码原样保留
/// - 选了「密钥」却没填路径：视为无效并回退到密码认证，避免把密码也一起清掉
/// - 未显式传 `authType`（CLI / MCP / 旧客户端）：有密钥路径即视为密钥认证
fn normalize_server_auth(params: &mut Value) {
    let key_path = params
        .get("sshKeyPath")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .unwrap_or("");
    let explicit = params
        .get("authType")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_ascii_lowercase());

    let use_key = match explicit.as_deref() {
        Some("key") => !key_path.is_empty(),
        Some("password") => false,
        _ => !key_path.is_empty(),
    };

    if use_key {
        params["sshKeyPath"] = json!(key_path);
        // 空串是给 db::update_server 的「显式清空密码」信号
        params["password"] = json!("");
    } else {
        params["sshKeyPath"] = json!(null);
    }
}

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
        normalize_server_auth(&mut params);
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
            if pwd.is_empty() {
                // 空密码一律存 NULL，避免库里出现 '' 这种歧义值
                server.password = None;
            } else if !pwd.starts_with("enc:") {
                server.password = Some(encrypt_password(pwd).await.map_err(|e| e.to_string())?);
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
        let mut params = params.clone();
        normalize_server_auth(&mut params);
        let mut server =
            serde_json::from_value::<Server>(params.clone()).map_err(|e| e.to_string())?;
        // 只在明文密码时加密，避免重复加密已加密的密码
        if let Some(ref pwd) = server.password {
            if !pwd.is_empty() && !pwd.starts_with("enc:") {
                server.password = Some(encrypt_password(pwd).await.map_err(|e| e.to_string())?);
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
            // 显式选了「密码」认证时忽略密钥路径，与保存时的互斥规则保持一致
            ssh_key_path: match params.get("authType").and_then(|v| v.as_str()) {
                Some("password") => None,
                _ => params
                    .get("sshKeyPath")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 密码认证：密钥路径必须写 NULL，绝不留空串
    #[test]
    fn password_auth_clears_ssh_key_path() {
        let mut p = json!({
            "id": "s1", "sshKeyPath": "", "password": "secret", "authType": "password"
        });
        normalize_server_auth(&mut p);
        assert!(p["sshKeyPath"].is_null(), "空串必须被规范成 NULL");
        assert_eq!(p["password"].as_str(), Some("secret"));
    }

    /// 密钥认证：清空密码（空串是给 db::update_server 的「显式清空」信号）
    #[test]
    fn key_auth_clears_password() {
        let mut p = json!({
            "id": "s1", "sshKeyPath": "  ~/.ssh/id_ed25519_github  ", "password": "secret", "authType": "key"
        });
        normalize_server_auth(&mut p);
        assert_eq!(p["sshKeyPath"].as_str(), Some("~/.ssh/id_ed25519_github"));
        assert_eq!(p["password"].as_str(), Some(""));
    }

    /// 选了密钥却没填路径：回退到密码认证，不能把密码也一起清掉
    #[test]
    fn key_auth_without_path_falls_back_to_password() {
        let mut p = json!({
            "id": "s1", "sshKeyPath": "", "password": "secret", "authType": "key"
        });
        normalize_server_auth(&mut p);
        assert!(p["sshKeyPath"].is_null());
        assert_eq!(p["password"].as_str(), Some("secret"), "不能把密码也清掉");
    }

    /// 未传 authType（CLI / MCP / 老客户端）：按有没有密钥路径推导
    #[test]
    fn auth_type_is_inferred_when_absent() {
        let mut p = json!({"id": "s1", "sshKeyPath": "", "password": "secret"});
        normalize_server_auth(&mut p);
        assert!(p["sshKeyPath"].is_null());
        assert_eq!(p["password"].as_str(), Some("secret"));

        let mut p = json!({"id": "s1", "sshKeyPath": "/root/.ssh/id_rsa", "password": "secret"});
        normalize_server_auth(&mut p);
        assert_eq!(p["sshKeyPath"].as_str(), Some("/root/.ssh/id_rsa"));
        assert_eq!(p["password"].as_str(), Some(""));
    }
}
