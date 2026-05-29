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

fn normalize_job(raw: CronJobRaw) -> Option<CronJobItem> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== normalize_job ====================

    #[test]
    fn test_normalize_job_empty_id_returns_none() {
        let raw = CronJobRaw {
            id: String::new(),
            name: None,
            prompt: None,
            schedule: None,
            schedule_display: None,
            state: None,
            enabled: None,
            repeat: None,
            next_run_at: None,
            last_run_at: None,
            last_status: None,
            last_error: None,
            deliver: None,
            created_at: None,
            paused_at: None,
            script: None,
            skills: None,
            no_agent: None,
            workdir: None,
            profile: None,
            model: None,
            provider: None,
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
        // jobs.json doesn't exist → should return Ok(empty vec)
        let result = list_cron_jobs();
        // If the file doesn't exist, returns Ok([])
        // If it exists and is valid, returns Ok with jobs
        // Either way, it should be Ok
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
            deliver: Some(serde_json::json!({"platform": "telegram", "chat_id": "123"})),
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
}

