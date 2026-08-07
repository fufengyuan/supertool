
//! High-level database operations — table browsing, data querying.
//! Extracted from tauri/src/commands/database.rs for sharing.

use crate::db_pool::{execute_mysql_query, execute_postgres_query, execute_sqlite_query, DbConnection};

pub async fn redis_keys(conn: &DbConnection, db_index: i64, pattern: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            // S9: KEYS → SCAN 迭代，避免大 key 空间阻塞 Redis（复用现成 redis_scan_keys）
            redis_scan_keys(conn, db_index, pattern, 1000).await
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_key_value(conn: &DbConnection, db_index: i64, key: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let key_type: String = redis::cmd("TYPE").arg(key).query_async(&mut c.clone()).await.map_err(|e| format!("Redis TYPE failed: {}", e))?;
            let val = match key_type.as_str() {
                "string" => { let v: String = redis::cmd("GET").arg(key).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?; serde_json::Value::String(v) }
                "hash" => { let v: std::collections::HashMap<String,String> = redis::cmd("HGETALL").arg(key).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?; serde_json::Value::Object(v.into_iter().map(|(k,v)|(k,serde_json::Value::String(v))).collect()) }
                "list" => { let v: Vec<String> = redis::cmd("LRANGE").arg(key).arg(0).arg(-1).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?; serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()) }
                "set" => { let v: Vec<String> = redis::cmd("SMEMBERS").arg(key).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?; serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()) }
                "zset" => { let v: Vec<String> = redis::cmd("ZRANGE").arg(key).arg(0).arg(-1).arg("WITHSCORES").query_async(&mut c.clone()).await.map_err(|e| e.to_string())?; serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()) }
                _ => serde_json::Value::Null
            };
            Ok(serde_json::json!({ "success": true, "value": val, "type": key_type }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_set_key(conn: &DbConnection, db_index: i64, key: &str, value: &str, ttl: i64) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            if ttl > 0 { redis::cmd("SETEX").arg(key).arg(ttl).arg(value).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SETEX failed: {}", e))?; }
            else { redis::cmd("SET").arg(key).arg(value).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SET failed: {}", e))?; }
            Ok(serde_json::json!({ "success": true }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_delete_key(conn: &DbConnection, db_index: i64, key: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            redis::cmd("DEL").arg(key).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis DEL failed: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_keys_by_type(conn: &DbConnection, db_index: i64, type_filter: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let mut all_keys: Vec<String> = Vec::new();
            let mut cursor: String = "0".to_string();
            loop {
                let (new_cursor, batch): (String, Vec<String>) = redis::cmd("SCAN").arg(&cursor).arg("COUNT").arg(1000).query_async(&mut c.clone()).await.map_err(|e| format!("Redis SCAN failed: {}", e))?;
                all_keys.extend(batch); cursor = new_cursor;
                if cursor == "0" { break; }
            }
            let mut keys_by_type: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for key in all_keys {
                let ktype: String = redis::cmd("TYPE").arg(&key).query_async(&mut c.clone()).await.map_err(|e| format!("Redis TYPE failed: {}", e))?;
                if type_filter == "*" || type_filter == ktype { keys_by_type.entry(ktype).or_default().push(key); }
            }
            Ok(serde_json::json!({ "success": true, "keysByType": keys_by_type }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_streams(conn: &DbConnection, db_index: i64, pattern: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let keys: Vec<String> = redis::cmd("KEYS").arg(pattern).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?;
            let mut streams = Vec::new();
            for key in keys {
                let len: i64 = redis::cmd("XLEN").arg(&key).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?;
                streams.push(serde_json::json!({"name": key, "length": len}));
            }
            Ok(serde_json::json!({ "success": true, "streams": streams }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_stream_messages(conn: &DbConnection, db_index: i64, stream: &str, start: &str, end: &str, count: i64) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let msgs: Vec<Vec<String>> = redis::cmd("XRANGE").arg(stream).arg(start).arg(end).arg("COUNT").arg(count).query_async(&mut c.clone()).await.map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "success": true, "messages": msgs }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_stream_add(conn: &DbConnection, db_index: i64, stream: &str, fields: &serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let mut cmd = redis::Cmd::new(); cmd.arg("XADD").arg(stream).arg("*");
            for (k, v) in fields { cmd.arg(k).arg(v.as_str().unwrap_or("")); }
            let id: String = cmd.query_async(&mut c.clone()).await.map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "success": true, "id": id }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

pub async fn redis_stream_delete(conn: &DbConnection, db_index: i64, stream: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            redis::cmd("DEL").arg(stream).query_async::<()>(&mut c.clone()).await.map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "success": true }))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

/// Insert a row into a table
pub async fn insert_table_row(conn: &DbConnection, table: &str, values: &serde_json::Map<String, serde_json::Value>, db_name: Option<&str>) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let (cols, vals): (Vec<String>, Vec<String>) = values.iter().map(|(k, v)| {
                let val = match v {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    _ => v.to_string(),
                };
                (format!("`{}`", k), val)
            }).unzip();
            let prefix = db_name.filter(|s| !s.is_empty()).map(|d| format!("`{}`.", d)).unwrap_or_default();
            let sql = format!("INSERT INTO {}{} ({}) VALUES ({})", prefix, table, cols.join(", "), vals.join(", "));
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let (cols, vals): (Vec<String>, Vec<String>) = values.iter().map(|(k, v)| {
                let val = match v {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    _ => v.to_string(),
                };
                (format!("\"{}\"", k), val)
            }).unzip();
            let sql = format!("INSERT INTO \"{}\" ({}) VALUES ({})", table, cols.join(", "), vals.join(", "));
            execute_sqlite_query(cfg, &sql).await
        }
        _ => Err("Only MySQL and SQLite supported for insert".to_string()),
    }
}

pub async fn test_connection(config: &crate::db_pool::DbConnectionConfig) -> Result<serde_json::Value, String> {
    match config.db_type.as_str() {
        "mysql" => { crate::db_pool::connect_mysql(config).await?; Ok(serde_json::json!({"success": true})) }
        "postgres" => { crate::db_pool::connect_postgres(config).await?; Ok(serde_json::json!({"success": true})) }
        "redis" => { crate::db_pool::connect_redis(config).await?; Ok(serde_json::json!({"success": true})) }
        "sqlite" => { Ok(serde_json::json!({"success": true})) }
        other => Err(format!("Unsupported: {}", other)),
    }
}

pub async fn update_table_row(conn: &DbConnection, table: &str, pk: &serde_json::Map<String, serde_json::Value>, values: &serde_json::Map<String, serde_json::Value>, db_name: Option<&str>) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let sets: Vec<String> = values.iter().map(|(k,v)| format!("`{}` = {}", k, sql_val(v))).collect();
            let wheres: Vec<String> = pk.iter().map(|(k,v)| format!("`{}` = {}", k, sql_val(v))).collect();
            let prefix = db_name.filter(|s|!s.is_empty()).map(|d|format!("`{}`.",d)).unwrap_or_default();
            execute_mysql_query(p, &format!("UPDATE {}{} SET {} WHERE {}", prefix, table, sets.join(", "), wheres.join(" AND "))).await
        }
        DbConnection::Sqlite(cfg) => {
            let sets: Vec<String> = values.iter().map(|(k,v)| format!("\"{}\" = {}", k, sql_val(v))).collect();
            let wheres: Vec<String> = pk.iter().map(|(k,v)| format!("\"{}\" = {}", k, sql_val(v))).collect();
            execute_sqlite_query(cfg, &format!("UPDATE \"{}\" SET {} WHERE {}", table, sets.join(", "), wheres.join(" AND "))).await
        }
        _ => Err("Only MySQL/SQLite supported for update".to_string()),
    }
}

pub async fn delete_table_row(conn: &DbConnection, table: &str, pk: &serde_json::Map<String, serde_json::Value>, db_name: Option<&str>) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let wheres: Vec<String> = pk.iter().map(|(k,v)| format!("`{}` = {}", k, sql_val(v))).collect();
            let prefix = db_name.filter(|s|!s.is_empty()).map(|d|format!("`{}`.",d)).unwrap_or_default();
            execute_mysql_query(p, &format!("DELETE FROM {}{} WHERE {}", prefix, table, wheres.join(" AND "))).await
        }
        DbConnection::Sqlite(cfg) => {
            let wheres: Vec<String> = pk.iter().map(|(k,v)| format!("\"{}\" = {}", k, sql_val(v))).collect();
            execute_sqlite_query(cfg, &format!("DELETE FROM \"{}\" WHERE {}", table, wheres.join(" AND "))).await
        }
        _ => Err("Only MySQL/SQLite supported for delete".to_string()),
    }
}

fn sql_val(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        _ => v.to_string(),
    }
}

pub async fn get_table_primary_keys(conn: &DbConnection, table: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &format!("SHOW KEYS FROM {} WHERE Key_name = 'PRIMARY'", table)).await,
        DbConnection::Postgres(c) => execute_postgres_query(c, &format!("SELECT a.attname FROM pg_index i JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) WHERE i.indrelid = '\"{}\"'::regclass AND i.indisprimary", table)).await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, &format!("PRAGMA table_info({})", table)).await,
        _ => Err("Not supported".to_string()),
    }
}

pub async fn get_views(conn: &DbConnection, db_name: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &format!("SHOW FULL TABLES FROM `{}` WHERE TABLE_TYPE LIKE 'VIEW'", db_name)).await,
        DbConnection::Postgres(c) => execute_postgres_query(c, "SELECT viewname FROM pg_views WHERE schemaname='public' ORDER BY viewname").await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, "SELECT name FROM sqlite_master WHERE type='view' AND name NOT LIKE 'sqlite_%' ORDER BY name").await,
        _ => Err("Not supported".to_string()),
    }
}

pub async fn get_table_structure(conn: &DbConnection, db_name: Option<&str>, table: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let db = db_name.unwrap_or("");
            let col_sql = format!(
                "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY, EXTRA, COLUMN_COMMENT, ORDINAL_POSITION \
                 FROM information_schema.COLUMNS WHERE TABLE_SCHEMA='{}' AND TABLE_NAME='{}' ORDER BY ORDINAL_POSITION", db, table);
            let col_result = execute_mysql_query(p, &col_sql).await?;
            let idx_sql = format!("SHOW INDEX FROM `{}`.`{}`", db, table);
            let idx_result = execute_mysql_query(p, &idx_sql).await?;
            Ok(serde_json::json!({
                "success": true, "rows": col_result.get("rows").cloned().unwrap_or_default(),
                "indexes": idx_result.get("rows").cloned().unwrap_or_default()
            }))
        }
        DbConnection::Postgres(c) => {
            let col_sql = format!(
                "SELECT c.column_name AS \"COLUMN_NAME\", \
                 CASE WHEN c.data_type = 'character varying' THEN 'varchar('||COALESCE(c.character_maximum_length::text,'255')||')' \
                      WHEN c.data_type = 'numeric' THEN 'numeric('||COALESCE(c.numeric_precision::text,'10')||','||COALESCE(c.numeric_scale::text,'0')||')' \
                      ELSE COALESCE(c.domain_name, c.data_type) END AS \"COLUMN_TYPE\", \
                 c.is_nullable AS \"IS_NULLABLE\", c.column_default AS \"COLUMN_DEFAULT\", \
                 c.ordinal_position AS \"ORDINAL_POSITION\", \
                 COALESCE(pgd.description,'') AS \"COLUMN_COMMENT\", \
                 CASE WHEN pk.column_name IS NOT NULL THEN 'PRI' ELSE '' END AS \"COLUMN_KEY\", '' AS \"EXTRA\" \
                 FROM information_schema.columns c \
                 LEFT JOIN pg_catalog.pg_statio_all_tables st ON st.schemaname=c.table_schema AND st.relname=c.table_name \
                 LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid=st.relid AND pgd.objsubid=c.ordinal_position \
                 LEFT JOIN (SELECT ku.column_name,ku.table_schema,ku.table_name FROM information_schema.table_constraints tc \
                   JOIN information_schema.key_column_usage ku ON tc.constraint_name=ku.constraint_name \
                   WHERE tc.constraint_type='PRIMARY KEY') pk ON pk.column_name=c.column_name \
                 WHERE c.table_schema='public' AND c.table_name='{}' ORDER BY c.ordinal_position", table);
            execute_postgres_query(c, &col_sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let sql = format!("PRAGMA table_info([{}])", table);
            execute_sqlite_query(cfg, &sql).await
        }
        _ => Err("Table structure not supported for this DB type".to_string()),
    }
}

pub async fn get_table_data(conn: &DbConnection, db_name: Option<&str>, table: &str, limit: i64, offset: i64) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let sql = if let Some(db) = db_name {
                format!("SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}", db, table, limit, offset)
            } else {
                format!("SELECT * FROM `{}` LIMIT {} OFFSET {}", table, limit, offset)
            };
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Postgres(c) => {
            let sql = format!("SELECT * FROM \"{}\" LIMIT {} OFFSET {}", table, limit, offset);
            execute_postgres_query(c, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let sql = format!("SELECT * FROM \"{}\" LIMIT {} OFFSET {}", table, limit, offset);
            execute_sqlite_query(cfg, &sql).await
        }
        _ => Err("Table data not supported for this DB type".to_string()),
    }
}

pub async fn get_databases(conn: &DbConnection) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, "SHOW DATABASES").await,
        DbConnection::Postgres(c) => execute_postgres_query(c, "SELECT datname FROM pg_database WHERE datistemplate=false").await,
        _ => Err("List databases not supported".to_string()),
    }
}

pub async fn get_tables(conn: &DbConnection, db_name: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &format!("SHOW TABLES FROM `{}`", db_name)).await,
        DbConnection::Postgres(c) => execute_postgres_query(c, "SELECT tablename FROM pg_tables WHERE schemaname='public' ORDER BY tablename").await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").await,
        _ => Err("List tables not supported".to_string()),
    }
}

pub async fn get_create_sql(conn: &DbConnection, table: &str, db_name: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &format!("SHOW CREATE TABLE `{}`.`{}`", db_name, table)).await,
        DbConnection::Postgres(c) => execute_postgres_query(c, &format!("SELECT '-- PostgreSQL: use pg_dump for table {}' AS sql", table)).await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, &format!("SELECT sql FROM sqlite_master WHERE name='{}'", table)).await,
        _ => Err("Not supported".to_string()),
    }
}

pub async fn backup_create(conn: &DbConnection) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => {
            let tables = execute_mysql_query(p, "SHOW TABLES").await?;
            let mut backup = serde_json::Map::new();
            if let Some(rows) = tables.get("rows").and_then(|r| r.as_array()) {
                for row in rows {
                    if let Some(name) = row.as_object().and_then(|o| o.values().next()).and_then(|v| v.as_str()) {
                        let data = execute_mysql_query(p, &format!("SELECT * FROM `{}`", name)).await?;
                        backup.insert(name.to_string(), data);
                    }
                }
            }
            Ok(serde_json::json!({ "success": true, "data": backup }))
        }
        _ => Err("Backup only supported for MySQL".to_string()),
    }
}

pub async fn backup_list(backup_dir: &str) -> Result<serde_json::Value, String> {
    let dir = std::path::Path::new(backup_dir);
    if !dir.exists() { return Ok(serde_json::json!({ "success": true, "backups": [] })); }
    let mut backups = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    backups.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy(),
                        "size": meta.len(),
                        "modified": meta.modified().ok().map(|t| format!("{:?}", t)),
                    }));
                }
            }
        }
    }
    Ok(serde_json::json!({ "success": true, "backups": backups }))
}

pub async fn backup_delete(file_path: &str) -> Result<serde_json::Value, String> {
    std::fs::remove_file(file_path).map_err(|e| format!("Delete failed: {}", e))?;
    Ok(serde_json::json!({ "success": true }))
}

pub async fn execute_query_direct(conn: &DbConnection, sql: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => crate::db_pool::execute_mysql_query(p, sql).await,
        DbConnection::Postgres(c) => crate::db_pool::execute_postgres_query(c, sql).await,
        DbConnection::Redis(c) => crate::db_pool::execute_redis_command(c, sql).await,
        DbConnection::Sqlite(cfg) => crate::db_pool::execute_sqlite_query(cfg, sql).await,
    }
}

pub async fn backup_restore(conn: &DbConnection, file_path: &str) -> Result<serde_json::Value, String> {
    let compressed = std::fs::read(file_path).map_err(|e| format!("Read failed: {}", e))?;
    if compressed.is_empty() { return Err("Empty backup file".to_string()); }
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str).map_err(|e| format!("Decompress failed: {}", e))?;
    let payload: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| format!("Parse failed: {}", e))?;
    let files = payload.get("files").and_then(|v| v.as_object()).ok_or_else(|| "Missing 'files' field".to_string())?;
    let mut executed = 0i64;
    let mut errors: Vec<String> = Vec::new();
    for (_filename, sql_content) in files.iter() {
        if let Some(sql) = sql_content.as_str() {
            if sql.trim().is_empty() { continue; }
            match execute_query_direct(conn, sql).await {
                Ok(_) => executed += 1,
                Err(e) => errors.push(format!("{}: {}", _filename, e)),
            }
        }
    }
    Ok(serde_json::json!({"success": errors.is_empty(), "executed": executed, "errors": errors}))
}

/// Redis: get key metadata (type + ttl)
pub async fn redis_key_info(conn: &DbConnection, db_index: i64, key: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let key_type: String = redis::cmd("TYPE").arg(key).query_async(&mut c.clone()).await.map_err(|e| format!("Redis TYPE failed: {}", e))?;
            let ttl: i64 = redis::cmd("TTL").arg(key).query_async(&mut c.clone()).await.map_err(|e| format!("Redis TTL failed: {}", e))?;
            Ok(serde_json::json!({"success": true, "type": key_type, "ttl": ttl}))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

/// Redis: scan keys with cursor-based iteration
pub async fn redis_scan_keys(conn: &DbConnection, db_index: i64, pattern: &str, count: i64) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            let mut keys = Vec::new();
            let mut cursor: String = "0".to_string();
            loop {
                let (new_cursor, batch): (String, Vec<String>) = redis::cmd("SCAN").arg(&cursor).arg("MATCH").arg(pattern).arg("COUNT").arg(count)
                    .query_async(&mut c.clone()).await.map_err(|e| format!("Redis SCAN failed: {}", e))?;
                keys.extend(batch);
                cursor = new_cursor;
                if cursor == "0" { break; }
            }
            Ok(serde_json::json!({"success": true, "keys": keys}))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}

/// Redis: add a key with specified type (string/hash/list/set)
pub async fn redis_add_key(conn: &DbConnection, db_index: i64, key: &str, key_type: &str, value: &serde_json::Value) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::Redis(c) => {
            redis::cmd("SELECT").arg(db_index).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SELECT failed: {}", e))?;
            match key_type {
                "string" => { redis::cmd("SET").arg(key).arg(value.as_str().unwrap_or("")).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SET failed: {}", e))?; }
                "hash" => {
                    if let Some(obj) = value.as_object() {
                        let mut cmd = redis::Cmd::new(); cmd.arg("HMSET").arg(key);
                        for (k, v) in obj { cmd.arg(k).arg(v.as_str().unwrap_or("")); }
                        cmd.query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis HMSET failed: {}", e))?;
                    }
                }
                "list" => { redis::cmd("RPUSH").arg(key).arg(value.as_str().unwrap_or("")).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis RPUSH failed: {}", e))?; }
                "set" => { redis::cmd("SADD").arg(key).arg(value.as_str().unwrap_or("")).query_async::<()>(&mut c.clone()).await.map_err(|e| format!("Redis SADD failed: {}", e))?; }
                _ => return Err(format!("Unsupported type: {}", key_type)),
            }
            Ok(serde_json::json!({"success": true}))
        }
        _ => Err("Not a Redis connection".to_string()),
    }
}
