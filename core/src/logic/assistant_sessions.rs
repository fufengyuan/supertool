use rusqlite::params;
use serde_json::{Value, json};

/// AI 配置助手历史会话 —— 会话列表 / 详情 / 保存 / 删除。
/// 消息以 JSON 数组整体落库（`messages` 列），会话标题由前端取首条用户消息生成；
/// 敏感字段（密码/密钥等）在写入前已由前端脱敏，这里不再二次处理。
impl super::CoreService {
    /// 会话列表：只返回元信息（不含消息体），供侧边栏展示
    pub async fn list_assistant_sessions(&self) -> Result<Value, String> {
        self.list_assistant_sessions_full(false).await
    }

    /// 会话列表，`with_messages` 为 true 时返回完整消息体
    pub async fn list_assistant_sessions_full(&self, with_messages: bool) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare(
                    "SELECT id, title, messages, createdAt, updatedAt FROM assistant_sessions ORDER BY updatedAt DESC",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "title": row.get::<_, String>("title")?,
                        "messages": if with_messages { row.get::<_, String>("messages")? } else { "[]".to_string() },
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let mut out: Vec<Value> = Vec::new();
            for r in rows {
                let mut v = r.map_err(|e| e.to_string())?;
                // messages 列是 JSON 字符串，反序列化成数组下发（列表模式下用空数组占位）
                if let Some(s) = v.get_mut("messages").and_then(|m| m.as_str()) {
                    v["messages"] = serde_json::from_str::<Value>(if with_messages { s } else { "[]" })
                        .unwrap_or(Value::Array(vec![]));
                }
                out.push(v);
            }
            Ok(json!(out))
        });
        result
    }

    /// 读取单个会话（含消息体），不存在返回 null
    pub async fn get_assistant_session(&self, id: &str) -> Result<Value, String> {
        let id = id.to_string();
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare(
                    "SELECT id, title, messages, createdAt, updatedAt FROM assistant_sessions WHERE id = ?1",
                )
                .map_err(|e| e.to_string())?;
            let mut rows = stmt
                .query_map(params![id], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "title": row.get::<_, String>("title")?,
                        "messages": row.get::<_, String>("messages")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                        "updatedAt": row.get::<_, String>("updatedAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            if let Some(row) = rows.next() {
                let mut v = row.map_err(|e| e.to_string())?;
                if let Some(s) = v.get_mut("messages").and_then(|m| m.as_str()) {
                    v["messages"] =
                        serde_json::from_str::<Value>(s).unwrap_or(Value::Array(vec![]));
                }
                Ok(v)
            } else {
                Ok(Value::Null)
            }
        });
        result
    }

    /// 保存会话：存在即更新（UPSERT），不存在即新建。messages 为 JSON 数组字符串。
    pub async fn save_assistant_session(
        &self,
        id: &str,
        title: &str,
        messages: &str,
    ) -> Result<Value, String> {
        let id = id.to_string();
        let title = title.to_string();
        let messages = messages.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO assistant_sessions (id, title, messages, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?4)
                     ON CONFLICT(id) DO UPDATE SET title = ?2, messages = ?3, updatedAt = ?4",
                    params![id, title, messages, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "title": title, "updatedAt": now}))
    }

    /// 删除会话（最后一条会话被删后会由前端自动新建空会话）
    pub async fn delete_assistant_session(&self, id: &str) -> Result<Value, String> {
        let id = id.to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM assistant_sessions WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"ok": true, "id": id}))
    }
}

#[cfg(test)]
mod assistant_session_tests {
    use super::*;

    fn temp_core(tag: &str) -> crate::logic::CoreService {
        let dir = std::env::temp_dir().join(format!(
            "st_ast_sess_{}_{}",
            tag,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        crate::logic::CoreService::new(
            crate::db::Database::new(&dir.join("t.db")).unwrap(),
            dir,
        )
    }

    #[tokio::test]
    async fn save_list_get_delete_roundtrip() {
        let core = temp_core("crud");
        let sess = core
            .save_assistant_session("s1", "给商城加一台服务器", r#"[{"id":"m1","role":"user","text":"你好"}]"#)
            .await
            .expect("保存会话应成功");
        assert_eq!(sess["id"], "s1");
        assert_eq!(sess["title"], "给商城加一台服务器");

        // 列表（不含消息体）
        let list = core.list_assistant_sessions().await.unwrap();
        assert_eq!(list.as_array().unwrap().len(), 1);
        assert_eq!(list[0]["id"], "s1");
        assert_eq!(list[0]["messages"].as_array().unwrap().len(), 0, "列表不含消息体");

        // 详情（含消息体）
        let got = core.get_assistant_session("s1").await.unwrap();
        assert_eq!(got["messages"].as_array().unwrap().len(), 1, "详情含消息体");
        assert_eq!(got["messages"][0]["text"], "你好");

        // 覆盖保存（UPSERT 更新标题与消息）
        core.save_assistant_session("s1", "改名后的会话", r#"[]"#)
            .await
            .unwrap();
        let re = core.get_assistant_session("s1").await.unwrap();
        assert_eq!(re["title"], "改名后的会话");

        // 不存在的会话返回 null
        assert!(core.get_assistant_session("nope").await.unwrap().is_null());

        // 删除
        core.delete_assistant_session("s1").await.unwrap();
        let list2 = core.list_assistant_sessions().await.unwrap();
        assert_eq!(list2.as_array().unwrap().len(), 0);
    }
}