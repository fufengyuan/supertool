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
pub enum DbConnection {
    MySql(MySqlPool),
    Postgres(PgClient),
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
    let mut conn = pool.get_conn().await.map_err(|e| format!("MySQL failed: {}", e))?;
    conn.ping().await.map_err(|e| format!("MySQL ping failed: {}", e))?;
    Ok(DbConnection::MySql(pool))
}

pub async fn execute_mysql_query(pool: &MySqlPool, sql: &str) -> Result<serde_json::Value, String> {
    let mut conn = pool.get_conn().await.map_err(|e| e.to_string())?;
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"USE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE");
    if is_write {
        conn.query_drop(sql).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true}))
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
    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
        .await.map_err(|e| format!("Postgres failed: {}", e))?;
    tokio::spawn(async move { connection.await.ok(); });
    Ok(DbConnection::Postgres(client))
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
    let conn = client
        .get_multiplexed_async_connection()
        .await
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
    let ty = row.columns()[i].type_();
    match ty {
        &Type::INT2 | &Type::INT4 | &Type::INT8 => row
            .try_get::<_, Option<i64>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |n| serde_json::json!(n)),
        &Type::FLOAT4 | &Type::FLOAT8 => row
            .try_get::<_, Option<f64>>(i)
            .ok()
            .flatten()
            .and_then(|n| serde_json::Number::from_f64(n).map(serde_json::Value::Number))
            .unwrap_or(serde_json::Value::Null),
        &Type::BOOL => row
            .try_get::<_, Option<bool>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
        // 时间类型分别处理：TIMESTAMPTZ 含时区、DATE/TIME 与 NaiveDateTime 不兼容（S1 修复）
        &Type::TIMESTAMP => row
            .try_get::<_, Option<chrono::NaiveDateTime>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |v| serde_json::Value::String(v.to_string())),
        &Type::TIMESTAMPTZ => row
            .try_get::<_, Option<chrono::DateTime<chrono::Utc>>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |v| {
                serde_json::Value::String(v.to_rfc3339())
            }),
        &Type::DATE => row
            .try_get::<_, Option<chrono::NaiveDate>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |v| serde_json::Value::String(v.to_string())),
        &Type::TIME => row
            .try_get::<_, Option<chrono::NaiveTime>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, |v| serde_json::Value::String(v.to_string())),
        &Type::TEXT
        | &Type::VARCHAR
        | &Type::BPCHAR
        | &Type::NAME
        | &Type::JSON
        | &Type::JSONB
        | &Type::UUID => row
            .try_get::<_, Option<String>>(i)
            .ok()
            .flatten()
            .map_or(serde_json::Value::Null, serde_json::Value::String),
        _ => {
            if let Ok(v) = row.try_get::<_, Option<&[u8]>>(i) {
                v.map_or(serde_json::Value::Null, |b| {
                    serde_json::Value::String(hex::encode(b))
                })
            } else if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                v.map_or(serde_json::Value::Null, serde_json::Value::String)
            } else {
                serde_json::Value::Null
            }
        }
    }
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

fn redis_value_to_json(val: &redis::Value) -> serde_json::Value {
    match val {
        redis::Value::Nil => serde_json::Value::Null,
        redis::Value::Int(n) => serde_json::json!(n),
        redis::Value::BulkString(d) => serde_json::Value::String(String::from_utf8_lossy(d).to_string()),
        redis::Value::Array(items) => serde_json::Value::Array(items.iter().map(redis_value_to_json).collect()),
        redis::Value::Okay => serde_json::Value::String("OK".to_string()),
        redis::Value::SimpleString(s) => serde_json::Value::String(s.clone()),
        _ => serde_json::Value::Null,
    }
}

// ── SQLite Query ─────────────────────────────────────────────────────────────

pub async fn execute_sqlite_query(config: &DbConnectionConfig, sql: &str) -> Result<serde_json::Value, String> {
    let path = config.path.as_ref().ok_or("SQLite path required")?;
    let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE");
    if is_write {
        conn.execute_batch(sql).map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true}))
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
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
    }
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
