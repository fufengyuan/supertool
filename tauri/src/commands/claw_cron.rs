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
