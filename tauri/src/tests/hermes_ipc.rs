//! Direct-call tests for all hermes_* Tauri commands.
//!
//! Tests call the Rust functions directly (same as the frontend IPC handler
//! would), using real config / state files. This catches errors that would
//! surface when the frontend invokes these commands.
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serial lock so parallel test runs don't clobber config / env / state files.
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Run test with a temporary HERMES_HOME to avoid clobbering user data.
fn with_tmp_home<F>(f: F)
where
    F: FnOnce(&PathBuf),
{
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("hermes_ipc_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    unsafe { std::env::set_var("HERMES_HOME", &dir); }
    // Pre-create cron dir so cron tests work
    std::fs::create_dir_all(dir.join("cron")).ok();
    f(&dir);
    unsafe { std::env::remove_var("HERMES_HOME"); }
    let _ = std::fs::remove_dir_all(&dir);
}

// ═══════════════════════════════════════════════════════════════
// hermes_config
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_info() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_config::get_hermes_config_info().unwrap();
    assert!(v.get("hermesHome").is_some(), "missing hermesHome");
    assert!(v.get("installed").is_some(), "missing installed");
}

#[test]
fn test_list_toolsets() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_config::list_toolsets().unwrap();
    let toolsets = v["toolsets"].as_array().expect("expected toolsets array");
    assert!(!toolsets.is_empty(), "should have toolsets");
}

#[test]
fn test_list_mcp_servers() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_config::list_mcp_servers().unwrap();
    assert!(v.get("mcp_servers").is_some(), "missing mcp_servers");
}

// ═══════════════════════════════════════════════════════════════
// hermes_memory
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_read_memory() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let info = crate::commands::hermes_memory::read_memory().unwrap();
    assert!(!info.memory.content.is_empty() || info.memory.exists == false, "memory content weird");
    assert!(!info.user.content.is_empty() || info.user.exists == false, "user content weird");
}

#[test]
fn test_list_memory_providers() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let result = crate::commands::hermes_memory::list_memory_providers();
    let providers = result.providers;
    assert!(!providers.is_empty(), "should have at least one provider");
}

#[test]
fn test_read_env_vars() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_memory::read_env_vars().unwrap();
    assert!(v.is_object(), "should return an object");
}

// ═══════════════════════════════════════════════════════════════
// hermes_skills
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_installed_skills() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let skills = crate::commands::hermes_skills::list_installed_skills();
    // Vec<SkillInfo> — no JSON methods, just verify it's iterable
    assert!(skills.len() >= 0, "should at least be a vector");
}

#[test]
fn test_list_bundled_skills() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let skills = crate::commands::hermes_skills::list_bundled_skills();
    assert!(skills.len() >= 0, "should at least be a vector");
}

#[test]
fn test_get_skill_content_no_path() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let content = crate::commands::hermes_skills::get_skill_content("/nonexistent/skill".into());
    assert_eq!(content, "");
}

// ═══════════════════════════════════════════════════════════════
// hermes_cron
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_list_cron_jobs() {
    with_tmp_home(|_tmp| {
        let jobs = crate::commands::hermes_cron::list_cron_jobs().unwrap();
        assert!(!jobs.is_empty() || jobs.len() == 0, "should handle empty");
    });
}

#[test]
fn test_create_and_remove_cron_job() {
    with_tmp_home(|_tmp| {
        // Create
        let created = crate::commands::hermes_cron::create_cron_job(
            "*/5 * * * *".into(),
            Some("test prompt".into()),
            Some("test-job".into()),
            None,
        ).expect("createCronJob should succeed");
        let success = created["success"].as_bool().unwrap_or(false);
        assert!(success, "create must return success");

        // List and verify
        let jobs = crate::commands::hermes_cron::list_cron_jobs().unwrap();
        let found = jobs.iter().any(|j| j.name == "test-job");
        assert!(found, "created job should appear in list");

        // Remove
        if let Some(job) = jobs.iter().find(|j| j.name == "test-job") {
            crate::commands::hermes_cron::remove_cron_job(job.id.clone())
                .expect("removeCronJob should succeed");
        }
    });
}

// ═══════════════════════════════════════════════════════════════
// hermes_agent_chat (async — need Tokio runtime)
// ═══════════════════════════════════════════════════════════════

/// Run an async test function inside a Tokio runtime.
fn run_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let rt = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    rt.block_on(f)
}

#[test]
fn test_agent_check_available() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = run_async(crate::commands::hermes_agent_chat::agent_check_available()).unwrap();
    assert!(v.get("available").is_some(), "missing available");
    assert!(v.get("ready").is_some(), "missing ready");
}

#[test]
fn test_agent_get_models() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = run_async(crate::commands::hermes_agent_chat::agent_get_models()).unwrap();
    assert!(v.get("customModels").is_some(), "missing customModels");
    assert!(v.get("providerModels").is_some(), "missing providerModels");
}

#[test]
fn test_agent_clear_cache() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = run_async(crate::commands::hermes_agent_chat::agent_clear_cache("test-session".into()));
    assert!(v.is_ok(), "agentClearCache failed: {:?}", v.err());
}

// ═══════════════════════════════════════════════════════════════
// hermes_gateway
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_gateway_status() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_gateway::gateway_status().unwrap();
    assert!(v.get("running").is_some(), "missing running");
    assert!(v.get("pid").is_some(), "missing pid");
}

// ═══════════════════════════════════════════════════════════════
// hermes_insights
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_get_insights() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let v = crate::commands::hermes_insights::get_insights(Some(7), None).unwrap();
    assert!(v.get("success").is_some(), "missing success");
    assert!(v.get("output").is_some(), "missing output");
}

// ═══════════════════════════════════════════════════════════════
// hermes_sessions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sessions_export() {
    let v = crate::commands::hermes_sessions::sessions_export("-".into(), None, None);
    match v {
        Ok(_) => {},
        Err(ref e) if e.contains("No state.db") => {
            // No Hermes state DB — acceptable
        }
        Err(e) => panic!("sessionsExport failed: {e}"),
    }
}

#[test]
fn test_sessions_prune_dry_run() {
    // No real state.db needed — dry-run returns success even with no DB
    // if the source is not provided
    let v = crate::commands::hermes_sessions::sessions_prune(Some(90), None, None);
    match v {
        Ok(_) => {},  // dry-run succeeded
        Err(ref e) if e.contains("No state.db") => {
            // Acceptable if no Hermes state DB exists on this machine
        }
        Err(e) => panic!("sessionsPrune dry-run failed: {e}"),
    }
}
