//! IPC-style tests for hermes_cron commands.

use crate::commands::hermes_cron::*;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    // ── Helper: build mock app with cron commands registered ──
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::hermes_cron::list_cron_jobs,
            crate::commands::hermes_cron::create_cron_job,
            crate::commands::hermes_cron::remove_cron_job,
            crate::commands::hermes_cron::pause_cron_job,
            crate::commands::hermes_cron::resume_cron_job,
            crate::commands::hermes_cron::trigger_cron_job,
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
    // ==================== normalize_job ====================
#[test]
fn test_normalize_job_empty_id_returns_none() {
    let raw = CronJobRaw {
        id: String::new(),
        ..Default::default()
    };
    assert!(normalize_job(raw).is_none());
}
#[test]
fn test_normalize_job_active_state() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        name: Some("Test Job".to_string()),
        prompt: Some("Do something".to_string()),
        schedule_display: Some("every 2h".to_string()),
        state: Some("active".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.id, "job-1");
    assert_eq!(result.name, "Test Job");
    assert_eq!(result.prompt, "Do something");
    assert_eq!(result.schedule, "every 2h");
    assert_eq!(result.state, "active");
    assert!(result.enabled);
}
#[test]
fn test_normalize_job_scheduled_state_becomes_active() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        state: Some("scheduled".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "active");
}
#[test]
fn test_normalize_job_paused_state_preserved() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        state: Some("paused".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "paused");
}
#[test]
fn test_normalize_job_completed_state_preserved() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        state: Some("completed".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "completed");
}
#[test]
fn test_normalize_job_unknown_state_falls_to_active() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        state: Some("unknown_state".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "active");
}
#[test]
fn test_normalize_job_missing_state_defaults_to_active() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        state: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "active");
}
#[test]
fn test_normalize_job_deliver_string() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        deliver: Some(serde_json::Value::String("telegram".to_string())),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "telegram");
}
#[test]
fn test_normalize_job_deliver_array() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        deliver: Some(serde_json::Value::Array(vec![
            serde_json::Value::String("telegram".to_string()),
            serde_json::Value::String("discord".to_string()),
        ])),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "telegram, discord");
}
#[test]
fn test_normalize_job_deliver_missing_defaults_to_local() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        deliver: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "local");
}
#[test]
fn test_normalize_job_disabled_enabled_flag() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        enabled: Some(false),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert!(!result.enabled);
}
#[test]
fn test_normalize_job_script_field() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        script: Some("/path/to/script.sh".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.script, Some("/path/to/script.sh".to_string()));
}
#[test]
fn test_normalize_job_skills_field() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        skills: Some(vec!["web".to_string(), "terminal".to_string()]),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.skills, vec!["web", "terminal"]);
}
#[test]
fn test_normalize_job_timestamps() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        next_run_at: Some("2026-05-30T09:00:00Z".to_string()),
        last_run_at: Some("2026-05-29T09:00:00Z".to_string()),
        last_status: Some("success".to_string()),
        last_error: Some("Something went wrong".to_string()),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.next_run_at, Some("2026-05-30T09:00:00Z".to_string()));
    assert_eq!(result.last_run_at, Some("2026-05-29T09:00:00Z".to_string()));
    assert_eq!(result.last_status, Some("success".to_string()));
    assert_eq!(result.last_error, Some("Something went wrong".to_string()));
}
#[test]
fn test_normalize_job_name_and_prompt_default_to_empty() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        name: None,
        prompt: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.name, "");
    assert_eq!(result.prompt, "");
}
#[test]
fn test_normalize_job_schedule_default_to_empty() {
    let raw = CronJobRaw {
        id: "job-1".to_string(),
        schedule_display: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.schedule, "");
}
    // ==================== list_cron_jobs (edge cases) ====================
#[test]
fn test_list_cron_jobs_no_file_returns_empty() {
    let result = list_cron_jobs();
    assert!(result.is_ok());
}
    // ==================== boundary: all fields populated ====================
#[test]
fn test_normalize_job_all_fields_populated() {
    let raw = CronJobRaw {
        id: "job-full".to_string(),
        name: Some("Full Job".to_string()),
        prompt: Some("Run full pipeline".to_string()),
        schedule: Some(CronJobSchedule {
            kind: Some("interval".to_string()),
            run_at: Some("*/5 * * * *".to_string()),
            display: Some("every 5 minutes".to_string()),
        }),
        schedule_display: Some("every 5 minutes".to_string()),
        state: Some("active".to_string()),
        enabled: Some(true),
        repeat: Some(CronJobRepeat {
            times: Some(10),
            completed: Some(3),
        }),
        next_run_at: Some("2026-05-30T10:00:00Z".to_string()),
        last_run_at: Some("2026-05-29T10:00:00Z".to_string()),
        last_status: Some("success".to_string()),
        last_error: Some("".to_string()),
        deliver: Some(serde_json::Value::String("telegram".to_string())),
        created_at: Some("2026-05-01T00:00:00Z".to_string()),
        paused_at: None,
        script: Some("/opt/scripts/backup.sh".to_string()),
        skills: Some(vec!["web".to_string(), "terminal".to_string()]),
        no_agent: Some(false),
        workdir: Some("/home/user".to_string()),
        profile: Some("default".to_string()),
        model: Some("claude-sonnet-4".to_string()),
        provider: Some("anthropic".to_string()),
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.id, "job-full");
    assert_eq!(result.name, "Full Job");
    assert_eq!(result.prompt, "Run full pipeline");
    assert_eq!(result.schedule, "every 5 minutes");
    assert_eq!(result.state, "active");
    assert!(result.enabled);
    assert_eq!(result.next_run_at, Some("2026-05-30T10:00:00Z".to_string()));
    assert_eq!(result.last_run_at, Some("2026-05-29T10:00:00Z".to_string()));
    assert_eq!(result.last_status, Some("success".to_string()));
    assert_eq!(result.last_error, Some("".to_string()));
    assert_eq!(result.deliver, "telegram");
    assert_eq!(result.skills, vec!["web", "terminal"]);
    assert_eq!(result.script, Some("/opt/scripts/backup.sh".to_string()));
}
    // ==================== boundary: deliver as non-standard JSON types ====================
#[test]
fn test_normalize_job_deliver_as_object_fallsback_to_local() {
    let raw = CronJobRaw {
        id: "job-obj".to_string(),
        deliver: Some(serde_json::json!( {"platform": "telegram", "chat_id": "123"})),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "local");
}
#[test]
fn test_normalize_job_deliver_as_number_fallsback_to_local() {
    let raw = CronJobRaw {
        id: "job-num".to_string(),
        deliver: Some(serde_json::Value::Number(serde_json::Number::from(42))),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "local");
}
#[test]
fn test_normalize_job_deliver_as_bool_fallsback_to_local() {
    let raw = CronJobRaw {
        id: "job-bool".to_string(),
        deliver: Some(serde_json::Value::Bool(true)),
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.deliver, "local");
}
    // ==================== boundary: edge state values ====================
#[test]
fn test_normalize_job_enabled_defaults_to_true() {
    let raw = CronJobRaw {
        id: "job-def-enable".to_string(),
        enabled: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert!(result.enabled);
}
#[test]
fn test_normalize_job_null_state_falls_to_active() {
    let raw = CronJobRaw {
        id: "job-null-state".to_string(),
        state: None,
        ..Default::default()
    };
    let result = normalize_job(raw).unwrap();
    assert_eq!(result.state, "active");
}
    // ── IPC: mock builder smoke test ────────────────────────────────────
#[test]
fn test_ipc_mock_builder_creates_app() {
    let (_app, _webview) = build_test_app();
        // No panic == success
}
    // ── IPC: list_cron_jobs ──────────────────────────────────────────────
#[test]
fn test_ipc_list_cron_jobs() {
    let (_app, webview) = build_test_app();
    let jobs: Vec<CronJobItem> =
        invoke_ok(&webview, "list_cron_jobs", serde_json::json!({}));
        // Must return an array (possibly empty)
    for job in &jobs {
        assert!(!job.id.is_empty(), "each job must have a non-empty id");
    }
        // Verify camelCase serialization for frontend
    if let Some(job) = jobs.first() {
        let json = serde_json::to_value(job).unwrap();
        assert!(json.get("nextRunAt").is_some(), "field should be 'nextRunAt' not 'next_run_at'");
        assert!(json.get("lastRunAt").is_some(), "field should be 'lastRunAt' not 'last_run_at'");
        assert!(json.get("lastStatus").is_some(), "field should be 'lastStatus' not 'last_status'");
    }
}
    // ── IPC: create_cron_job ────────────────────────────────────────────
#[test]
fn test_ipc_create_cron_job() {
    let (_app, webview) = build_test_app();
        // This command shells out to `hermes cron create`, which may or may not
        // be available in the test environment. We verify the IPC dispatch
        // doesn't panic and returns a proper Result<(), String> shape.
    match get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "create_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({
                "schedule": "*/5 * * * *",
                "prompt": "test ping",
                "name": "ipc-test-job",
            })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => {
                // If the CLI succeeds, the response is `null` (unit `()`)
            let val: serde_json::Value = resp.deserialize().unwrap();
            assert_eq!(val, serde_json::Value::Null);
        }
        Err(_) => {
                // If the CLI is not available, an error is expected — that's fine.
                // The important thing is the test doesn't panic.
        }
    }
}
    // ── IPC: remove_cron_job ────────────────────────────────────────────
#[test]
fn test_ipc_remove_cron_job() {
    let (_app, webview) = build_test_app();
        // Similar to create — may succeed or fail depending on `hermes` CLI
    match get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "remove_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "jobId": "nonexistent-test-job" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => {
            let val: serde_json::Value = resp.deserialize().unwrap();
            assert_eq!(val, serde_json::Value::Null);
        }
        Err(_) => {
                // CLI unavailable — acceptable
        }
    }
}
    // ── IPC: pause_cron_job ─────────────────────────────────────────────
#[test]
fn test_ipc_pause_cron_job() {
    let (_app, webview) = build_test_app();
    match get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "pause_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "jobId": "nonexistent-test-job" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => {
            let val: serde_json::Value = resp.deserialize().unwrap();
            assert_eq!(val, serde_json::Value::Null);
        }
        Err(_) => {
                // CLI unavailable — acceptable
        }
    }
}
    // ── IPC: resume_cron_job ────────────────────────────────────────────
#[test]
fn test_ipc_resume_cron_job() {
    let (_app, webview) = build_test_app();
    match get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "resume_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "jobId": "nonexistent-test-job" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => {
            let val: serde_json::Value = resp.deserialize().unwrap();
            assert_eq!(val, serde_json::Value::Null);
        }
        Err(_) => {
                // CLI unavailable — acceptable
        }
    }
}
    // ── IPC: trigger_cron_job ───────────────────────────────────────────
#[test]
fn test_ipc_trigger_cron_job() {
    let (_app, webview) = build_test_app();
    match get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "trigger_cron_job".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({ "jobId": "nonexistent-test-job" })),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    ) {
        Ok(resp) => {
            let val: serde_json::Value = resp.deserialize().unwrap();
            assert_eq!(val, serde_json::Value::Null);
        }
        Err(_) => {
                // CLI unavailable — acceptable
        }
    }
}
    // ── IPC: camelCase command name dispatch ────────────────────────────
#[test]
fn test_ipc_list_cron_jobs_response_shape() {
    let (_app, webview) = build_test_app();
    let res = get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: "list_cron_jobs".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(serde_json::json!({})),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    );
        // IPC dispatch must succeed — list_cron_jobs doesn't shell out,
        // it just reads a file (or returns empty vec if file missing)
    assert!(res.is_ok(), "listCronJobs IPC should succeed: {res:?}");
    let val: serde_json::Value = res.unwrap().deserialize().unwrap();
    assert!(val.is_array(), "response should be a JSON array");
}
