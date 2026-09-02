//! External database connection pool management.
//!
//! Manages connections to MySQL, PostgreSQL, Redis, and SQLite databases.
//! Shared between Tauri and GPUI apps.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex as TokioMutex;

// Native database drivers
use mysql_async::{Pool as MySqlPool, Row, prelude::Queryable};
use redis::aio::MultiplexedConnection as RedisConn;
use tokio_postgres::{Client as PgClient, NoTls};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConnectionConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: String,
    pub host: String,
    pub port: i64,
    #[serde(alias = "user")]
    pub username: String,
    pub password: Option<String>,
    #[serde(rename = "dbName", alias = "database")]
    pub db_name: Option<String>,
    #[serde(rename = "dbIndex")]
    pub db_index: Option<i64>,
    pub path: Option<String>,
}

/// Native connection pool - stores actual driver connections
///
/// `Clone` 供 GUI 连接池在锁内取出连接后解锁执行（PG 的 Client 不是 Clone，
/// 用 `Arc<PgClient>` 包装；deref coercion 保证所有 `&PgClient` 调用点无需改动）。
#[derive(Clone)]
pub enum DbConnection {
    MySql(MySqlPool),
    Postgres(std::sync::Arc<PgClient>),
    Redis(RedisConn),
    Sqlite(DbConnectionConfig),
}

/// Global connection pool
pub static CONNECTION_POOL: LazyLock<TokioMutex<HashMap<String, DbConnection>>> =
    LazyLock::new(|| TokioMutex::new(HashMap::new()));

// ── MySQL ───────────────────────────────────────────────────────────────────

pub async fn connect_mysql(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let decrypted_pw = config.password.as_deref()
        .map(|pw| crate::encryption::try_decrypt_password(pw));
    let opts = mysql_async::OptsBuilder::default()
        .ip_or_hostname(config.host.clone())
        .tcp_port(config.port as u16)
        .user(Some(config.username.clone()))
        .pass(decrypted_pw)
        .db_name(config.db_name.clone());
    let pool = MySqlPool::new(opts);
    // 5s 连接超时：主机不可达时快速失败（自 tauri 版吸收）
    let mut conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        pool.get_conn(),
    )
    .await
    .map_err(|_| format!("MySQL connect timeout (host: {})", config.host))?
    .map_err(|e| format!("MySQL failed: {}", e))?;
    conn.ping().await.map_err(|e| format!("MySQL ping failed: {}", e))?;
    Ok(DbConnection::MySql(pool))
}

pub async fn execute_mysql_query(pool: &MySqlPool, sql: &str) -> Result<serde_json::Value, String> {
    let mut conn = pool.get_conn().await.map_err(|e| e.to_string())?;
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"USE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE"|"RENAME"|"GRANT"|"REVOKE");
    if is_write {
        conn.query_drop(sql).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true, "rows": []}))
    } else {
        let rows: Vec<Row> = conn.query(sql).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true, "rows": mysql_rows_to_json(&rows)}))
    }
}

fn mysql_rows_to_json(rows: &[Row]) -> Vec<serde_json::Value> {
    if rows.is_empty() { return vec![]; }
    let columns = rows[0].columns();
    rows.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        for (i, col) in columns.as_ref().iter().enumerate() {
            let val = row.as_ref(i);
            let jv = match val {
                Some(mysql_async::Value::NULL) => serde_json::Value::Null,
                Some(mysql_async::Value::Bytes(b)) => serde_json::Value::String(String::from_utf8_lossy(b).to_string()),
                Some(mysql_async::Value::Int(n)) => serde_json::Value::Number(serde_json::Number::from(*n)),
                Some(mysql_async::Value::UInt(n)) => serde_json::Value::Number(serde_json::Number::from(*n)),
                Some(mysql_async::Value::Float(f)) => serde_json::Number::from_f64(*f as f64).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null),
                Some(mysql_async::Value::Double(f)) => serde_json::Number::from_f64(*f).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null),
                Some(mysql_async::Value::Date(y,m,d,h,min,s,_)) => serde_json::Value::String(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",y,m,d,h,min,s)),
                // TIME 类型（可带天数/负值）：避免落到 Debug 格式（自 tauri 版吸收）
                Some(mysql_async::Value::Time(neg, d, h, m, s, _)) => serde_json::Value::String(format!(
                    "{}{}d {:02}:{:02}:{:02}",
                    if *neg { "-" } else { "" },
                    d, h, m, s
                )),
                _ => serde_json::Value::String(format!("{:?}", val)),
            };
            obj.insert(col.name_str().to_string(), jv);
        }
        serde_json::Value::Object(obj)
    }).collect()
}

// ── PostgreSQL ──────────────────────────────────────────────────────────────

pub async fn connect_postgres(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let decrypted_pw = config.password.as_deref()
        .map(|pw| crate::encryption::try_decrypt_password(pw));
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.username,
        decrypted_pw.unwrap_or_default(),
        config.db_name.as_deref().unwrap_or("postgres"),
    );
    // 5s 连接超时：主机不可达时快速失败，避免调用方无限等待（自 tauri 版吸收）
    let (client, connection) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio_postgres::connect(&conn_str, NoTls),
    )
    .await
    .map_err(|_| format!("PostgreSQL connect timeout (host: {})", config.host))?
    .map_err(|e| format!("PostgreSQL failed: {}", e))?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("PostgreSQL connection error: {}", e);
        }
    });
    // ping 校验：确认连接真正可用（自 tauri 版吸收）
    client
        .batch_execute("SELECT 1")
        .await
        .map_err(|e| format!("PostgreSQL ping failed: {}", e))?;
    Ok(DbConnection::Postgres(std::sync::Arc::new(client)))
}

// ── Redis ───────────────────────────────────────────────────────────────────

pub async fn connect_redis(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let db_idx = config.db_index.unwrap_or(0);
    let url = if let Some(ref pw) = config.password {
        let decrypted = crate::encryption::try_decrypt_password(pw);
        format!(
            "redis://:{}@{}:{}/{}",
            decrypted, config.host, config.port, db_idx
        )
    } else {
        format!("redis://{}:{}/{}", config.host, config.port, db_idx)
    };
    let client = redis::Client::open(url.as_str()).map_err(|e| format!("Redis URL: {}", e))?;
    // 5s 连接超时：主机不可达时快速失败（自 tauri 版吸收）
    let conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get_multiplexed_async_connection(),
    )
    .await
    .map_err(|_| format!("Redis connect timeout (host: {})", config.host))?
    .map_err(|e| format!("Redis connect: {}", e))?;
    let _: String = redis::cmd("PING")
        .query_async(&mut conn.clone())
        .await
        .map_err(|e| format!("Redis ping: {}", e))?;
    Ok(DbConnection::Redis(conn))
}

// ── SQLite ──────────────────────────────────────────────────────────────────

pub async fn connect_sqlite(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    Ok(DbConnection::Sqlite(config.clone()))
}

// ── PostgreSQL Query ─────────────────────────────────────────────────────────

/// 展开 tokio_postgres 错误的完整信息。
///
/// `Error::Db` 的默认 Display 只输出 "db error"，把 PostgreSQL 返回的 severity /
/// message / detail / hint 全丢掉。统一展开后，GUI 与 CLI 通过 core 都能看到真实
/// 错误原因（表不存在、权限、SQL 语法等），否则排查只能靠猜。
pub fn pg_error_detail(e: &tokio_postgres::Error) -> String {
    let Some(db_err) = e.as_db_error() else {
        return e.to_string();
    };
    let mut msg = format!(
        "{} [{}]: {}",
        db_err.severity(),
        db_err.code().code(),
        db_err.message()
    );
    if let Some(d) = db_err.detail() {
        msg.push_str(&format!(" | 详情: {d}"));
    }
    if let Some(h) = db_err.hint() {
        msg.push_str(&format!(" | 建议: {h}"));
    }
    if let (Some(s), Some(t)) = (db_err.schema(), db_err.table()) {
        msg.push_str(&format!(" | 位置: {s}.{t}"));
    }
    msg
}

/// 把表名解析为 PostgreSQL 的 `"schema"."table"` 限定名。
///
/// PG 的层级是 **database → schema → table**，而连接时已通过 `dbname=` 选定 database，
/// 一条连接无法跨库查询。MySQL 的 `db.table` 语义不能直接套用：把 db_name 拼成
/// `"db_name"."table"` 时，PG 会把它当成 schema=db_name 去解析，必然报
/// `relation "xxx.yyy" does not exist`。这里让 PG 自己按 search_path 解析并回读真实
/// schema；同时兼容调用方传入已限定的 `public.users` / `"public"."users"`。
/// （自 tauri 版下沉，GUI/CLI 共用同一 PG 表名解析逻辑）
pub async fn pg_qualify_table(client: &PgClient, table: &str) -> String {
    let table = table.trim();
    if table.is_empty() {
        return String::new();
    }

    // 已带 schema 限定：拆开分别加引号（兼容调用方预置的双引号）
    if let Some((schema, name)) = split_schema_qualified(table) {
        return format!(
            "\"{}\".\"{}\"",
            pg_quote_inner(schema),
            pg_quote_inner(name)
        );
    }

    // 未限定：交给 PG 按 search_path 解析，回读真实 schema。
    // 表不存在时 to_regclass 返回 NULL（不报错），走下面的回退。
    let sql = format!(
        "SELECT n.nspname FROM pg_class c \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE c.oid = to_regclass('{}')",
        pg_escape_literal(table)
    );
    if let Ok(Some(row)) = client.query_opt(&sql, &[]).await {
        let schema: String = row.get(0);
        if !schema.is_empty() {
            return format!("\"{}\".\"{}\"", pg_quote_inner(&schema), pg_quote_inner(table));
        }
    }

    // 解析不到（表不存在 / 无权限 / 大写表名）：不带 schema，让 PG 按 search_path 处理。
    // 大写表名走这里反而是对的——`"Users"` 保留大小写，能命中 public.Users。
    format!("\"{}\"", pg_quote_inner(table))
}

/// 拆分 `schema.table`（rfind 最后一个点；两侧都不能为空）
fn split_schema_qualified(name: &str) -> Option<(&str, &str)> {
    let dot = name.rfind('.')?;
    let (schema, tbl) = name.split_at(dot);
    let tbl = &tbl[1..];
    if schema.is_empty() || tbl.is_empty() {
        return None;
    }
    Some((schema, tbl))
}

/// 去掉标识符外层的双引号，并把内部的双引号按 SQL 规则转义（`public` / `"public"` → `public`）
fn pg_quote_inner(ident: &str) -> String {
    let s = ident.trim();
    let s = s
        .strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .unwrap_or(s);
    s.replace('"', "\"\"")
}

/// SQL 字符串字面量转义（单引号加倍）
fn pg_escape_literal(s: &str) -> String {
    s.replace('\'', "''")
}

pub async fn execute_postgres_query(client: &PgClient, sql: &str) -> Result<serde_json::Value, String> {
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE");
    if is_write {
        client
            .execute(sql, &[])
            .await
            .map_err(|e| format!("PostgreSQL query failed: {}", pg_error_detail(&e)))?;
        Ok(serde_json::json!({"success": true}))
    } else {
        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| format!("PostgreSQL query failed: {}", pg_error_detail(&e)))?;
        let result: Vec<serde_json::Value> = rows.iter().map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col) in row.columns().iter().enumerate() {
                let val = pg_value_to_json(row, i);
                obj.insert(col.name().to_string(), val);
            }
            serde_json::Value::Object(obj)
        }).collect();
        Ok(serde_json::json!({"success": true, "rows": result}))
    }
}

fn pg_value_to_json(row: &tokio_postgres::Row, i: usize) -> serde_json::Value {
    use tokio_postgres::types::Type;
    // 精确类型分支：tokio_postgres 的 try_get 按列类型严格匹配，
    // 用统一 i64 读 INT2/INT4 列、统一 f64 读 FLOAT4 列都会类型不匹配返回
    // Null（曾导致 int4 主键列在结果里全部变 null）。时间类型分别处理：
    // TIMESTAMPTZ/DATE/TIME 与 NaiveDateTime 不兼容（S1 修复）。
    let val = match row.columns()[i].type_() {
        &Type::INT2 => row.try_get::<_, i16>(i).ok()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
        &Type::INT4 => row.try_get::<_, i32>(i).ok()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
        &Type::INT8 => row.try_get::<_, i64>(i).ok()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
        &Type::FLOAT4 => row.try_get::<_, f32>(i).ok()
            .and_then(|v| serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number)),
        &Type::FLOAT8 => row.try_get::<_, f64>(i).ok()
            .and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)),
        &Type::BOOL => row.try_get::<_, bool>(i).ok().map(serde_json::Value::Bool),
        &Type::TEXT | &Type::VARCHAR | &Type::BPCHAR | &Type::NAME => row
            .try_get::<_, Option<String>>(i)
            .ok()
            .flatten()
            .map(serde_json::Value::String),
        &Type::TIMESTAMP => row
            .try_get::<_, Option<chrono::NaiveDateTime>>(i)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::String(v.to_string())),
        &Type::TIMESTAMPTZ => row
            .try_get::<_, Option<chrono::DateTime<chrono::Utc>>>(i)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::String(v.to_rfc3339())),
        &Type::DATE => row
            .try_get::<_, Option<chrono::NaiveDate>>(i)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::String(v.to_string())),
        &Type::TIME => row
            .try_get::<_, Option<chrono::NaiveTime>>(i)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::String(v.to_string())),
        _ => {
            // 其余类型（含 JSON/JSONB/UUID/NUMERIC/bytea…）：先试字符串，失败再试
            // 字节（bytea → hex）。与 tauri 生产版本行为一致。
            if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                Some(v.map_or(serde_json::Value::Null, serde_json::Value::String))
            } else if let Ok(v) = row.try_get::<_, Option<&[u8]>>(i) {
                Some(v.map_or(serde_json::Value::Null, |b| {
                    serde_json::Value::String(hex::encode(b))
                }))
            } else {
                None
            }
        }
    };
    val.unwrap_or(serde_json::Value::Null)
}

// ── Redis Query ──────────────────────────────────────────────────────────────

pub async fn execute_redis_command(conn: &RedisConn, cmd: &str) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty Redis command".to_string());
    }
    // 通用 Cmd 构造：参数缺失由 redis 服务端报错而非本地索引 panic（B7 修复）
    let mut cmd_builder = redis::Cmd::new();
    cmd_builder.arg(parts[0]);
    for part in &parts[1..] {
        cmd_builder.arg(part);
    }
    let result: redis::Value = cmd_builder
        .query_async(&mut conn.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(redis_value_to_json(&result))
}

/// redis::Value → JSON（pub：GUI 的 Redis keys/stream 命令在取回原始 Value 后复用此转换）
pub fn redis_value_to_json(val: &redis::Value) -> serde_json::Value {
    match val {
        redis::Value::Nil => serde_json::Value::Null,
        redis::Value::Int(n) => serde_json::json!(n),
        redis::Value::BulkString(d) => serde_json::Value::String(String::from_utf8_lossy(d).to_string()),
        redis::Value::Array(items) => serde_json::Value::Array(items.iter().map(redis_value_to_json).collect()),
        redis::Value::Okay => serde_json::Value::String("OK".to_string()),
        redis::Value::SimpleString(s) => serde_json::Value::String(s.clone()),
        // Map（XINFO 等返回 k-v 对）： BulkString key 解码，非字符串 key 跳过（自 tauri 版吸收）
        redis::Value::Map(pairs) => {
            let obj: serde_json::Map<String, serde_json::Value> = pairs
                .iter()
                .filter_map(|(k, v)| {
                    if let redis::Value::BulkString(kb) = k {
                        Some((
                            String::from_utf8_lossy(kb).to_string(),
                            redis_value_to_json(v),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

// ── SQLite Query ─────────────────────────────────────────────────────────────

pub async fn execute_sqlite_query(config: &DbConnectionConfig, sql: &str) -> Result<serde_json::Value, String> {
    let path = config.path.as_ref().ok_or("SQLite path required")?.to_string();
    let sql = sql.to_string();
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE");
    // spawn_blocking + 读写分离打开：查询 READ_ONLY、写 READ_WRITE，
    // 避免在 async runtime 里同步阻塞（自 tauri 版吸收）
    tokio::task::spawn_blocking(move || {
        let flags = if is_write {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
        };
        let conn = rusqlite::Connection::open_with_flags(&path, flags).map_err(|e| e.to_string())?;
        if is_write {
            conn.execute_batch(&sql).map_err(|e| e.to_string())?;
            return Ok(serde_json::json!({"success": true, "rows": []}));
        }
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count).map(|i| stmt.column_name(i).unwrap_or("?").to_string()).collect();
        let rows = stmt.query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for i in 0..col_count {
                let val = match row.get::<_, rusqlite::types::Value>(i) {
                    Ok(rusqlite::types::Value::Null) => serde_json::Value::Null,
                    Ok(rusqlite::types::Value::Integer(n)) => serde_json::json!(n),
                    Ok(rusqlite::types::Value::Real(f)) => serde_json::json!(f),
                    Ok(rusqlite::types::Value::Text(s)) => serde_json::Value::String(s),
                    Ok(rusqlite::types::Value::Blob(b)) => serde_json::Value::String(hex::encode(b)),
                    _ => serde_json::Value::Null,
                };
                obj.insert(col_names[i].clone(), val);
            }
            Ok(serde_json::Value::Object(obj))
        }).map_err(|e| e.to_string())?;
        let items: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
        Ok(serde_json::json!({"success": true, "rows": items}))
    })
    .await
    .map_err(|e| format!("SQLite task failed: {}", e))?
}

// ── Query Dispatcher ─────────────────────────────────────────────────────────

pub async fn execute_query(conn: &DbConnection, sql: &str) -> Result<serde_json::Value, String> {
    match conn {
        DbConnection::MySql(p) => execute_mysql_query(p, sql).await,
        DbConnection::Postgres(c) => execute_postgres_query(c, sql).await,
        DbConnection::Redis(c) => execute_redis_command(c, sql).await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, sql).await,
    }
}

pub async fn add_connection(id: String, conn: DbConnection) {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.insert(id, conn);
}

pub async fn remove_connection(id: &str) {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.remove(id);
}

pub async fn get_connection(id: &str) -> Option<String> {
    let pool = CONNECTION_POOL.lock().await;
    pool.get(id).map(|_| id.to_string())
}

pub async fn list_connections() -> Vec<(String, String)> {
    let pool = CONNECTION_POOL.lock().await;
    pool.keys().map(|k| (k.clone(), String::new())).collect()
}

#[cfg(test)]
mod pg_qualify_tests {
    use super::*;

    #[test]
    fn split_schema_qualified_basic() {
        assert_eq!(split_schema_qualified("public.users"), Some(("public", "users")));
        assert_eq!(split_schema_qualified("app.v2.events"), Some(("app.v2", "events")));
        // 已带引号的整段视为 schema 部分（调用方预置限定）
        assert_eq!(split_schema_qualified("\"public\".users"), Some(("\"public\"", "users")));
        // 无点 / 空段 → None
        assert_eq!(split_schema_qualified("users"), None);
        assert_eq!(split_schema_qualified(".users"), None);
        assert_eq!(split_schema_qualified("users."), None);
    }

    #[test]
    fn pg_quote_inner_strips_and_escapes() {
        assert_eq!(pg_quote_inner("public"), "public");
        assert_eq!(pg_quote_inner("\"public\""), "public");
        // 内部双引号按 SQL 规则加倍
        assert_eq!(pg_quote_inner("he\"llo"), "he\"\"llo");
        assert_eq!(pg_quote_inner("  Users  "), "Users");
    }

    #[test]
    fn pg_escape_literal_doubles_quotes() {
        assert_eq!(pg_escape_literal("it's"), "it''s");
        assert_eq!(pg_escape_literal("plain"), "plain");
    }
}
