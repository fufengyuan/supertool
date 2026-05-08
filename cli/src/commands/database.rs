use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_db(client: &ApiClient, action: &DbCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        DbCommands::List { json } => {
            let resp: serde_json::Value = client.request("db:list", None)?;
            let conns = resp
                .get("connections")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
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
            let conns: Vec<serde_json::Value> = client.request("db:list", None).unwrap_or_default();
            let name = conns
                .iter()
                .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
                .and_then(|c| c.get("name").and_then(|v| v.as_str()))
                .unwrap_or(id.as_str());
            let resp: serde_json::Value =
                client.request("db:disconnect", Some(&serde_json::json!({"id": id})))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("数据库已断开: {}", name));
            } else {
                anyhow::bail!(
                    "断开失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        DbCommands::Query { db_id, sql, json } => {
            let resp: serde_json::Value =
                client.request("db:query", Some(&serde_json::json!({"dbId": db_id, "sql": sql})))?;
            // Check if approval is required
            if resp
                .get("requiresApproval")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let msg = resp
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("数据库已开启安全审核");
                anyhow::bail!("⚠️ {}\n请使用 GUI 执行 SQL 查询。", msg);
            }
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(rows) = resp.get("rows").and_then(|v| v.as_array()) {
                    if *json {
                        print_json(&rows);
                    } else {
                        if rows.is_empty() {
                            println!("  (空结果)");
                        } else {
                            if let Some(first) = rows.first().and_then(|r| r.as_object()) {
                                let headers: Vec<String> = first.keys().map(|k| k.clone()).collect();
                                println!("  {}", headers.join(" | "));
                                println!("  {}", "─".repeat(60));
                            }
                            for row in rows {
                                if let Some(obj) = row.as_object() {
                                    let vals: Vec<String> = obj
                                        .values()
                                        .map(|v| v.as_str().unwrap_or(&v.to_string()).to_string())
                                        .collect();
                                    println!("  {}", vals.join(" | "));
                                }
                            }
                            println!("\n  共 {} 条记录", rows.len());
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        DbCommands::Tables { db_id, db, json } => {
            let resp: serde_json::Value = client.request(
                "db:get-tables",
                Some(&serde_json::json!({"dbId": db_id, "db": db.as_deref().unwrap_or("")})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let tables = resp
                    .get("tables")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if *json {
                    print_json(&tables);
                } else {
                    println!("  表 ({}):", tables.len());
                    for t in &tables {
                        if let Some(s) = t.as_str() {
                            println!("    {}", s);
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "获取失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        DbCommands::Databases { db_id, json } => {
            let resp: serde_json::Value = client.request(
                "db:get-databases",
                Some(&serde_json::json!({"dbId": db_id})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let dbs = resp
                    .get("databases")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if *json {
                    print_json(&dbs);
                } else {
                    println!("  数据库 ({}):", dbs.len());
                    for d in &dbs {
                        if let Some(s) = d.as_str() {
                            println!("    {}", s);
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "获取失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        DbCommands::Redis { db_id, action } => cmd_redis(client, db_id, action)?,
    }
    Ok(())
}

pub fn cmd_redis(client: &ApiClient, db_id: &str, action: &RedisCommands) -> Result<()> {
    // Check if connection requires approval (for destructive Redis operations)
    match action {
        RedisCommands::Set { .. } | RedisCommands::Delete { .. } => {
            let conns: Vec<serde_json::Value> = client.request("db:list", None).unwrap_or_default();
            if let Some(conn) = conns
                .iter()
                .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(db_id))
            {
                if conn
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let name = conn
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(db_id);
                    anyhow::bail!("⚠️ 数据库「{}」已开启安全审核，CLI 不支持修改 Redis 数据。请在 GUI 中操作。", name);
                }
            }
        }
        _ => {}
    }
    match action {
        RedisCommands::Keys { pattern } => {
            let resp: serde_json::Value = client.request(
                "db:redis:keys",
                Some(&serde_json::json!({"dbId": db_id, "pattern": pattern})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let keys = resp
                    .get("keys")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                println!("  Redis Keys ({}):", keys.len());
                for k in &keys {
                    if let Some(s) = k.as_str() {
                        println!("    {}", s);
                    }
                }
            } else {
                anyhow::bail!(
                    "获取失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::Get { key } => {
            let resp: serde_json::Value =
                client.request("db:redis:get", Some(&serde_json::json!({"dbId": db_id, "key": key})))?;
            print_json(&resp);
        }
        RedisCommands::Set { key, value } => {
            let resp: serde_json::Value = client.request(
                "db:redis:set",
                Some(&serde_json::json!({"dbId": db_id, "key": key, "value": value})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Redis 键已设置: {} = {}", key, value));
            } else {
                anyhow::bail!(
                    "设置失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::Delete { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:delete",
                Some(&serde_json::json!({"dbId": db_id, "key": key})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Redis 键已删除: {}", key));
            } else {
                anyhow::bail!(
                    "删除失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::Type { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("TYPE {}", key)})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!(
                    "  类型: {}",
                    resp.get("result").and_then(|v| v.as_str()).unwrap_or("")
                );
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::Ttl { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("TTL {}", key)})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let ttl = resp.get("result").and_then(|v| v.as_i64()).unwrap_or(-1);
                let ttl_str = if ttl == -1 {
                    "永不过期"
                } else if ttl == -2 {
                    "键不存在"
                } else {
                    &format!("{} 秒", ttl)
                };
                println!("  剩余时间: {}", ttl_str);
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::HGet { key, field } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("HGET \"{}\" \"{}\"", key, field)})),
            )?;
            print_json(&resp);
        }
        RedisCommands::HGetAll { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("HGETALL \"{}\"", key)})),
            )?;
            print_json(&resp);
        }
        RedisCommands::HLen { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("HLEN {}", key)})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!(
                    "  Hash 字段数: {}",
                    resp.get("result").and_then(|v| v.as_u64()).unwrap_or(0)
                );
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::LRange { key, start, stop } => {
            let s = start.unwrap_or(0);
            let e = stop.unwrap_or(-1);
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("LRANGE {} {} {}", key, s, e)})),
            )?;
            print_json(&resp);
        }
        RedisCommands::LLen { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("LLEN {}", key)})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!(
                    "  List 长度: {}",
                    resp.get("result").and_then(|v| v.as_u64()).unwrap_or(0)
                );
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        RedisCommands::SMembers { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("SMEMBERS {}", key)})),
            )?;
            print_json(&resp);
        }
        RedisCommands::SCard { key } => {
            let resp: serde_json::Value = client.request(
                "db:redis:command",
                Some(&serde_json::json!({"dbId": db_id, "command": format!("SCARD {}", key)})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!(
                    "  Set 成员数: {}",
                    resp.get("result").and_then(|v| v.as_u64()).unwrap_or(0)
                );
            } else {
                anyhow::bail!(
                    "查询失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
    }
    Ok(())
}
