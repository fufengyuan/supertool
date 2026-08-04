use rusqlite::params;
use serde_json::{Value, json};

/// Settings module — extracted from mod.rs
///

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
                        let enc = crate::encryption::encrypt_password(&pw)
                            .map_err(|e| format!("db_connections 密码加密失败: {e}"))?;
                        obj.insert("password".to_string(), json!(enc));
                    }
                }
            }
        }
        let value = serde_json::to_string(&arr).map_err(|e| e.to_string())?;
        self.set_setting("db_connections", &value).await
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
        let core = temp_db();
        // 模拟旧版 Electron 遗留密文（salt:iv:authTag:data，含冒号，不应被再次加密）
        let legacy = "wmVifu6/z6hV2mIquJNA3A==:8It3qFuDuPsDnCo6:J+epAapVg4ivn81fpJFvFg==:20f0Md8yd+ZCfic3eDmIti0=";
        let conns = json!([{"id": "c1", "name": "旧连接", "type": "mysql", "password": legacy}]);
        core.set_setting("db_connections", &serde_json::to_string(&conns).unwrap())
            .await
            .unwrap();

        let got = core.get_db_connections().await.unwrap();
        let arr = got.as_array().unwrap();
        let dec = arr[0]["password"].as_str().unwrap();
        assert_ne!(dec, legacy, "Electron 密文应被解密");
        assert!(!dec.contains(':'), "解密结果不应仍是密文形态，实际: {}", dec);
    }
}
