//! Claw Agent configuration — read/write API key, base URL, model.
//!
//! Persists to `~/.claw/config.json`.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Claw 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClawConfig {
    /// API Key
    pub api_key: String,
    /// Base URL (OpenAI-compatible endpoint, e.g. https://api.openai.com/v1)
    #[serde(default)]
    pub base_url: String,
    /// Model name (e.g. claude-sonnet-4-6, gpt-4.1-mini)
    #[serde(default = "default_model")]
    pub model: String,
    /// Provider label (for display only, routing is automatic)
    #[serde(default)]
    pub provider: String,
}

fn default_model() -> String {
    "claude-sonnet-4-6".to_string()
}

/// 配置文件路径
fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("config.json")
}

/// 读取 Claw 配置
pub fn read_claw_config() -> Result<ClawConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(ClawConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse ~/.claw/config.json: {e}"))
}

/// 写入 Claw 配置
pub fn write_claw_config(config: &ClawConfig) -> Result<(), String> {
    let path = config_path();
    let parent = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory: {e}"))?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    log::info!("[claw_config] Saved config to {}", path.display());
    Ok(())
}

/// 获取 Claw 配置（Tauri command）
#[tauri::command(rename_all = "camelCase")]
pub fn claw_config_get() -> Result<serde_json::Value, String> {
    let config = read_claw_config()?;
    Ok(serde_json::json!({
        // !!! CRITICAL: return the RAW key, NEVER mask it in the backend !!!
        // Backend masking caused a hard-to-find bug: the frontend saves
        // the masked key back to disk, corrupting the real key.
        // The frontend is responsible for display-level masking only.
        "apiKey": config.api_key,
        "hasApiKey": !config.api_key.is_empty(),
        "baseUrl": config.base_url,
        "model": config.model,
        "provider": config.provider,
    }))
}

/// 保存 Claw 配置（Tauri command）
#[tauri::command(rename_all = "camelCase")]
pub fn claw_config_set(
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut config = read_claw_config()?;

    if let Some(key) = api_key {
        config.api_key = key;
    }
    if let Some(url) = base_url {
        config.base_url = url;
    }
    if let Some(m) = model {
        config.model = m;
    }
    if let Some(p) = provider {
        config.provider = p;
    }

    write_claw_config(&config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Claw config saved",
    }))
}
