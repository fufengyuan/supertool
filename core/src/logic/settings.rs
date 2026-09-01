use rusqlite::params;
use serde_json::{Value, json};

/// Settings module — extracted from mod.rs
///

/// 密钥轮换的明文暂存（prepare 与 commit 之间的中间态）
static PENDING_ROTATION: tokio::sync::Mutex<Option<Vec<(String, String, String, String)>>> =
    tokio::sync::Mutex::const_new(None);

impl super::CoreService {
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

    // ============ DB Connections（settings 中 db_connections 的加解密存取） ============

    /// 读取 db_connections（JSON 数组），对每项的 password 解密后返回
    pub async fn get_db_connections(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let raw: String = db
                .conn()
                .query_row(
                    "SELECT value FROM settings WHERE key = 'db_connections'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_default();
            if raw.trim().is_empty() {
                return Ok(json!([]));
            }
            let mut arr: Vec<Value> = serde_json::from_str(&raw).unwrap_or_default();
            for item in arr.iter_mut() {
                if let Some(obj) = item.as_object_mut() {
                    if let Some(pw) = obj.get("password").and_then(|v| v.as_str()) {
                        if !pw.is_empty() {
                            let dec = crate::encryption::try_decrypt_password(pw);
                            obj.insert("password".to_string(), json!(dec));
                        }
                    }
                }
            }
            Ok(json!(arr))
        });
        result
    }

    /// 保存 db_connections（JSON 数组），对每项明文 password 加密后落盘；
    /// 已是密文（Electron 格式 salt:iv:authTag:data，恰好 4 段冒号分隔）的项跳过，避免二次加密
    pub async fn set_db_connections(&self, connections: Value) -> Result<Value, String> {
        let mut arr: Vec<Value> = if let Some(list) = connections.as_array() {
            list.clone()
        } else {
            vec![connections]
        };
        for item in arr.iter_mut() {
            if let Some(obj) = item.as_object_mut() {
                if let Some(pw) = obj.get("password").and_then(|v| v.as_str()) {
                    let pw = pw.to_string();
                    if !pw.is_empty() && !looks_like_electron_ciphertext(&pw) {
                        let enc = crate::encryption::encrypt_password(&pw).await
                            .map_err(|e| format!("db_connections 密码加密失败: {e}"))?;
                        obj.insert("password".to_string(), json!(enc));
                    }
                }
            }
        }
        let value = serde_json::to_string(&arr).map_err(|e| e.to_string())?;
        self.set_setting("db_connections", &value).await
    }

    // ============ 加密密钥轮换 ============

    /// 修改加密密钥：旧密钥解密所有 `enc:` 形态密文 → 新密钥重加密，单事务提交。
    /// 必须在 encryption::set_custom_key 之前调用（仍用旧密钥读），成功后调用方再切新密钥。
    /// 返回 (重加密总数, 解密失败列表)。解密失败的条目保留原密文（不破坏数据）。
    pub async fn rotate_encryption_key_prepare(&self) -> Result<(usize, Vec<String>), String> {
        let old_key = crate::encryption::peek_active_key().await;

        // 收集所有含密文的表/列。⚠️ 约定：新增任何「入库前用 encrypt_password 加密」的列，
        // 必须同步加进 TARGETS（否则密钥轮换后该列密文用新密钥解不开，永久不可读）。
        // 当前覆盖：服务器密码、邮箱 SMTP 密码、nginx 密码、db_connections、ai_providers.apiKey。
        const TARGETS: &[(&str, &str)] = &[
            ("servers", "password"),
            ("alert_email_config", "smtp_password"),
            ("nginx_passwords", "pass"),
        ];
        // settings 里的 db_connections（JSON 数组内 password）与 ai_providers（JSON 内 apiKey）
        // 结构化处理单独做
        let mut decrypted_values: Vec<(String, String, String, String)> = Vec::new(); // (table, col, id, plain)
        let mut failed: Vec<String> = Vec::new();

        for (table, col) in TARGETS {
            // 密码列可能存裸 base64（encrypt_password 输出）或 enc: 前缀形态，
            // 统一扫非空值，交给解密函数判断格式
            let sql = format!(
                "SELECT rowid, {} FROM {} WHERE {} IS NOT NULL AND {} != ''",
                col, table, col, col
            );
            let rows: Vec<(i64, String)> = self.db_read(|conn| -> Result<Vec<(i64, String)>, String> {
                let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
                let it = stmt
                    .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(it)
            })??;
            for (rowid, enc) in rows {
                // enc: 前缀 + Tauri/AES-GCM 密文（try_decrypt 内部已处理 enc: 剥离？无——
                // 服务器加密存储为 "enc:<b64>"，解密入口 try_decrypt_password 需先剥前缀）
                let b64 = enc.strip_prefix("enc:").unwrap_or(&enc);
                match crate::encryption::decrypt_password_with_key(b64, &old_key).await {
                    Ok(plain) => decrypted_values.push((table.to_string(), col.to_string(), rowid.to_string(), plain)),
                    Err(e) => failed.push(format!("{}.{}, rowid={}: {}", table, col, rowid, e)),
                }
            }
        }

        // db_connections（settings 表 JSON）
        if let Ok(Value::String(raw)) = self.get_setting("db_connections").await {
            if let Ok(mut arr) = serde_json::from_str::<Value>(&raw) {
                if let Some(items) = arr.as_array_mut() {
                    for item in items.iter_mut() {
                        let dec_password = item.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let dec_id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                        if let (Some(pw), Some(id)) = (dec_password, dec_id) {
                            let b64 = pw.strip_prefix("enc:").unwrap_or(&pw);
                            match crate::encryption::decrypt_password_with_key(b64, &old_key).await {
                                Ok(plain) => decrypted_values.push(("settings.db_connections".to_string(), "password".to_string(), id, plain)),
                                Err(e) => failed.push(format!("db_connections, id={}: {}", id, e)),
                            }
                        }
                    }
                }
            }
        }

        // ai_providers（settings 表 JSON 内 apiKey）
        if let Ok(Value::String(raw)) = self.get_setting("ai_providers").await {
            if let Ok(mut arr) = serde_json::from_str::<Value>(&raw) {
                if let Some(items) = arr.as_array_mut() {
                    for item in items.iter_mut() {
                        let dec_key = item.get("apiKey").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let dec_id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_default();
                        if let Some(k) = dec_key {
                            let b64 = k.strip_prefix("enc:").unwrap_or(&k);
                            match crate::encryption::decrypt_password_with_key(b64, &old_key).await {
                                Ok(plain) => decrypted_values.push(("settings.ai_providers".to_string(), "apiKey".to_string(), dec_id, plain)),
                                Err(e) => failed.push(format!("ai_providers, id={}: {}", dec_id, e)),
                            }
                        }
                    }
                }
            }
        }

        let total = decrypted_values.len();
        // 保存明文清单到内存缓存，等 set_custom_key 成功后由 commit_rotation 写回
        *PENDING_ROTATION.lock().await = Some(decrypted_values);
        Ok((total, failed))
    }

    /// 把 pending 明文用**新密钥**重加密写回（单事务），此时 active key 仍是旧密钥。
    /// 全部写回成功后才由调用方切换 active key —— 若此步失败，active key 未变、
    /// 存量密文仍是旧密钥可解，重试安全，不会出现"密钥已切换但密文解不开"的不可逆态。
    /// 返回重加密成功条数；pending 无论成败都会清空。
    pub async fn commit_rotation(&self, new_key: &[u8; 32]) -> Result<usize, String> {
        let pending = PENDING_ROTATION.lock().await.take();
        let Some(items) = pending else { return Ok(0) };

        // 表级条目（settings. 前缀留给 JSON 处理）：(table, col, rowid, plain)
        let table_items: Vec<(String, String, String, String)> = items
            .iter()
            .filter(|(t, _, _, _)| !t.starts_with("settings."))
            .cloned()
            .collect();
        // settings JSON 条目
        let dbconn_plain: Vec<(String, String)> = items
            .iter()
            .filter(|(t, _, _, _)| t == "settings.db_connections")
            .map(|(_, _, id, p)| (id.clone(), p.clone()))
            .collect();
        let ai_plain: Vec<(String, String)> = items
            .iter()
            .filter(|(t, _, _, _)| t == "settings.ai_providers")
            .map(|(_, _, id, p)| (id.clone(), p.clone()))
            .collect();

        // 用新密钥一次性把所有明文加密成新密文（先于写库，加密失败不落库）
        let mut table_updates: Vec<(String, String, String, String)> = Vec::new();
        for (table, col, id, plain) in &table_items {
            let enc = crate::encryption::encrypt_password_with_key(plain, new_key).await?;
            table_updates.push((table.clone(), col.clone(), id.clone(), enc));
        }
        let mut dbconn_new: Vec<(String, String)> = Vec::new();
        for (id, plain) in &dbconn_plain {
            dbconn_new.push((id.clone(), crate::encryption::encrypt_password_with_key(plain, new_key).await?));
        }
        let mut ai_new: Vec<(String, String)> = Vec::new();
        for (id, plain) in &ai_plain {
            ai_new.push((id.clone(), crate::encryption::encrypt_password_with_key(plain, new_key).await?));
        }

        // 单事务写回：表级 UPDATE + settings JSON 重写，全成或全败
        let mut count = 0usize;
        let result: Result<Result<(), String>, String> = self.db_write_tx(|conn| {
            for (table, col, rowid, enc) in &table_updates {
                let rowid = rowid
                    .parse::<i64>()
                    .map_err(|_| format!("{} 无效 rowid: {}", table, rowid))?;
                conn.execute(
                    &format!("UPDATE {} SET \"{}\" = ?1 WHERE rowid = ?2", table, col),
                    rusqlite::params![enc, rowid],
                )
                .map_err(|e| format!("{} 重加密写回失败: {}", table, e))?;
                count += 1;
            }
            if !dbconn_new.is_empty() {
                let raw: String = conn
                    .query_row("SELECT value FROM settings WHERE key = 'db_connections'", [], |r| r.get(0))
                    .unwrap_or_default();
                if let Ok(mut arr) = serde_json::from_str::<Value>(&raw) {
                    if let Some(list) = arr.as_array_mut() {
                        for item in list.iter_mut() {
                            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some((_, enc)) = dbconn_new.iter().find(|(i, _)| i == id) {
                                if let Some(obj) = item.as_object_mut() { obj.insert("password".to_string(), json!(enc)); }
                                count += 1;
                            }
                        }
                    }
                    conn.execute(
                        "INSERT OR REPLACE INTO settings (key, value) VALUES ('db_connections', ?1)",
                        rusqlite::params![serde_json::to_string(&arr).map_err(|e| e.to_string())?],
                    )
                    .map_err(|e| format!("db_connections 写回失败: {}", e))?;
                }
            }
            if !ai_new.is_empty() {
                let raw: String = conn
                    .query_row("SELECT value FROM settings WHERE key = 'ai_providers'", [], |r| r.get(0))
                    .unwrap_or_default();
                if let Ok(mut arr) = serde_json::from_str::<Value>(&raw) {
                    if let Some(list) = arr.as_array_mut() {
                        for item in list.iter_mut() {
                            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some((_, enc)) = ai_new.iter().find(|(i, _)| i == id) {
                                if let Some(obj) = item.as_object_mut() { obj.insert("apiKey".to_string(), json!(enc)); }
                                count += 1;
                            }
                        }
                    }
                    conn.execute(
                        "INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_providers', ?1)",
                        rusqlite::params![serde_json::to_string(&arr).map_err(|e| e.to_string())?],
                    )
                    .map_err(|e| format!("ai_providers 写回失败: {}", e))?;
                }
            }
            Ok(())
        });
        result.map_err(|e| format!("db_write failed: {}", e))??;
        Ok(count)
    }

    /// 清除密钥轮换的明文暂存（任一步失败/中断时调用，避免明文长期驻留内存）
    pub async fn clear_pending_rotation(&self) {
        PENDING_ROTATION.lock().await.take();
    }

    // ============ Projects ============
}

/// Electron 版密文形态为 `salt:iv:authTag:data`（4 段、各段非空的冒号分隔字符串）。
/// 用「恰好 4 段且各段非空」而非「含冒号」判断，避免把含冒号的合法明文密码误判为密文而跳过加密。
fn looks_like_electron_ciphertext(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    parts.len() == 4 && parts.iter().all(|p| !p.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::logic::CoreService;

    use std::sync::atomic::{AtomicUsize, Ordering};
    static DB_SEQ: AtomicUsize = AtomicUsize::new(0);

    fn temp_db() -> CoreService {
        // 每个测试用独立序号，避免并行测试共用同一临时 DB 路径互相干扰
        let seq = DB_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("supertool_st_settings_test_{}_{}", std::process::id(), seq));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings_test.db");
        let _ = std::fs::remove_file(&path);
        let db = Database::new(&path).unwrap();
        CoreService::new(db, dir)
    }

    #[tokio::test]
    async fn db_connections_roundtrip_encrypts_and_decrypts() {
        let core = temp_db();
        let conns = json!([
            {"id": "c1", "name": "新连接", "type": "mysql", "password": "plain-secret"},
            {"id": "c2", "name": "空密码", "type": "sqlite", "password": ""},
            {"id": "c3", "name": "含冒号密码", "type": "postgresql", "password": "p@ss:word"},
        ]);
        core.set_db_connections(conns).await.unwrap();

        // 落盘应为密文（明文不得直接写入 settings）
        let raw: String = core
            .db_read(|c| {
                c.query_row(
                    "SELECT value FROM settings WHERE key='db_connections'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_default()
            })
            .unwrap();
        assert!(!raw.contains("plain-secret"), "明文不应直接落盘: {}", raw);
        assert!(!raw.contains("p@ss:word"), "含冒号的明文密码也应加密落盘: {}", raw);

        // 读取应解回明文
        let got = core.get_db_connections().await.unwrap();
        let arr = got.as_array().unwrap();
        assert_eq!(arr[0]["password"], "plain-secret");
        assert_eq!(arr[1]["password"], "");
        assert_eq!(arr[2]["password"], "p@ss:word");
    }

    #[tokio::test]
    async fn get_decrypts_legacy_electron_ciphertext() {
        // 注入可控 Electron 口令，让解密逻辑自足（不依赖宿主 ~/.supertool 里的密钥文件）
        crate::encryption::set_electron_secret_for_test(Some("legacy-passphrase".to_string()));
        let core = temp_db();
        // 用注入的口令构造 Electron 密文（salt:iv:authTag:data，含冒号，不应被再次加密）
        let legacy = crate::encryption::encrypt_password_electron("plain-secret").unwrap();
        assert!(legacy.contains(':'), "Electron 密文应为 colon 分隔，实际: {}", legacy);
        let conns = json!([{"id": "c1", "name": "旧连接", "type": "mysql", "password": legacy}]);
        core.set_setting("db_connections", &serde_json::to_string(&conns).unwrap())
            .await
            .unwrap();

        let got = core.get_db_connections().await.unwrap();
        let arr = got.as_array().unwrap();
        let dec = arr[0]["password"].as_str().unwrap();
        assert_ne!(dec, legacy, "Electron 密文应被解密");
        assert!(!dec.contains(':'), "解密结果不应仍是密文形态，实际: {}", dec);
        assert_eq!(dec, "plain-secret", "应解回明文");
    }
}
