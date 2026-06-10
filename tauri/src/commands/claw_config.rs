//! Claw Agent configuration — read/write API key, base URL, model.
//!
//! Persists to `~/.claw/settings.json`.
//!
//! ## Format compatibility
//!
//! **On disk** the file uses the upstream Claw CLI format:
//! ```json
//! {
//!   "provider": { "kind": "openai", "apiKey": "...", "baseUrl": "..." },
//!   "model": "claude-sonnet-4-6",
//!   "maxIterations": 25,
//!   ...
//! }
//! ```
//!
//! **In memory / frontend** we keep flat fields (provider as string) for simplicity.
//! `read_claw_config` handles both formats (old flat → auto-migrate),
//! `write_claw_config` always writes the upstream-compatible nested format.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Claw 配置结构（内存表示 — flat fields for frontend simplicity）
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
/// Uses the exact same resolution as upstream ConfigLoader::default_config_home():
/// 1. $CLAW_CONFIG_HOME if set
/// 2. $HOME/.claw (using std::env::var_os("HOME"), matching upstream behavior)
/// 3. .claw (relative fallback)
fn config_path() -> PathBuf {
    let config_home = std::env::var_os("CLAW_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".claw")))
        .unwrap_or_else(|| PathBuf::from(".claw"));
    config_home.join("settings.json")
}

/// Parse `provider` from a raw JSON value, handling both flat-string and nested-object formats.
///
/// Upstream format (nested object — compatible with ConfigLoader):
/// ```json
/// { "provider": { "kind": "openai", "apiKey": "...", "baseUrl": "..." } }
/// ```
///
/// Legacy format (flat string — our old format):
/// ```json
/// { "provider": "openai", "api_key": "...", "base_url": "..." }
/// ```
fn parse_provider(value: &serde_json::Value) -> (String, String, String) {
    match value.get("provider") {
        // Upstream format: provider is an object with kind/apiKey/baseUrl
        Some(serde_json::Value::Object(obj)) => {
            let kind = obj
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let api_key = obj
                .get("apiKey")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let base_url = obj
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (kind, api_key, base_url)
        }
        // Legacy format: provider is a flat string; api_key/base_url at root
        Some(serde_json::Value::String(s)) => {
            let api_key = value
                .get("api_key")
                .or_else(|| value.get("apiKey"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let base_url = value
                .get("base_url")
                .or_else(|| value.get("baseUrl"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (s.clone(), api_key, base_url)
        }
        // No provider field at all — fall back to root-level fields
        None => {
            let api_key = value
                .get("api_key")
                .or_else(|| value.get("apiKey"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let base_url = value
                .get("base_url")
                .or_else(|| value.get("baseUrl"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (String::new(), api_key, base_url)
        }
        // Unexpected type — return defaults
        _ => (String::new(), String::new(), String::new()),
    }
}

/// 读取 Claw 配置（兼容新旧格式）
pub fn read_claw_config() -> Result<ClawConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(ClawConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;

    // Parse provider from both formats
    let (provider, api_key, base_url) = parse_provider(&value);

    let model = value
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-sonnet-4-6")
        .to_string();

    // Agent behavior fields use camelCase in Upstram format, snake_case in legacy
    fn get_u32(value: &serde_json::Value, keys: &[&str], default: u32) -> u32 {
        for key in keys {
            if let Some(v) = value.get(*key).and_then(|v| v.as_u64()) {
                return v as u32;
            }
        }
        default
    }
    fn get_string(value: &serde_json::Value, keys: &[&str], default: &str) -> String {
        for key in keys {
            if let Some(v) = value.get(*key).and_then(|v| v.as_str()) {
                return v.to_string();
            }
        }
        default.to_string()
    }
    fn get_bool(value: &serde_json::Value, keys: &[&str], default: bool) -> bool {
        for key in keys {
            if let Some(v) = value.get(*key).and_then(|v| v.as_bool()) {
                return v;
            }
        }
        default
    }

    Ok(ClawConfig {
        api_key,
        base_url,
        model,
        provider,
        max_iterations: get_u32(&value, &["maxIterations", "max_iterations"], 25),
        skill_bytes_cap: get_u32(&value, &["skillBytesCap", "skill_bytes_cap"], 200 * 1024),
        max_retries: get_u32(&value, &["maxRetries", "max_retries"], 1),
        reasoning_effort: get_string(&value, &["reasoningEffort", "reasoning_effort"], ""),
        tool_output_truncation: get_u32(
            &value,
            &["toolOutputTruncation", "tool_output_truncation"],
            100_000,
        ),
        auto_compaction: get_bool(&value, &["autoCompaction", "auto_compaction"], true),
    })
}

/// 写入 Claw 配置（始终使用上游兼容的嵌套格式）
pub fn write_claw_config(config: &ClawConfig) -> Result<(), String> {
    let path = config_path();
    let parent = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    // Build provider object in upstream format
    let mut provider_obj = serde_json::Map::new();
    if !config.provider.is_empty() {
        provider_obj.insert(
            "kind".to_string(),
            serde_json::Value::String(config.provider.clone()),
        );
    }
    if !config.api_key.is_empty() {
        provider_obj.insert(
            "apiKey".to_string(),
            serde_json::Value::String(config.api_key.clone()),
        );
    }
    if !config.base_url.is_empty() {
        provider_obj.insert(
            "baseUrl".to_string(),
            serde_json::Value::String(config.base_url.clone()),
        );
    }

    let mut root = serde_json::Map::new();
    root.insert(
        "provider".to_string(),
        serde_json::Value::Object(provider_obj),
    );
    root.insert(
        "model".to_string(),
        serde_json::Value::String(config.model.clone()),
    );
    root.insert(
        "maxIterations".to_string(),
        serde_json::Value::Number(serde_json::Number::from(config.max_iterations)),
    );
    root.insert(
        "skillBytesCap".to_string(),
        serde_json::Value::Number(serde_json::Number::from(config.skill_bytes_cap)),
    );
    root.insert(
        "maxRetries".to_string(),
        serde_json::Value::Number(serde_json::Number::from(config.max_retries)),
    );
    root.insert(
        "reasoningEffort".to_string(),
        serde_json::Value::String(config.reasoning_effort.clone()),
    );
    root.insert(
        "toolOutputTruncation".to_string(),
        serde_json::Value::Number(serde_json::Number::from(config.tool_output_truncation)),
    );
    root.insert(
        "autoCompaction".to_string(),
        serde_json::Value::Bool(config.auto_compaction),
    );

    let content = serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    log::info!("[claw_config] Saved config to {}", path.display());
    Ok(())
}

/// 获取 Claw 配置（Tauri command — 展平为前端期望的格式）
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

// ── Permission Mode ──────────────────────────────────────────────────────

/// Valid permission mode values.
const PERMISSION_MODES: &[&str] = &["allow", "ask", "deny"];

/// Tauri command: get the current permission mode from settings.json.
///
/// Reads `permissions.mode` from `~/.claw/settings.json`. Returns
/// the mode string or the default "ask" if not set.
#[tauri::command(rename_all = "camelCase")]
pub fn claw_get_permission_mode() -> Result<serde_json::Value, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(serde_json::json!({
            "mode": "ask",
            "configured": false,
        }));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;

    let mode = value
        .get("permissions")
        .and_then(|p| p.as_object())
        .and_then(|p| p.get("mode"))
        .and_then(|m| m.as_str())
        .unwrap_or("ask")
        .to_string();

    Ok(serde_json::json!({
        "mode": mode,
        "configured": mode != "ask",
    }))
}

/// Tauri command: set the permission mode in settings.json.
///
/// Writes `permissions.mode` to `~/.claw/settings.json`, preserving
/// all existing settings. Supports: "allow", "ask", "deny".
#[tauri::command(rename_all = "camelCase")]
pub fn claw_set_permission_mode(mode: String) -> Result<serde_json::Value, String> {
    if !PERMISSION_MODES.contains(&mode.as_str()) {
        return Err(format!(
            "Invalid permission mode '{}'. Supported: {}",
            mode,
            PERMISSION_MODES.join(", ")
        ));
    }

    let path = config_path();

    // Read existing settings.json or start with empty object
    let mut value: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Set permissions.mode
    let permissions = value
        .as_object_mut()
        .ok_or("settings.json root must be an object")?;
    permissions
        .entry("permissions")
        .or_insert(serde_json::json!({}))
        .as_object_mut()
        .ok_or("'permissions' must be an object")?
        .insert("mode".to_string(), serde_json::Value::String(mode.clone()));

    // Write back
    let parent = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to serialize: {e}"))?;
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;

    log::info!(
        "[claw_config] Permission mode set to '{}' in {}",
        mode,
        path.display()
    );

    Ok(serde_json::json!({
        "success": true,
        "mode": mode,
    }))
}
