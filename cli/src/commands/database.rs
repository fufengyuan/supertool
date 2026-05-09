use crate::runtime::CliRuntime;
use crate::types::*;
use crate::output::*;
use anyhow::Result;

pub async fn cmd_db(rt: &mut CliRuntime, action: &DbCommands) -> Result<()> {
    check_db(rt).await?;
    match action {
        DbCommands::List { json } => {
            let conns = get_db_connections(rt).await?;
            if *json {
                print_json(&conns);
            } else {
                println!("\n  数据库连接 ({}):", conns.len());
                // 按类型分组
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for c in &conns {
                    let r#type = c.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_uppercase();
                    if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == &r#type) {
                        items.push(c);
                    } else {
                        groups.push((r#type, vec![c]));
                    }
                }
                for (r#type, items) in &groups {
                    println!("▸ {}", r#type);
                    for c in items {
                        let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}  {}", id, name);
                    }
                    println!();
                }
            }
        }
        DbCommands::Disconnect { id } => {
            anyhow::bail!("⚠️ CLI 暂不支持数据库断开连接，请使用 GUI 操作。")
        }
        DbCommands::Query { db_id, sql, .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持外部数据库查询 ({}), 请使用 GUI 或直连数据库。", db_id)
        }
        DbCommands::Tables { db_id, .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持获取外部数据库表结构 ({}), 请使用 GUI。", db_id)
        }
        DbCommands::Databases { db_id, .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持获取外部数据库列表 ({}), 请使用 GUI。", db_id)
        }
        DbCommands::Redis { db_id, action } => cmd_redis(db_id, action)?,
    }
    Ok(())
}

pub fn cmd_redis(db_id: &str, action: &RedisCommands) -> Result<()> {
    match action {
        RedisCommands::Keys { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::Get { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::Set { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::Delete { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::Type { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::Ttl { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::HGet { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::HGetAll { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::HLen { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::LRange { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::LLen { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::SMembers { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
        RedisCommands::SCard { .. } => {
            anyhow::bail!("⚠️ CLI 暂不支持 Redis 操作 ({}), 请使用 GUI 或 redis-cli。", db_id)
        }
    }
}

async fn check_db(rt: &mut CliRuntime) -> Result<()> {
    let _ = rt.core
        .db_read(|conn| conn.prepare("SELECT 1").map(|_| ()))
        .map_err(|e| anyhow::anyhow!("数据库连接失败: {}", e))?;
    Ok(())
}

async fn get_db_connections(rt: &mut CliRuntime) -> Result<Vec<serde_json::Value>> {
    let setting = rt.core.get_setting("db_connections").await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let connections: Vec<serde_json::Value> = match setting {
        serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
        serde_json::Value::Array(arr) => arr,
        _ => vec![],
    };
    // 移除密码字段，规范化 requiresApproval
    Ok(connections.into_iter().map(|mut c| {
        if let Some(obj) = c.as_object_mut() {
            obj.remove("password");
            let ra = obj.get("requiresApproval").and_then(|v| v.as_bool()).unwrap_or(false);
            obj.insert("requiresApproval".to_string(), serde_json::Value::Bool(ra));
        }
        c
    }).collect())
}
