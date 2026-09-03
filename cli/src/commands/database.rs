use crate::output;
use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;
use supertool_core::db_pool::DbConnectionConfig;

pub async fn cmd_db(rt: &mut CliRuntime, action: &DbCommands) -> Result<()> {
    check_db(rt).await?;
    match action {
        DbCommands::List { json } => {
            let conns = get_db_connections(rt).await?;
            if *json || rt.json_mode {
                print_json(&conns);
            } else {
                println!("\n  数据库连接 ({}):", conns.len());
                // 按类型分组
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for c in &conns {
                    let r#type = c
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_uppercase();
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
        DbCommands::Disconnect { id: _ } => {
            // CLI is stateless — connections are created and dropped per command.
            // Nothing to disconnect.
            print_success("CLI 无状态连接，无需断开。");
        }
        DbCommands::Query { db_id, sql, json } => {
            // 生产环境护栏：审批连接上只读 SQL 放行，写 SQL 拦截
            if !supertool_core::logic::sql_safety::is_read_only_sql(sql) {
                check_db_connection_write(rt, db_id).await?;
            }
            let config = get_db_config(rt, db_id).await?;
            let resp = rt
                .core
                .execute_db_query(config, sql)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
                print_json(&resp);
            } else {
                print_query_result(&resp, sql);
            }
        }
        DbCommands::Tables { db_id, db, json } => {
            let config = get_db_config(rt, db_id).await?;
            let db_name = db.as_deref().or(config.db_name.as_deref()).unwrap_or("");
            let resp = rt
                .core
                .db_get_tables(&config, db_name)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
                print_json(&resp);
            } else {
                print_tables(&resp, db_name);
            }
        }
        DbCommands::Databases { db_id, json } => {
            let config = get_db_config(rt, db_id).await?;
            let resp = rt
                .core
                .db_get_databases(&config)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
                print_json(&resp);
            } else {
                print_databases(&resp);
            }
        }
        DbCommands::Structure {
            db_id,
            db,
            table,
            json,
        } => {
            let config = get_db_config(rt, db_id).await?;
            let db_ref = db.as_deref().or(config.db_name.as_deref());
            let resp = rt
                .core
                .db_get_table_structure(&config, db_ref, table)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
                print_json(&resp);
            } else {
                print_table_structure(&resp, table);
            }
        }
        DbCommands::Data {
            db_id,
            db,
            table,
            limit,
            offset,
            json,
        } => {
            let config = get_db_config(rt, db_id).await?;
            let db_ref = db.as_deref().or(config.db_name.as_deref());
            let resp = rt
                .core
                .db_get_table_data(&config, db_ref, table, *limit, *offset)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
                print_json(&resp);
            } else {
                print_table_data(&resp, table);
            }
        }
        DbCommands::Redis { db_id, action, json } => {
            cmd_redis(rt, db_id, *json, action).await?
        }
    }
    Ok(())
}

pub async fn cmd_redis(
    rt: &mut CliRuntime,
    db_id: &str,
    json: bool,
    action: &RedisCommands,
) -> Result<()> {
    // 命令级 -j 或全局 --json 任一开启即 JSON 输出
    rt.set_json(json);
    let config = get_db_config(rt, db_id).await?;
    let db_index = config.db_index.unwrap_or(0);

    match action {
        RedisCommands::Keys { pattern } => {
            let pat = pattern.as_deref().unwrap_or("*");
            let resp = rt
                .core
                .db_redis_list_keys(&config, db_index, pat)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let keys = resp
                .get("keys")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if rt.json_mode {
                print_json(&resp);
            } else if keys.is_empty() {
                println!("  (无匹配 key)");
            } else {
                println!("  Redis keys ({}):", keys.len());
                for k in &keys {
                    println!("    {}", k.as_str().unwrap_or(""));
                }
            }
        }
        RedisCommands::Get { key } => {
            let resp = rt
                .core
                .db_redis_get_value(&config, db_index, key)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                let key_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                let value = resp.get("value").cloned().unwrap_or(serde_json::Value::Null);
                println!("  {} ({}) = {}", key, key_type, format_json_value(&value));
            }
        }
        RedisCommands::Set { key, value } => {
            // 生产环境护栏：写命令在审批连接上拦截
            check_db_connection_write(rt, db_id).await?;
            let resp = rt
                .core
                .db_redis_set_key(&config, db_index, key, value, 0)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                print_success(&format!("SET {} = OK", key));
            }
        }
        RedisCommands::Delete { key } => {
            // 生产环境护栏：写命令在审批连接上拦截
            check_db_connection_write(rt, db_id).await?;
            let resp = rt
                .core
                .db_redis_delete_key(&config, db_index, key)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                print_success(&format!("DEL {} = OK", key));
            }
        }
        RedisCommands::Type { key } => {
            let resp = rt
                .core
                .db_redis_key_info(&config, db_index, key)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                let key_type = resp.get("type").and_then(|v| v.as_str()).unwrap_or("none");
                println!("  {} → {}", key, key_type);
            }
        }
        RedisCommands::Ttl { key } => {
            let resp = rt
                .core
                .db_redis_key_info(&config, db_index, key)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                let ttl = resp.get("ttl").and_then(|v| v.as_i64()).unwrap_or(-2);
                let desc = match ttl {
                    -1 => "永不过期".to_string(),
                    -2 => "key 不存在".to_string(),
                    n => format!("{} 秒", n),
                };
                println!("  {} TTL: {}", key, desc);
            }
        }
        RedisCommands::HGet { key, field } => {
            let cmd_str = format!("HGET {} {}", key, field);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {}:{} = {}", key, field, format_json_value(&resp));
            }
        }
        RedisCommands::HGetAll { key } => {
            let cmd_str = format!("HGETALL {}", key);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} (hash):", key);
                println!("    {}", format_json_value(&resp));
            }
        }
        RedisCommands::HLen { key } => {
            let cmd_str = format!("HLEN {}", key);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} HLEN = {}", key, format_json_value(&resp));
            }
        }
        RedisCommands::LRange { key, start, stop } => {
            let s = start.unwrap_or(0);
            let e = stop.unwrap_or(-1);
            let cmd_str = format!("LRANGE {} {} {}", key, s, e);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} [{}..{}] (list):", key, s, e);
                println!("    {}", format_json_value(&resp));
            }
        }
        RedisCommands::LLen { key } => {
            let cmd_str = format!("LLEN {}", key);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} LLEN = {}", key, format_json_value(&resp));
            }
        }
        RedisCommands::SMembers { key } => {
            let cmd_str = format!("SMEMBERS {}", key);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} (set):", key);
                println!("    {}", format_json_value(&resp));
            }
        }
        RedisCommands::SCard { key } => {
            let cmd_str = format!("SCARD {}", key);
            let resp = rt
                .core
                .db_redis_exec(&config, db_index, &cmd_str)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&resp);
            } else {
                println!("  {} SCARD = {}", key, format_json_value(&resp));
            }
        }
    }
    Ok(())
}

// ── Helpers ──

async fn check_db(rt: &mut CliRuntime) -> Result<()> {
    let _ = rt
        .core
        .db_read(|conn| conn.prepare("SELECT 1").map(|_| ()))
        .map_err(|e| anyhow::anyhow!("数据库连接失败: {}", e))?;
    Ok(())
}

/// Get DB connections list (passwords stripped) — for `db list` display.
async fn get_db_connections(rt: &mut CliRuntime) -> Result<Vec<serde_json::Value>> {
    let setting = rt
        .core
        .get_setting("db_connections")
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let connections: Vec<serde_json::Value> = match setting {
        serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
        serde_json::Value::Array(arr) => arr,
        _ => vec![],
    };
    // 移除密码字段，规范化 requiresApproval
    Ok(connections
        .into_iter()
        .map(|mut c| {
            if let Some(obj) = c.as_object_mut() {
                obj.remove("password");
                let ra = obj
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                obj.insert("requiresApproval".to_string(), serde_json::Value::Bool(ra));
            }
            c
        })
        .collect())
}

/// Get a single DbConnectionConfig (WITH password) for actual DB operations.
async fn get_db_config(rt: &mut CliRuntime, db_id: &str) -> Result<DbConnectionConfig> {
    let setting = rt
        .core
        .get_setting("db_connections")
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let connections: Vec<serde_json::Value> = match setting {
        serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
        serde_json::Value::Array(arr) => arr,
        _ => vec![],
    };
    let conn = connections
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(db_id))
        .ok_or_else(|| anyhow::anyhow!("未找到数据库连接: {}（可用 stool db list 核对 id）", db_id))?;
    serde_json::from_value(conn.clone())
        .map_err(|e| anyhow::anyhow!("解析连接配置失败: {}", e))
}

/// 生产环境护栏：数据库连接开启审批（requiresApproval=true）时禁止写操作（只读查询放行）
async fn check_db_connection_write(rt: &mut CliRuntime, db_id: &str) -> Result<()> {
    let setting = rt
        .core
        .get_setting("db_connections")
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let connections: Vec<serde_json::Value> = match setting {
        serde_json::Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
        serde_json::Value::Array(arr) => arr,
        _ => vec![],
    };
    let conn = connections
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(db_id))
        .ok_or_else(|| anyhow::anyhow!("未找到数据库连接: {}（可用 stool db list 核对 id）", db_id))?;
    if conn
        .get("requiresApproval")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let name = conn.get("name").and_then(|v| v.as_str()).unwrap_or(db_id);
        return Err(output::fail(
            output::EXIT_UNAUTHORIZED,
            format!(
                "数据库连接「{}」已开启审批（生产环境），CLI 禁止写操作。只读查询不受限，请在 GUI 中操作写操作。",
                name
            ),
        ));
    }
    Ok(())
}

// ── Output formatters ──

fn format_json_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "(nil)".to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "(empty)".to_string()
            } else {
                format!(
                    "[{}]",
                    arr.iter()
                        .map(|i| match i {
                            serde_json::Value::String(s) => s.clone(),
                            _ => i.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                "(empty)".to_string()
            } else {
                format!(
                    "{{{}}}",
                    obj.iter()
                        .map(|(k, v)| match v {
                            serde_json::Value::String(s) => format!("{}: {}", k, s),
                            _ => format!("{}: {}", k, v),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        other => other.to_string(),
    }
}

fn print_query_result(resp: &serde_json::Value, sql: &str) {
    if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
        let err = resp
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");
        println!("  ❌ 查询失败: {}", err);
        return;
    }
    if let Some(rows) = resp.get("rows").and_then(|v| v.as_array()) {
        if rows.is_empty() {
            println!("  查询无结果 (0 行)");
            return;
        }
        // Extract column names from first row
        let columns: Vec<String> = if let Some(first) = rows.first() {
            if let Some(obj) = first.as_object() {
                obj.keys().cloned().collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        // Print table header
        let col_widths: Vec<usize> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let max_cell = rows
                    .iter()
                    .map(|r| {
                        r.get(&columns[i])
                            .map(|v| cell_str(v).len())
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);
                col.len().max(max_cell).min(40)
            })
            .collect();

        let separator: String = col_widths
            .iter()
            .map(|w| "-".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("+");
        let header: String = columns
            .iter()
            .enumerate()
            .map(|(i, col)| format!(" {:<width$} ", col, width = col_widths[i]))
            .collect::<Vec<_>>()
            .join("|");
        println!("  SQL: {}", sql);
        println!("  +{}+", separator);
        println!("  |{}|", header);
        println!("  +{}+", separator);
        for row in rows.iter().take(200) {
            let line: String = columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let val = row.get(col).map(cell_str).unwrap_or_default();
                    format!(" {:<width$} ", val, width = col_widths[i])
                })
                .collect::<Vec<_>>()
                .join("|");
            println!("  |{}|", line);
        }
        if rows.len() > 200 {
            println!("  ... (省略 {} 行)", rows.len() - 200);
        }
        println!("  +{}+", separator);
        println!("  ({} 行)", rows.len());
    } else {
        // Write operation
        print_success("执行成功");
    }
}

fn cell_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

fn print_databases(resp: &serde_json::Value) {
    if let Some(dbs) = resp.get("databases").and_then(|v| v.as_array()) {
        println!("  数据库列表 ({}):", dbs.len());
        for db in dbs {
            let name = db.as_str().unwrap_or("");
            println!("    {}", name);
        }
    } else if let Some(arr) = resp.as_array() {
        println!("  数据库列表 ({}):", arr.len());
        for db in arr {
            let name = db
                .as_str()
                .or_else(|| db.get("name").and_then(|v| v.as_str()))
                .unwrap_or("");
            println!("    {}", name);
        }
    } else {
        println!("  {}", format_json_value(resp));
    }
}

fn print_tables(resp: &serde_json::Value, db_name: &str) {
    if let Some(tables) = resp.get("tables").and_then(|v| v.as_array()) {
        println!("  表列表 [{}] ({}):", db_name, tables.len());
        for t in tables {
            let name = t
                .as_str()
                .or_else(|| t.get("name").and_then(|v| v.as_str()))
                .unwrap_or("");
            println!("    {}", name);
        }
    } else if let Some(arr) = resp.as_array() {
        println!("  表列表 [{}] ({}):", db_name, arr.len());
        for t in arr {
            let name = t
                .as_str()
                .or_else(|| t.get("name").and_then(|v| v.as_str()))
                .unwrap_or("");
            println!("    {}", name);
        }
    } else {
        println!("  {}", format_json_value(resp));
    }
}

fn print_table_structure(resp: &serde_json::Value, table: &str) {
    println!("  表结构 [{}]:", table);
    println!("  {}", "─".repeat(60));
    if let Some(columns) = resp.get("columns").and_then(|v| v.as_array()) {
        for col in columns {
            let name = col.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let col_type = col
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let nullable = col
                .get("nullable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let key = col.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let default = col.get("default").map(|v| v.to_string()).unwrap_or_default();
            println!(
                "    {} {} {} {} {}",
                name,
                col_type,
                if nullable { "NULL" } else { "NOT NULL" },
                key,
                if default.is_empty() {
                    "".to_string()
                } else {
                    format!("DEFAULT {}", default)
                }
            );
        }
    } else if let Some(arr) = resp.as_array() {
        for col in arr {
            println!("    {}", format_json_value(col));
        }
    } else {
        println!("    {}", format_json_value(resp));
    }
}

fn print_table_data(resp: &serde_json::Value, table: &str) {
    println!("  表数据 [{}]:", table);
    println!("  {}", "─".repeat(60));
    if let Some(rows) = resp.get("rows").and_then(|v| v.as_array()) {
        if rows.is_empty() {
            println!("    (无数据)");
            return;
        }
        for (i, row) in rows.iter().enumerate() {
            println!("    [{}] {}", i, format_json_value(row));
        }
        println!("  ({} 行)", rows.len());
    } else if let Some(arr) = resp.as_array() {
        for (i, row) in arr.iter().enumerate() {
            println!("    [{}] {}", i, format_json_value(row));
        }
        println!("  ({} 行)", arr.len());
    } else {
        println!("    {}", format_json_value(resp));
    }
}
