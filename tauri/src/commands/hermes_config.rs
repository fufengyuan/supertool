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
pub(crate) fn config_path() -> PathBuf {
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

/// Generate a fixed API key for SuperTool local use
/// Uses a predictable key so Gateway and SuperTool stay in sync
fn generate_api_key() -> String {
    // Fixed key for local development - must match what Gateway expects
    // This ensures SuperTool and Gateway always use the same key
    "supertool-local-key".to_string()
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
    let mut new_content = lines.join("\n");
    if !new_content.ends_with("\n") {
        new_content.push('\n');
    }
    std::fs::write(&env_path, &new_content).map_err(|e| format!("Failed to write .env: {}", e))?;
    
    log::info!("[ensure_api_server_config] Auto-configured Hermes API server with new key");
    
    // Restart gateway to apply new config
    let restart_result = std::process::Command::new("/bin/bash")
        .args(["-l", "-c", "hermes gateway restart"])
        .output();
    
    match restart_result {
        Ok(output) if output.status.success() => {
            log::info!("[ensure_api_server_config] Gateway restarted successfully");
        }
        Ok(output) => {
            log::warn!("[ensure_api_server_config] Gateway restart failed: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            log::warn!("[ensure_api_server_config] Failed to restart gateway: {}", e);
        }
    }
    
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
/// If custom_key is provided, use it instead of auto-generating
#[tauri::command(rename_all = "camelCase")]
pub fn agent_configure_api_server(custom_key: Option<String>) -> Result<serde_json::Value, String> {
    let key = if let Some(custom) = custom_key {
        // User provided a custom key, use it
        set_api_server_key(&custom)?;
        custom
    } else {
        // Auto-configure
        ensure_api_server_config()?
    };
    
    Ok(serde_json::json!({
        "success": true,
        "apiKey": key,
        "message": "Hermes API server configured. Gateway will be restarted.",
    }))
}

/// Set a specific API key for Hermes API server
fn set_api_server_key(key: &str) -> Result<(), String> {
    let env_path = hermes_env_path();
    
    // Read existing content (if file exists)
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Build new content - remove existing API_SERVER_* lines and add new ones
    let mut lines: Vec<String> = existing_content
        .lines()
        .filter(|line| {
            let l = line.trim();
            !l.starts_with("API_SERVER_ENABLED=") && !l.starts_with("API_SERVER_KEY=")
        })
        .map(|s| s.to_string())
        .collect();
    
    lines.push("API_SERVER_ENABLED=true".to_string());
    lines.push(format!("API_SERVER_KEY={}", key));
    
    // Ensure .hermes directory exists
    let hermes_dir = env_path.parent().unwrap();
    std::fs::create_dir_all(hermes_dir)
        .map_err(|e| format!("Failed to create .hermes directory: {}", e))?;
    
    // Write new content
    let mut new_content = lines.join("\n");
    if !new_content.ends_with("\n") {
        new_content.push('\n');
    }
    std::fs::write(&env_path, &new_content).map_err(|e| format!("Failed to write .env: {}", e))?;
    
    log::info!("[set_api_server_key] Set API_SERVER_KEY={}", key);
    
    // Restart gateway to apply new config
    let restart_result = std::process::Command::new("/bin/bash")
        .args(["-l", "-c", "hermes gateway restart"])
        .output();
    
    match restart_result {
        Ok(output) if output.status.success() => {
            log::info!("[set_api_server_key] Gateway restarted successfully");
        }
        Ok(output) => {
            log::warn!(
                "[set_api_server_key] Gateway restart failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            log::warn!("[set_api_server_key] Failed to restart gateway: {}", e);
        }
    }
    
    Ok(())
}

// ============================================================================
// Hermes Toolset Management (platform_toolsets.cli + mcp_servers)
// ============================================================================

/// All 16 Hermes toolset definitions with their key, label, and description
pub(crate) const ALL_TOOLSETS: &[(&str, &str, &str)] = &[
    ("web",            "Web",            "Web search and capture"),
    ("browser",        "Browser",        "Web browsing"),
    ("terminal",       "Terminal",        "Shell commands"),
    ("file",           "File",            "Read/write files"),
    ("code_execution", "Code Execution",  "Execute Python code"),
    ("vision",         "Vision",          "Image analysis"),
    ("image_gen",      "Image Gen",       "Generate images"),
    ("tts",            "TTS",             "Text to speech"),
    ("skills",         "Skills",          "Load and manage skills"),
    ("memory",         "Memory",          "Persistent memory"),
    ("session_search", "Session Search",  "Search history"),
    ("clarify",        "Clarify",         "Ask questions"),
    ("delegation",     "Delegation",       "Spawn sub-agents"),
    ("cronjob",        "Cron Job",        "Schedule tasks"),
    ("moa",            "MOA",             "Mixture of Agents"),
    ("todo",           "Todo",            "Task list"),
];

/// Read the CLI toolset enable list from a parsed YAML Value.
/// Returns Some(list) when `platform_toolsets.cli` is a non-empty array.
/// Returns None when `platform_toolsets` or `platform_toolsets.cli` is missing or empty.
pub(crate) fn read_enabled_toolsets_from_value(root: &serde_yaml::Value) -> Option<Vec<String>> {
    let platform = root.get("platform_toolsets")?;
    let cli = platform.get("cli")?;
    let arr = cli.as_sequence()?;
    if arr.is_empty() {
        return None;
    }
    let keys: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if keys.is_empty() {
        return None;
    }
    Some(keys)
}

/// List all 16 Hermes toolsets with enabled/disabled status based on
/// `platform_toolsets.cli` in ~/.hermes/config.yaml.
///
/// When `platform_toolsets.cli` does not exist, is empty, or is not present,
/// all toolsets are considered enabled (backward-compatible default).
#[tauri::command(rename_all = "camelCase")]
pub fn list_toolsets() -> Result<serde_json::Value, String> {
    let enabled_keys = read_config_yaml().ok()
        .and_then(|root| read_enabled_toolsets_from_value(&root));

    let toolsets: Vec<serde_json::Value> = ALL_TOOLSETS.iter().map(|(key, label, desc)| {
        let enabled = enabled_keys
            .as_ref()
            .map(|keys| keys.contains(&key.to_string()))
            .unwrap_or(true); // default: enabled
        serde_json::json!({
            "key": key,
            "label": label,
            "description": desc,
            "enabled": enabled,
        })
    }).collect();

    Ok(serde_json::json!({ "toolsets": toolsets }))
}

/// Enable or disable a toolset in `platform_toolsets.cli`.
///
/// When enabling, the toolset key is added to the list if not already present.
/// When disabling, the toolset key is removed from the list.
/// If the list doesn't exist yet, it is created with all 16 toolsets enabled,
/// then the specified toolset is removed.
#[tauri::command(rename_all = "camelCase")]
pub fn set_toolset_enabled(key: String, enabled: bool) -> Result<serde_json::Value, String> {
    let path = config_path();

    // Read existing config YAML or start with empty
    let yaml_content = if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        String::new()
    };

    let mut root: serde_yaml::Value = if yaml_content.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&yaml_content)
            .map_err(|e| format!("Failed to parse config.yaml: {}", e))?
    };

    // Navigate to / create platform_toolsets.cli
    let platform = root
        .as_mapping_mut()
        .ok_or("Config root is not a mapping")?;

    let platform_entry = platform
        .entry(serde_yaml::Value::String("platform_toolsets".to_string()))
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    let cli_entry = platform_entry
        .as_mapping_mut()
        .ok_or("platform_toolsets is not a mapping")?
        .entry(serde_yaml::Value::String("cli".to_string()))
        .or_insert_with(|| serde_yaml::Value::Sequence(serde_yaml::Sequence::new()));

    let cli_seq = cli_entry
        .as_sequence_mut()
        .ok_or("platform_toolsets.cli is not a list")?;

    if enabled {
        // Add key if not already present
        let key_exists = cli_seq.iter().any(|v| v.as_str() == Some(&key));
        if !key_exists {
            cli_seq.push(serde_yaml::Value::String(key.clone()));
        }
    } else {
        // When the user first disables a toolset and cli list is empty (or new),
        // pre-populate with all 16 toolsets so the "explicit allowlist" semantics
        // work correctly — only the disabled toolset gets removed.
        if cli_seq.is_empty() {
            for (tool_key, _, _) in ALL_TOOLSETS {
                cli_seq.push(serde_yaml::Value::String(tool_key.to_string()));
            }
        }
        // Remove key if present
        cli_seq.retain(|v| v.as_str() != Some(&key));
    }

    // Write back atomically
    let new_content = serde_yaml::to_string(&root)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    let tmp_path = path.with_extension("yaml.tmp");
    std::fs::write(&tmp_path, &new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename config: {}", e))?;

    Ok(serde_json::json!({ "success": true }))
}

/// List all configured MCP servers from ~/.hermes/config.yaml.
///
/// Returns an array of { name, type, detail } objects.
/// - `type` is "http" when the server has a `url` field, "stdio" when it has a `command` field.
/// - `detail` shows the URL (http) or command + args (stdio).
#[tauri::command(rename_all = "camelCase")]
pub fn list_mcp_servers() -> Result<serde_json::Value, String> {
    let root = read_config_yaml()?;

    let servers = match root.get("mcp_servers") {
        Some(serde_yaml::Value::Mapping(map)) => {
            let mut result = Vec::new();
            for (name_val, config) in map {
                let name = name_val.as_str().unwrap_or("unknown").to_string();
                if let Some(cfg_map) = config.as_mapping() {
                    if let Some(url_val) = cfg_map.get(&serde_yaml::Value::String("url".to_string())) {
                        result.push(serde_json::json!({
                            "name": name,
                            "type": "http",
                            "detail": url_val.as_str().unwrap_or(""),
                        }));
                    } else if let Some(cmd_val) = cfg_map.get(&serde_yaml::Value::String("command".to_string())) {
                        let cmd = cmd_val.as_str().unwrap_or("");
                        let args_str = cfg_map
                            .get(&serde_yaml::Value::String("args".to_string()))
                            .and_then(|a| a.as_sequence())
                            .map(|seq| {
                                seq.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            })
                            .unwrap_or_default();
                        let detail = if args_str.is_empty() {
                            cmd.to_string()
                        } else {
                            format!("{} {}", cmd, args_str)
                        };
                        result.push(serde_json::json!({
                            "name": name,
                            "type": "stdio",
                            "detail": detail,
                        }));
                    }
                }
            }
            result
        }
        _ => Vec::new(),
    };

    Ok(serde_json::json!({ "mcp_servers": servers }))
}

/// Get Hermes Agent configuration info (home path, version).
#[tauri::command(rename_all = "camelCase")]
pub fn get_hermes_config_info() -> Result<serde_json::Value, String> {
    let config_path = config_path();
    let hermes_home = config_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let installed = hermes_is_installed();
    let config_exists = config_path.exists();
    let version = if installed {
        // Try reading version from installed package
        std::process::Command::new("hermes")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    Ok(serde_json::json!({
        "hermesHome": hermes_home,
        "configExists": config_exists,
        "installed": installed,
        "version": version,
    }))
}

/// Export Hermes Agent config.yaml content as a string.
#[tauri::command(rename_all = "camelCase")]
pub fn export_hermes_config() -> Result<serde_json::Value, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(serde_json::json!({
            "success": true,
            "content": "",
            "message": "No config.yaml found",
        }));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(serde_json::json!({
        "success": true,
        "content": content,
    }))
}

/// Import Hermes Agent config.yaml from provided content.
#[tauri::command(rename_all = "camelCase")]
pub fn import_hermes_config(content: String) -> Result<serde_json::Value, String> {
    let path = config_path();
    // Ensure .hermes directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    // Validate YAML before writing
    serde_yaml::from_str::<serde_yaml::Value>(&content)
        .map_err(|e| format!("Invalid YAML content: {}", e))?;
    // Write atomically
    let tmp_path = path.with_extension("yaml.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("Failed to write temp config: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename config: {}", e))?;
    Ok(serde_json::json!({
        "success": true,
        "message": "Config imported successfully",
    }))
}

/// Set a specific config key in ~/.hermes/config.yaml using dot-notation.
/// e.g. key="agent.service_tier", value="fast"
#[tauri::command(rename_all = "camelCase")]
pub fn hermes_set_config(key: String, value: serde_json::Value) -> Result<serde_json::Value, String> {
    let path = config_path();
    let yaml_content = if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("读取配置失败: {e}"))?
    } else {
        String::new()
    };

    let mut root: serde_yaml::Value = if yaml_content.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&yaml_content)
            .map_err(|e| format!("解析 config.yaml 失败: {e}"))?
    };

    // 将 JSON value 转为 YAML value
    fn json_to_yaml(v: &serde_json::Value) -> serde_yaml::Value {
        match v {
            serde_json::Value::Null => serde_yaml::Value::Null,
            serde_json::Value::Bool(b) => serde_yaml::Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    serde_yaml::Value::Number(i.into())
                } else if let Some(f) = n.as_f64() {
                    serde_yaml::Value::Number(serde_yaml::Number::from(f))
                } else {
                    serde_yaml::Value::Null
                }
            }
            serde_json::Value::String(s) => serde_yaml::Value::String(s.clone()),
            serde_json::Value::Array(arr) => {
                serde_yaml::Value::Sequence(arr.iter().map(json_to_yaml).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut m = serde_yaml::Mapping::new();
                for (k, v) in obj {
                    m.insert(serde_yaml::Value::String(k.clone()), json_to_yaml(v));
                }
                serde_yaml::Value::Mapping(m)
            }
        }
    }

    // 按 dot-notation 路径导航（如 "agent.service_tier"）
    let keys: Vec<&str> = key.split('.').collect();
    let mut current = &mut root;
    for k in &keys[..keys.len() - 1] {
        let entry = current
            .as_mapping_mut()
            .ok_or(format!("配置路径 '{key}' 中间节点不是 mapping"))?
            .entry(serde_yaml::Value::String(k.to_string()))
            .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        current = entry;
    }

    // 设置最后一个 key
    let last_key = keys.last().ok_or("空 key")?;
    current
        .as_mapping_mut()
        .ok_or(format!("配置路径 '{key}' 的父节点不是 mapping"))?
        .insert(serde_yaml::Value::String(last_key.to_string()), json_to_yaml(&value));

    // 原子写入
    let new_content = serde_yaml::to_string(&root)
        .map_err(|e| format!("序列化配置失败: {e}"))?;
    let tmp_path = path.with_extension("yaml.tmp");
    std::fs::write(&tmp_path, &new_content)
        .map_err(|e| format!("写入配置失败: {e}"))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("更新配置失败: {e}"))?;

    Ok(serde_json::json!({ "success": true, "key": key }))
}

/// Read and parse ~/.hermes/config.yaml as a generic YAML value.
fn read_config_yaml() -> Result<serde_yaml::Value, String> {
    let path = config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.yaml: {}", e))
}
