//! Hermes config.yaml management (pure Rust, no Python bridge)
//!
//! Reads/writes ~/.hermes/config.yaml for custom model management.
//! This replaces the Python bridge functions: agent_get/add/remove_models.
//!
//! Config structure:
//! ```yaml
//! model:
//!   default: "gpt-4"
//! custom_models:
//!   - "gpt-4"
//!   - "claude-3-opus"
//! ```
//!
//! Model list is dynamically fetched from ~/.hermes/models_dev_cache.json
//! (maintained by Hermes Agent from models.dev API), ensuring real-time sync.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Hermes config.yaml model section
#[derive(Debug, Serialize, Deserialize)]
struct ModelConfig {
    #[serde(default)]
    default: Option<String>,
    #[serde(default, alias = "model")]
    model: Option<String>,
    #[serde(default)]
    provider: Option<String>,
}

/// Hermes config.yaml root structure
#[derive(Debug, Serialize, Deserialize)]
struct HermesConfig {
    #[serde(default)]
    model: Option<ModelConfig>,
    #[serde(default)]
    custom_models: Vec<String>,
}

/// Models.dev cache entry for a single model
#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

/// Models.dev cache entry for a provider
#[derive(Debug, Deserialize)]
struct ProviderEntry {
    #[serde(default)]
    env: Vec<String>,
    #[serde(default)]
    models: HashMap<String, ModelEntry>,
}

/// Models.dev cache file structure (~/.hermes/models_dev_cache.json)
type ModelsDevCache = HashMap<String, ProviderEntry>;

/// Get path to Hermes config.yaml
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("config.yaml")
}

/// Get path to Hermes models.dev cache
fn models_cache_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("models_dev_cache.json")
}

/// Read Hermes config.yaml, return default model + custom models list
fn read_config() -> Result<HermesConfig, String> {
    let path = config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.yaml: {}", e))
}

/// Read models.dev cache file, return provider -> models mapping
fn read_models_cache() -> Result<ModelsDevCache, String> {
    let path = models_cache_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read models cache: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse models_dev_cache.json: {}", e))
}

/// Write Hermes config.yaml atomically
fn write_config(config: &HermesConfig) -> Result<(), String> {
    let path = config_path();
    let content = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    // Write to temp file then rename for atomicity
    let tmp_path = path.with_extension("yaml.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename config: {}", e))?;
    Ok(())
}

/// Check if Hermes Agent is installed (run_agent.py exists)
pub fn hermes_is_installed() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".hermes").join("hermes-agent").join("run_agent.py"))
        .filter(|p| p.exists())
        .is_some()
}

/// Get custom models, default model, and all available provider models from Hermes config
/// Provider models are dynamically fetched from ~/.hermes/models_dev_cache.json
/// Returns all models from all providers (user can select, API call will fail if key not configured)
pub fn get_models() -> Result<serde_json::Value, String> {
    let config = read_config()?;

    let default_model = config
        .model
        .as_ref()
        .and_then(|m| m.default.as_deref().or(m.model.as_deref()))
        .unwrap_or("")
        .to_string();

    let active_provider = config
        .model
        .as_ref()
        .and_then(|m| m.provider.as_deref())
        .unwrap_or("")
        .to_string();

    // 从 models.dev 缓存获取所有供应商模型
    let cache = read_models_cache().unwrap_or_default();

    // 收集所有供应商的模型，添加供应商前缀以便前端分组显示
    let mut provider_models: Vec<String> = Vec::new();
    for (provider_id, provider_entry) in &cache {
        for model_id in provider_entry.models.keys() {
            // 格式化为 "provider/model"，前端可解析供应商分组
            provider_models.push(format!("{}/{}", provider_id, model_id));
        }
    }

    Ok(serde_json::json!({
        "customModels": config.custom_models,
        "defaultModel": default_model,
        "activeProvider": active_provider,
        "providerModels": provider_models,
    }))
}
/// Add a model to Hermes config
pub fn add_model(model: String) -> Result<serde_json::Value, String> {
    let mut config = read_config()?;

    // Check if already exists
    if config.custom_models.contains(&model) {
        return Err(format!("Model '{}' already exists", model));
    }

    config.custom_models.push(model.clone());
    write_config(&config)?;

    Ok(serde_json::json!({
        "success": true,
        "model": model,
        "customModels": config.custom_models,
    }))
}

/// Remove a model from Hermes config
pub fn remove_model(model: String) -> Result<serde_json::Value, String> {
    let mut config = read_config()?;

    let pos = config
        .custom_models
        .iter()
        .position(|m| m == &model)
        .ok_or_else(|| format!("Model '{}' not found", model))?;

    config.custom_models.remove(pos);
    write_config(&config)?;

    Ok(serde_json::json!({
        "success": true,
        "model": model,
        "customModels": config.custom_models,
    }))
}

/// Set the default model in Hermes config (persists to config.yaml)
/// Empty string means "use system default" - don't write to config
pub fn set_default_model(model: String) -> Result<serde_json::Value, String> {
    if model.is_empty() {
        // Empty model = use system default, don't modify config
        return Ok(serde_json::json!({ "success": true, "model": "" }));
    }
    
    let mut config = read_config()?;
    if config.model.is_none() {
        config.model = Some(ModelConfig {
            default: None,
            model: None,
            provider: None,
        });
    }
    if let Some(ref mut m) = config.model {
        m.default = Some(model.clone());
    }
    write_config(&config)?;
    Ok(serde_json::json!({ "success": true, "model": model }))
}

// ============================================================================
// Hermes API Server Configuration
// ============================================================================

/// Get path to Hermes .env file
fn hermes_env_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join(".env")
}

/// Generate a random API key (32 chars hex)
fn generate_api_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Simple hash-like key generation (not cryptographically secure, but sufficient for local use)
    format!("{:x}", timestamp % 0xFFFFFFFFFFFFFFFF)
}

/// Check if Hermes API server is configured
/// Returns (enabled, has_key, key_value)
pub fn check_api_server_config() -> (bool, bool, String) {
    let env_path = hermes_env_path();
    if !env_path.exists() {
        return (false, false, String::new());
    }
    
    let content = std::fs::read_to_string(&env_path).unwrap_or_default();
    let enabled = content.lines().any(|line| {
        let line = line.trim();
        line.starts_with("API_SERVER_ENABLED=true") || 
        line.starts_with("API_SERVER_ENABLED=1") ||
        line.starts_with("API_SERVER_ENABLED=yes")
    });
    
    let key_line = content.lines().find(|line| line.trim().starts_with("API_SERVER_KEY="));
    let has_key = key_line.is_some();
    let key = key_line
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    
    (enabled, has_key, key)
}

/// Ensure Hermes API server is configured with an API key
/// If not configured, auto-generate a key and write to .env
/// Returns the API key that should be used
pub fn ensure_api_server_config() -> Result<String, String> {
    let (enabled, has_key, existing_key) = check_api_server_config();
    
    if enabled && has_key && !existing_key.is_empty() {
        // Already configured with a key
        return Ok(existing_key);
    }
    
    // Need to configure
    let env_path = hermes_env_path();
    let new_key = generate_api_key();
    
    // Read existing content (if file exists)
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Build new content
    let mut lines: Vec<String> = existing_content.lines()
        .filter(|line| {
            let l = line.trim();
            !l.starts_with("API_SERVER_ENABLED=") && !l.starts_with("API_SERVER_KEY=")
        })
        .map(|s| s.to_string())
        .collect();
    
    lines.push("API_SERVER_ENABLED=true".to_string());
    lines.push(format!("API_SERVER_KEY={}", new_key));
    
    // Ensure .hermes directory exists
    let hermes_dir = env_path.parent().unwrap();
    std::fs::create_dir_all(hermes_dir)
        .map_err(|e| format!("Failed to create .hermes directory: {}", e))?;
    
    // Write new content
    let new_content = lines.join("\n");
    if !new_content.ends_with("\n") {
        new_content.push('\n');
    }
    std::fs::write(&env_path, &new_content)
        .map_err(|e| format!("Failed to write .env: {}", e))?;
    
    log::info!("[ensure_api_server_config] Auto-configured Hermes API server with new key");
    Ok(new_key)
}

/// Check API server status and return config info for frontend
#[tauri::command(rename_all = "camelCase")]
pub fn agent_api_server_status() -> Result<serde_json::Value, String> {
    let installed = hermes_is_installed();
    let (enabled, has_key, key) = check_api_server_config();
    
    // Check if API server is actually running (health check)
    let running = if installed && enabled {
        // Try to connect to health endpoint
        let client = reqwest::blocking::Client::new();
        client
            .get("http://localhost:8642/health")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    } else {
        false
    };
    
    Ok(serde_json::json!({
        "installed": installed,
        "configured": enabled && has_key,
        "running": running,
        "needsRestart": enabled && has_key && !running,
        "api_key": key,  // Return key so frontend can use it for requests
    }))
}

/// Auto-configure Hermes API server and return the API key
#[tauri::command(rename_all = "camelCase")]
pub fn agent_configure_api_server() -> Result<serde_json::Value, String> {
    let key = ensure_api_server_config()?;
    
    Ok(serde_json::json!({
        "success": true,
        "apiKey": key,
        "message": "Hermes API server configured. Please restart gateway: hermes gateway restart",
    }))
}
