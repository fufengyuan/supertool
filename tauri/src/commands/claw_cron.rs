//! Claw Cron — persistent cron job management at ~/.claw/cron.json

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronJobInfo {
    pub id: String,
    pub schedule: String,
    pub prompt: String,
    pub description: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub run_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CronStore {
    jobs: Vec<CronJobInfo>,
}

fn cron_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("cron.json")
}

fn load_store() -> CronStore {
    let path = cron_file();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(CronStore { jobs: Vec::new() }),
        Err(_) => CronStore { jobs: Vec::new() },
    }
}

fn save_store(store: &CronStore) -> Result<(), String> {
    let path = cron_file();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("cron_{}", ts)
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_list_cron_jobs() -> Vec<CronJobInfo> {
    load_store().jobs
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_create_cron_job(
    schedule: String,
    prompt: String,
    description: Option<String>,
) -> Result<CronJobInfo, String> {
    let mut store = load_store();
    let job = CronJobInfo {
        id: generate_id(),
        schedule,
        prompt,
        description: description.unwrap_or_default(),
        enabled: true,
        last_run: None,
        run_count: 0,
    };
    store.jobs.push(job.clone());
    save_store(&store)?;
    Ok(job)
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_delete_cron_job(cron_id: String) -> Result<(), String> {
    let mut store = load_store();
    let before = store.jobs.len();
    store.jobs.retain(|j| j.id != cron_id);
    if store.jobs.len() == before {
        return Err(format!("Cron job '{}' not found", cron_id));
    }
    save_store(&store)
}

#[tauri::command(rename_all = "camelCase")]
pub fn claw_toggle_cron_job(cron_id: String) -> Result<CronJobInfo, String> {
    let mut store = load_store();
    let job = store
        .jobs
        .iter_mut()
        .find(|j| j.id == cron_id)
        .ok_or_else(|| format!("Cron job '{}' not found", cron_id))?;
    job.enabled = !job.enabled;
    let result = job.clone();
    save_store(&store)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
