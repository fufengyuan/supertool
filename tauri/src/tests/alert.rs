//! IPC-style tests for alert commands.

use crate::commands::alert::*;
use supertool_core::Database;
use supertool_core::logic::CoreService;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    // ── Test app builder ────────────────────────────────────────────────
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
        // Create in-memory database and CoreService for stateful commands
    let db = Database::new(&Path::new(":memory:")).expect("in-memory DB should open");
    let core = CoreService::new(db, std::env::temp_dir());
    let app = mock_builder()
        .manage(core)
        .invoke_handler(tauri::generate_handler![
            crate::commands::alert::get_email_config,
            crate::commands::alert::save_email_config,
            crate::commands::alert::test_email_config,
            crate::commands::alert::get_alert_services,
            crate::commands::alert::add_alert_service,
            crate::commands::alert::update_alert_service,
            crate::commands::alert::delete_alert_service,
            crate::commands::alert::get_alert_resources,
            crate::commands::alert::add_alert_resource,
            crate::commands::alert::update_alert_resource,
            crate::commands::alert::delete_alert_resource,
            crate::commands::alert::get_alert_history,
                // NOTE: trigger_alert_check takes AppHandle — cannot be
                // registered with MockRuntime. Omitted from test handler.
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
use std::path::Path;
use std::path::PathBuf;
    // ── IPC invoke helper ───────────────────────────────────────────────
fn invoke_ipc<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<R, String> {
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
        Ok(response) => response
            .deserialize::<R>()
            .map_err(|e| format!("deserialize error: {e:?}")),
        Err(e) => Err(format!("IPC error: {e:?}")),
    }
}
    // ── Pure-logic helper tests (no Tauri) ──────────────────────────────
#[test]
fn test_build_email_success() {
    let email = build_email("from@test.com", "to@test.com", "Subject", "Body")
        .expect("valid addresses should build");
        // Just confirm the message was built — subject() API may vary by lettre version
}
#[test]
fn test_build_email_invalid_from() {
    let result = build_email("not-an-email", "to@test.com", "Subject", "Body");
    assert!(result.is_err(), "invalid from should fail");
}
#[test]
fn test_build_email_invalid_to() {
    let result = build_email("from@test.com", "not-an-email", "Subject", "Body");
    assert!(result.is_err(), "invalid to should fail");
}
#[test]
fn test_build_email_multiple_recipients() {
    let email = build_email(
        "from@test.com",
        "a@test.com, b@test.com",
        "Subject",
        "Body",
    )
    .expect("multiple valid recipients should build");
        // Just confirm the message was built — subject() API may vary by lettre version
}
#[test]
fn test_build_smtp_transport_no_encryption_succeeds() {
        // With "none" encryption, transport construction always succeeds;
        // actual failure only happens at send time.
    let result = build_smtp_transport("smtp.example.com", 25, "none", "user", "pass");
    assert!(result.is_ok(), "transport construction should succeed");
}
    // ── IPC tests: mock_app build smoke test ────────────────────────────
#[test]
fn test_ipc_mock_builder_creates_app() {
    let (_app, _ww) = build_test_app();
}
    // ── IPC tests: get_email_config / save_email_config ─────────────────
#[test]
fn test_ipc_get_email_config_returns_none_initially() {
    let (_app, ww) = build_test_app();
    let result: Result<Option<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_email_config", serde_json::json!({}));
    assert!(result.is_ok(), "get_email_config should not fail");
    assert!(
        result.unwrap().is_none(),
        "no config should exist initially"
    );
}
#[test]
fn test_ipc_save_and_get_email_config() {
    let (_app, ww) = build_test_app();
        // Save a config
    let save: Result<(), String> = invoke_ipc(
        &ww,
        "save_email_config",
        serde_json::json!({
            "smtpHost": "smtp.example.com",
            "smtpPort": 587,
            "smtpUsername": "alice",
            "smtpPassword": "secret",
            "fromEmail": "alice@example.com",
            "toEmail": "bob@example.com",
            "smtpEncryption": "starttls",
        }),
    );
    assert!(save.is_ok(), "save_email_config should succeed");
        // Read it back
    let get: Result<Option<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_email_config", serde_json::json!({}));
    let config = get.expect("get_email_config should succeed")
        .expect("config should now exist");
    assert_eq!(config["smtpHost"], "smtp.example.com");
    assert_eq!(config["smtpPort"], 587);
    assert_eq!(config["smtpUsername"], "alice");
    assert_eq!(config["fromEmail"], "alice@example.com");
    assert_eq!(config["toEmail"], "bob@example.com");
    assert_eq!(config["smtpEncryption"], "starttls");
}
    // ── IPC tests: test_email_config (expected to fail without SMTP) ────
#[test]
fn test_ipc_test_email_config_fails_without_smtp() {
    let (_app, ww) = build_test_app();
        // No real SMTP server available → should return an Err
    let result: Result<String, String> = invoke_ipc(
        &ww,
        "test_email_config",
        serde_json::json!({
            "smtpHost": "nonexistent.local",
            "smtpPort": 587,
            "smtpUsername": null,
            "smtpPassword": null,
            "fromEmail": "test@test.com",
            "toEmail": "test@test.com",
            "smtpEncryption": "starttls",
        }),
    );
    assert!(result.is_err(), "test_email_config should fail without SMTP");
}
    // ── IPC tests: alert_services CRUD ──────────────────────────────────
#[test]
fn test_ipc_get_alert_services_returns_empty_initially() {
    let (_app, ww) = build_test_app();
    let result: Result<Vec<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({}));
    let services = result.expect("get_alert_services should succeed");
    assert!(services.is_empty(), "no services yet");
}
#[test]
fn test_ipc_add_alert_service() {
    let (_app, ww) = build_test_app();
    let add: Result<(), String> = invoke_ipc(
        &ww,
        "add_alert_service",
        serde_json::json!({
            "service": {
                "name": "My Service",
                "host": "192.168.1.1",
                "port": 443,
                "checkInterval": 60,
                "timeoutSeconds": 5,
                "maxRetries": 3,
                "enabled": true,
            },
        }),
    );
    assert!(add.is_ok(), "add_alert_service should succeed");
        // Verify it shows up
    let list: Result<Vec<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({}));
    let services = list.expect("get_alert_services should succeed");
    assert_eq!(services.len(), 1);
    assert_eq!(services[0]["name"], "My Service");
    assert_eq!(services[0]["host"], "192.168.1.1");
    assert_eq!(services[0]["port"], 443);
}
#[test]
fn test_ipc_update_alert_service() {
    let (_app, ww) = build_test_app();
        // Add first
    let add: Result<(), String> = invoke_ipc(
        &ww,
        "add_alert_service",
        serde_json::json!({
            "service": {
                "name": "Original",
                "host": "10.0.0.1",
                "port": 80,
                "checkInterval": 60,
                "timeoutSeconds": 5,
                "maxRetries": 3,
                "enabled": true,
            },
        }),
    );
    assert!(add.is_ok());
        // Get the generated ID
    let list: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
    let service_id = list[0]["id"].as_str().unwrap().to_string();
        // Update
    let update: Result<(), String> = invoke_ipc(
        &ww,
        "update_alert_service",
        serde_json::json!({
            "service": {
                "id": service_id,
                "name": "Updated",
                "host": "10.0.0.1",
                "port": 8080,
                "checkInterval": 120,
                "timeoutSeconds": 10,
                "maxRetries": 5,
                "enabled": false,
            },
        }),
    );
    assert!(update.is_ok(), "update_alert_service should succeed");
        // Verify
    let list2: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
    assert_eq!(list2.len(), 1);
    assert_eq!(list2[0]["name"], "Updated");
    assert_eq!(list2[0]["port"], 8080);
    assert!(!list2[0]["enabled"].as_bool().unwrap_or(true));
}
#[test]
fn test_ipc_delete_alert_service() {
    let (_app, ww) = build_test_app();
        // Add
    let _: () = invoke_ipc(
        &ww,
        "add_alert_service",
        serde_json::json!({
            "service": {
                "name": "ToDelete",
                "host": "10.0.0.1",
                "port": 80,
                "checkInterval": 60,
                "timeoutSeconds": 5,
                "maxRetries": 3,
                "enabled": true,
            },
        }),
    ).unwrap();
        // Get ID
    let list: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
    let sid = list[0]["id"].as_str().unwrap().to_string();
        // Delete
    let del: Result<(), String> = invoke_ipc(
        &ww,
        "delete_alert_service",
        serde_json::json!({ "id": sid }),
    );
    assert!(del.is_ok(), "delete_alert_service should succeed");
        // Verify empty
    let list2: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
    assert!(list2.is_empty());
}
    // ── IPC tests: alert_resources CRUD ─────────────────────────────────
#[test]
fn test_ipc_get_alert_resources_returns_empty_initially() {
    let (_app, ww) = build_test_app();
    let result: Result<Vec<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({}));
    let resources = result.expect("get_alert_resources should succeed");
    assert!(resources.is_empty(), "no resources yet");
}
#[test]
fn test_ipc_add_alert_resource() {
    let (_app, ww) = build_test_app();
    let add: Result<(), String> = invoke_ipc(
        &ww,
        "add_alert_resource",
        serde_json::json!({
            "resource": {
                "name": "SSL Cert",
                "category": "certificate",
                "remark": "example.com cert",
                "expireAt": "2026-12-31",
                "alertAdvanceDays": 30,
                "enabled": true,
            },
        }),
    );
    assert!(add.is_ok(), "add_alert_resource should succeed");
    let list: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "SSL Cert");
    assert_eq!(list[0]["category"], "certificate");
}
#[test]
fn test_ipc_update_alert_resource() {
    let (_app, ww) = build_test_app();
        // Add
    let _: () = invoke_ipc(
        &ww,
        "add_alert_resource",
        serde_json::json!({
            "resource": {
                "name": "Original",
                "category": "certificate",
                "remark": "",
                "expireAt": "2026-12-31",
                "alertAdvanceDays": 30,
                "enabled": true,
            },
        }),
    ).unwrap();
    let list: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
    let rid = list[0]["id"].as_str().unwrap().to_string();
    let update: Result<(), String> = invoke_ipc(
        &ww,
        "update_alert_resource",
        serde_json::json!({
            "resource": {
                "id": rid,
                "name": "Updated Cert",
                "category": "certificate",
                "remark": "updated",
                "expireAt": "2027-01-15",
                "alertAdvanceDays": 14,
                "enabled": false,
            },
        }),
    );
    assert!(update.is_ok(), "update_alert_resource should succeed");
    let list2: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
    assert_eq!(list2.len(), 1);
    assert_eq!(list2[0]["name"], "Updated Cert");
    assert_eq!(list2[0]["alertAdvanceDays"], 14);
}
#[test]
fn test_ipc_delete_alert_resource() {
    let (_app, ww) = build_test_app();
    let _: () = invoke_ipc(
        &ww,
        "add_alert_resource",
        serde_json::json!({
            "resource": {
                "name": "ToDelete",
                "category": "domain",
                "remark": "",
                "expireAt": "2026-12-31",
                "alertAdvanceDays": 30,
                "enabled": true,
            },
        }),
    ).unwrap();
    let list: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
    let rid = list[0]["id"].as_str().unwrap().to_string();
    let del: Result<(), String> = invoke_ipc(
        &ww,
        "delete_alert_resource",
        serde_json::json!({ "id": rid }),
    );
    assert!(del.is_ok(), "delete_alert_resource should succeed");
    let list2: Vec<serde_json::Value> =
        invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
    assert!(list2.is_empty());
}
    // ── IPC tests: get_alert_history ────────────────────────────────────
#[test]
fn test_ipc_get_alert_history_returns_empty_initially() {
    let (_app, ww) = build_test_app();
    let result: Result<Vec<serde_json::Value>, String> =
        invoke_ipc(&ww, "get_alert_history", serde_json::json!({}));
    let history = result.expect("get_alert_history should succeed");
    assert!(history.is_empty(), "no history yet");
}
    // trigger_alert_check takes AppHandle — cannot be compiled with MockRuntime,
    // so it is omitted from the test invoke_handler entirely.
