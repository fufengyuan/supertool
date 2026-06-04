//! IPC-style tests for claw_cron commands.

use crate::commands::claw_cron::*;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    // ── Helper: build a mock app + webview with cron commands registered ──
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::claw_cron::claw_list_cron_jobs,
            crate::commands::claw_cron::claw_create_cron_job,
            crate::commands::claw_cron::claw_delete_cron_job,
            crate::commands::claw_cron::claw_toggle_cron_job,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
    /// Send IPC request and deserialize the response as `R`.
    /// Panics on IPC or deserialization failure.
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
    // ── Existing direct tests ────────────────────────────────────────────
#[test]
fn test_list_cron_jobs_returns_array() {
    let jobs = claw_list_cron_jobs();
        // Should always return an array (possibly empty)
    assert!(jobs.iter().all(|j| !j.id.is_empty()));
        // Verify camelCase serialization
    if let Some(job) = jobs.first() {
        let json = serde_json::to_value(job).unwrap();
            // These fields must be camelCase for the frontend
        assert!(json.get("lastRun").is_some(), "field should be 'lastRun' not 'last_run'");
        assert!(json.get("runCount").is_some(), "field should be 'runCount' not 'run_count'");
        assert!(json.get("schedule").is_some());
        assert!(json.get("prompt").is_some());
        assert!(json.get("enabled").is_some());
    }
}
#[test]
fn test_create_and_delete_cron_job() {
        // Create
    let job = claw_create_cron_job(
        "*/5 * * * *".to_string(),
        "test ping".to_string(),
        Some("unit test".to_string()),
    ).expect("create should succeed");
    assert_eq!(job.schedule, "*/5 * * * *");
    assert_eq!(job.prompt, "test ping");
    assert_eq!(job.description, "unit test");
    assert!(job.enabled);
    assert!(job.run_count == 0);
        // Verify listed
    let jobs = claw_list_cron_jobs();
    assert!(jobs.iter().any(|j| j.id == job.id));
        // Toggle
    let toggled = claw_toggle_cron_job(job.id.clone()).expect("toggle should succeed");
    assert!(!toggled.enabled);
        // Re-toggle
    let toggled_again = claw_toggle_cron_job(job.id.clone()).expect("re-toggle should succeed");
    assert!(toggled_again.enabled);
        // Delete
    claw_delete_cron_job(job.id.clone()).expect("delete should succeed");
        // Verify gone
    let jobs_after = claw_list_cron_jobs();
    assert!(jobs_after.iter().all(|j| j.id != job.id));
}
#[test]
fn test_delete_nonexistent_returns_error() {
    let result = claw_delete_cron_job("nonexistent_id".to_string());
    assert!(result.is_err());
}
#[test]
fn test_toggle_nonexistent_returns_error() {
    let result = claw_toggle_cron_job("nonexistent_id".to_string());
    assert!(result.is_err());
}
#[test]
fn test_create_without_description() {
    let job = claw_create_cron_job(
        "0 0 * * *".to_string(),
        "daily".to_string(),
        None,
    ).expect("create without desc should succeed");
    assert_eq!(job.description, "");
    claw_delete_cron_job(job.id).ok();
}
    // ── IPC-style tests: simulate frontend calls via get_ipc_response ────
#[test]
fn test_ipc_mock_builder_creates_app() {
    let (_app, _webview) = build_test_app();
        // No panic == success
}
#[test]
fn test_ipc_list_cron_jobs() {
    let (_app, webview) = build_test_app();
    let jobs: Vec<CronJobInfo> = invoke_ok(&webview, "claw_list_cron_jobs", serde_json::json!({}));
        // Verify camelCase field names for frontend
    if let Some(job) = jobs.first() {
        let json = serde_json::to_value(job).unwrap();
        assert!(json.get("lastRun").is_some(), "field should be 'lastRun' not 'last_run'");
        assert!(json.get("runCount").is_some(), "field should be 'runCount' not 'run_count'");
    }
}
#[test]
fn test_ipc_create_cron_job_shape() {
    let (_app, webview) = build_test_app();
    let job: CronJobInfo = invoke_ok(
        &webview,
        "claw_create_cron_job",
        serde_json::json!({
            "schedule": "*/10 * * * *",
            "prompt": "ipc test ping",
            "description": "ipc unit test",
        }),
    );
        // Assert shape of the IPC response (not file side-effects)
    assert!(!job.id.is_empty(), "should have an id");
    assert_eq!(job.schedule, "*/10 * * * *");
    assert_eq!(job.prompt, "ipc test ping");
    assert_eq!(job.description, "ipc unit test");
    assert!(job.enabled, "new jobs should be enabled");
    assert_eq!(job.run_count, 0, "new jobs should have run_count=0");
    assert!(job.last_run.is_none(), "new jobs should have no last_run");
        // Verify camelCase serialization of the returned value
    let json = serde_json::to_value(&job).unwrap();
    assert!(json.get("lastRun").is_some(), "field should be 'lastRun' not 'last_run'");
    assert!(json.get("runCount").is_some(), "field should be 'runCount' not 'run_count'");
        // Now verify IPC toggle works on the same created job
    let toggled: CronJobInfo = invoke_ok(
        &webview,
        "claw_toggle_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    assert!(!toggled.enabled, "first toggle should disable");
    assert_eq!(toggled.id, job.id);
    let re_toggled: CronJobInfo = invoke_ok(
        &webview,
        "claw_toggle_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    assert!(re_toggled.enabled, "second toggle should re-enable");
    assert_eq!(re_toggled.id, job.id);
        // Verify IPC delete works on the same created job — response is `()`
    let deleted: () = invoke_ok(
        &webview,
        "claw_delete_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    let _ = deleted;
        // Verify IPC delete of already-deleted job returns an error
    let res = get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "claw_delete_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "cronId": &job.id })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(res.is_err(), "deleting already-deleted job should error via IPC");
        // Verify IPC toggle of deleted job returns an error
    let res2 = get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "claw_toggle_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "cronId": &job.id })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(res2.is_err(), "toggling deleted job should error via IPC");
        // Clean the file for subsequent tests
    let _ = std::fs::write(cron_file(), "{\"jobs\":[]}");
}
#[test]
fn test_ipc_delete_returns_unit_shape() {
    let (_app, webview) = build_test_app();
        // First create a job (direct call — reliable)
    let job = claw_create_cron_job(
        "0 0 * * *".to_string(),
        "delete shape test".to_string(),
        None,
    ).expect("direct create should succeed");
        // IPC delete — assert the IPC response deserializes as `()`
    let result: () = invoke_ok(
        &webview,
        "claw_delete_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    let _ = result; // `()` is the expected shape for `Result<(), String>` returning Ok
}
#[test]
fn test_ipc_toggle_cron_job_shape() {
    let (_app, webview) = build_test_app();
        // First create a job (direct call — reliable)
    let job = claw_create_cron_job(
        "*/2 * * * *".to_string(),
        "toggle shape test".to_string(),
        None,
    ).expect("direct create should succeed");
        // IPC toggle — assert the toggle response shape
    let toggled: CronJobInfo = invoke_ok(
        &webview,
        "claw_toggle_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    assert!(!toggled.enabled, "toggled job should be disabled");
    assert_eq!(toggled.id, job.id);
    assert_eq!(toggled.schedule, job.schedule);
    assert_eq!(toggled.prompt, job.prompt);
        // IPC toggle back
    let toggled_again: CronJobInfo = invoke_ok(
        &webview,
        "claw_toggle_cron_job",
        serde_json::json!({ "cronId": job.id }),
    );
    assert!(toggled_again.enabled, "re-toggled job should be enabled");
        // Cleanup
    claw_delete_cron_job(job.id).ok();
}
#[test]
fn test_ipc_delete_nonexistent_returns_error() {
    let (_app, webview) = build_test_app();
    let res = get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "claw_delete_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "cronId": "nonexistent_id" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(res.is_err(), "deleting nonexistent cron job should error via IPC");
}
#[test]
fn test_ipc_toggle_nonexistent_returns_error() {
    let (_app, webview) = build_test_app();
    let res = get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "claw_toggle_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "cronId": "nonexistent_id" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
    assert!(res.is_err(), "toggling nonexistent cron job should error via IPC");
}
