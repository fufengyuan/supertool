//! Hermes config management — using hermes-config ultra crate.
//!
//! Replaces ad-hoc path resolution and YAML manipulation with
//! hermes_config::paths and hermes_config::loader APIs.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use hermes_config::paths;

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

fn models_cache_path() -> PathBuf {
    paths::hermes_home().join("models_dev_cache.json")
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

/// Check if Hermes Agent is installed
pub fn hermes_is_installed() -> bool {
    let home = paths::hermes_home();
    home.join("hermes-agent").join("run_agent.py").exists()
}

/// Get custom models, default model, and all available provider models
pub fn get_models() -> Result<serde_json::Value, String> {
    let config_path = paths::config_path();
    let content = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        return Ok(serde_json::json!({
            "customModels": Vec::<String>::new(),
            "defaultModel": "",
            "activeProvider": "",
            "providerModels": Vec::<String>::new(),
        }));
    };

    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let default_model = yaml["model"]["default"]
        .as_str()
        .or_else(|| yaml["model"]["model"].as_str())
        .unwrap_or("")
        .to_string();

    let active_provider = yaml["model"]["provider"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let custom_models: Vec<String> = yaml["custom_models"]
        .as_sequence()
        .map(|seq| seq.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // From models.dev cache
    let cache = read_models_cache().unwrap_or_default();
    let mut provider_models: Vec<String> = Vec::new();
    for (provider_id, entry) in &cache {
        for model_id in entry.models.keys() {
            provider_models.push(format!("{}/{}", provider_id, model_id));
        }
    }

    Ok(serde_json::json!({
        "customModels": custom_models,
        "defaultModel": default_model,
        "activeProvider": active_provider,
        "providerModels": provider_models,
    }))
}

/// Read raw YAML from config path
fn read_config_yaml() -> Result<serde_yaml::Value, String> {
    let path = paths::config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.yaml: {}", e))
}

/// Write YAML back to config path atomically (via hermes-config loader)
fn write_config_yaml(root: &serde_yaml::Value) -> Result<(), String> {
    let path = paths::config_path();
    let content = serde_yaml::to_string(root)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    let tmp = path.with_extension("yaml.tmp");
    std::fs::write(&tmp, &content)
        .map_err(|e| format!("Failed to write temp config: {}", e))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| format!("Failed to rename config: {}", e))?;
    Ok(())
}

// ── Model management ──────────────────────────────────────────

/// Add a model to Hermes config
pub fn add_model(model: String) -> Result<serde_json::Value, String> {
    let content = read_config_yaml()?;
    let mut map = content.as_mapping().cloned().unwrap_or_default();
    let mut custom: Vec<serde_yaml::Value> = map
        .remove(&serde_yaml::Value::String("custom_models".into()))
        .and_then(|v| v.as_sequence().cloned())
        .unwrap_or_default();

    if custom.iter().any(|v| v.as_str() == Some(&model)) {
        return Err(format!("Model '{}' already exists", model));
    }
    custom.push(serde_yaml::Value::String(model.clone()));

    map.insert(serde_yaml::Value::String("custom_models".into()), serde_yaml::Value::Sequence(custom.clone()));
    write_config_yaml(&serde_yaml::Value::Mapping(map))?;

    let models: Vec<String> = custom.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    Ok(serde_json::json!({
        "success": true,
        "model": model,
        "customModels": models,
    }))
}

/// Remove a model from Hermes config
pub fn remove_model(model: String) -> Result<serde_json::Value, String> {
    let content = read_config_yaml()?;
    let mut map = content.as_mapping().cloned().unwrap_or_default();
    let mut custom: Vec<serde_yaml::Value> = map
        .remove(&serde_yaml::Value::String("custom_models".into()))
        .and_then(|v| v.as_sequence().cloned())
        .unwrap_or_default();

    let pos = custom.iter().position(|v| v.as_str() == Some(&model))
        .ok_or_else(|| format!("Model '{}' not found", model))?;
    custom.remove(pos);

    map.insert(serde_yaml::Value::String("custom_models".into()), serde_yaml::Value::Sequence(custom.clone()));
    write_config_yaml(&serde_yaml::Value::Mapping(map))?;

    let models: Vec<String> = custom.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    Ok(serde_json::json!({
        "success": true,
        "model": model,
        "customModels": models,
    }))
}

/// Set the default model in Hermes config
pub fn set_default_model(model: String) -> Result<serde_json::Value, String> {
    if model.is_empty() {
        return Ok(serde_json::json!({ "success": true, "model": "" }));
    }

    let content = read_config_yaml()?;
    let mut map = content.as_mapping().cloned().unwrap_or_default();

    let model_entry = map
        .entry(serde_yaml::Value::String("model".into()))
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

    if let Some(m) = model_entry.as_mapping_mut() {
        m.insert(serde_yaml::Value::String("default".into()), serde_yaml::Value::String(model.clone()));
    }

    write_config_yaml(&serde_yaml::Value::Mapping(map))?;
    Ok(serde_json::json!({ "success": true, "model": model }))
}

// ── API Server Configuration ──────────────────────────────────

/// Check if Hermes API server is configured
pub fn check_api_server_config() -> (bool, bool, String) {
    let env_path = paths::env_path();
    if !env_path.exists() {
        return (false, false, String::new());
    }

    let content = std::fs::read_to_string(&env_path).unwrap_or_default();
    let enabled = content.lines().any(|line| {
        let l = line.trim();
        l.starts_with("API_SERVER_ENABLED=true")
            || l.starts_with("API_SERVER_ENABLED=1")
            || l.starts_with("API_SERVER_ENABLED=yes")
    });

    let key_line = content.lines().find(|line| line.trim().starts_with("API_SERVER_KEY="));
    let has_key = key_line.is_some();
    let key = key_line
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    (enabled, has_key, key)
}

fn generate_api_key() -> String {
    "supertool-local-key".to_string()
}

/// Ensure Hermes API server is configured with an API key
pub fn ensure_api_server_config() -> Result<String, String> {
    let (enabled, has_key, existing_key) = check_api_server_config();
    if enabled && has_key && !existing_key.is_empty() {
        return Ok(existing_key);
    }

    let env_path = paths::env_path();
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };

    let new_key = generate_api_key();
    let mut lines: Vec<String> = existing_content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("API_SERVER_ENABLED=") && !t.starts_with("API_SERVER_KEY=")
        })
        .map(String::from)
        .collect();

    lines.push("API_SERVER_ENABLED=true".to_string());
    lines.push(format!("API_SERVER_KEY={}", new_key));

    let hermes_dir = env_path.parent().unwrap();
    std::fs::create_dir_all(hermes_dir)
        .map_err(|e| format!("Failed to create .hermes directory: {}", e))?;

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    std::fs::write(&env_path, &new_content)
        .map_err(|e| format!("Failed to write .env: {}", e))?;

    log::info!("[hermes_config] Auto-configured Hermes API server");

    // Restart gateway
    let _ = std::process::Command::new("/bin/bash")
        .args(["-l", "-c", "hermes gateway restart"])
        .output();

    Ok(new_key)
}

fn set_api_server_key(key: &str) -> Result<(), String> {
    let env_path = paths::env_path();
    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut lines: Vec<String> = existing_content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("API_SERVER_ENABLED=") && !t.starts_with("API_SERVER_KEY=")
        })
        .map(String::from)
        .collect();

    lines.push("API_SERVER_ENABLED=true".to_string());
    lines.push(format!("API_SERVER_KEY={}", key));

    let hermes_dir = env_path.parent().unwrap();
    std::fs::create_dir_all(hermes_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    std::fs::write(&env_path, &new_content)
        .map_err(|e| format!("Failed to write .env: {}", e))?;

    let _ = std::process::Command::new("/bin/bash")
        .args(["-l", "-c", "hermes gateway restart"])
        .output();

    Ok(())
}

// ── Tauri Commands ────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn agent_api_server_status() -> Result<serde_json::Value, String> {
    let installed = hermes_is_installed();
    let (enabled, has_key, key) = check_api_server_config();

    let running = if installed && enabled {
        reqwest::blocking::Client::builder()
            .no_proxy()
            .build()
            .ok()
            .and_then(|c| {
                c.get("http://localhost:8642/health")
                    .timeout(std::time::Duration::from_secs(2))
                    .send()
                    .ok()
            })
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
        "api_key": key,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn agent_configure_api_server(custom_key: Option<String>) -> Result<serde_json::Value, String> {
    let key = if let Some(custom) = custom_key {
        set_api_server_key(&custom)?;
        custom
    } else {
        ensure_api_server_config()?
    };

    Ok(serde_json::json!({
        "success": true,
        "apiKey": key,
        "message": "Hermes API server configured. Gateway will be restarted.",
    }))
}

// ── Toolset Management ────────────────────────────────────────

pub(crate) const ALL_TOOLSETS: &[(&str, &str, &str)] = &[
    ("web",            "Web",            "Web search and capture"),
    ("browser",        "Browser",        "Web browsing"),
    ("terminal",       "Terminal",       "Shell commands"),
    ("file",           "File",           "Read/write files"),
    ("code_execution", "Code Execution", "Execute Python code"),
    ("vision",         "Vision",         "Image analysis"),
    ("image_gen",      "Image Gen",      "Generate images"),
    ("tts",            "TTS",            "Text to speech"),
    ("skills",         "Skills",         "Load and manage skills"),
    ("memory",         "Memory",         "Persistent memory"),
    ("session_search", "Session Search", "Search history"),
    ("clarify",        "Clarify",        "Ask questions"),
    ("delegation",     "Delegation",     "Spawn sub-agents"),
    ("cronjob",        "Cron Job",       "Schedule tasks"),
    ("moa",            "MOA",            "Mixture of Agents"),
    ("todo",           "Todo",           "Task list"),
];

pub(crate) fn read_enabled_toolsets_from_value(root: &serde_yaml::Value) -> Option<Vec<String>> {
    let platform = root.get("platform_toolsets")?;
    let cli = platform.get("cli")?;
    let arr = cli.as_sequence()?;
    if arr.is_empty() {
        return None;
    }
    let keys: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    if keys.is_empty() { None } else { Some(keys) }
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_toolsets() -> Result<serde_json::Value, String> {
    let enabled_keys = read_config_yaml()
        .ok()
        .and_then(|root| read_enabled_toolsets_from_value(&root));

    let toolsets: Vec<serde_json::Value> = ALL_TOOLSETS
        .iter()
        .map(|(key, label, desc)| {
            let enabled = enabled_keys
                .as_ref()
                .map(|keys| keys.contains(&key.to_string()))
                .unwrap_or(true);
            serde_json::json!({
                "key": key,
                "label": label,
                "description": desc,
                "enabled": enabled,
            })
        })
        .collect();

    Ok(serde_json::json!({ "toolsets": toolsets }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_toolset_enabled(key: String, enabled: bool) -> Result<serde_json::Value, String> {
    let path = paths::config_path();
    let yaml_content = if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        String::new()
    };

    let mut root: serde_yaml::Value = if yaml_content.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&yaml_content)
            .map_err(|e| format!("Failed to parse config.yaml: {}", e))?
    };

    let platform = root.as_mapping_mut().ok_or("Config root is not a mapping")?;

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
        if !cli_seq.iter().any(|v| v.as_str() == Some(&key)) {
            cli_seq.push(serde_yaml::Value::String(key));
        }
    } else {
        if cli_seq.is_empty() {
            for (tk, _, _) in ALL_TOOLSETS {
                cli_seq.push(serde_yaml::Value::String(tk.to_string()));
            }
        }
        cli_seq.retain(|v| v.as_str() != Some(&key));
    }

    write_config_yaml(&root)?;
    Ok(serde_json::json!({ "success": true }))
}

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
                        let detail = if args_str.is_empty() { cmd.to_string() } else { format!("{} {}", cmd, args_str) };
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

#[tauri::command(rename_all = "camelCase")]
pub fn get_hermes_config_info() -> Result<serde_json::Value, String> {
    let cfg_path = paths::config_path();
    let home = cfg_path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let installed = hermes_is_installed();
    let config_exists = cfg_path.exists();
    let version = if installed {
        std::process::Command::new("hermes")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    Ok(serde_json::json!({
        "hermesHome": home,
        "configExists": config_exists,
        "installed": installed,
        "version": version,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn export_hermes_config() -> Result<serde_json::Value, String> {
    let path = paths::config_path();
    if !path.exists() {
        return Ok(serde_json::json!({ "success": true, "content": "", "message": "No config.yaml found" }));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    Ok(serde_json::json!({ "success": true, "content": content }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn import_hermes_config(content: String) -> Result<serde_json::Value, String> {
    let path = paths::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    // Validate YAML
    serde_yaml::from_str::<serde_yaml::Value>(&content)
        .map_err(|e| format!("Invalid YAML: {}", e))?;
    let tmp = path.with_extension("yaml.tmp");
    std::fs::write(&tmp, &content).map_err(|e| format!("Failed to write: {}", e))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("Failed to rename: {}", e))?;
    Ok(serde_json::json!({ "success": true, "message": "Config imported successfully" }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn hermes_set_config(key: String, value: serde_json::Value) -> Result<serde_json::Value, String> {
    let path = paths::config_path();
    let yaml_content = if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?
    } else {
        String::new()
    };

    let mut root: serde_yaml::Value = if yaml_content.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&yaml_content).map_err(|e| format!("解析 config.yaml 失败: {e}"))?
    };

    fn json_to_yaml(v: &serde_json::Value) -> serde_yaml::Value {
        match v {
            serde_json::Value::Null => serde_yaml::Value::Null,
            serde_json::Value::Bool(b) => serde_yaml::Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() { serde_yaml::Value::Number(i.into()) }
                else if let Some(f) = n.as_f64() { serde_yaml::Value::Number(serde_yaml::Number::from(f)) }
                else { serde_yaml::Value::Null }
            }
            serde_json::Value::String(s) => serde_yaml::Value::String(s.clone()),
            serde_json::Value::Array(arr) => serde_yaml::Value::Sequence(arr.iter().map(json_to_yaml).collect()),
            serde_json::Value::Object(obj) => {
                let mut m = serde_yaml::Mapping::new();
                for (k, v) in obj { m.insert(serde_yaml::Value::String(k.clone()), json_to_yaml(v)); }
                serde_yaml::Value::Mapping(m)
            }
        }
    }

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

    let last_key = keys.last().ok_or("空 key")?;
    current
        .as_mapping_mut()
        .ok_or(format!("配置路径 '{key}' 的父节点不是 mapping"))?
        .insert(serde_yaml::Value::String(last_key.to_string()), json_to_yaml(&value));

    let new_content = serde_yaml::to_string(&root).map_err(|e| format!("序列化配置失败: {e}"))?;
    let tmp = path.with_extension("yaml.tmp");
    std::fs::write(&tmp, &new_content).map_err(|e| format!("写入配置失败: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("更新配置失败: {e}"))?;

    Ok(serde_json::json!({ "success": true, "key": key }))
}
