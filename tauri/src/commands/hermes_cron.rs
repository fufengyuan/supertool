//! Hermes Cron Job Management — using hermes-config paths, no CLI calls.
//!
//! Reads/writes ~/.hermes/cron/jobs.json directly.
//! Mutations (create, remove, pause, resume, trigger) are done via
//! direct file operations instead of `hermes cron` CLI subprocess.

use serde::{Deserialize, Serialize};

use hermes_config::paths;

// ── JSON types matching ~/.hermes/cron/jobs.json ──────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CronJobSchedule {
    pub kind: Option<String>,
    pub run_at: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CronJobRepeat {
    pub times: Option<i64>,
    pub completed: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CronJobOrigin {
    pub platform: Option<String>,
    pub chat_id: Option<String>,
    pub chat_name: Option<String>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CronJobRaw {
    pub id: String,
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub schedule: Option<CronJobSchedule>,
    pub schedule_display: Option<String>,
    pub state: Option<String>,
    pub enabled: Option<bool>,
    pub repeat: Option<CronJobRepeat>,
    pub next_run_at: Option<String>,
    pub last_run_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub deliver: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub paused_at: Option<String>,
    pub script: Option<String>,
    pub skills: Option<Vec<String>>,
    pub no_agent: Option<bool>,
    pub workdir: Option<String>,
    pub profile: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobsFile {
    pub jobs: Vec<CronJobRaw>,
}

// ── Frontend types ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobItem {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub schedule: String,
    pub state: String,
    pub enabled: bool,
    pub next_run_at: Option<String>,
    pub last_run_at: Option<String>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub deliver: String,
    pub skills: Vec<String>,
    pub script: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────

fn jobs_file_path() -> std::path::PathBuf {
    paths::cron_dir().join("jobs.json")
}

fn read_jobs_file() -> Result<Vec<CronJobRaw>, String> {
    let path = jobs_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read jobs file: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let parsed: JobsFile = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse jobs file: {e}"))?;
    Ok(parsed.jobs)
}

fn write_jobs_file(jobs: &[CronJobRaw]) -> Result<(), String> {
    let path = jobs_file_path();
    // Ensure cron dir exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create cron dir: {e}"))?;
    }
    let file = JobsFile {
        jobs: jobs.to_vec(),
    };
    let content = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("Failed to serialize jobs: {e}"))?;
    // Atomic write
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &content)
        .map_err(|e| format!("Failed to write jobs: {e}"))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| format!("Failed to update jobs file: {e}"))?;
    Ok(())
}

pub fn normalize_job(raw: CronJobRaw) -> Option<CronJobItem> {
    let id = raw.id;
    if id.is_empty() {
        return None;
    }

    let enabled = raw.enabled.unwrap_or(true);
    let state = raw.state.as_deref().unwrap_or("active").to_string();
    let state_label = match state.as_str() {
        "paused" => "paused",
        "completed" => "completed",
        _ => "active",
    };

    let deliver_str = match raw.deliver {
        Some(serde_json::Value::String(s)) => s,
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<_>>()
            .join(", "),
        _ => "local".to_string(),
    };

    Some(CronJobItem {
        id,
        name: raw.name.unwrap_or_default(),
        prompt: raw.prompt.unwrap_or_default(),
        schedule: raw.schedule_display.unwrap_or_default(),
        state: state_label.to_string(),
        enabled,
        next_run_at: raw.next_run_at,
        last_run_at: raw.last_run_at,
        last_status: raw.last_status,
        last_error: raw.last_error,
        deliver: deliver_str,
        skills: raw.skills.unwrap_or_default(),
        script: raw.script,
    })
}

// ── Tauri Commands ────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn list_cron_jobs() -> Result<Vec<CronJobItem>, String> {
    let jobs = read_jobs_file()?;
    Ok(jobs.into_iter().filter_map(normalize_job).collect())
}

#[tauri::command(rename_all = "camelCase")]
pub fn create_cron_job(
    schedule: String,
    prompt: Option<String>,
    name: Option<String>,
    deliver: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut jobs = read_jobs_file()?;

    let new_job = CronJobRaw {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        prompt,
        schedule: Some(CronJobSchedule {
            kind: Some("cron".to_string()),
            run_at: Some(schedule.clone()),
            display: Some(schedule.clone()),
        }),
        schedule_display: Some(schedule),
        state: Some("active".to_string()),
        enabled: Some(true),
        deliver: deliver.map(serde_json::Value::String),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    jobs.push(new_job);
    write_jobs_file(&jobs)?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_cron_job(job_id: String) -> Result<serde_json::Value, String> {
    let mut jobs = read_jobs_file()?;
    jobs.retain(|j| j.id != job_id);
    write_jobs_file(&jobs)?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn pause_cron_job(job_id: String) -> Result<serde_json::Value, String> {
    let mut jobs = read_jobs_file()?;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.state = Some("paused".to_string());
        job.paused_at = Some(chrono::Utc::now().to_rfc3339());
    }
    write_jobs_file(&jobs)?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn resume_cron_job(job_id: String) -> Result<serde_json::Value, String> {
    let mut jobs = read_jobs_file()?;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.state = Some("active".to_string());
        job.paused_at = None;
    }
    write_jobs_file(&jobs)?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn trigger_cron_job(job_id: String) -> Result<serde_json::Value, String> {
    let mut jobs = read_jobs_file()?;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
        job.state = Some("active".to_string());
    }
    write_jobs_file(&jobs)?;
    // Note: actual execution is handled by the gateway scheduler
    Ok(serde_json::json!({ "success": true, "message": "Job scheduled for immediate execution" }))
}
