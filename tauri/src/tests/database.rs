//! IPC-style tests for database commands.

use crate::commands::database::*;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    // ── Helpers ──────────────────────────────────────────────────────────
fn create_test_config(path: &str) -> DbConnectionConfig {
    DbConnectionConfig {
        id: "test-id".into(),
        name: "test".into(),
        db_type: "sqlite".into(),
        host: String::new(),
        port: 0,
        username: String::new(),
        password: None,
        db_name: None,
        db_index: None,
        path: Some(path.into()),
    }
}
    /// Build a mock Tauri app with all database commands registered
    /// (except db_compare_structures which takes AppHandle — see its comment).
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
                // Connection management (5 main commands)
            crate::commands::database::db_connect,
            crate::commands::database::db_disconnect,
            crate::commands::database::db_query,
            crate::commands::database::db_get_tables,
            crate::commands::database::db_get_databases,
                // Table introspection
            crate::commands::database::db_get_table_structure,
            crate::commands::database::db_get_table_data,
            crate::commands::database::db_get_table_primary_keys,
            crate::commands::database::db_get_views,
            crate::commands::database::db_get_create_sql,
                // Compare & Sync
                // NOTE: db_compare_structures takes AppHandle — cannot be
                // registered with MockRuntime. Omitted from test handler.
            crate::commands::database::db_execute_structure_sync,
            crate::commands::database::db_compare_data,
            crate::commands::database::db_execute_data_sync,
                // Backup
            crate::commands::database::db_backup_create,
            crate::commands::database::db_backup_list,
            crate::commands::database::db_backup_restore,
            crate::commands::database::db_backup_delete,
                // Redis operations
            crate::commands::database::db_redis_databases,
            crate::commands::database::db_redis_keys,
            crate::commands::database::db_redis_keys_tree,
            crate::commands::database::db_redis_keys_by_type,
            crate::commands::database::db_redis_key_info,
            crate::commands::database::db_redis_key_value,
            crate::commands::database::db_redis_set_key,
            crate::commands::database::db_redis_add_key,
            crate::commands::database::db_redis_delete_key,
            crate::commands::database::db_redis_exec,
            crate::commands::database::db_redis_scan_keys,
            crate::commands::database::db_redis_streams,
            crate::commands::database::db_redis_stream_info,
            crate::commands::database::db_redis_stream_messages,
            crate::commands::database::db_redis_stream_add,
            crate::commands::database::db_redis_stream_del,
            crate::commands::database::db_redis_stream_delete,
            crate::commands::database::db_redis_stream_group_create,
            crate::commands::database::db_redis_stream_group_destroy,
            crate::commands::database::db_redis_stream_consumers,
            crate::commands::database::db_redis_stream_pending,
            crate::commands::database::db_redis_stream_claim,
            crate::commands::database::db_redis_stream_ack,
            crate::commands::database::db_redis_stream_retry,
            crate::commands::database::db_redis_stream_trim,
            crate::commands::database::db_redis_zset_range,
            crate::commands::database::db_redis_zset_remove,
                // Row CRUD
            crate::commands::database::db_insert_table_row,
            crate::commands::database::db_update_table_row,
            crate::commands::database::db_delete_table_row,
            crate::commands::database::db_get_table_data_filtered,
                // Test
            crate::commands::database::db_test,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
    /// Send an IPC invoke and deserialize the success response as `R`.
fn invoke_ok<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> R {
    let res = get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    res.unwrap_or_else(|e| panic!("IPC '{cmd}' failed: {e:?}"))
        .deserialize::<R>()
        .unwrap()
}
    /// Send an IPC invoke that is expected to return an error (String),
    /// and return that error message.
fn invoke_err(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> String {
    let res = get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    match res {
        Ok(response) => {
                // Command returned Ok – this is unexpected; try to explain
            let explain = response
                .deserialize::<serde_json::Value>()
                .ok()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "<no body>".into());
            panic!("IPC '{cmd}' expected Err but got Ok: {explain}")
        }
        Err(e) => format!("{e:?}"),
    }
}
    // ═══════════════════════════════════════════════════════════════════════
    // Pure-logic helper tests (no Tauri runtime)
    // ═══════════════════════════════════════════════════════════════════════
#[tokio::test]
async fn test_sqlite_execute_query_returns_rows() {
    let dir =
        std::env::temp_dir().join(format!("supertool_test_sqlite_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("test.db");
    let path_str = db_path.to_str().unwrap();
        // Create a test SQLite database with schema
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO users (name, age) VALUES ('Alice', 30), ('Bob', 25), ('Charlie', 35)",
            [],
        )
        .unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, title TEXT, user_id INTEGER)", []).unwrap();
    }
    let cfg = create_test_config(path_str);
        // Test: list tables from sqlite_master
    let result = execute_sqlite_query(&cfg, "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").await.unwrap();
    let rows = result.get("rows").and_then(|r| r.as_array()).unwrap();
    assert_eq!(rows.len(), 2, "expected 2 tables, got: {:?}", rows);
    let names: Vec<&str> = rows
        .iter()
        .filter_map(|r| r.get("name").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"posts"), "should contain posts table");
    assert!(names.contains(&"users"), "should contain users table");
        // Test: query a table
    let result = execute_sqlite_query(&cfg, "SELECT id, name, age FROM users ORDER BY age")
        .await
        .unwrap();
    let rows = result.get("rows").and_then(|r| r.as_array()).unwrap();
    assert_eq!(rows.len(), 3, "expected 3 users, got: {:?}", rows);
    assert_eq!(rows[0].get("name").and_then(|v| v.as_str()), Some("Bob"));
    assert_eq!(rows[0].get("age").and_then(|v| v.as_i64()), Some(25));
    assert_eq!(
        rows[2].get("name").and_then(|v| v.as_str()),
        Some("Charlie")
    );
        // Cleanup
    let _ = std::fs::remove_file(&db_path);
}
#[test]
fn test_connect_sqlite_valid_path() {
    let dir =
        std::env::temp_dir().join(format!("supertool_test_connect_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("valid.db");
    let path_str = db_path.to_str().unwrap();
        // Create the file
    rusqlite::Connection::open(&db_path).unwrap();
    let cfg = create_test_config(path_str);
    assert!(connect_sqlite(&cfg).is_ok(), "valid path should succeed");
    let _ = std::fs::remove_file(&db_path);
}
#[test]
fn test_connect_sqlite_missing_path_returns_error() {
    let cfg = DbConnectionConfig {
        path: None,
        ..create_test_config("/nonexistent/path.db")
    };
    let result = connect_sqlite(&cfg);
    assert!(result.is_err(), "missing path should fail");
    assert!(result.unwrap_err().contains("path is required"));
}
#[test]
fn test_connect_sqlite_nonexistent_file_returns_error() {
    let cfg = create_test_config("/tmp/__nonexistent_supertool_test_db__.db");
    let result = connect_sqlite(&cfg);
    assert!(result.is_err(), "nonexistent file should fail");
}
    // ═══════════════════════════════════════════════════════════════════════
    // IPC-style tests via tauri::test::get_ipc_response
    // ═══════════════════════════════════════════════════════════════════════
#[test]
fn test_ipc_db_disconnect_returns_success() {
        /// db_disconnect works without an active connection — it just removes
        /// the id from the pool if present. This verifies IPC routing is wired.
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ok(&ww, "db_disconnect", serde_json::json!({ "id": "nonexistent" }));
    assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
}
#[test]
fn test_ipc_db_redis_databases_returns_default() {
        /// db_redis_databases returns a hard-coded default without needing a
        /// real Redis connection. Verifies IPC routing + return shape.
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ok(&ww, "db_redis_databases", serde_json::json!({ "id": "any" }));
    assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
    let dbs = result.get("databases").and_then(|v| v.as_array());
    assert!(dbs.is_some(), "response should contain 'databases' array");
    assert_eq!(dbs.unwrap().len(), 1);
}
#[test]
fn test_ipc_db_backup_list_returns_array() {
        /// db_backup_list scans the backup directory. With no backups present
        /// it returns an empty array. Verifies IPC routing.
    let (_app, ww) = build_test_app();
    let result: serde_json::Value =
        invoke_ok(&ww, "db_backup_list", serde_json::json!({}));
    assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
    let backups = result.get("backups").and_then(|v| v.as_array());
    assert!(backups.is_some(), "response should contain 'backups' array");
}
#[test]
fn test_ipc_db_query_no_connection() {
        /// Without a prior db_connect call, db_query should error with
        /// "Connection not found". This proves IPC dispatch works for
        /// commands that take multiple params.
    let (_app, ww) = build_test_app();
    let err = invoke_err(
        &ww,
        "db_query",
        serde_json::json!({ "id": "nonexistent", "sql": "SELECT 1" }),
    );
    assert!(
        err.contains("Connection not found"),
        "expected 'Connection not found' error, got: {err}"
    );
}
#[test]
fn test_ipc_db_get_tables_no_connection() {
    let (_app, ww) = build_test_app();
    let err = invoke_err(
        &ww,
        "db_get_tables",
        serde_json::json!({ "id": "nonexistent", "dbName": "test" }),
    );
    assert!(
        err.contains("Connection not found"),
        "expected 'Connection not found' error, got: {err}"
    );
}
#[test]
fn test_ipc_db_get_databases_no_connection() {
    let (_app, ww) = build_test_app();
    let err = invoke_err(
        &ww,
        "db_get_databases",
        serde_json::json!({ "id": "nonexistent" }),
    );
    assert!(
        err.contains("Connection not found"),
        "expected 'Connection not found' error, got: {err}"
    );
}
#[test]
fn test_ipc_db_test_sqlite_missing_path() {
        /// db_test with missing SQLite path should error. Verifies IPC routing
        /// for commands taking a structured config parameter.
        /// Note: the parameter is named `config` in the Rust signature, so the
        /// IPC body must wrap fields in `{"config": {...}}`.
    let (_app, ww) = build_test_app();
    let err = invoke_err(
        &ww,
        "db_test",
        serde_json::json!({
            "config": {
                "id": "test-id",
                "name": "test",
                "type": "sqlite",
                "host": "",
                "port": 0,
                "username": "",
                "password": null,
                "dbName": null,
                "dbIndex": null,
                "path": null,
            }
        }),
    );
    assert!(
        err.contains("path is required"),
        "expected 'path is required' error, got: {err}"
    );
}
#[test]
fn test_ipc_db_backup_delete_nonexistent() {
        /// db_backup_delete on a nonexistent file should error.
    let (_app, ww) = build_test_app();
    let err = invoke_err(
        &ww,
        "db_backup_delete",
        serde_json::json!({ "file": "/tmp/__supertool_nonexistent_backup__.db" }),
    );
    assert!(
        err.contains("Failed to delete") || err.contains("No such file"),
        "expected file-not-found error, got: {err}"
    );
}
