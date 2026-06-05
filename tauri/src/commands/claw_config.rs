//! Claw Agent configuration — read/write API key, base URL, model.
//!
//! Persists to `~/.claw/config.json`.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Claw 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // ── Agent behavior settings ──

    /// Maximum tool loop iterations per turn (default: 25, CLI: usize::MAX)
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// Maximum bytes for Hermes skills injection into system prompt (default: 200KB)
    #[serde(default = "default_skill_bytes_cap")]
    pub skill_bytes_cap: u32,
    /// Maximum retries on transient streaming errors (default: 1)
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Reasoning effort level: "low", "medium", "high", or empty (default)
    #[serde(default)]
    pub reasoning_effort: String,
    /// Tool output truncation threshold in chars (default: 100000)
    #[serde(default = "default_tool_output_truncation")]
    pub tool_output_truncation: u32,
    /// Enable auto-compaction of old messages when context is large (default: true)
    #[serde(default = "default_auto_compaction")]
    pub auto_compaction: bool,
}

impl Default for ClawConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::new(),
            model: default_model(),
            provider: String::new(),
            max_iterations: default_max_iterations(),
            skill_bytes_cap: default_skill_bytes_cap(),
            max_retries: default_max_retries(),
            reasoning_effort: String::new(),
            tool_output_truncation: default_tool_output_truncation(),
            auto_compaction: default_auto_compaction(),
        }
    }
}

fn default_model() -> String {
    "claude-sonnet-4-6".to_string()
}
fn default_max_iterations() -> u32 {
    25
}
fn default_skill_bytes_cap() -> u32 {
    200 * 1024 // 200KB
}
fn default_max_retries() -> u32 {
    1
}
fn default_tool_output_truncation() -> u32 {
    100_000 // 100K chars
}
fn default_auto_compaction() -> bool {
    true
}

/// 配置文件路径
fn config_path() -> PathBuf {
    // Use upstream's config_home (supports CLAW_CONFIG_HOME env var)
    let config_home = std::env::var_os("CLAW_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".claw")))
        .unwrap_or_else(|| PathBuf::from("~/.claw"));
    config_home.join("settings.json")
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
        "maxIterations": config.max_iterations,
        "skillBytesCap": config.skill_bytes_cap,
        "maxRetries": config.max_retries,
        "reasoningEffort": config.reasoning_effort,
        "toolOutputTruncation": config.tool_output_truncation,
        "autoCompaction": config.auto_compaction,
    }))
}

/// 保存 Claw 配置（Tauri command）
#[tauri::command(rename_all = "camelCase")]
pub fn claw_config_set(
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    max_iterations: Option<u32>,
    skill_bytes_cap: Option<u32>,
    max_retries: Option<u32>,
    reasoning_effort: Option<String>,
    tool_output_truncation: Option<u32>,
    auto_compaction: Option<bool>,
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
    if let Some(v) = max_iterations {
        config.max_iterations = v.clamp(1, 200);
    }
    if let Some(v) = skill_bytes_cap {
        config.skill_bytes_cap = v.clamp(10 * 1024, 2 * 1024 * 1024); // 10KB - 2MB
    }
    if let Some(v) = max_retries {
        config.max_retries = v.min(10);
    }
    if let Some(v) = reasoning_effort {
        config.reasoning_effort = v;
    }
    if let Some(v) = tool_output_truncation {
        config.tool_output_truncation = v.clamp(10_000, 10_000_000); // 10K - 10M chars
    }
    if let Some(v) = auto_compaction {
        config.auto_compaction = v;
    }

    write_claw_config(&config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": "Claw config saved",
    }))
}
