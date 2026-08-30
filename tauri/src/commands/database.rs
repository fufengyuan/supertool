use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tauri::Emitter;
use tokio::sync::Mutex as TokioMutex;

// Native database drivers
use mysql_async::{Pool as MySqlPool, Row, prelude::Queryable};
use redis::aio::MultiplexedConnection as RedisConn;
use tokio_postgres::{Client as PgClient, NoTls};

use crate::commands::es::EsClient;

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

/// Native connection pool - stores actual driver connections, not configs
#[derive(Clone)]
pub enum DbConnection {
    MySql(MySqlPool),
    Postgres(std::sync::Arc<tokio_postgres::Client>),
    Redis(RedisConn),
    Sqlite(DbConnectionConfig), // SQLite is file-based, config is enough
    Elasticsearch(EsClient),    // HTTP REST client
}

pub static CONNECTION_POOL: LazyLock<TokioMutex<HashMap<String, DbConnection>>> =
    LazyLock::new(|| TokioMutex::new(HashMap::new()));

/// Redis 连接配置备份：`db_connect` 时记下，供按 db 单独建连使用。
static REDIS_CONFIGS: LazyLock<TokioMutex<HashMap<String, DbConnectionConfig>>> =
    LazyLock::new(|| TokioMutex::new(HashMap::new()));

/// 按 `{id}:{db}` 缓存「URL 已绑定 db」的连接。
/// redis-rs 的 `MultiplexedConnection` clone 共享同一条连接，而 `SELECT` 是**连接级**状态：
/// 在共享连接上「SELECT dbN + 后续命令」会被并发请求插队串库（表现为某些 db 展开为空、
/// 计数对不上、点 key 读不到值）。改成每个 db 一条连接后，根本不再需要 SELECT。
static REDIS_DB_CONNS: LazyLock<TokioMutex<HashMap<String, RedisConn>>> =
    LazyLock::new(|| TokioMutex::new(HashMap::new()));

/// 取（必要时建立）指定 db 的 Redis 连接。
async fn redis_conn_for(id: &str, db_index: i64) -> Result<DbConnection, String> {
    let key = format!("{id}:{db_index}");
    if let Some(c) = REDIS_DB_CONNS.lock().await.get(&key).cloned() {
        return Ok(DbConnection::Redis(c));
    }
    let cfg = {
        let map = REDIS_CONFIGS.lock().await;
        map.get(id)
            .cloned()
            .or_else(|| {
                // 兼容：连接由更早的路径建立、没备份配置时退回共享连接（仍走 SELECT 语义）
                None
            })
            .ok_or_else(|| "Connection not found".to_string())?
    };
    let mut db_cfg = cfg;
    db_cfg.db_index = Some(db_index);
    let conn = connect_redis(&db_cfg).await?;
    if let DbConnection::Redis(ref c) = conn {
        REDIS_DB_CONNS.lock().await.insert(key, c.clone());
    }
    Ok(conn)
}

// ============ MySQL ============

pub async fn connect_mysql(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let decrypted_pw = config
        .password
        .as_deref()
        .map(|pw| supertool_core::encryption::try_decrypt_password(pw));
    let opts = mysql_async::OptsBuilder::default()
        .ip_or_hostname(config.host.clone())
        .tcp_port(config.port as u16)
        .user(Some(config.username.clone()))
        .pass(decrypted_pw)
        .db_name(config.db_name.clone());
    let pool = MySqlPool::new(opts);
    // S2: 连接超时（5s），避免主机不可达时 UI 无限转圈
    let mut conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        pool.get_conn(),
    )
    .await
    .map_err(|_| format!("MySQL connect timeout (host: {})", config.host))?
    .map_err(|e| format!("MySQL connection failed: {}", e))?;
    conn.ping()
        .await
        .map_err(|e| format!("MySQL ping failed: {}", e))?;
    Ok(DbConnection::MySql(pool))
}

/// 取 MySQL 表的列元数据：列名 → (数据类型, 是否可空)。查不到返回空表（退回原有拼接方式）。
async fn mysql_column_meta(
    pool: &MySqlPool,
    db_name: Option<&str>,
    table: &str,
) -> HashMap<String, (String, bool)> {
    let schema_cond = match db_name.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => format!("TABLE_SCHEMA = '{}'", s.replace('\'', "''")),
        None => "TABLE_SCHEMA = DATABASE()".to_string(),
    };
    let sql = format!(
        "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE FROM information_schema.COLUMNS WHERE {} AND TABLE_NAME = '{}'",
        schema_cond,
        table.replace('\'', "''")
    );
    let mut meta = HashMap::new();
    let rows = match execute_mysql_query(pool, &sql).await {
        Ok(v) => v.get("rows").and_then(|r| r.as_array()).cloned().unwrap_or_default(),
        Err(e) => {
            log::warn!("[db] 读取列元数据失败，退回原样拼接: {}", e);
            return meta;
        }
    };
    // information_schema 列名大小写随 MySQL 版本/配置变化，取值时两种都试
    let cell = |row: &serde_json::Value, col: &str| -> Option<String> {
        row.get(col)
            .or_else(|| row.get(&col.to_lowercase()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    for row in rows {
        let name = match cell(&row, "COLUMN_NAME") {
            Some(n) => n,
            None => continue,
        };
        let ty = cell(&row, "DATA_TYPE").unwrap_or_default().to_lowercase();
        let nullable = cell(&row, "IS_NULLABLE")
            .unwrap_or_else(|| "YES".to_string())
            .eq_ignore_ascii_case("yes");
        meta.insert(name, (ty, nullable));
    }
    meta
}

/// 按列类型生成 SQL 字面量。空串写进数值/时间列会被 MySQL 判成
/// `Truncated incorrect DOUBLE value: ''`（ERROR 1292），这里改成：可空列 → NULL，
/// 非空列 → 直接给出看得懂的错误。
fn mysql_literal(
    col: &str,
    meta: &HashMap<String, (String, bool)>,
    v: &serde_json::Value,
) -> Result<String, String> {
    if let serde_json::Value::String(s) = v {
        if s.trim().is_empty() {
            if let Some((ty, nullable)) = meta.get(col) {
                let typed = matches!(
                    ty.as_str(),
                    "tinyint"
                        | "smallint"
                        | "mediumint"
                        | "int"
                        | "integer"
                        | "bigint"
                        | "decimal"
                        | "numeric"
                        | "float"
                        | "double"
                        | "real"
                        | "bit"
                        | "year"
                        | "date"
                        | "datetime"
                        | "timestamp"
                        | "time"
                );
                if typed {
                    if *nullable {
                        return Ok("NULL".to_string());
                    }
                    return Err(format!(
                        "列 `{}` 是 {} 类型且不允许为空，请填写有效值",
                        col, ty
                    ));
                }
            }
        }
    }
    Ok(match v {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        _ => v.to_string(),
    })
}

async fn execute_mysql_query(pool: &MySqlPool, sql: &str) -> Result<serde_json::Value, String> {
    let mut conn = pool
        .get_conn()
        .await
        .map_err(|e| format!("MySQL connection failed: {}", e))?;
    // DDL/DML statements (ALTER, CREATE, DROP, INSERT, UPDATE, DELETE, USE, BEGIN, COMMIT, ROLLBACK) don't return rows
    let upper = sql.trim().to_uppercase();
    let first_word = upper.split_whitespace().next().unwrap_or("");
    let is_ddl = matches!(
        first_word,
        "ALTER"
            | "CREATE"
            | "DROP"
            | "INSERT"
            | "UPDATE"
            | "DELETE"
            | "USE"
            | "BEGIN"
            | "COMMIT"
            | "ROLLBACK"
            | "TRUNCATE"
            | "RENAME"
            | "GRANT"
            | "REVOKE"
    );
    if is_ddl {
        conn.query_drop(sql)
            .await
            .map_err(|e| format!("MySQL query failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "rows": [] }))
    } else {
        let rows: Vec<Row> = conn
            .query(sql)
            .await
            .map_err(|e| format!("MySQL query failed: {}", e))?;
        let result = mysql_rows_to_json(&rows);
        Ok(serde_json::json!({ "success": true, "rows": result }))
    }
}

fn mysql_rows_to_json(rows: &[Row]) -> Vec<serde_json::Value> {
    if rows.is_empty() {
        return vec![];
    }
    let columns = rows[0].columns();
    rows.iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col) in columns.as_ref().iter().enumerate() {
                let val = row.as_ref(i);
                let json_val = match val {
                    Some(mysql_async::Value::NULL) => serde_json::Value::Null,
                    Some(mysql_async::Value::Bytes(b)) => {
                        serde_json::Value::String(String::from_utf8_lossy(b).to_string())
                    }
                    Some(mysql_async::Value::Int(n)) => {
                        serde_json::Value::Number(serde_json::Number::from(*n))
                    }
                    Some(mysql_async::Value::UInt(n)) => {
                        serde_json::Value::Number(serde_json::Number::from(*n))
                    }
                    Some(mysql_async::Value::Float(f)) => serde_json::Number::from_f64(*f as f64)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    Some(mysql_async::Value::Double(f)) => serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    Some(mysql_async::Value::Date(y, m, d, h, min, s, _)) => {
                        serde_json::Value::String(format!(
                            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                            y, m, d, h, min, s
                        ))
                    }
                    Some(mysql_async::Value::Time(neg, d, h, m, s, _)) => {
                        serde_json::Value::String(format!(
                            "{}{}d {}:{}:{}",
                            if *neg { "-" } else { "" },
                            d,
                            h,
                            m,
                            s
                        ))
                    }
                    _ => serde_json::Value::Null,
                };
                obj.insert(col.name_str().to_string(), json_val);
            }
            serde_json::Value::Object(obj)
        })
        .collect()
}

// ============ PostgreSQL ============

pub async fn connect_postgres(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let decrypted_pw = config
        .password
        .as_deref()
        .map(|pw| supertool_core::encryption::try_decrypt_password(pw))
        .unwrap_or_default();
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host,
        config.port,
        config.username,
        decrypted_pw,
        config.db_name.as_deref().unwrap_or("postgres")
    );
    // S2: 连接超时（5s），避免主机不可达时 UI 无限转圈
    let (client, connection) = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio_postgres::connect(&conn_str, NoTls),
    )
    .await
    .map_err(|_| format!("PostgreSQL connect timeout (host: {})", config.host))?
    .map_err(|e| format!("PostgreSQL connection failed: {}", e))?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("PostgreSQL connection error: {}", e);
        }
    });
    client
        .batch_execute("SELECT 1")
        .await
        .map_err(|e| format!("PostgreSQL ping failed: {}", e))?;
    Ok(DbConnection::Postgres(std::sync::Arc::new(client)))
}

async fn execute_postgres_query(client: &PgClient, sql: &str) -> Result<serde_json::Value, String> {
    let rows = client
        .query(sql, &[])
        .await
        .map_err(|e| format!("PostgreSQL query failed: {}", e))?;
    let result: Vec<serde_json::Value> = rows.iter().map(pg_row_to_json).collect();
    Ok(serde_json::json!({ "success": true, "rows": result }))
}

fn pg_row_to_json(row: &tokio_postgres::Row) -> serde_json::Value {
    use tokio_postgres::types::Type;
    let columns = row.columns();
    let mut obj = serde_json::Map::new();
    for (i, col) in columns.iter().enumerate() {
        let val = match *col.type_() {
            Type::INT2 => row
                .try_get::<_, i16>(i)
                .ok()
                .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
            Type::INT4 => row
                .try_get::<_, i32>(i)
                .ok()
                .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
            Type::INT8 => row
                .try_get::<_, i64>(i)
                .ok()
                .map(|v| serde_json::Value::Number(serde_json::Number::from(v))),
            Type::FLOAT4 => row.try_get::<_, f32>(i).ok().and_then(|v| {
                serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number)
            }),
            Type::FLOAT8 => row
                .try_get::<_, f64>(i)
                .ok()
                .and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number)),
            Type::BOOL => row.try_get::<_, bool>(i).ok().map(serde_json::Value::Bool),
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME | Type::JSON | Type::JSONB => {
                row.try_get::<_, String>(i)
                    .ok()
                    .map(serde_json::Value::String)
            }
            // S1 修复：时间类型分别处理，TIMESTAMPTZ/DATE/TIME 与 NaiveDateTime 不兼容会返回 NULL
            Type::TIMESTAMP => row
                .try_get::<_, chrono::NaiveDateTime>(i)
                .ok()
                .map(|v| serde_json::Value::String(v.to_string())),
            Type::TIMESTAMPTZ => row
                .try_get::<_, chrono::DateTime<chrono::Utc>>(i)
                .ok()
                .map(|v| serde_json::Value::String(v.to_rfc3339())),
            Type::DATE => row
                .try_get::<_, chrono::NaiveDate>(i)
                .ok()
                .map(|v| serde_json::Value::String(v.to_string())),
            Type::TIME => row
                .try_get::<_, chrono::NaiveTime>(i)
                .ok()
                .map(|v| serde_json::Value::String(v.to_string())),
            _ => row
                .try_get::<_, String>(i)
                .ok()
                .map(serde_json::Value::String)
                .or_else(|| {
                    row.try_get::<_, Vec<u8>>(i)
                        .ok()
                        .map(|b| serde_json::Value::String(hex_encode(&b)))
                }),
        }
        .unwrap_or(serde_json::Value::Null);
        obj.insert(col.name().to_string(), val);
    }
    serde_json::Value::Object(obj)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ============ Redis ============

pub async fn connect_redis(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    let db_idx = config.db_index.unwrap_or(0);
    let url = if let Some(ref pw) = config.password {
        let decrypted = supertool_core::encryption::try_decrypt_password(pw);
        format!(
            "redis://:{}@{}:{}/{}",
            decrypted, config.host, config.port, db_idx
        )
    } else {
        format!("redis://{}:{}/{}", config.host, config.port, db_idx)
    };
    let client =
        redis::Client::open(url.as_str()).map_err(|e| format!("Redis connection failed: {}", e))?;
    // S2: 连接超时（5s），避免主机不可达时 UI 无限转圈
    let conn = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get_multiplexed_async_connection(),
    )
    .await
    .map_err(|_| format!("Redis connect timeout (host: {})", config.host))?
    .map_err(|e| format!("Redis connection failed: {}", e))?;
    let _: String = redis::cmd("PING")
        .query_async(&mut conn.clone())
        .await
        .map_err(|e| format!("Redis ping failed: {}", e))?;
    Ok(DbConnection::Redis(conn))
}

async fn execute_redis_command(
    conn: &RedisConn,
    command: &str,
) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty Redis command".to_string());
    }
    let mut cmd = redis::Cmd::new();
    cmd.arg(parts[0]);
    for part in &parts[1..] {
        cmd.arg(part);
    }
    let result: redis::Value = cmd
        .query_async(&mut conn.clone())
        .await
        .map_err(|e| format!("Redis command failed: {}", e))?;
    Ok(serde_json::json!({ "success": true, "result": redis_value_to_json(&result) }))
}

fn redis_value_to_json(val: &redis::Value) -> serde_json::Value {
    match val {
        redis::Value::Nil => serde_json::Value::Null,
        redis::Value::Int(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        redis::Value::BulkString(b) => {
            serde_json::Value::String(String::from_utf8_lossy(b).to_string())
        }
        redis::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(redis_value_to_json).collect())
        }
        redis::Value::Okay => serde_json::Value::String("OK".to_string()),
        redis::Value::SimpleString(s) => serde_json::Value::String(s.clone()),
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

// ============ SQLite (rusqlite - sync, wrapped in spawn_blocking) ============

pub(crate) async fn execute_sqlite_query(
    config: &DbConnectionConfig,
    sql: &str,
) -> Result<serde_json::Value, String> {
    let db_path = config
        .path
        .as_deref()
        .ok_or_else(|| "SQLite database path required".to_string())?;
    let db_path = db_path.to_string();
    let sql = sql.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("SQLite open failed: {}", e))?;
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("SQLite prepare failed: {}", e))?;
        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let rows = stmt
            .query_map([], |row| {
                let mut obj = serde_json::Map::new();
                for (i, name) in column_names.iter().enumerate() {
                    let val: rusqlite::types::Value = row.get(i).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            i,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;
                    let json_val = match val {
                        rusqlite::types::Value::Null => serde_json::Value::Null,
                        rusqlite::types::Value::Integer(n) => {
                            serde_json::Value::Number(serde_json::Number::from(n))
                        }
                        rusqlite::types::Value::Real(f) => serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null),
                        rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                        rusqlite::types::Value::Blob(b) => {
                            serde_json::Value::String(hex_encode(&b))
                        }
                    };
                    obj.insert(name.clone(), json_val);
                }
                Ok(serde_json::Value::Object(obj))
            })
            .map_err(|e| format!("SQLite query failed: {}", e))?;
        let result: Result<Vec<_>, _> = rows.collect();
        let rows = result.map_err(|e| format!("SQLite row collect failed: {}", e))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "rows": rows }))
    })
    .await
    .map_err(|e| format!("SQLite task failed: {}", e))?
}

/// Execute SQLite write operations (INSERT, UPDATE, DELETE, etc.)
async fn execute_sqlite_write(
    config: &DbConnectionConfig,
    sql: &str,
) -> Result<serde_json::Value, String> {
    let db_path = config
        .path
        .as_deref()
        .ok_or_else(|| "SQLite database path required".to_string())?;
    let db_path = db_path.to_string();
    let sql = sql.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .map_err(|e| format!("SQLite open failed: {}", e))?;
        conn.execute(&sql, [])
            .map_err(|e| format!("SQLite execute failed: {}", e))?;
        Ok::<_, String>(serde_json::json!({ "success": true, "rows": [] }))
    })
    .await
    .map_err(|e| format!("SQLite task failed: {}", e))?
}

// ============ Tauri Commands ============

#[tauri::command(rename_all = "camelCase")]
pub async fn db_connect(config: DbConnectionConfig) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] db_connect() called, type={}, host={}",
        config.db_type,
        supertool_core::logic::log_sanitizer::sanitize_string(&config.host)
    );
    let conn = match config.db_type.as_str() {
        "mysql" => connect_mysql(&config).await?,
        "postgres" => connect_postgres(&config).await?,
        "redis" => connect_redis(&config).await?,
        "sqlite" => DbConnection::Sqlite(config.clone()),
        "elasticsearch" => DbConnection::Elasticsearch(crate::commands::es::connect_es(&config).await?),
        other => return Err(format!("Unsupported database type: {}", other)),
    };
    if config.db_type == "redis" {
        REDIS_CONFIGS.lock().await.insert(config.id.clone(), config.clone());
        // 配置可能变更（host/密码/默认 db），清掉旧的按 db 缓存的连接
        REDIS_DB_CONNS.lock().await.retain(|k, _| !k.starts_with(&format!("{}:", config.id)));
    }
    let mut pool = CONNECTION_POOL.lock().await;
    pool.insert(config.id.clone(), conn);
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_disconnect(id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_disconnect() called");
    CONNECTION_POOL.lock().await.remove(&id);
    REDIS_CONFIGS.lock().await.remove(&id);
    REDIS_DB_CONNS.lock().await.retain(|k, _| !k.starts_with(&format!("{id}:")));
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_query(id: String, sql: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_query() called");
    // 只读白名单（与 CLI 审批连接同规则）：SELECT/SHOW/EXPLAIN/DESC/PRAGMA 查询；
    // WITH 可携带写语句不放行；PRAGMA 赋值形式拦截。写操作请走 db_execute_write
    if !supertool_core::logic::sql_safety::is_read_only_sql(&sql) {
        let first = sql.trim().split_whitespace().next().unwrap_or("");
        return Err(format!(
            "Only read-only queries allowed. Blocked: {}（写操作请使用专用命令）",
            first
        ));
    }
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found. Call db:connect first.".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &sql).await,
        DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), &sql).await,
        DbConnection::Redis(c) => execute_redis_command(c, &sql).await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, &sql).await,
        DbConnection::Elasticsearch(_) => Err("Elasticsearch 不支持 SQL 查询".to_string()),
    }
}

/// GUI 专用写通道（DROP TABLE / ALTER / 结构同步等）。
/// CLI/MCP 层对审批连接仍走 is_read_only_sql 白名单，此处仅限 GUI 用户手动操作。
#[tauri::command(rename_all = "camelCase")]
pub async fn db_execute_write(id: String, sql: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_execute_write() called");
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found. Call db:connect first.".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &sql).await,
        DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), &sql).await,
        // SQLite 写操作必须用 READ_WRITE 连接（execute_sqlite_query 是只读连接）
        DbConnection::Sqlite(cfg) => execute_sqlite_write(cfg, &sql).await,
        DbConnection::Redis(_) => Err("Redis 不支持 SQL 写操作".to_string()),
        DbConnection::Elasticsearch(_) => Err("Elasticsearch 不支持 SQL 写操作".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_tables(id: String, db_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_get_tables() called");
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => execute_mysql_query(p, &format!("SHOW TABLES FROM {}", quote_ident(&db_name, "mysql"))).await,
        DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename").await,
        DbConnection::Sqlite(cfg) => execute_sqlite_query(cfg, "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").await,
        _ => Err("Unsupported database type for listing tables".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_databases(id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_get_databases() called");
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            let result = execute_mysql_query(p, "SHOW DATABASES").await;
            // Filter out system databases
            result.map(|mut v| {
                if let Some(rows) = v.get_mut("rows").and_then(|r| r.as_array_mut()) {
                    rows.retain(|row| {
                        row.as_object()
                            .and_then(|obj| obj.values().next())
                            .and_then(|v| v.as_str())
                            .map(|name| {
                                !matches!(
                                    name,
                                    "information_schema" | "performance_schema" | "mysql" | "sys"
                                )
                            })
                            .unwrap_or(true)
                    });
                }
                v
            })
        }
        DbConnection::Postgres(c) => {
            let result = execute_postgres_query(
                c,
                "SELECT datname FROM pg_database WHERE datistemplate = false",
            )
            .await;
            result.map(|mut v| {
                if let Some(rows) = v.get_mut("rows").and_then(|r| r.as_array_mut()) {
                    rows.retain(|row| {
                        row.as_object()
                            .and_then(|obj| obj.values().next())
                            .and_then(|v| v.as_str())
                            .map(|name| !matches!(name, "postgres" | "template0" | "template1"))
                            .unwrap_or(true)
                    });
                }
                v
            })
        }
        DbConnection::Sqlite(cfg) => Ok(
            serde_json::json!({ "success": true, "rows": [cfg.path.as_deref().unwrap_or("main")] }),
        ),
        _ => Err("Unsupported database type for listing databases".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_table_structure(
    id: String,
    db_name: Option<String>,
    table: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_get_table_structure() called");
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            let db = db_name.unwrap_or_default();
            // Query columns
            let col_sql = format!(
                "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY, EXTRA, COLUMN_COMMENT, ORDINAL_POSITION FROM information_schema.COLUMNS WHERE TABLE_SCHEMA='{}' AND TABLE_NAME='{}' ORDER BY ORDINAL_POSITION",
                db.replace('\'', "''"), table.replace('\'', "''")
            );
            let col_result = execute_mysql_query(p, &col_sql).await?;
            // Query indexes
            let idx_sql = format!("SHOW INDEX FROM `{}`.`{}`", db, table);
            let idx_result = execute_mysql_query(p, &idx_sql).await?;
            // Merge into one response
            let cols = col_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let idxs = idx_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            Ok(serde_json::json!({ "success": true, "rows": cols, "indexes": idxs }))
        }
        DbConnection::Postgres(c) => {
            // Query columns with length/precision/comment
            let col_sql = format!(
                "SELECT c.column_name AS \"COLUMN_NAME\", \
                 CASE \
                   WHEN c.data_type = 'character varying' THEN 'varchar(' || COALESCE(c.character_maximum_length::text, '255') || ')' \
                   WHEN c.data_type = 'character' THEN 'char(' || COALESCE(c.character_maximum_length::text, '1') || ')' \
                   WHEN c.data_type = 'numeric' THEN 'numeric(' || COALESCE(c.numeric_precision::text, '10') || ',' || COALESCE(c.numeric_scale::text, '0') || ')' \
                   WHEN c.domain_name IS NOT NULL THEN c.domain_name \
                   ELSE c.data_type \
                 END AS \"COLUMN_TYPE\", \
                 c.is_nullable AS \"IS_NULLABLE\", \
                 c.column_default AS \"COLUMN_DEFAULT\", \
                 c.ordinal_position AS \"ORDINAL_POSITION\", \
                 COALESCE(pgd.description, '') AS \"COLUMN_COMMENT\", \
                 CASE WHEN pk.column_name IS NOT NULL THEN 'PRI' ELSE '' END AS \"COLUMN_KEY\", \
                 '' AS \"EXTRA\" \
                 FROM information_schema.columns c \
                 LEFT JOIN pg_catalog.pg_statio_all_tables st ON st.schemaname = c.table_schema AND st.relname = c.table_name \
                 LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = st.relid AND pgd.objsubid = c.ordinal_position \
                 LEFT JOIN ( \
                   SELECT ku.column_name, ku.table_schema, ku.table_name \
                   FROM information_schema.table_constraints tc \
                   JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name AND tc.table_schema = ku.table_schema \
                   WHERE tc.constraint_type = 'PRIMARY KEY' \
                 ) pk ON pk.column_name = c.column_name AND pk.table_schema = c.table_schema AND pk.table_name = c.table_name \
                 WHERE c.table_schema='public' AND c.table_name='{}' \
                 ORDER BY c.ordinal_position",
                table.replace('\'', "''")
            );
            let col_result = execute_postgres_query(c, &col_sql).await?;
            // Query indexes
            let idx_sql = format!(
                "SELECT i.relname AS index_name, ix.indisunique AS is_unique, ix.indisprimary AS is_primary, \
                 array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) AS columns \
                FROM pg_class t JOIN pg_index ix ON t.oid = ix.indrelid \
                JOIN pg_class i ON i.oid = ix.indexrelid \
                JOIN pg_namespace n ON n.oid = t.relnamespace \
                JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey) \
                WHERE n.nspname = 'public' AND t.relname = '{}' \
                GROUP BY i.relname, ix.indisunique, ix.indisprimary ORDER BY i.relname",
                table.replace('\'', "''")
            );
            let idx_result = execute_postgres_query(c, &idx_sql).await?;
            let cols = col_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let idxs = idx_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            Ok(serde_json::json!({ "success": true, "rows": cols, "indexes": idxs }))
        }
        DbConnection::Sqlite(cfg) => {
            // SQLite: PRAGMA table_info returns cid, name, type, notnull, dflt_value, pk
            // Convert to MySQL-compatible format
            let col_sql = format!(
                "SELECT \
                    name AS \"COLUMN_NAME\", \
                    type AS \"COLUMN_TYPE\", \
                    CASE WHEN \"notnull\" = 1 THEN 'NO' ELSE 'YES' END AS \"IS_NULLABLE\", \
                    \"dflt_value\" AS \"COLUMN_DEFAULT\", \
                    CASE WHEN \"pk\" > 0 THEN 'PRI' ELSE '' END AS \"COLUMN_KEY\", \
                    '' AS \"EXTRA\", \
                    '' AS \"COLUMN_COMMENT\", \
                    \"cid\" + 1 AS \"ORDINAL_POSITION\" \
                FROM pragma_table_info('{}') ORDER BY \"cid\"",
                table.replace('\'', "''")
            );
            let col_result = execute_sqlite_query(cfg, &col_sql).await?;
            // SQLite indexes: PRAGMA index_list + index_info (need to query columns for each index)
            let idx_list_sql = format!(
                "SELECT name, \"unique\" FROM pragma_index_list('{}')",
                table
            );
            let idx_list_result = execute_sqlite_query(cfg, &idx_list_sql).await?;
            let cols = col_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));

            // Build index array with column info
            let idx_rows = idx_list_result
                .get("rows")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let mut indexes_with_columns: Vec<serde_json::Value> = vec![];
            if let serde_json::Value::Array(rows) = idx_rows {
                for row in rows {
                    let idx_name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    // Query columns for this specific index
                    let idx_info_sql = format!(
                        "SELECT name FROM pragma_index_info('{}') ORDER BY seqno",
                        idx_name
                    );
                    let idx_info_result = execute_sqlite_query(cfg, &idx_info_sql).await?;
                    let col_rows = idx_info_result
                        .get("rows")
                        .cloned()
                        .unwrap_or(serde_json::Value::Array(vec![]));
                    let columns: Vec<String> = if let serde_json::Value::Array(cr) = col_rows {
                        cr.iter()
                            .filter_map(|c| {
                                c.get("name")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            })
                            .collect()
                    } else {
                        vec![]
                    };
                    let is_unique = row.get("unique").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
                    indexes_with_columns.push(serde_json::json!({
                        "name": idx_name,
                        "columns": columns,
                        "isUnique": is_unique,
                        "isPrimary": false  // SQLite primary keys are not in index_list
                    }));
                }
            }

            Ok(
                serde_json::json!({ "success": true, "rows": cols, "indexes": indexes_with_columns }),
            )
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_table_data(
    id: String,
    db_name: String,
    table: String,
    limit: usize,
    offset: usize,
    order_by: Option<String>,
    order_dir: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_get_table_data() called");
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    let safe_dir = match order_dir.as_deref().unwrap_or("").to_uppercase().as_str() {
        "DESC" => "DESC",
        _ => "ASC",
    };
    let safe_order =
        order_by.filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_'));
    match &conn {
        DbConnection::MySql(p) => {
            let order = safe_order
                .map(|s| format!(" ORDER BY `{}` {}", s, safe_dir))
                .unwrap_or_default();
            let sql = format!(
                "SELECT * FROM `{}`.`{}`{} LIMIT {} OFFSET {}",
                db_name, table, order, limit, offset
            );
            let resp = execute_mysql_query(p, &sql).await?;
            // B5: 返回 total 供前端分页
            let mut total = 0u64;
            if let Ok(cv) = execute_mysql_query(p, &format!("SELECT COUNT(*) AS c FROM `{}`.`{}`", db_name, table)).await {
                if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                    total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        DbConnection::Postgres(c) => {
            let order = safe_order
                .map(|s| format!(" ORDER BY \"{}\" {}", s, safe_dir))
                .unwrap_or_default();
            let sql = format!(
                "SELECT * FROM \"{}\".\"{}\"{} LIMIT {} OFFSET {}",
                db_name, table, order, limit, offset
            );
            let resp = execute_postgres_query(c, &sql).await?;
            let mut total = 0u64;
            if let Ok(cv) = execute_postgres_query(c, &format!("SELECT COUNT(*) AS c FROM \"{}\".\"{}\"", db_name, table)).await {
                if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                    total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        DbConnection::Sqlite(cfg) => {
            let order = safe_order
                .map(|s| format!(" ORDER BY \"{}\" {}", s, safe_dir))
                .unwrap_or_default();
            let resp = execute_sqlite_query(
                cfg,
                &format!(
                    "SELECT * FROM \"{}\"{} LIMIT {} OFFSET {}",
                    table, order, limit, offset
                ),
            )
            .await?;
            let mut total = 0u64;
            if let Ok(cv) = execute_sqlite_query(cfg, &format!("SELECT COUNT(*) AS c FROM \"{}\"", table)).await {
                if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                    total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_table_primary_keys(
    id: String,
    table: String,
    db_name: String,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            let sql = format!(
                "SELECT COLUMN_NAME FROM information_schema.KEY_COLUMN_USAGE WHERE TABLE_SCHEMA='{}' AND TABLE_NAME='{}' AND CONSTRAINT_NAME='PRIMARY' ORDER BY ORDINAL_POSITION",
                db_name.replace('\'', "''"), table.replace('\'', "''")
            );
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Postgres(c) => {
            let sql = format!(
                "SELECT kcu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name WHERE tc.table_schema='public' AND tc.table_name='{}' AND tc.constraint_type='PRIMARY KEY'",
                table.replace('\'', "''")
            );
            execute_postgres_query(c, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            // SQLite: PRAGMA table_info returns pk column index > 0
            let sql = format!(
                "SELECT name FROM pragma_table_info('{}') WHERE pk > 0 ORDER BY pk",
                table.replace('\'', "''")
            );
            execute_sqlite_query(cfg, &sql).await
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_views(id: String, db_name: String) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            execute_mysql_query(
                p,
                &format!(
                    "SHOW FULL TABLES FROM `{}` WHERE Table_type = 'VIEW'",
                    db_name
                ),
            )
            .await
        }
        DbConnection::Postgres(c) => {
            execute_postgres_query(
                c,
                "SELECT table_name FROM information_schema.views WHERE table_schema='public'",
            )
            .await
        }
        DbConnection::Sqlite(cfg) => {
            execute_sqlite_query(
                cfg,
                "SELECT name FROM sqlite_master WHERE type='view' AND name NOT LIKE 'sqlite_%'",
            )
            .await
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_create_sql(
    id: String,
    table: String,
    db_name: String,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            let sql = format!("SHOW CREATE TABLE {}.{}", quote_ident(&db_name, "mysql"), quote_ident(&table, "mysql"));
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Postgres(_c) => {
            Err("PostgreSQL: use pg_dump for CREATE TABLE SQL".to_string())
        }
        DbConnection::Sqlite(cfg) => {
            // SQLite: get CREATE statement from sqlite_master
            let sql = format!(
                "SELECT sql AS 'Create Table' FROM sqlite_master WHERE type='table' AND name='{}'",
                table
            );
            execute_sqlite_query(cfg, &sql).await
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

// Internal helper (not a Tauri command)
async fn get_conn_type(id: &str) -> String {
    let pool = CONNECTION_POOL.lock().await;
    match pool.get(id) {
        Some(DbConnection::MySql(_)) => "mysql".to_string(),
        Some(DbConnection::Postgres(_)) => "postgresql".to_string(),
        Some(DbConnection::Sqlite(_)) => "sqlite".to_string(),
        _ => "mysql".to_string(),
    }
}

/// Quote identifier based on database type
fn quote_ident(name: &str, db_type: &str) -> String {
    match db_type {
        "postgresql" => format!("\"{}\"", name.replace('"', "\"\"")),
        "sqlite" => format!("\"{}\"", name.replace('"', "\"\"")),
        _ => format!("`{}`", name.replace('`', "``")),
    }
}

// Compare & Sync
#[tauri::command(rename_all = "camelCase")]
pub async fn db_compare_structures(
    app: tauri::AppHandle,
    source_id: String,
    source_db: String,
    target_id: String,
    target_db: String,
    _table_name: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_compare_structures() called");
    let db_type = get_conn_type(&target_id).await;

    // Get table lists from both databases
    let src_tables_result = db_get_tables(source_id.clone(), source_db.clone()).await;
    let tgt_tables_result = db_get_tables(target_id.clone(), target_db.clone()).await;

    let src_tables: Vec<String> = match &src_tables_result {
        Ok(v) => v
            .get("rows")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|row| {
                        row.as_object()
                            .and_then(|obj| obj.values().next())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => vec![],
    };
    let tgt_tables: Vec<String> = match &tgt_tables_result {
        Ok(v) => v
            .get("rows")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|row| {
                        row.as_object()
                            .and_then(|obj| obj.values().next())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => vec![],
    };

    // Case-insensitive table matching (like Electron)
    let src_lower_map: std::collections::HashMap<String, String> = src_tables
        .iter()
        .map(|t| (t.to_lowercase(), t.clone()))
        .collect();
    let tgt_lower_map: std::collections::HashMap<String, String> = tgt_tables
        .iter()
        .map(|t| (t.to_lowercase(), t.clone()))
        .collect();
    let src_lower_set: std::collections::HashSet<&str> =
        src_lower_map.keys().map(|s| s.as_str()).collect();
    let tgt_lower_set: std::collections::HashSet<&str> =
        tgt_lower_map.keys().map(|s| s.as_str()).collect();

    let only_source: Vec<String> = src_lower_set
        .difference(&tgt_lower_set)
        .map(|k| src_lower_map[*k].clone())
        .collect();
    let only_target: Vec<String> = tgt_lower_set
        .difference(&src_lower_set)
        .map(|k| tgt_lower_map[*k].clone())
        .collect();
    // common tables as (source_actual_name, target_actual_name) pairs
    let common: Vec<(String, String)> = src_lower_set
        .intersection(&tgt_lower_set)
        .map(|k| (src_lower_map[*k].clone(), tgt_lower_map[*k].clone()))
        .collect();

    let mut diffs: Vec<serde_json::Value> = vec![];

    // Tables only in source — generate CREATE TABLE SQL
    for t in &only_source {
        let create_sql =
            match db_get_table_structure(source_id.clone(), Some(source_db.clone()), t.to_string())
                .await
            {
                Ok(v) => {
                    let cols = v
                        .get("rows")
                        .and_then(|r| r.as_array())
                        .cloned()
                        .unwrap_or_default();
                    generate_create_table_sql(t, &cols)
                }
                Err(_) => format!("-- Failed to get structure for table '{}'", t),
            };
        diffs.push(serde_json::json!({
            "tableName": t,
            "diffType": "table_only_in_source",
            "sourceValue": format!("表 {} 仅存在于源数据库", t),
            "sql": create_sql
        }));
    }

    // Tables only in target
    for t in &only_target {
        diffs.push(serde_json::json!({
            "tableName": t,
            "diffType": "table_only_in_target",
            "targetValue": format!("表 {} 仅存在于目标数据库", t),
            "sql": format!("-- Table '{}' only exists in target", t)
        }));
    }

    // Compare column structures for common tables — use ACTUAL table names from each DB
    let total_tables = only_source.len() + only_target.len() + common.len();
    let mut progress: usize = only_source.len() + only_target.len();

    for (src_table, tgt_table) in &common {
        progress += 1;
        let _ = app.emit(
            "db:compare-progress",
            serde_json::json!({
                "current": progress,
                "total": total_tables,
                "table": src_table,
            }),
        );

        let src_struct = db_get_table_structure(
            source_id.clone(),
            Some(source_db.clone()),
            src_table.to_string(),
        )
        .await;
        let tgt_struct = db_get_table_structure(
            target_id.clone(),
            Some(target_db.clone()),
            tgt_table.to_string(),
        )
        .await;

        let src_resp = src_struct.as_ref().ok();
        let tgt_resp = tgt_struct.as_ref().ok();

        let src_cols = src_resp
            .and_then(|v| v.get("rows"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let tgt_cols = tgt_resp
            .and_then(|v| v.get("rows"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        // Build column maps: lowercase name -> (actual name, column info) — case-insensitive matching
        let src_map: std::collections::HashMap<String, (String, serde_json::Value)> = src_cols
            .iter()
            .filter_map(|c| {
                c.get("COLUMN_NAME")
                    .and_then(|v| v.as_str())
                    .map(|name| (name.to_lowercase(), (name.to_string(), c.clone())))
            })
            .collect();
        let tgt_map: std::collections::HashMap<String, (String, serde_json::Value)> = tgt_cols
            .iter()
            .filter_map(|c| {
                c.get("COLUMN_NAME")
                    .and_then(|v| v.as_str())
                    .map(|name| (name.to_lowercase(), (name.to_string(), c.clone())))
            })
            .collect();

        // Columns only in source
        for (key, (name, col)) in &src_map {
            if !tgt_map.contains_key(key) {
                let col_def = build_col_def_from_info(col, &db_type);
                diffs.push(serde_json::json!({
                    "tableName": tgt_table,
                    "diffType": "column_added",
                    "columnName": name,
                    "sourceValue": col,
                    "sql": format!("ALTER TABLE {} ADD COLUMN {};", quote_ident(tgt_table, &db_type), col_def)
                }));
            }
        }

        // Columns only in target
        for (key, (name, col)) in &tgt_map {
            if !src_map.contains_key(key) {
                diffs.push(serde_json::json!({
                    "tableName": tgt_table,
                    "diffType": "column_removed",
                    "columnName": name,
                    "targetValue": col,
                    "sql": format!("-- Column '{}' only exists in target", name)
                }));
            }
        }

        // Columns in both — compare attributes (case-insensitive)
        for (key, (name, src_col)) in &src_map {
            if let Some((_, tgt_col)) = tgt_map.get(key) {
                let src_type = src_col
                    .get("COLUMN_TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let tgt_type = tgt_col
                    .get("COLUMN_TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let src_nullable = src_col
                    .get("IS_NULLABLE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NO");
                let tgt_nullable = tgt_col
                    .get("IS_NULLABLE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NO");
                let src_default = src_col
                    .get("COLUMN_DEFAULT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NULL");
                let tgt_default = tgt_col
                    .get("COLUMN_DEFAULT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("NULL");
                let src_comment = src_col
                    .get("COLUMN_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let tgt_comment = tgt_col
                    .get("COLUMN_COMMENT")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let src_extra = src_col.get("EXTRA").and_then(|v| v.as_str()).unwrap_or("");
                let tgt_extra = tgt_col.get("EXTRA").and_then(|v| v.as_str()).unwrap_or("");

                let mut changes: Vec<String> = vec![];
                if src_type != tgt_type {
                    changes.push(format!("类型: {} → {}", tgt_type, src_type));
                }
                if src_nullable != tgt_nullable {
                    changes.push(format!("允许NULL: {} → {}", tgt_nullable, src_nullable));
                }
                if src_default != tgt_default {
                    changes.push(format!("默认值: {} → {}", tgt_default, src_default));
                }
                if src_comment != tgt_comment {
                    changes.push(format!("注释: '{}' → '{}'", tgt_comment, src_comment));
                }
                if src_extra != tgt_extra {
                    changes.push(format!("额外: {} → {}", tgt_extra, src_extra));
                }

                if !changes.is_empty() {
                    let col_def = build_col_def_from_info(src_col, &db_type);
                    diffs.push(serde_json::json!({
                        "tableName": tgt_table,
                        "diffType": "column_modified",
                        "columnName": name,
                        "sourceValue": src_col,
                        "targetValue": tgt_col,
                        "sql": format!("ALTER TABLE {} MODIFY COLUMN {};", quote_ident(tgt_table, &db_type), col_def)
                    }));
                }
            }
        }

        // === Compare indexes (case-insensitive, skip PRIMARY) ===
        let src_idxs = src_resp
            .and_then(|v| v.get("indexes"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        let tgt_idxs = tgt_resp
            .and_then(|v| v.get("indexes"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        // Group index rows by index name → { name, unique, columns[] }
        let src_idx_map = group_indexes(&src_idxs);
        let tgt_idx_map = group_indexes(&tgt_idxs);

        for (idx_key, src_idx) in &src_idx_map {
            let idx_name = src_idx["name"].as_str().unwrap_or("");
            if idx_name.to_uppercase() == "PRIMARY" {
                continue;
            }
            if let Some(tgt_idx) = tgt_idx_map.get(idx_key) {
                // Index exists in both — compare columns and uniqueness
                let src_cols_str = src_idx["columns"].as_str().unwrap_or("");
                let tgt_cols_str = tgt_idx["columns"].as_str().unwrap_or("");
                let src_unique = src_idx["unique"].as_bool().unwrap_or(false);
                let tgt_unique = tgt_idx["unique"].as_bool().unwrap_or(false);
                if src_cols_str != tgt_cols_str || src_unique != tgt_unique {
                    diffs.push(serde_json::json!({
                        "tableName": tgt_table,
                        "diffType": "index_modified",
                        "sourceValue": src_idx,
                        "targetValue": tgt_idx,
                        "sql": format!("DROP INDEX IF EXISTS {} ON {}; CREATE INDEX {} ON {} ({});",
                            quote_ident(idx_name, &db_type), quote_ident(tgt_table, &db_type), quote_ident(idx_name, &db_type), quote_ident(tgt_table, &db_type), src_cols_str)
                    }));
                }
            } else {
                // Index only in source → need to create
                let src_cols_str = src_idx["columns"].as_str().unwrap_or("");
                let unique_str = if src_idx["unique"].as_bool().unwrap_or(false) {
                    "UNIQUE "
                } else {
                    ""
                };
                diffs.push(serde_json::json!({
                    "tableName": tgt_table,
                    "diffType": "index_added",
                    "sourceValue": src_idx,
                    "sql": format!("CREATE {}INDEX {} ON {} ({});", unique_str, quote_ident(idx_name, &db_type), quote_ident(tgt_table, &db_type), src_cols_str)
                }));
            }
        }

        for (idx_key, tgt_idx) in &tgt_idx_map {
            let idx_name = tgt_idx["name"].as_str().unwrap_or("");
            if idx_name.to_uppercase() == "PRIMARY" {
                continue;
            }
            if !src_idx_map.contains_key(idx_key) {
                diffs.push(serde_json::json!({
                    "tableName": tgt_table,
                    "diffType": "index_removed",
                    "targetValue": tgt_idx,
                    "sql": format!("DROP INDEX IF EXISTS {} ON {};", quote_ident(idx_name, &db_type), quote_ident(tgt_table, &db_type))
                }));
            }
        }

        // === Compare primary keys ===
        let src_pks = extract_primary_keys(&src_cols);
        let tgt_pks = extract_primary_keys(&tgt_cols);
        let src_pk_str = src_pks.join(",");
        let tgt_pk_str = tgt_pks.join(",");
        if !src_pk_str.is_empty() && !tgt_pk_str.is_empty() && src_pk_str != tgt_pk_str {
            diffs.push(serde_json::json!({
                "tableName": tgt_table,
                "diffType": "primary_key_changed",
                "sourceValue": src_pks,
                "targetValue": tgt_pks,
                "sql": format!("-- Primary key differs: source({}) vs target({}) — manual intervention required", src_pk_str, tgt_pk_str)
            }));
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "source_tables": src_tables_result.unwrap_or(serde_json::json!({})),
        "target_tables": tgt_tables_result.unwrap_or(serde_json::json!({})),
        "diffs": diffs
    }))
}

/// Group SHOW INDEX rows into { name, unique, columns } objects
/// MySQL SHOW INDEX returns: Key_name, Column_name, Non_unique, Seq_in_index
fn group_indexes(
    indexes: &[serde_json::Value],
) -> std::collections::HashMap<String, serde_json::Value> {
    let mut map: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for idx in indexes {
        let name = idx
            .get("Key_name")
            .or_else(|| idx.get("index_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        map.entry(name).or_default().push(idx.clone());
    }

    let mut result = std::collections::HashMap::new();
    for (name, rows) in map {
        let key = name.to_lowercase();
        if rows.is_empty() {
            continue;
        }
        let first = &rows[0];
        let non_unique = first
            .get("Non_unique")
            .or_else(|| first.get("is_unique"))
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        let is_unique = non_unique == 0;
        // Collect columns sorted by Seq_in_index
        let mut col_rows: Vec<(i64, String)> = rows
            .iter()
            .filter_map(|r| {
                let seq = r.get("Seq_in_index").and_then(|v| v.as_i64()).unwrap_or(1);
                let col = r
                    .get("Column_name")
                    .or_else(|| r.get("column_names"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Some((seq, col))
            })
            .collect();
        col_rows.sort_by_key(|(seq, _)| *seq);
        let columns: Vec<String> = col_rows.into_iter().map(|(_, c)| c).collect();
        let col_str = columns.join(",");

        result.insert(
            key,
            serde_json::json!({
                "name": name,
                "unique": is_unique,
                "columns": col_str
            }),
        );
    }
    result
}

/// Extract primary key column names from information_schema columns (COLUMN_KEY = 'PRI')
fn extract_primary_keys(cols: &[serde_json::Value]) -> Vec<String> {
    let mut pks: Vec<(i64, String)> = cols
        .iter()
        .filter_map(|c| {
            let key = c.get("COLUMN_KEY").and_then(|v| v.as_str()).unwrap_or("");
            if key == "PRI" {
                let name = c
                    .get("COLUMN_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pos = c
                    .get("ORDINAL_POSITION")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                Some((pos, name))
            } else {
                None
            }
        })
        .collect();
    pks.sort_by_key(|(pos, _)| *pos);
    pks.into_iter().map(|(_, name)| name).collect()
}

/// Generate CREATE TABLE SQL from information_schema column data
fn generate_create_table_sql(table: &str, cols: &[serde_json::Value]) -> String {
    generate_create_table_sql_with_type(table, cols, "mysql")
}

fn generate_create_table_sql_with_type(
    table: &str,
    cols: &[serde_json::Value],
    db_type: &str,
) -> String {
    let mut col_defs: Vec<String> = Vec::new();
    let mut primary_keys: Vec<String> = Vec::new();

    for col in cols {
        let name = col
            .get("COLUMN_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let col_type = col
            .get("COLUMN_TYPE")
            .and_then(|v| v.as_str())
            .unwrap_or("TEXT");
        let nullable = col
            .get("IS_NULLABLE")
            .and_then(|v| v.as_str())
            .unwrap_or("YES");
        let default = col.get("COLUMN_DEFAULT");
        let extra = col.get("EXTRA").and_then(|v| v.as_str()).unwrap_or("");
        let comment = col
            .get("COLUMN_COMMENT")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let key = col.get("COLUMN_KEY").and_then(|v| v.as_str()).unwrap_or("");

        let mut def = format!("  {} {}", quote_ident(name, db_type), col_type);
        if nullable == "NO" {
            def.push_str(" NOT NULL");
        }
        if let Some(d) = default {
            if !d.is_null() {
                def.push_str(&format!(" DEFAULT {}", d));
            }
        }
        if !extra.is_empty() {
            def.push_str(&format!(" {}", extra));
        }
        if db_type == "mysql" && !comment.is_empty() {
            def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        col_defs.push(def);

        if key == "PRI" {
            primary_keys.push(quote_ident(name, db_type));
        }
    }

    if !primary_keys.is_empty() {
        col_defs.push(format!("  PRIMARY KEY ({})", primary_keys.join(", ")));
    }

    let mut sql = format!(
        "CREATE TABLE {} (\n{}\n);",
        quote_ident(table, db_type),
        col_defs.join(",\n")
    );

    // PostgreSQL: add COMMENT ON statements separately
    if db_type == "postgresql" {
        let table_ref = quote_ident(table, db_type);
        let comments: Vec<String> = cols
            .iter()
            .filter_map(|c| {
                let name = c.get("COLUMN_NAME").and_then(|v| v.as_str())?;
                let comment = c.get("COLUMN_COMMENT").and_then(|v| v.as_str())?;
                if comment.is_empty() {
                    return None;
                }
                Some(format!(
                    "COMMENT ON COLUMN {}.{} IS '{}';",
                    table_ref,
                    quote_ident(name, db_type),
                    comment.replace('\'', "''")
                ))
            })
            .collect();
        if !comments.is_empty() {
            sql.push_str("\n");
            sql.push_str(&comments.join("\n"));
        }
    }

    sql
}

/// Build a column definition string from information_schema column info
fn build_col_def_from_info(col: &serde_json::Value, db_type: &str) -> String {
    let name = col
        .get("COLUMN_NAME")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let col_type = col
        .get("COLUMN_TYPE")
        .and_then(|v| v.as_str())
        .unwrap_or("TEXT");
    let nullable = col
        .get("IS_NULLABLE")
        .and_then(|v| v.as_str())
        .unwrap_or("YES");
    let default = col.get("COLUMN_DEFAULT");
    let extra = col.get("EXTRA").and_then(|v| v.as_str()).unwrap_or("");
    let comment = col
        .get("COLUMN_COMMENT")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut def = format!("{} {}", quote_ident(name, db_type), col_type);
    if nullable == "NO" {
        def.push_str(" NOT NULL");
    }
    if let Some(d) = default {
        if !d.is_null() {
            def.push_str(&format!(" DEFAULT {}", d));
        }
    }
    if !extra.is_empty() {
        def.push_str(&format!(" {}", extra));
    }
    if db_type == "mysql" && !comment.is_empty() {
        def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
    }
    def
}

#[allow(non_snake_case)]
#[tauri::command(rename_all = "camelCase")]
pub async fn db_execute_structure_sync(
    id: String,
    sqls: Vec<String>,
    targetDbName: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] db_execute_structure_sync() called, {} statements",
        sqls.len()
    );
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };

    let mut executed = 0u64;
    let mut errors: Vec<String> = vec![];

    // B3 修复：MySQL/SQLite 用同一物理连接执行事务；Postgres 单连接客户端本身即真事务
    if let DbConnection::MySql(p) = &conn {
        let (executed, errors) = execute_mysql_transaction(p, &sqls, if targetDbName.is_empty() { None } else { Some(targetDbName.as_str()) }).await?;
        return Ok(serde_json::json!({ "success": errors.is_empty(), "executed": executed, "errors": errors }));
    }
    if let DbConnection::Sqlite(cfg) = &conn {
        let (executed, errors) = execute_sqlite_transaction(cfg, &sqls).await?;
        return Ok(serde_json::json!({ "success": errors.is_empty(), "executed": executed, "errors": errors }));
    }

    // PostgreSQL: BEGIN/中间/COMMIT 在同一 client 连接上
    let begin_result = match &conn {
        DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), "BEGIN").await,
        _ => Err("Unsupported database type".to_string()),
    };
    if let Err(e) = begin_result {
        return Ok(
            serde_json::json!({ "success": false, "executed": 0, "errors": vec![format!("Failed to start transaction: {}", e)] }),
        );
    }

    for sql in &sqls {
        let trimmed = sql.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        let r = match &conn {
            DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), trimmed).await,
            _ => Err("Unsupported database type".to_string()),
        };

        match r {
            Ok(_) => executed += 1,
            Err(e) => {
                errors.push(format!(
                    "Error executing \"{}\": {}",
                    trimmed.chars().take(100).collect::<String>(),
                    e
                ));
                let _ = match &conn {
                    DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), "ROLLBACK").await,
                    _ => Err("Unsupported".to_string()),
                };
                return Ok(
                    serde_json::json!({ "success": false, "executed": executed, "errors": errors }),
                );
            }
        }
    }

    if let Err(e) = match &conn {
        DbConnection::Postgres(c) => execute_postgres_query(c.as_ref(), "COMMIT").await,
        _ => Err("Unsupported".to_string()),
    } {
        errors.push(format!("COMMIT failed: {}", e));
        return Ok(serde_json::json!({ "success": false, "executed": executed, "errors": errors }));
    }

    Ok(serde_json::json!({ "success": true, "executed": executed, "errors": errors }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_compare_data(
    source_id: String,
    target_id: String,
    table_name: String,
    primary_keys: Vec<String>,
    source_db: String,
    target_db: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] db_compare_data() called for table: {}",
        table_name
    );

    // Fetch all rows from source and target
    let src_result = db_get_table_data(
        source_id,
        source_db,
        table_name.clone(),
        100000,
        0,
        None,
        None,
    )
    .await;
    let tgt_result = db_get_table_data(
        target_id,
        target_db,
        table_name.clone(),
        100000,
        0,
        None,
        None,
    )
    .await;

    let src_rows: Vec<serde_json::Value> = match &src_result {
        Ok(v) => v
            .get("rows")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default(),
        Err(e) => return Err(format!("Source query failed: {}", e)),
    };
    let tgt_rows: Vec<serde_json::Value> = match &tgt_result {
        Ok(v) => v
            .get("rows")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default(),
        Err(e) => return Err(format!("Target query failed: {}", e)),
    };

    // Build maps keyed by primary key values
    let mut src_map: std::collections::HashMap<String, &serde_json::Value> =
        std::collections::HashMap::new();
    let mut tgt_map: std::collections::HashMap<String, &serde_json::Value> =
        std::collections::HashMap::new();

    for row in &src_rows {
        let key = row_key(row, &primary_keys);
        src_map.insert(key, row);
    }
    for row in &tgt_rows {
        let key = row_key(row, &primary_keys);
        tgt_map.insert(key, row);
    }

    let mut diffs: Vec<serde_json::Value> = vec![];
    let mut total_inserts = 0;
    let mut total_updates = 0;
    let mut total_deletes = 0;

    // Find inserts and updates
    for (key, src_row) in &src_map {
        if !tgt_map.contains_key(key) {
            let pk = extract_pk(src_row, &primary_keys);
            diffs.push(serde_json::json!({
                "diffType": "insert",
                "primaryKey": pk,
                "sourceRow": src_row
            }));
            total_inserts += 1;
        } else {
            let tgt_row = tgt_map[key];
            if rows_differ(src_row, tgt_row, &primary_keys) {
                let pk = extract_pk(src_row, &primary_keys);
                diffs.push(serde_json::json!({
                    "diffType": "update",
                    "primaryKey": pk,
                    "sourceRow": src_row,
                    "targetRow": tgt_row
                }));
                total_updates += 1;
            }
        }
    }

    // Find deletes
    for (key, tgt_row) in &tgt_map {
        if !src_map.contains_key(key) {
            let pk = extract_pk(tgt_row, &primary_keys);
            diffs.push(serde_json::json!({
                "diffType": "delete",
                "primaryKey": pk,
                "targetRow": tgt_row
            }));
            total_deletes += 1;
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "diffs": diffs,
        "totalInserts": total_inserts,
        "totalUpdates": total_updates,
        "totalDeletes": total_deletes
    }))
}

/// Build a row key from primary key values (case-insensitive PK lookup)
fn row_key(row: &serde_json::Value, pks: &[String]) -> String {
    // Build lowercase -> actual key mapping for case-insensitive lookup
    let row_map: std::collections::HashMap<String, &serde_json::Value> = match row.as_object() {
        Some(obj) => obj.iter().map(|(k, v)| (k.to_lowercase(), v)).collect(),
        None => return "<UNDEF_ROW>".to_string(),
    };
    let parts: Vec<String> = pks
        .iter()
        .map(|pk| match row_map.get(&pk.to_lowercase()) {
            None => "<UNDEF>".to_string(),
            Some(v) if v.is_null() => "<NULL>".to_string(),
            Some(v) => v.to_string(),
        })
        .collect();
    parts.join("|||")
}

/// Extract primary key values from a row (case-insensitive)
fn extract_pk(row: &serde_json::Value, pks: &[String]) -> serde_json::Value {
    let row_map: std::collections::HashMap<String, &serde_json::Value> = match row.as_object() {
        Some(obj) => obj.iter().map(|(k, v)| (k.to_lowercase(), v)).collect(),
        None => return serde_json::Value::Null,
    };
    let mut pk = serde_json::Map::new();
    for p in pks {
        if let Some(v) = row_map.get(&p.to_lowercase()) {
            pk.insert(p.clone(), (*v).clone());
        }
    }
    serde_json::Value::Object(pk)
}

/// Check if two rows differ (excluding primary key columns, case-insensitive column matching)
fn rows_differ(src: &serde_json::Value, tgt: &serde_json::Value, pks: &[String]) -> bool {
    let src_obj = match src.as_object() {
        Some(o) => o,
        None => return true,
    };
    let tgt_obj = match tgt.as_object() {
        Some(o) => o,
        None => return true,
    };

    // Build lowercase -> value maps for case-insensitive lookup
    let src_map: std::collections::HashMap<String, &serde_json::Value> =
        src_obj.iter().map(|(k, v)| (k.to_lowercase(), v)).collect();
    let tgt_map: std::collections::HashMap<String, &serde_json::Value> =
        tgt_obj.iter().map(|(k, v)| (k.to_lowercase(), v)).collect();
    let pk_lower: std::collections::HashSet<String> =
        pks.iter().map(|p| p.to_lowercase()).collect();

    for (col_lower, src_val) in &src_map {
        if pk_lower.contains(col_lower) {
            continue;
        }
        let tgt_val = match tgt_map.get(col_lower) {
            Some(v) => v,
            None => return true,
        };
        if src_val.is_null() && tgt_val.is_null() {
            continue;
        }
        if src_val.is_null() || tgt_val.is_null() {
            return true;
        }
        if src_val.is_object() || tgt_val.is_object() || src_val.is_array() || tgt_val.is_array() {
            if serde_json::to_string(src_val).unwrap_or_default()
                != serde_json::to_string(tgt_val).unwrap_or_default()
            {
                return true;
            }
        } else if src_val.to_string() != tgt_val.to_string() {
            return true;
        }
    }
    false
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_execute_data_sync(params: serde_json::Value) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] db_execute_data_sync() called");

    let target_id = params
        .get("targetConnectionId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let table_name = params
        .get("tableName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let db_name = params
        .get("targetDbName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let primary_keys: Vec<String> = params
        .get("primaryKeys")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let diffs = params
        .get("diffs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let use_transaction = params
        .get("useTransaction")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(target_id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };

    let mut inserted = 0u64;
    let mut updated = 0u64;
    let mut deleted = 0u64;
    let mut errors: Vec<String> = vec![];

    // 先收集语句（保留 diffType 用于计数），再由各数据库类型在同一连接上执行事务（B3 修复）
    let mut statements: Vec<(String, String)> = vec![];
    for diff in &diffs {
        let diff_type = diff.get("diffType").and_then(|v| v.as_str()).unwrap_or("");
        let source_row = diff.get("sourceRow");
        let pk = diff.get("primaryKey");
        let sql = match diff_type {
            "insert" => source_row.map(|row| generate_insert_sql(table_name, row, &conn)),
            "update" => source_row.map(|row| generate_update_sql(table_name, row, &primary_keys, &conn)),
            "delete" => pk.map(|pk_val| generate_delete_sql(table_name, pk_val, &primary_keys, &conn)),
            _ => None,
        };
        match sql {
            Some(s) => statements.push((diff_type.to_string(), s)),
            None => errors.push(format!("{}: missing row/primaryKey", diff_type)),
        }
    }

    // 各数据库类型在事务连接上执行（MySQL/SQLite 用同一物理连接，PG 用同一 client）
    let mut count = |t: &str| match t {
        "insert" => inserted += 1,
        "update" => updated += 1,
        "delete" => deleted += 1,
        _ => {}
    };
    match &conn {
        DbConnection::MySql(p) => {
            let mut c = p.get_conn().await.map_err(|e| e.to_string())?;
            if !db_name.is_empty() {
                let _ = c.query_drop(format!("USE `{}`", db_name.replace('`', "``"))).await;
            }
            if use_transaction {
                c.query_drop("BEGIN").await.map_err(|e| format!("BEGIN failed: {}", e))?;
            }
            for (t, sql) in &statements {
                if let Err(e) = c.query_drop(sql).await {
                    errors.push(format!("{}: {}", t, e));
                    if use_transaction { let _ = c.query_drop("ROLLBACK").await; break; }
                } else {
                    count(t);
                }
            }
            if use_transaction && errors.is_empty() {
                let _ = c.query_drop("COMMIT").await;
            }
        }
        DbConnection::Sqlite(cfg) => {
            let db_path = cfg.path.as_deref().ok_or("SQLite database path required")?.to_string();
            let statements = statements.clone();
            let use_transaction = use_transaction;
            let (n_i, n_u, n_d, errs) = tokio::task::spawn_blocking(move || {
                let mut c = rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                    .map_err(|e| format!("SQLite open failed: {}", e))?;
                let mut n_i = 0u64; let mut n_u = 0u64; let mut n_d = 0u64; let mut errs: Vec<String> = vec![];
                if use_transaction {
                    let tx = c.transaction().map_err(|e| format!("SQLite BEGIN failed: {}", e))?;
                    for (t, sql) in &statements {
                        if let Err(e) = tx.execute_batch(sql) {
                            errs.push(format!("{}: {}", t, e));
                            return Ok::<_, String>((n_i, n_u, n_d, errs)); // tx drop 自动回滚
                        }
                        match t.as_str() { "insert" => n_i += 1, "update" => n_u += 1, "delete" => n_d += 1, _ => {} }
                    }
                    tx.commit().map_err(|e| format!("SQLite COMMIT failed: {}", e))?;
                } else {
                    for (t, sql) in &statements {
                        if let Err(e) = c.execute_batch(sql) {
                            errs.push(format!("{}: {}", t, e));
                            break;
                        }
                        match t.as_str() { "insert" => n_i += 1, "update" => n_u += 1, "delete" => n_d += 1, _ => {} }
                    }
                }
                Ok((n_i, n_u, n_d, errs))
            })
            .await
            .map_err(|e| e.to_string())??;
            inserted += n_i; updated += n_u; deleted += n_d; errors.extend(errs);
        }
        DbConnection::Postgres(c) => {
            if use_transaction {
                let _ = execute_postgres_query(c.as_ref(), "BEGIN").await;
            }
            for (t, sql) in &statements {
                let r = execute_postgres_query(c.as_ref(), sql).await;
                match r {
                    Ok(_) => count(t),
                    Err(e) => {
                        errors.push(format!("{}: {}", t, e));
                        if use_transaction { let _ = execute_postgres_query(c.as_ref(), "ROLLBACK").await; break; }
                    }
                }
            }
            if use_transaction && errors.is_empty() {
                let _ = execute_postgres_query(c.as_ref(), "COMMIT").await;
            }
        }
        _ => return Err("Unsupported DB type".to_string()),
    }

    Ok(serde_json::json!({
        "success": errors.is_empty(),
        "inserted": inserted,
        "updated": updated,
        "deleted": deleted,
        "errors": errors
    }))
}

fn generate_insert_sql(table: &str, row: &serde_json::Value, conn: &DbConnection) -> String {
    let db_type = match &conn {
        DbConnection::MySql(_) => "mysql",
        DbConnection::Postgres(_) => "postgresql",
        DbConnection::Sqlite(_) => "sqlite",
        _ => "mysql",
    };
    let obj = match row.as_object() {
        Some(o) => o,
        None => return format!("-- Invalid row for INSERT"),
    };
    let columns: Vec<String> = obj.keys().cloned().collect();
    let values: Vec<String> = columns.iter().map(|c| escape_sql_value(&obj[c])).collect();
    let col_list: Vec<String> = columns.iter().map(|c| quote_ident(c, db_type)).collect();

    let keyword = match db_type {
        "mysql" => "INSERT IGNORE",
        "sqlite" => "INSERT OR IGNORE",
        "postgresql" => "INSERT",
        _ => "INSERT",
    };
    let table_ref = quote_ident(table, db_type);

    let mut sql = format!(
        "{} INTO {} ({}) VALUES ({});",
        keyword,
        table_ref,
        col_list.join(", "),
        values.join(", ")
    );
    if db_type == "postgresql" {
        sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT DO NOTHING;",
            table_ref,
            col_list.join(", "),
            values.join(", ")
        );
    }
    sql
}

fn generate_update_sql(
    table: &str,
    row: &serde_json::Value,
    pks: &[String],
    conn: &DbConnection,
) -> String {
    let db_type = match &conn {
        DbConnection::MySql(_) => "mysql",
        DbConnection::Postgres(_) => "postgresql",
        DbConnection::Sqlite(_) => "sqlite",
        _ => "mysql",
    };
    let obj = match row.as_object() {
        Some(o) => o,
        None => return "-- Invalid row for UPDATE".to_string(),
    };
    let non_pk_cols: Vec<&String> = obj.keys().filter(|k| !pks.contains(*k)).collect();
    if non_pk_cols.is_empty() {
        return "-- No non-PK columns to update".to_string();
    }

    let set_clause: Vec<String> = non_pk_cols
        .iter()
        .map(|c| {
            format!(
                "{} = {}",
                quote_ident(c, db_type),
                escape_sql_value(&obj[*c])
            )
        })
        .collect();

    let where_clause: Vec<String> = pks
        .iter()
        .map(|pk| match obj.get(pk) {
            None | Some(serde_json::Value::Null) => format!("{} IS NULL", quote_ident(pk, db_type)),
            Some(v) => format!("{} = {}", quote_ident(pk, db_type), escape_sql_value(v)),
        })
        .collect();

    format!(
        "UPDATE {} SET {} WHERE {};",
        quote_ident(table, db_type),
        set_clause.join(", "),
        where_clause.join(" AND ")
    )
}

fn generate_delete_sql(
    table: &str,
    pk_val: &serde_json::Value,
    pks: &[String],
    conn: &DbConnection,
) -> String {
    let db_type = match &conn {
        DbConnection::MySql(_) => "mysql",
        DbConnection::Postgres(_) => "postgresql",
        DbConnection::Sqlite(_) => "sqlite",
        _ => "mysql",
    };
    let where_clause: Vec<String> = pks
        .iter()
        .map(|pk| match pk_val.get(pk) {
            None | Some(serde_json::Value::Null) => format!("{} IS NULL", quote_ident(pk, db_type)),
            Some(v) => format!("{} = {}", quote_ident(pk, db_type), escape_sql_value(v)),
        })
        .collect();

    format!(
        "DELETE FROM {} WHERE {};",
        quote_ident(table, db_type),
        where_clause.join(" AND ")
    )
}

fn escape_sql_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => {
            if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('\'', "''")
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            format!("'{}'", escaped)
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            let json = serde_json::to_string(val).unwrap_or_default();
            let escaped = json.replace('\\', "\\\\").replace('\'', "''");
            format!("'{}'", escaped)
        }
    }
}

// Backup
#[tauri::command(rename_all = "camelCase")]
pub async fn db_backup_create(
    id: String,
    db_name: String,
    objects: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // Export data to JSON
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    let mut backup = serde_json::Map::new();
    for obj in &objects {
        if let Some(table) = obj.get("table").and_then(|v| v.as_str()) {
            let data = match &conn {
                DbConnection::MySql(p) => {
                    execute_mysql_query(p, &format!("SELECT * FROM `{}`.`{}`", db_name, table))
                        .await
                }
                DbConnection::Postgres(c) => {
                    execute_postgres_query(
                        c,
                        &format!("SELECT * FROM \"{}\".\"{}\"", db_name, table),
                    )
                    .await
                }
                DbConnection::Sqlite(cfg) => {
                    execute_sqlite_query(cfg, &format!("SELECT * FROM \"{}\"", table)).await
                }
                _ => Err("Unsupported".to_string()),
            };
            backup.insert(table.to_string(), data.unwrap_or(serde_json::Value::Null));
        }
    }
    Ok(serde_json::json!({ "success": true, "data": backup }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_backup_list(_id: Option<String>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "success": true, "backups": [] }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_backup_restore(id: String, file: String) -> Result<serde_json::Value, String> {
    log::info!(
        "[db_backup_restore] Restoring backup for connection '{}' from file '{}'",
        id,
        file
    );

    // 1. Read file as binary (nb3 = gzipped JSON)
    let compressed = std::fs::read(&file).map_err(|e| format!("读取备份文件失败: {}", e))?;
    if compressed.is_empty() {
        return Err("备份文件为空（0 字节）".to_string());
    }

    // 2. Decompress gzip
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .map_err(|e| format!("解压备份文件失败（非有效的 .nb3 格式）: {}", e))?;

    // 3. Parse JSON payload
    let payload: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("解析备份文件失败: {}", e))?;

    let files = payload
        .get("files")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "备份文件格式错误：缺少 files 字段".to_string())?;
    let metadata = payload.get("metadata");

    // 4. Get connection from pool
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id)
            .cloned()
            .ok_or_else(|| format!("连接 '{}' 未找到，请先连接数据库", id))?
    };

    // Log restore info
    if let Some(meta) = metadata {
        let db_name = meta
            .get("databaseName")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let obj_count = meta
            .get("objects")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        log::info!(
            "[db_backup_restore] Restoring to database '{}', {} objects, {} SQL files",
            db_name,
            obj_count,
            files.len()
        );
    }

    // 5. Execute all SQL statements against the database connection
    let mut executed = 0i64;
    let mut errors: Vec<String> = Vec::new();

    for (filename, sql_content) in files.iter() {
        let sql = match sql_content.as_str() {
            Some(s) => s,
            None => {
                errors.push(format!("{}: 非字符串内容", filename));
                continue;
            }
        };
        let trimmed = sql.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
            continue; // Skip comments/empty
        }

        match &conn {
            DbConnection::MySql(p) => match p.get_conn().await {
                Ok(mut mysql_conn) => {
                    if let Err(e) = mysql_conn.query_drop(trimmed).await {
                        errors.push(format!("{}: MySQL 执行失败: {}", filename, e));
                    } else {
                        executed += 1;
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: MySQL 连接失败: {}", filename, e));
                }
            },
            DbConnection::Postgres(c) => match c.batch_execute(trimmed).await {
                Ok(_) => executed += 1,
                Err(e) => errors.push(format!("{}: PostgreSQL 执行失败: {}", filename, e)),
            },
            DbConnection::Sqlite(cfg) => {
                let db_path = match cfg.path.as_deref() {
                    Some(p) => p.to_string(),
                    None => {
                        errors.push(format!("{}: SQLite 数据库路径缺失", filename));
                        continue;
                    }
                };
                let sql_owned = trimmed.to_string();
                match tokio::task::spawn_blocking(move || {
                    let conn = rusqlite::Connection::open(&db_path)
                        .map_err(|e| format!("SQLite 打开失败: {}", e))?;
                    conn.execute_batch(&sql_owned)
                        .map_err(|e| format!("SQLite 执行失败: {}", e))?;
                    Ok::<_, String>(())
                })
                .await
                {
                    Ok(Ok(_)) => executed += 1,
                    Ok(Err(e)) => errors.push(format!("{}: {}", filename, e)),
                    Err(e) => errors.push(format!("{}: SQLite 任务失败: {}", filename, e)),
                }
            }
            DbConnection::Redis(_) => {
                return Err("Redis 不支持备份恢复操作".to_string());
            }
            DbConnection::Elasticsearch(_) => {
                return Err("Elasticsearch 不支持备份恢复操作".to_string());
            }
        }
    }

    let success = errors.is_empty();
    if !success {
        log::warn!(
            "[db_backup_restore] Restore completed with {} errors: {}",
            errors.len(),
            errors.join("; ")
        );
    }
    log::info!(
        "[db_backup_restore] Restore complete: executed={}, errors={}",
        executed,
        errors.len()
    );

    Ok(serde_json::json!({
        "success": success,
        "executed": executed,
        "errors": errors,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_backup_delete(file: String) -> Result<serde_json::Value, String> {
    std::fs::remove_file(&file).map_err(|e| format!("Failed to delete: {}", e))?;
    Ok(serde_json::json!({ "success": true }))
}

// Redis operations
#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_databases(id: String) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    if let DbConnection::Redis(c) = conn {
        // Query INFO keyspace to get per-db key counts
        let info: String = redis::cmd("INFO")
            .arg("keyspace")
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis INFO failed: {}", e))?;

        // Parse lines like: db0:keys=5,expires=0,avg_ttl=0
        let mut databases: Vec<serde_json::Value> = Vec::new();
        for line in info.lines() {
            let line = line.trim();
            if !line.starts_with("db") {
                continue;
            }
            // Format: dbN:keys=K,...
            if let Some(colon_idx) = line.find(':') {
                let db_part = &line[..colon_idx]; // "db0"
                let rest = &line[colon_idx + 1..]; // "keys=5,expires=0,avg_ttl=0"
                if let Some(db_num_str) = db_part.strip_prefix("db") {
                    if let Ok(db_num) = db_num_str.parse::<i64>() {
                        let keys = rest
                            .split(',')
                            .find_map(|kv| {
                                let mut parts = kv.splitn(2, '=');
                                if parts.next() == Some("keys") {
                                    parts.next().and_then(|v| v.parse::<i64>().ok())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0);
                        databases.push(serde_json::json!({ "db": db_num, "keys": keys }));
                    }
                }
            }
        }

        // If no databases found in INFO (empty Redis), default to db0
        if databases.is_empty() {
            databases.push(serde_json::json!({ "db": 0, "keys": 0 }));
        }

        Ok(serde_json::json!({ "success": true, "databases": databases }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_keys(
    id: String,
    db_index: i64,
    pattern: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        // S9: KEYS → SCAN 迭代，避免大 key 空间阻塞 Redis
        let keys = redis_scan_all_keys(&c, &pattern, 100_000).await?;
        Ok(serde_json::json!({ "success": true, "keys": keys }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_keys_tree(
    id: String,
    db_index: i64,
    pattern: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    let DbConnection::Redis(c) = conn else {
        return Err("Not a Redis connection".to_string());
    };

    // 前端传的是「前缀」（如 `models:`），`*`/空表示整库。SCAN 必须补尾部通配符：
    // 之前把 `models:` 原样当 pattern，匹配不到任何 key → 二级目录展不开、计数恒为 0。
    let prefix = if pattern == "*" || pattern.is_empty() {
        String::new()
    } else {
        pattern.clone()
    };
    let match_pattern = if prefix.is_empty() {
        "*".to_string()
    } else {
        format!("{}*", prefix)
    };

    let keys = redis_scan_all_keys(&c, &match_pattern, 100_000).await?;

    // 按前缀之后的「下一段」分组：还带 `:` 的算文件夹（count = 该前缀下 key 总数），
    // 其余是当前层叶子。分组在服务端做，前端只需按前缀拼接完整 key。
    let mut folders: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut leaves: Vec<String> = Vec::new();
    for k in &keys {
        let rest = k.strip_prefix(prefix.as_str()).unwrap_or(k.as_str());
        match rest.find(':') {
            Some(i) if i > 0 => *folders.entry(rest[..i].to_string()).or_insert(0) += 1,
            _ => leaves.push(rest.to_string()),
        }
    }

    // 叶子补真实类型（数量可控才逐个 TYPE，避免大 key 空间下 N 次往返）
    let probe_types = leaves.len() <= 300;
    let mut leaf_objs: Vec<serde_json::Value> = Vec::with_capacity(leaves.len());
    for name in &leaves {
        let mut key_type = "string".to_string();
        if probe_types {
            let full = format!("{}{}", prefix, name);
            if let Ok(Some(t)) = redis::cmd("TYPE")
                .arg(&full)
                .query_async::<Option<String>>(&mut c.clone())
                .await
            {
                key_type = t;
            }
        }
        leaf_objs.push(serde_json::json!({ "name": name, "type": key_type }));
    }

    let folder_objs: Vec<serde_json::Value> = folders
        .into_iter()
        .map(|(name, count)| serde_json::json!({ "name": name, "count": count }))
        .collect();

    Ok(serde_json::json!({
        "success": true,
        "folders": folder_objs,
        "leaves": leaf_objs,
        "hasMore": keys.len() >= 100_000,
        "scanned": keys.len(),
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_keys_by_type(
    id: String,
    db_index: i64,
    pattern: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let match_pattern = if pattern.is_empty() || pattern == "*" { "*" } else { &pattern };
        let all_keys = redis_scan_all_keys(&c, match_pattern, 100_000).await?;

        // 按类型分组：TYPE 用 pipeline 分批，避免逐键往返（5k 键曾耗时 60s+）
        let mut keys_by_type: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for chunk in all_keys.chunks(500) {
            let mut pipe = redis::pipe();
            for key in chunk {
                pipe.cmd("TYPE").arg(key);
            }
            let types: Vec<String> = pipe
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis TYPE pipeline failed: {}", e))?;
            for (key, ktype) in chunk.iter().zip(types.into_iter()) {
                keys_by_type.entry(ktype).or_default().push(key.clone());
            }
        }
        Ok(serde_json::json!({ "success": true, "keysByType": keys_by_type }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_key_info(
    id: String,
    db_index: i64,
    key: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let key_type: String = redis::cmd("TYPE")
            .arg(&key)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis TYPE failed: {}", e))?;
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis TTL failed: {}", e))?;
        // Compute length based on type
        let length: i64 = match key_type.as_str() {
            "string" => redis::cmd("STRLEN").arg(&key).query_async(&mut c.clone()).await,
            "hash" => redis::cmd("HLEN").arg(&key).query_async(&mut c.clone()).await,
            "list" => redis::cmd("LLEN").arg(&key).query_async(&mut c.clone()).await,
            "set" => redis::cmd("SCARD").arg(&key).query_async(&mut c.clone()).await,
            "zset" => redis::cmd("ZCARD").arg(&key).query_async(&mut c.clone()).await,
            "stream" => redis::cmd("XLEN").arg(&key).query_async(&mut c.clone()).await,
            _ => Ok(0i64),
        }
        .map_err(|e| format!("Redis length command failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "type": key_type, "ttl": ttl, "length": length }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

/// 按字节读 Redis 值并做 UTF-8 容错解码。
/// Redis 值是二进制安全的：之前直接反序列化成 String/HashMap<String,String>，
/// 遇到序列化 blob（protobuf、Java 序列化等）会报「Cannot convert from UTF-8」，
/// 表现为「获取键值失败」。现在改成 lossy 解码并告诉前端该值不是合法 UTF-8。
async fn redis_raw_value(c: &RedisConn, key: &str, key_type: &str) -> Result<(serde_json::Value, bool), String> {
    let mut binary = false;
    let mut dec = |b: Vec<u8>| match String::from_utf8(b) {
        Ok(s) => s,
        Err(e) => {
            binary = true;
            String::from_utf8_lossy(e.as_bytes()).into_owned()
        }
    };
    let val = match key_type {
        "string" => {
            // Option：键可能在 TYPE 与 GET 之间过期，直接解 String 会报错
            let v: Option<Vec<u8>> = redis::cmd("GET")
                .arg(key)
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?;
            match v {
                Some(b) => serde_json::Value::String(dec(b)),
                None => serde_json::Value::Null,
            }
        }
        "hash" => {
            let flat: Vec<Vec<u8>> = redis::cmd("HGETALL")
                .arg(key)
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?;
            let mut obj = serde_json::Map::new();
            for pair in flat.chunks(2) {
                let f = dec(pair[0].clone());
                let v = pair.get(1).map(|b| dec(b.clone())).unwrap_or_default();
                obj.insert(f, serde_json::Value::String(v));
            }
            serde_json::Value::Object(obj)
        }
        "list" => {
            let v: Vec<Vec<u8>> = redis::cmd("LRANGE")
                .arg(key)
                .arg(0)
                .arg(-1)
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?;
            serde_json::Value::Array(v.into_iter().map(|b| serde_json::Value::String(dec(b))).collect())
        }
        "set" => {
            let v: Vec<Vec<u8>> = redis::cmd("SMEMBERS")
                .arg(key)
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?;
            serde_json::Value::Array(v.into_iter().map(|b| serde_json::Value::String(dec(b))).collect())
        }
        "zset" => {
            let v: Vec<Vec<u8>> = redis::cmd("ZRANGE")
                .arg(key)
                .arg(0)
                .arg(-1)
                .arg("WITHSCORES")
                .query_async(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?;
            serde_json::Value::Array(v.into_iter().map(|b| serde_json::Value::String(dec(b))).collect())
        }
        _ => serde_json::Value::Null,
    };
    Ok((val, binary))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_key_value(
    id: String,
    db_index: i64,
    key: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let key_type: String = redis::cmd("TYPE")
            .arg(&key)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis TYPE failed: {}", e))?;
        let (val, binary) = redis_raw_value(&c, &key, &key_type).await?;
        Ok(serde_json::json!({ "success": true, "value": val, "type": key_type, "binary": binary }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_set_key(
    id: String,
    db_index: i64,
    key: String,
    value: String,
    ttl: i64,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        if ttl > 0 {
            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl)
                .arg(&value)
                .query_async::<()>(&mut c.clone())
                .await
                .map_err(|e| format!("Redis SETEX failed: {}", e))?;
        } else {
            redis::cmd("SET")
                .arg(&key)
                .arg(&value)
                .query_async::<()>(&mut c.clone())
                .await
                .map_err(|e| format!("Redis SET failed: {}", e))?;
        }
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_add_key(
    id: String,
    db_index: i64,
    key_type: String,
    key: String,
    value: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        match key_type.as_str() {
            "string" => redis::cmd("SET")
                .arg(&key)
                .arg(value.as_str().unwrap_or(""))
                .query_async::<()>(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?,
            "hash" => {
                if let Some(obj) = value.as_object() {
                    let mut cmd = redis::Cmd::new();
                    cmd.arg("HMSET").arg(&key);
                    for (k, v) in obj {
                        cmd.arg(k).arg(v.as_str().unwrap_or(""));
                    }
                    cmd.query_async::<()>(&mut c.clone())
                        .await
                        .map_err(|e| format!("Redis command failed: {}", e))?;
                }
            }
            "list" => redis::cmd("RPUSH")
                .arg(&key)
                .arg(value.as_str().unwrap_or(""))
                .query_async::<()>(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?,
            "set" => redis::cmd("SADD")
                .arg(&key)
                .arg(value.as_str().unwrap_or(""))
                .query_async::<()>(&mut c.clone())
                .await
                .map_err(|e| format!("Redis command failed: {}", e))?,
            _ => return Err(format!("Unsupported Redis type: {}", key_type)),
        }
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_delete_key(
    id: String,
    db_index: i64,
    key: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut c.clone())
            .await
            .map_err(|e| format!("Redis DEL failed: {}", e))?;
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_exec(
    id: String,
    db_index: i64,
    command: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }
        let mut cmd = redis::Cmd::new();
        cmd.arg(parts[0]);
        for p in &parts[1..] {
            cmd.arg(p);
        }
        let result: redis::Value = cmd
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis exec failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "result": redis_value_to_json(&result) }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[allow(unused_variables)]
#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_scan_keys(
    id: String,
    db_index: i64,
    pattern: String,
    type_filter: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        // S9: KEYS → SCAN 迭代，避免大 key 空间阻塞 Redis
        let keys = redis_scan_all_keys(&c, &pattern, 100_000).await?;
        Ok(serde_json::json!({ "success": true, "keys": keys }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

/// S9: 用 SCAN 迭代替代 KEYS（大 key 空间下 KEYS 会阻塞 Redis 服务端）
async fn redis_scan_all_keys(
    conn: &RedisConn,
    pattern: &str,
    max: usize,
) -> Result<Vec<String>, String> {
    // 键按字节取回：Redis 键是二进制安全的，个别非 UTF-8 键（序列化 blob、protobuf 等）
    // 若直接反序列化成 Vec<String> 会让整次 SCAN 报
    // 「Cannot convert from UTF-8」，表现为整个库展开为空。这里统一 lossy 解码。
    let mut raw: Vec<Vec<u8>> = Vec::new();
    let mut cursor: u64 = 0;
    loop {
        let (next_cursor, batch): (u64, Vec<Vec<u8>>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(1000)
            .query_async(&mut conn.clone())
            .await
            .map_err(|e| format!("Redis SCAN failed: {}", e))?;
        raw.extend(batch);
        cursor = next_cursor;
        if cursor == 0 || raw.len() >= max {
            break;
        }
    }
    raw.truncate(max);
    Ok(raw.iter().map(|k| String::from_utf8_lossy(k).into_owned()).collect())
}

// Stream commands
#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_streams(
    id: String,
    db_index: i64,
    pattern: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        // S9: KEYS → SCAN 迭代，避免大 key 空间阻塞 Redis
        let keys = redis_scan_all_keys(&c, &pattern, 100_000).await?;
        Ok(serde_json::json!({ "success": true, "streams": keys }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_info(
    id: String,
    db_index: i64,
    stream: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let info: HashMap<String, String> = redis::cmd("XINFO")
            .arg("STREAM")
            .arg(&stream)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XINFO failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "info": info }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_messages(
    id: String,
    db_index: i64,
    stream: String,
    count: i64,
    start: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let msgs: Vec<Vec<(String, Vec<(String, String)>)>> = redis::cmd("XRANGE")
            .arg(&stream)
            .arg(&start)
            .arg("+")
            .arg("COUNT")
            .arg(count)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XRANGE failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "messages": msgs }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_add(
    id: String,
    db_index: i64,
    stream: String,
    data: HashMap<String, String>,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let mut cmd = redis::Cmd::new();
        cmd.arg("XADD").arg(&stream).arg("*");
        for (k, v) in &data {
            cmd.arg(k).arg(v);
        }
        let msg_id: String = cmd
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XADD failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "message_id": msg_id }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_del(
    id: String,
    db_index: i64,
    stream: String,
) -> Result<serde_json::Value, String> {
    db_redis_stream_delete(id, db_index, stream).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_delete(
    id: String,
    db_index: i64,
    stream: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        redis::cmd("DEL")
            .arg(&stream)
            .query_async::<()>(&mut c.clone())
            .await
            .map_err(|e| format!("Redis DEL failed: {}", e))?;
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_group_create(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&stream)
            .arg(&group)
            .arg("$")
            .arg("MKSTREAM")
            .query_async::<()>(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XGROUP CREATE failed: {}", e))?;
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_group_destroy(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        redis::cmd("XGROUP")
            .arg("DESTROY")
            .arg(&stream)
            .arg(&group)
            .query_async::<()>(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XGROUP DESTROY failed: {}", e))?;
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_consumers(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let consumers: Vec<HashMap<String, String>> = redis::cmd("XINFO")
            .arg("CONSUMERS")
            .arg(&stream)
            .arg(&group)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XINFO CONSUMERS failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "consumers": consumers }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_pending(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let pending: Vec<Vec<(String, redis::Value)>> = redis::cmd("XPENDING")
            .arg(&stream)
            .arg(&group)
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XPENDING failed: {}", e))?;
        Ok(
            serde_json::json!({ "success": true, "pending": serde_json::Value::Array(pending.iter().map(|inner| serde_json::Value::Array(inner.iter().map(|(k, v)| serde_json::json!({ "key": k, "value": redis_value_to_json(v) })).collect())).collect()) }),
        )
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_claim(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
    consumer: String,
    msg_ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let mut cmd = redis::Cmd::new();
        cmd.arg("XCLAIM")
            .arg(&stream)
            .arg(&group)
            .arg(&consumer)
            .arg("0");
        for msg_id in &msg_ids {
            cmd.arg(msg_id);
        }
        let result: redis::Value = cmd
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XCLAIM failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "result": redis_value_to_json(&result) }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_ack(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
    msg_ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let mut cmd = redis::Cmd::new();
        cmd.arg("XACK").arg(&stream).arg(&group);
        for msg_id in &msg_ids {
            cmd.arg(msg_id);
        }
        let count: i64 = cmd
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XACK failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "acknowledged": count }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_retry(
    id: String,
    db_index: i64,
    stream: String,
    group: String,
    consumer: String,
    msg_ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    db_redis_stream_claim(id, db_index, stream, group, consumer, msg_ids).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_stream_trim(
    id: String,
    db_index: i64,
    stream: String,
    count: i64,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        redis::cmd("XTRIM")
            .arg(&stream)
            .arg("MAXLEN")
            .arg(count)
            .query_async::<()>(&mut c.clone())
            .await
            .map_err(|e| format!("Redis XTRIM failed: {}", e))?;
        Ok(serde_json::json!({ "success": true }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_zset_range(
    id: String,
    db_index: i64,
    key: String,
    start: i64,
    stop: i64,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let result: Vec<String> = redis::cmd("ZRANGE")
            .arg(&key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis ZRANGE failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "result": result }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_redis_zset_remove(
    id: String,
    db_index: i64,
    key: String,
    members: Vec<String>,
) -> Result<serde_json::Value, String> {
    let conn = redis_conn_for(&id, db_index).await?;
    if let DbConnection::Redis(c) = conn {
        let mut cmd = redis::Cmd::new();
        cmd.arg("ZREM").arg(&key);
        for m in &members {
            cmd.arg(m);
        }
        let removed: i64 = cmd
            .query_async(&mut c.clone())
            .await
            .map_err(|e| format!("Redis ZREM failed: {}", e))?;
        Ok(serde_json::json!({ "success": true, "removed": removed }))
    } else {
        Err("Not a Redis connection".to_string())
    }
}

// Row CRUD
#[tauri::command(rename_all = "camelCase")]
pub async fn db_insert_table_row(
    id: String,
    table_name: String,
    values_json: serde_json::Value,
    db_name: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    let obj = values_json
        .as_object()
        .ok_or("values_json must be an object")?;
    if obj.is_empty() {
        return Err("Empty values".to_string());
    }
    match &conn {
        DbConnection::MySql(p) => {
            let meta = mysql_column_meta(p, db_name.as_deref(), &table_name).await;
            let mut cols: Vec<String> = Vec::with_capacity(obj.len());
            let mut vals: Vec<String> = Vec::with_capacity(obj.len());
            for (k, v) in obj.iter() {
                cols.push(format!("`{}`", k));
                vals.push(mysql_literal(k, &meta, v)?);
            }
            let prefix = db_name
                .filter(|s| !s.is_empty())
                .map(|d| format!("`{}`.", d))
                .unwrap_or_default();
            let sql = format!(
                "INSERT INTO {}`{}` ({}) VALUES ({})",
                prefix,
                table_name,
                cols.join(", "),
                vals.join(", ")
            );
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let (cols, vals): (Vec<String>, Vec<String>) = obj
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::Null => "NULL".to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                        _ => v.to_string(),
                    };
                    (format!("\"{}\"", k), val)
                })
                .unzip();
            let sql = format!(
                "INSERT INTO \"{}\" ({}) VALUES ({})",
                table_name,
                cols.join(", "),
                vals.join(", ")
            );
            execute_sqlite_write(cfg, &sql).await
        }
        _ => Err("Only MySQL and SQLite supported for now".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_update_table_row(
    id: String,
    table_name: String,
    primary_key_json: serde_json::Value,
    values_json: serde_json::Value,
    db_name: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    let pk_obj = primary_key_json
        .as_object()
        .ok_or("primary_key_json must be an object")?;
    let val_obj = values_json
        .as_object()
        .ok_or("values_json must be an object")?;
    match &conn {
        DbConnection::MySql(p) => {
            let meta = mysql_column_meta(p, db_name.as_deref(), &table_name).await;
            let mut sets: Vec<String> = Vec::with_capacity(val_obj.len());
            for (k, v) in val_obj.iter() {
                sets.push(format!("`{}` = {}", k, mysql_literal(k, &meta, v)?));
            }
            let mut wheres: Vec<String> = Vec::with_capacity(pk_obj.len());
            for (k, v) in pk_obj.iter() {
                if matches!(v, serde_json::Value::Null) {
                    wheres.push(format!("`{}` IS NULL", k));
                } else {
                    wheres.push(format!("`{}` = {}", k, mysql_literal(k, &meta, v)?));
                }
            }
            let prefix = db_name
                .filter(|s| !s.is_empty())
                .map(|d| format!("`{}`.", d))
                .unwrap_or_default();
            let sql = format!(
                "UPDATE {}`{}` SET {} WHERE {}",
                prefix,
                table_name,
                sets.join(", "),
                wheres.join(" AND ")
            );
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let sets: Vec<String> = val_obj
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::Null => "NULL".to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                        _ => v.to_string(),
                    };
                    format!("\"{}\" = {}", k, val)
                })
                .collect();
            let wheres: Vec<String> = pk_obj
                .iter()
                .map(|(k, v)| {
                    if matches!(v, serde_json::Value::Null) {
                        format!("\"{}\" IS NULL", k)
                    } else {
                        let val = match v {
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                            _ => v.to_string(),
                        };
                        format!("\"{}\" = {}", k, val)
                    }
                })
                .collect();
            let sql = format!(
                "UPDATE \"{}\" SET {} WHERE {}",
                table_name,
                sets.join(", "),
                wheres.join(" AND ")
            );
            execute_sqlite_write(cfg, &sql).await
        }
        _ => Err("Only MySQL and SQLite supported for now".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_delete_table_row(
    id: String,
    table_name: String,
    primary_key_json: serde_json::Value,
    db_name: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    let pk_obj = primary_key_json
        .as_object()
        .ok_or("primary_key_json must be an object")?;
    match &conn {
        DbConnection::MySql(p) => {
            let wheres: Vec<String> = pk_obj
                .iter()
                .map(|(k, v)| {
                    if matches!(v, serde_json::Value::Null) {
                        format!("`{}` IS NULL", k)
                    } else {
                        let val = match v {
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                            _ => v.to_string(),
                        };
                        format!("`{}` = {}", k, val)
                    }
                })
                .collect();
            let prefix = db_name
                .filter(|s| !s.is_empty())
                .map(|d| format!("`{}`.", d))
                .unwrap_or_default();
            let sql = format!(
                "DELETE FROM {}`{}` WHERE {}",
                prefix,
                table_name,
                wheres.join(" AND ")
            );
            execute_mysql_query(p, &sql).await
        }
        DbConnection::Sqlite(cfg) => {
            let wheres: Vec<String> = pk_obj
                .iter()
                .map(|(k, v)| {
                    if matches!(v, serde_json::Value::Null) {
                        format!("\"{}\" IS NULL", k)
                    } else {
                        let val = match v {
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                            _ => v.to_string(),
                        };
                        format!("\"{}\" = {}", k, val)
                    }
                })
                .collect();
            let sql = format!(
                "DELETE FROM \"{}\" WHERE {}",
                table_name,
                wheres.join(" AND ")
            );
            execute_sqlite_write(cfg, &sql).await
        }
        _ => Err("Only MySQL and SQLite supported for now".to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn db_get_table_data_filtered(
    id: String,
    db_name: String,
    table_name: String,
    filters_json: serde_json::Value,
    limit: usize,
    offset: usize,
    sort_column: Option<String>,
    sort_dir: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = {
        let pool = CONNECTION_POOL.lock().await;
        pool.get(&id).cloned().ok_or_else(|| "Connection not found".to_string())?
    };
    match &conn {
        DbConnection::MySql(p) => {
            let where_sql = build_where_from_filters(&filters_json, &|c| format!("`{}`", c));
            let mut sql = format!("SELECT * FROM `{}`.`{}`", db_name, table_name);
            if !where_sql.is_empty() {
                sql.push_str(&format!(" WHERE {}", where_sql));
            }
            if let Some(col) = sort_column {
                if !col.is_empty() && col.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let dir = match sort_dir.as_deref().unwrap_or("").to_uppercase().as_str() {
                        "DESC" => "DESC",
                        _ => "ASC",
                    };
                    sql.push_str(&format!(" ORDER BY `{}` {}", col, dir));
                }
            }
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
            let resp = execute_mysql_query(p, &sql).await?;
            let mut total = 0u64;
            if !where_sql.is_empty() {
                if let Ok(cv) = execute_mysql_query(p, &format!("SELECT COUNT(*) AS c FROM `{}`.`{}` WHERE {}", db_name, table_name, where_sql)).await {
                    if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                        total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        DbConnection::Sqlite(cfg) => {
            let where_sql = build_where_from_filters(&filters_json, &|c| format!("\"{}\"", c));
            let mut sql = format!("SELECT * FROM \"{}\"", table_name);
            if !where_sql.is_empty() {
                sql.push_str(&format!(" WHERE {}", where_sql));
            }
            if let Some(col) = sort_column {
                if !col.is_empty() && col.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let dir = match sort_dir.as_deref().unwrap_or("").to_uppercase().as_str() {
                        "DESC" => "DESC",
                        _ => "ASC",
                    };
                    sql.push_str(&format!(" ORDER BY \"{}\" {}", col, dir));
                }
            }
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
            let resp = execute_sqlite_query(cfg, &sql).await?;
            let mut total = 0u64;
            if !where_sql.is_empty() {
                if let Ok(cv) = execute_sqlite_query(cfg, &format!("SELECT COUNT(*) AS c FROM \"{}\" WHERE {}", table_name, where_sql)).await {
                    if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                        total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        DbConnection::Postgres(c) => {
            let where_sql = build_where_from_filters(&filters_json, &|c| format!("\"{}\"", c));
            let mut sql = format!("SELECT * FROM \"{}\".\"{}\"", db_name, table_name);
            if !where_sql.is_empty() {
                sql.push_str(&format!(" WHERE {}", where_sql));
            }
            if let Some(col) = sort_column {
                if !col.is_empty() && col.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let dir = match sort_dir.as_deref().unwrap_or("").to_uppercase().as_str() {
                        "DESC" => "DESC",
                        _ => "ASC",
                    };
                    sql.push_str(&format!(" ORDER BY \"{}\" {}", col, dir));
                }
            }
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
            let resp = execute_postgres_query(c, &sql).await?;
            let mut total = 0u64;
            if !where_sql.is_empty() {
                if let Ok(cv) = execute_postgres_query(c, &format!("SELECT COUNT(*) AS c FROM \"{}\".\"{}\" WHERE {}", db_name, table_name, where_sql)).await {
                    if let Some(rows) = cv.get("rows").and_then(|v| v.as_array()) {
                        total = rows.first().and_then(|r| r.get("c")).and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }
            }
            let mut obj = match resp { serde_json::Value::Object(o) => o, other => { let mut m = serde_json::Map::new(); m.insert("rows".into(), other); m } };
            obj.insert("total".into(), serde_json::json!(total));
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err("Unsupported database type".to_string()),
    }
}

/// 从 FilterCondition[] 数组构建 WHERE 子句（B4 修复：原实现只认对象、数组被静默忽略）
/// `quote` 用于列名引用（MySQL 反引号 / PG+SQLite 双引号），operator 白名单防注入
fn build_where_from_filters(filters_json: &serde_json::Value, quote: &dyn Fn(&str) -> String) -> String {
    let mut clauses: Vec<(String, String)> = Vec::new(); // (clause, logic 连接词)
    if let Some(arr) = filters_json.as_array() {
        for cond in arr {
            let Some(obj) = cond.as_object() else { continue };
            let col = obj.get("column").and_then(|v| v.as_str()).unwrap_or("");
            if col.is_empty() { continue; }
            let op = obj.get("operator").and_then(|v| v.as_str()).unwrap_or("=").to_uppercase();
            let logic = obj.get("logic").and_then(|v| v.as_str()).unwrap_or("AND").to_uppercase();
            let getv = |k: &str| obj.get(k).and_then(|x| x.as_str()).unwrap_or("");
            let esc = |v: &str| v.replace('\'', "''");
            let q = quote(&col);
            let clause = match op.as_str() {
                "=" => Some(format!("{} = '{}'", q, esc(getv("value")))),
                "!=" => Some(format!("{} != '{}'", q, esc(getv("value")))),
                ">" => Some(format!("{} > '{}'", q, esc(getv("value")))),
                "<" => Some(format!("{} < '{}'", q, esc(getv("value")))),
                ">=" => Some(format!("{} >= '{}'", q, esc(getv("value")))),
                "<=" => Some(format!("{} <= '{}'", q, esc(getv("value")))),
                "LIKE" => Some(format!("{} LIKE '%{}%'", q, esc(getv("value")))),
                "NOT LIKE" => Some(format!("{} NOT LIKE '%{}%'", q, esc(getv("value")))),
                "IN" | "NOT IN" => {
                    let items: Vec<String> = getv("value").split(',').map(|x| format!("'{}'", esc(x.trim()))).filter(|x| x != "''").collect();
                    if items.is_empty() { None } else { Some(format!("{} {} ({})", q, op, items.join(", "))) }
                }
                "BETWEEN" => {
                    let a = getv("value"); let b = getv("value2");
                    if a.is_empty() || b.is_empty() { None } else { Some(format!("{} BETWEEN '{}' AND '{}'", q, esc(a), esc(b))) }
                }
                "IS NULL" => Some(format!("{} IS NULL", q)),
                "IS NOT NULL" => Some(format!("{} IS NOT NULL", q)),
                _ => None,
            };
            if let Some(c) = clause {
                clauses.push((c, (if logic == "OR" { " OR " } else { " AND " }).to_string()));
            }
        }
    } else if let Some(obj) = filters_json.as_object() {
        // 兼容旧对象格式 {col: value}
        for (k, v) in obj {
            let q = quote(k);
            let clause = if v.is_null() {
                format!("{} IS NULL", q)
            } else {
                let val = match v {
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    other => other.to_string(),
                };
                format!("{} = {}", q, val)
            };
            clauses.push((clause, " AND ".to_string()));
        }
    }
    if clauses.is_empty() { return String::new(); }
    let mut out = String::new();
    for (i, (c, logic)) in clauses.iter().enumerate() {
        if i > 0 { out.push_str(logic); }
        out.push_str(c);
    }
    out
}

/// B3 修复：MySQL 事务（同一物理连接执行 BEGIN/SQL/COMMIT，pool.get_conn 轮询会破坏事务）
async fn execute_mysql_transaction(
    pool: &MySqlPool,
    sqls: &[String],
    use_db: Option<&str>,
) -> Result<(u64, Vec<String>), String> {
    let mut conn = pool.get_conn().await.map_err(|e| format!("get_conn failed: {}", e))?;
    // USE 必须在事务连接上执行，否则 DDL 落在默认库（structure_sync 修复）
    if let Some(db) = use_db {
        let _ = conn.query_drop(format!("USE `{}`", db.replace('`', "``"))).await;
    }
    conn.query_drop("BEGIN").await.map_err(|e| format!("BEGIN failed: {}", e))?;
    let mut executed = 0u64;
    let mut errors: Vec<String> = vec![];
    for sql in sqls {
        let t = sql.trim();
        if t.is_empty() || t.starts_with("--") { continue; }
        if let Err(e) = conn.query_drop(t).await {
            errors.push(format!(
                "Error executing \"{}\": {}",
                t.chars().take(100).collect::<String>(),
                e
            ));
            let _ = conn.query_drop("ROLLBACK").await;
            return Ok((executed, errors));
        }
        executed += 1;
    }
    conn.query_drop("COMMIT")
        .await
        .map_err(|e| format!("COMMIT failed: {}", e))?;
    Ok((executed, errors))
}

/// B3 修复：SQLite 事务（rusqlite transaction 同一连接，drop 自动回滚）
async fn execute_sqlite_transaction(
    config: &DbConnectionConfig,
    sqls: &[String],
) -> Result<(u64, Vec<String>), String> {
    let db_path = config
        .path
        .as_deref()
        .ok_or_else(|| "SQLite database path required".to_string())?
        .to_string();
    let sqls: Vec<String> = sqls.to_vec();
    tokio::task::spawn_blocking(move || {
        let mut conn = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .map_err(|e| format!("SQLite open failed: {}", e))?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("SQLite BEGIN failed: {}", e))?;
        let mut executed = 0u64;
        let mut errors: Vec<String> = vec![];
        for sql in sqls {
            let t = sql.trim();
            if t.is_empty() || t.starts_with("--") { continue; }
            if let Err(e) = tx.execute_batch(t) {
                errors.push(format!(
                    "Error executing \"{}\": {}",
                    t.chars().take(100).collect::<String>(),
                    e
                ));
                // tx drop 自动 ROLLBACK
                return Ok::<_, String>((executed, errors));
            }
            executed += 1;
        }
        tx.commit().map_err(|e| format!("SQLite COMMIT failed: {}", e))?;
        Ok((executed, errors))
    })
    .await
    .map_err(|e| e.to_string())?
}


// Test connection
#[tauri::command(rename_all = "camelCase")]
pub async fn db_test(config: DbConnectionConfig) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] db_test() called, type={}, host={}",
        config.db_type,
        supertool_core::logic::log_sanitizer::sanitize_string(&config.host)
    );
    match config.db_type.as_str() {
        "mysql" => {
            connect_mysql(&config).await?;
            Ok(serde_json::json!({ "success": true }))
        }
        "postgres" => {
            connect_postgres(&config).await?;
            Ok(serde_json::json!({ "success": true }))
        }
        "redis" => {
            connect_redis(&config).await?;
            Ok(serde_json::json!({ "success": true }))
        }
        "sqlite" => {
            connect_sqlite(&config)?;
            Ok(serde_json::json!({ "success": true }))
        }
        "elasticsearch" => {
            crate::commands::es::connect_es(&config).await?;
            Ok(serde_json::json!({ "success": true }))
        }
        other => Err(format!("Unsupported database type: {}", other)),
    }
}

pub(crate) fn connect_sqlite(config: &DbConnectionConfig) -> Result<(), String> {
    let path = config
        .path
        .as_deref()
        .ok_or("SQLite: database path is required")?;
    let expanded = shellexpand::tilde(path);
    // Check file exists and is readable
    std::fs::metadata(expanded.as_ref())
        .map_err(|e| format!("SQLite: cannot access database file '{}': {}", expanded, e))?;
    Ok(())
}



// (Legacy UDS router compatibility aliases removed — CLI uses HTTP API directly)
