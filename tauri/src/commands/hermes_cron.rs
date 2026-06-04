//! Hermes Cron Job Management
//!
//! Manages scheduled cron jobs by wrapping the `hermes cron` CLI commands.
//! List reads ~/.hermes/cron/jobs.json directly for structured data;
//! mutations (create, remove, pause, resume, trigger) use the CLI.

use serde::{Deserialize, Serialize};
use std::process::Command;

// ---- shell helpers (same pattern as profile.rs) ----

fn run_with_user_env(cmd: &str, args: &[&str]) -> Result<String, String> {
    let full_cmd = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };
    let output = Command::new("/bin/bash")
        .args(["-l", "-c", &full_cmd])
        .output()
        .map_err(|e| format!("Failed to run command via shell: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {stderr}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_hermes_path() -> String {
    let local_hermes = dirs::home_dir()
        .map(|h| h.join(".local/bin/hermes"))
        .map(|p| p.to_string_lossy().to_string());

    if let Some(path) = local_hermes {
        let check = Command::new("/bin/bash")
            .args(["-l", "-c", &format!("test -x {path} && echo exists")])
            .output();
        if let Ok(output) = check {
            if String::from_utf8_lossy(&output.stdout).contains("exists") {
                return path;
            }
        }
    }

    "hermes".to_string()
}

fn run_cron_cli(args: &[String]) -> Result<String, String> {
    let hermes = get_hermes_path();
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_with_user_env(&format!("{hermes} cron"), &args_str)
}

// ---- JSON types matching ~/.hermes/cron/jobs.json ----

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CronJobSchedule {
    pub kind: Option<String>,
    pub run_at: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CronJobRepeat {
    pub times: Option<i64>,
    pub completed: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CronJobOrigin {
    pub platform: Option<String>,
    pub chat_id: Option<String>,
    pub chat_name: Option<String>,
    pub thread_id: Option<String>,
}

/// Raw job entry as stored in jobs.json
#[derive(Debug, Serialize, Deserialize, Default)]
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

/// Jobs file top-level structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JobsFile {
    pub jobs: Vec<CronJobRaw>,
}

// ---- Public API types ----

/// Clean cron job model returned to the frontend
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

pub(crate) fn normalize_job(raw: CronJobRaw) -> Option<CronJobItem> {
    let id = raw.id;
    if id.is_empty() {
        return None;
    }

    let enabled = raw.enabled.unwrap_or(true);
    let state = raw.state.as_deref().unwrap_or("active").to_string();
    // Normalize state: "scheduled" → "active", "paused" → "paused"
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

fn hermes_home() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Failed to resolve home directory")
        .join(".hermes")
}

// ---- Tauri commands ----

/// List all cron jobs by reading ~/.hermes/cron/jobs.json
#[tauri::command(rename_all = "camelCase")]
pub fn list_cron_jobs() -> Result<Vec<CronJobItem>, String> {
    let jobs_path = hermes_home().join("cron").join("jobs.json");

    if !jobs_path.exists() {
        return Ok(vec![]);
    }

    let content =
        std::fs::read_to_string(&jobs_path).map_err(|e| format!("Failed to read jobs file: {e}"))?;

    if content.trim().is_empty() {
        return Ok(vec![]);
    }

    let parsed: JobsFile =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse jobs file: {e}"))?;

    let jobs: Vec<CronJobItem> = parsed.jobs.into_iter().filter_map(normalize_job).collect();
    Ok(jobs)
}

/// Create a new cron job via `hermes cron create`
#[tauri::command(rename_all = "camelCase")]
pub fn create_cron_job(
    schedule: String,
    prompt: Option<String>,
    name: Option<String>,
    deliver: Option<String>,
) -> Result<(), String> {
    let mut args = vec!["create".to_string(), schedule.clone()];

    if let Some(n) = name {
        if !n.is_empty() {
            args.push("--name".to_string());
            args.push(n);
        }
    }
    if let Some(d) = deliver {
        if !d.is_empty() {
            args.push("--deliver".to_string());
            args.push(d);
        }
    }
    if let Some(p) = prompt {
        if !p.is_empty() {
            args.push("--".to_string());
            args.push(p);
        }
    }

    run_cron_cli(&args)?;
    Ok(())
}

/// Remove a cron job via `hermes cron remove`
#[tauri::command(rename_all = "camelCase")]
pub fn remove_cron_job(job_id: String) -> Result<(), String> {
    run_cron_cli(&["remove".to_string(), job_id])?;
    Ok(())
}

/// Pause a cron job via `hermes cron pause`
#[tauri::command(rename_all = "camelCase")]
pub fn pause_cron_job(job_id: String) -> Result<(), String> {
    run_cron_cli(&["pause".to_string(), job_id])?;
    Ok(())
}

/// Resume a paused cron job via `hermes cron resume`
#[tauri::command(rename_all = "camelCase")]
pub fn resume_cron_job(job_id: String) -> Result<(), String> {
    run_cron_cli(&["resume".to_string(), job_id])?;
    Ok(())
}

/// Trigger a cron job immediately via `hermes cron run`
#[tauri::command(rename_all = "camelCase")]
pub fn trigger_cron_job(job_id: String) -> Result<(), String> {
    run_cron_cli(&["run".to_string(), job_id])?;
    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================
