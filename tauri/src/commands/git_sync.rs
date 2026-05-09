use supertool_core::logic::CoreService;
use serde_json::json;
use std::process::Command;
use tauri::State;

/// Default config structure stored as JSON in the settings table
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitSyncConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub remote_url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default)]
    pub ssh_key: String,
    #[serde(default)]
    pub last_sync: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_interval() -> u64 {
    30 // minutes
}

impl Default for GitSyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            remote_url: String::new(),
            branch: default_branch(),
            interval: default_interval(),
            ssh_key: String::new(),
            last_sync: None,
            status: "not_configured".to_string(),
            error: None,
        }
    }
}

// =================== Helper: run git command ===================

fn run_git(app_data_dir: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(supertool_core::logic::git::find_git())
        .current_dir(app_data_dir)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// =================== Helper: load / save config ===================

fn load_config(core: &CoreService) -> Result<GitSyncConfig, String> {
    let raw = core
        .db_read(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = 'git_sync_config'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .map_err(|e| e.to_string())?;

    match raw {
        Some(value) => {
            serde_json::from_str(&value).map_err(|e| format!("Failed to parse git_sync_config: {}", e))
        }
        None => Ok(GitSyncConfig::default()),
    }
}

fn save_config(core: &CoreService, config: &GitSyncConfig) -> Result<(), String> {
    let value = serde_json::to_string(config).map_err(|e| e.to_string())?;
    let _ = core.db_write(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["git_sync_config", value],
        )
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

// =================== Commands ===================

/// Get current git sync status and configuration
#[tauri::command(rename_all = "camelCase")]
pub async fn git_sync_status(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_sync_status() called");
    let config = load_config(&core)?;
    Ok(json!({
        "enabled": config.enabled,
        "remote_url": config.remote_url,
        "branch": config.branch,
        "interval": config.interval,
        "ssh_key": config.ssh_key,
        "last_sync": config.last_sync,
        "status": config.status,
        "error": config.error,
    }))
}

/// Save git sync configuration
#[tauri::command(rename_all = "camelCase")]
pub async fn git_sync_configure(
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_sync_configure() called");
    let mut config = load_config(&core)?;

    if let Some(v) = params.get("enabled").and_then(|v| v.as_bool()) {
        config.enabled = v;
    }
    if let Some(v) = params.get("remote_url").and_then(|v| v.as_str()) {
        config.remote_url = v.to_string();
    }
    if let Some(v) = params.get("branch").and_then(|v| v.as_str()) {
        config.branch = v.to_string();
    }
    if let Some(v) = params.get("interval").and_then(|v| v.as_u64()) {
        config.interval = v;
    }
    if let Some(v) = params.get("ssh_key").and_then(|v| v.as_str()) {
        config.ssh_key = v.to_string();
    }

    // Reset error/status when reconfiguring
    config.error = None;
    if !config.remote_url.is_empty() {
        config.status = "configured".to_string();
    } else {
        config.status = "not_configured".to_string();
    }

    save_config(&core, &config)?;

    Ok(json!({
        "success": true,
        "enabled": config.enabled,
        "remote_url": config.remote_url,
        "branch": config.branch,
        "interval": config.interval,
    }))
}

/// Initialize a git repo in the app data directory and set the remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_sync_init(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_sync_init() called");
    let config = load_config(&core)?;

    if config.remote_url.is_empty() {
        return Err("Cannot init: no remote_url configured".to_string());
    }

    // Get the app data dir path
    let app_dir_str = core
        .get_app_path()
        .await
        .map(|v| v.as_str().unwrap_or("").to_string())
        .map_err(|e| format!("Failed to get app path: {}", e))?;

    let app_dir_path = std::path::Path::new(&app_dir_str);

    // Check if already initialized
    if app_dir_path.join(".git").exists() {
        return Ok(json!({
            "success": true,
            "message": "Git repo already initialized"
        }));
    }

    // git init
    run_git(app_dir_path, &["init"])?;

    // Set remote
    run_git(
        app_dir_path,
        &["remote", "add", "origin", &config.remote_url],
    )?;

    // Update config status
    let mut updated_config = config;
    updated_config.status = "initialized".to_string();
    updated_config.error = None;
    save_config(&core, &updated_config)?;

    Ok(json!({
        "success": true,
        "message": "Git repo initialized and remote set"
    }))
}

/// Perform git pull from remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_sync_pull(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_sync_pull() called");
    let config = load_config(&core)?;

    if config.remote_url.is_empty() {
        return Err("Cannot pull: no remote_url configured".to_string());
    }

    let app_dir_str = core
        .get_app_path()
        .await
        .map(|v| v.as_str().unwrap_or("").to_string())
        .map_err(|e| format!("Failed to get app path: {}", e))?;

    let app_dir_path = std::path::Path::new(&app_dir_str);

    if !app_dir_path.join(".git").exists() {
        return Err("Git repo not initialized. Run git_sync_init first.".to_string());
    }

    // git fetch
    run_git(app_dir_path, &["fetch", "origin"])?;

    // git pull
    let output = run_git(
        app_dir_path,
        &["pull", "--rebase", "origin", &config.branch],
    )?;

    // Update last_sync time
    let mut updated_config = config;
    updated_config.last_sync = Some(chrono::Utc::now().to_rfc3339());
    updated_config.status = "synced".to_string();
    updated_config.error = None;
    save_config(&core, &updated_config)?;

    Ok(json!({
        "success": true,
        "output": output,
        "last_sync": updated_config.last_sync
    }))
}

/// Perform git add, commit, and push
#[tauri::command(rename_all = "camelCase")]
pub async fn git_sync_push(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_sync_push() called");
    let config = load_config(&core)?;

    if config.remote_url.is_empty() {
        return Err("Cannot push: no remote_url configured".to_string());
    }

    let app_dir_str = core
        .get_app_path()
        .await
        .map(|v| v.as_str().unwrap_or("").to_string())
        .map_err(|e| format!("Failed to get app path: {}", e))?;

    let app_dir_path = std::path::Path::new(&app_dir_str);

    if !app_dir_path.join(".git").exists() {
        return Err("Git repo not initialized. Run git_sync_init first.".to_string());
    }

    // git add .
    run_git(app_dir_path, &["add", "."])?;

    // Check if there are changes to commit
    let status_output = run_git(app_dir_path, &["status", "--porcelain"])?;

    if status_output.is_empty() {
        return Ok(json!({
            "success": true,
            "message": "Nothing to commit, working tree clean"
        }));
    }

    // git commit
    let commit_msg = format!(
        "Auto-sync: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    run_git(
        app_dir_path,
        &["commit", "-m", &commit_msg],
    )?;

    // git push
    let output = run_git(
        app_dir_path,
        &["push", "origin", &config.branch],
    )?;

    // Update last_sync time
    let mut updated_config = config;
    updated_config.last_sync = Some(chrono::Utc::now().to_rfc3339());
    updated_config.status = "synced".to_string();
    updated_config.error = None;
    save_config(&core, &updated_config)?;

    Ok(json!({
        "success": true,
        "output": output,
        "last_sync": updated_config.last_sync
    }))
}
