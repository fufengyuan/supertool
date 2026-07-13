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
    let decrypted_pw = config.password.as_deref()
        .map(|pw| crate::encryption::try_decrypt_password(pw));
    let addr = format!("redis://{}:{}@{}:{}",
        if decrypted_pw.is_some() { "" } else { "" },
        decrypted_pw.as_deref().unwrap_or(""),
        config.host, config.port,
    );
    let client = redis::Client::open(addr.as_str()).map_err(|e| format!("Redis URL: {}", e))?;
    let conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("Redis connect: {}", e))?;
    Ok(DbConnection::Redis(conn))
}

// ── SQLite ──────────────────────────────────────────────────────────────────

pub async fn connect_sqlite(config: &DbConnectionConfig) -> Result<DbConnection, String> {
    Ok(DbConnection::Sqlite(config.clone()))
}

// ── PostgreSQL Query ─────────────────────────────────────────────────────────

pub async fn execute_postgres_query(client: &PgClient, sql: &str) -> Result<serde_json::Value, String> {
    let upper = sql.trim().to_uppercase();
    let first = upper.split_whitespace().next().unwrap_or("");
    let is_write = matches!(first, "ALTER"|"CREATE"|"DROP"|"INSERT"|"UPDATE"|"DELETE"|"BEGIN"|"COMMIT"|"ROLLBACK"|"TRUNCATE");
    if is_write {
        client.execute(sql, &[]).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true}))
    } else {
        let rows = client.query(sql, &[]).await.map_err(|e| e.to_string())?;
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
    if let Ok(v) = row.try_get::<_, Option<i64>>(i) { return v.map_or(serde_json::Value::Null, |n| serde_json::json!(n)); }
    if let Ok(v) = row.try_get::<_, Option<f64>>(i) { return v.map_or(serde_json::Value::Null, |n| serde_json::json!(n)); }
    if let Ok(v) = row.try_get::<_, Option<bool>>(i) { return v.map_or(serde_json::Value::Null, |n| serde_json::json!(n)); }
    if let Ok(v) = row.try_get::<_, Option<String>>(i) { return v.map_or(serde_json::Value::Null, serde_json::Value::String); }
    if let Ok(v) = row.try_get::<_, Option<&[u8]>>(i) { return v.map_or(serde_json::Value::Null, |b| serde_json::Value::String(hex::encode(b))); }
    serde_json::Value::Null
}

// ── Redis Query ──────────────────────────────────────────────────────────────

pub async fn execute_redis_command(conn: &RedisConn, cmd: &str) -> Result<serde_json::Value, String> {
    use redis::AsyncCommands;
    let mut c = conn.clone();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return Err("Empty command".to_string()); }
    let command = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();
    let result: Result<redis::Value, redis::RedisError> = match command.to_uppercase().as_str() {
        "GET" => c.get(args[0]).await,
        "SET" => { let _: () = c.set(args[0], args[1]).await.map_err(|e| e.to_string())?; Ok(redis::Value::Okay) }
        "DEL" => { let n: i64 = c.del(args[0]).await.map_err(|e| e.to_string())?; Ok(redis::Value::Int(n)) }
        "KEYS" => c.keys(args[0]).await,
        "EXISTS" => c.exists(args[0]).await,
        "TTL" => c.ttl(args[0]).await,
        "TYPE" => { let t: String = redis::cmd("TYPE").arg(args[0]).query_async(&mut c).await.map_err(|e| e.to_string())?; Ok(redis::Value::SimpleString(t)) }
        _ => redis::cmd(command).arg(&args).query_async(&mut c).await,
    };
    result.map(|v| redis_value_to_json(&v)).map_err(|e| e.to_string())
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
