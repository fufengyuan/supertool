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

use serde::{Deserialize, Serialize};
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

/// Get path to Hermes config.yaml
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
        .join("config.yaml")
}

/// Read Hermes config.yaml, return default model + custom models list
fn read_config() -> Result<HermesConfig, String> {
    let path = config_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.yaml: {}", e))
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

/// Get custom models and default model from Hermes config
pub fn get_models() -> Result<serde_json::Value, String> {
    let config = read_config()?;

    let default_model = config
        .model
        .as_ref()
        .and_then(|m| m.default.as_deref().or(m.model.as_deref()))
        .unwrap_or("")
        .to_string();

    // 读取活跃供应商（仅显示该供应商的模型）
    let active_provider = config
        .model
        .as_ref()
        .and_then(|m| m.provider.as_deref())
        .unwrap_or("")
        .to_string();

    Ok(serde_json::json!({
        "customModels": config.custom_models,
        "defaultModel": default_model,
        "activeProvider": active_provider,
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
pub fn set_default_model(model: String) -> Result<serde_json::Value, String> {
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
