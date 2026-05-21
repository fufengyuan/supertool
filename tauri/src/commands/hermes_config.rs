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

/// Predefined model lists for common providers (from models.dev)
/// These are the most commonly used models for each provider.
const PROVIDER_MODELS: &[(&str, &[&str])] = &[
    // OpenAI
    ("openai", &[
        "gpt-4.1", "gpt-4.1-mini", "gpt-4.1-nano",
        "gpt-4o", "gpt-4o-mini", "gpt-4o-audio-preview",
        "gpt-4-turbo", "gpt-4", "gpt-4-32k",
        "gpt-3.5-turbo", "gpt-3.5-turbo-16k",
        "o1", "o1-mini", "o1-pro", "o3", "o3-mini", "o4-mini",
    ]),
    // Anthropic
    ("anthropic", &[
        "claude-opus-4-6", "claude-opus-4-5", "claude-opus-4-1-20250514", "claude-opus-4",
        "claude-sonnet-4-5", "claude-sonnet-4", "claude-3-7-sonnet",
        "claude-3-5-sonnet", "claude-3-5-sonnet-v2", "claude-3-5-haiku",
        "claude-3-haiku", "claude-3-opus", "claude-3-sonnet",
    ]),
    // Google / Gemini
    ("google", &[
        "gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.5-flash-lite",
        "gemini-2.0-pro", "gemini-2.0-flash", "gemini-2.0-flash-lite",
        "gemini-1.5-pro", "gemini-1.5-flash", "gemini-1.5-flash-8b",
        "gemini-1.0-pro", "gemini-pro",
    ]),
    ("gemini", &[
        "gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.5-flash-lite",
        "gemini-2.0-pro", "gemini-2.0-flash", "gemini-2.0-flash-lite",
        "gemini-1.5-pro", "gemini-1.5-flash", "gemini-1.5-flash-8b",
        "gemini-1.0-pro", "gemini-pro",
    ]),
    // Alibaba / Qwen / GLM
    ("alibaba", &[
        "glm-5", "glm-4-plus", "glm-4-air", "glm-4-airx", "glm-4-flash", "glm-4-long",
        "qwen3-235b-a22b", "qwen3-32b", "qwen3-14b", "qwen3-8b", "qwen3-4b",
        "qwen-max", "qwen-max-latest", "qwen-plus", "qwen-turbo", "qwen-long",
        "qwen2.5-max", "qwen2.5-plus", "qwen2.5-turbo", "qwen2.5-72b", "qwen2.5-32b",
    ]),
    ("qwen", &[
        "glm-5", "glm-4-plus", "glm-4-air", "glm-4-airx", "glm-4-flash", "glm-4-long",
        "qwen3-235b-a22b", "qwen3-32b", "qwen3-14b", "qwen3-8b", "qwen3-4b",
        "qwen-max", "qwen-max-latest", "qwen-plus", "qwen-turbo", "qwen-long",
        "qwen2.5-max", "qwen2.5-plus", "qwen2.5-turbo", "qwen2.5-72b", "qwen2.5-32b",
    ]),
    // DeepSeek
    ("deepseek", &[
        "deepseek-r1", "deepseek-r1-0528", "deepseek-reasoner",
        "deepseek-v3", "deepseek-v3-0324", "deepseek-chat",
        "deepseek-coder", "deepseek-prover-v2",
    ]),
    // xAI (Grok)
    ("x-ai", &[
        "grok-3", "grok-3-fast", "grok-3-mini", "grok-3-mini-fast",
        "grok-2-1212", "grok-2-vision-1212", "grok-beta",
    ]),
    ("xai", &[
        "grok-3", "grok-3-fast", "grok-3-mini", "grok-3-mini-fast",
        "grok-2-1212", "grok-2-vision-1212", "grok-beta",
    ]),
    // Mistral AI
    ("mistral", &[
        "mistral-large-2", "mistral-large", "mistral-medium",
        "mistral-small", "mistral-small-3", "mistral-small-2501",
        "codestral", "codestral-2501", "ministral-8b", "ministral-3b",
        "pixtral-12b", "pixtral-large",
    ]),
    // Meta (Llama)
    ("meta", &[
        "llama-4-maverick", "llama-4-scout",
        "llama-3.3-70b", "llama-3.2-90b", "llama-3.2-11b", "llama-3.2-3b", "llama-3.2-1b",
        "llama-3.1-405b", "llama-3.1-70b", "llama-3.1-8b",
        "llama-guard-3", "llama-guard-4",
    ]),
    // OpenRouter (aggregator - shows models from multiple providers)
    ("openrouter", &[
        "anthropic/claude-opus-4", "anthropic/claude-sonnet-4", "anthropic/claude-3.5-sonnet",
        "openai/gpt-4o", "openai/gpt-4o-mini", "openai/o1", "openai/o3-mini",
        "google/gemini-2.5-pro", "google/gemini-2.5-flash",
        "deepseek/deepseek-r1", "deepseek/deepseek-v3",
        "meta-llama/llama-4-maverick", "meta-llama/llama-4-scout",
        "mistral/mistral-large", "x-ai/grok-3",
    ]),
    // MiniMax
    ("minimax", &[
        "mini-max-01", "abab-6.5s-chat", "abab-6.5g-chat", "abab-6.5t-chat",
        "abab-5.5-chat", "abab-5.5s-chat", "speech-01-turbo",
    ]),
    ("minimax-cn", &[
        "mini-max-01", "abab-6.5s-chat", "abab-6.5g-chat", "abab-6.5t-chat",
        "abab-5.5-chat", "abab-5.5s-chat", "speech-01-turbo",
    ]),
    // Moonshot / Kimi
    ("moonshot", &["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k", "kimi-latest"]),
    ("kimi", &["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k", "kimi-latest"]),
    ("kimi-coding", &["kimi-latest", "kimi-dev", "kimi-code"]),
    // StepFun
    ("stepfun", &["step-2-16k", "step-1-8k", "step-1v-8k"]),
    // Cohere
    ("cohere", &["command-r-plus", "command-r", "command-a", "command", "command-light"]),
    // Nous Portal
    ("nous", &["nous-hermes-2-mixtral", "nous-capye", "nous-booste"]),
    // NVIDIA NIM
    ("nvidia", &["meta/llama-3.1-405b", "meta/llama-3.1-70b", "mistral/mistral-large"]),
];

/// Get custom models, default model, and provider models from Hermes config
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

    // 获取活跃供应商的预定义模型列表
    let prefix = if active_provider == "openrouter" {
        String::new()
    } else {
        format!("{}/", active_provider)
    };
    let provider_models: Vec<String> = PROVIDER_MODELS
        .iter()
        .find(|(p, _)| *p == active_provider)
        .map(|(_, models)| models.iter().map(|m| format!("{}{}", prefix, m)).collect())
        .unwrap_or_default();

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
