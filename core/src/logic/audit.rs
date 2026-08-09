//! 操作审计：记录 CLI / GUI / AI 的写操作（参数已脱敏），供追溯与合规。
//! 表结构见 core/src/db/mod.rs 的 `audit_logs`。

use crate::logic::CoreService;
use rusqlite::params;
use serde_json::{Value, json};

/// 审计记录条目（list_audit 返回结构）
#[derive(Debug, serde::Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub actor_type: String,
    pub actor_name: String,
    pub command: String,
    pub args_json: String,
    pub target: String,
    pub result: String,
    pub duration_ms: i64,
    pub created_at: String,
}

impl CoreService {
    /// 写入一条审计记录。调用方负责参数脱敏（应经 log_sanitizer）。
    /// 审计失败静默返回 Err，不阻塞主流程（由调用方决定是否忽略）。
    pub fn record_audit(
        &self,
        actor_type: &str,
        actor_name: &str,
        command: &str,
        args_json: &str,
        target: &str,
        result: &str,
        duration_ms: i64,
    ) -> Result<(), String> {
        let created_at = chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        self.with_db(|db| {
            let conn = db.conn();
            conn.execute(
                "INSERT INTO audit_logs (actor_type, actor_name, command, args_json, target, result, duration_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    actor_type,
                    actor_name,
                    command,
                    args_json,
                    target,
                    result,
                    duration_ms,
                    created_at
                ],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        })
    }

    /// 查询审计记录（按时间倒序），支持按 actor_type / result 过滤
    pub fn list_audit(
        &self,
        actor_type: Option<&str>,
        result: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Value, String> {
        let limit = limit.clamp(1, 1000);
        self.with_db(|db| {
            let conn = db.conn();
            let mut sql = String::from(
                "SELECT id, actor_type, actor_name, command, args_json, target, result, duration_ms, created_at
                 FROM audit_logs WHERE 1=1",
            );
            let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if let Some(a) = actor_type {
                sql.push_str(" AND actor_type = ?");
                params_vec.push(Box::new(a.to_string()));
            }
            if let Some(r) = result {
                sql.push_str(" AND result = ?");
                params_vec.push(Box::new(r.to_string()));
            }
            sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");
            params_vec.push(Box::new(limit as i64));
            params_vec.push(Box::new(offset as i64));

            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("审计查询失败: {}", e))?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())), |row| {
                    Ok(AuditEntry {
                        id: row.get(0)?,
                        actor_type: row.get(1)?,
                        actor_name: row.get(2)?,
                        command: row.get(3)?,
                        args_json: row.get(4)?,
                        target: row.get(5)?,
                        result: row.get(6)?,
                        duration_ms: row.get(7)?,
                        created_at: row.get(8)?,
                    })
                })
                .map_err(|e| format!("审计查询失败: {}", e))?;

            let mut entries: Vec<Value> = Vec::new();
            for row in rows.flatten() {
                entries.push(json!({
                    "id": row.id,
                    "actorType": row.actor_type,
                    "actorName": row.actor_name,
                    "command": row.command,
                    "argsJson": row.args_json,
                    "target": row.target,
                    "result": row.result,
                    "durationMs": row.duration_ms,
                    "createdAt": row.created_at,
                }));
            }
            Ok(Value::Array(entries))
        })
    }
}
