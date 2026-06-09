//! Agent Chat via ultra crate AgentLoop (direct in-process, no HTTP bridge)
//!
//! Uses hermes_agent::AgentLoop::run_stream() directly instead of
//! HTTP calls to a separate Hermes Gateway process.
//!
//! Event mapping (same names as before — frontend unchanged):
//!   agent-delta: content streaming
//!   agent-reasoning-delta: reasoning/thinking tokens
//!   agent-tool-start: tool call began
//!   agent-tool-complete: tool call finished
//!   agent-done: stream finished
//!   agent-error: error occurred
//!   agent-usage: token usage info

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use hermes_agent::agent_loop::ToolRegistry as AgentToolRegistry;
use hermes_agent::{
    attach_discovered_memory, AgentCallbacks, AgentConfig, AgentLoop,
};
use hermes_config::loader::load_config;
use hermes_config::GatewayConfig;
use hermes_core::{LlmProvider, Message, StreamChunk, ToolSchema};
use hermes_environments::BackendManager;
use hermes_skills::{FileSkillStore, SkillManager};
use serde_json::json;
use tauri::{AppHandle, Emitter};

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

lazy_static::lazy_static! {
    static ref ABORT_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref CURRENT_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
}

// ---------------------------------------------------------------------------
// Helpers — config / provider / tool building
// ---------------------------------------------------------------------------

/// Resolve provider name + model name from a `provider:model` string.
fn resolve_provider_and_model(config: &GatewayConfig, model: &str) -> (String, String) {
    let trimmed = model.trim();
    if let Some((provider, model_name)) = trimmed.split_once(':') {
        return (provider.trim().to_string(), model_name.trim().to_string());
    }
    if let Some((provider, _)) = config.llm_providers.iter().find(|(_, cfg)| {
        cfg.model.as_deref().map(str::trim).filter(|m| !m.is_empty()) == Some(trimmed)
    }) {
        return (provider.to_string(), trimmed.to_string());
    }
    if config.llm_providers.len() == 1 {
        if let Some((provider, _)) = config.llm_providers.iter().next() {
            return (provider.to_string(), trimmed.to_string());
        }
    }
    ("openai".to_string(), trimmed.to_string())
}

fn normalize_provider(name: &str) -> String {
    match name.trim().to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => "anthropic",
        "aws" | "aws-bedrock" | "amazon-bedrock" | "amazon" => "bedrock",
        "azure" => "azure-foundry",
        "step" | "step-plan" => "stepfun",
        "moonshot" | "kimi" => "kimi",
        "alibaba" | "alibaba-coding-plan" => "qwen",
        "github-copilot" | "github-models" => "copilot",
        "google" | "google-gemini" => "gemini",
        other => other,
    }
    .to_string()
}

/// Build an AgentConfig from Hermes config.yaml + model string.
fn build_agent_config(config: &GatewayConfig, model: &str) -> AgentConfig {
    let (provider_name, model_name) = resolve_provider_and_model(config, model);

    let runtime_providers = config
        .llm_providers
        .iter()
        .map(|(name, cfg)| {
            (
                name.clone(),
                hermes_agent::agent_loop::RuntimeProviderConfig {
                    api_key: cfg.api_key.clone(),
                    api_key_env: cfg.api_key_env.clone(),
                    base_url: cfg.base_url.clone(),
                    api_mode: None,
                    command: cfg.command.clone(),
                    args: cfg.args.clone(),
                    oauth_token_url: cfg.oauth_token_url.clone(),
                    oauth_client_id: cfg.oauth_client_id.clone(),
                },
            )
        })
        .collect();

    AgentConfig {
        max_turns: config.max_turns,
        budget: config.budget.clone(),
        model: model_name,
        stream: true,
        hermes_home: config.home_dir.clone(),
        provider: Some(provider_name),
        platform: Some("tauri".to_string()),
        runtime_providers,
        pass_session_id: true,
        ..AgentConfig::default()
    }
}

/// Build an LLM provider from config + model string.
fn build_provider(config: &GatewayConfig, model: &str) -> Arc<dyn LlmProvider> {
    let (provider_name, model_name) = resolve_provider_and_model(config, model);
    let runtime_provider = normalize_provider(&provider_name);

    let provider_cfg = config
        .llm_providers
        .get(&provider_name)
        .or_else(|| config.llm_providers.get(&runtime_provider));

    let base_url = provider_cfg.and_then(|c| c.base_url.clone());

    let api_key = provider_cfg
        .and_then(|c| c.api_key.as_deref())
        .map(|s| s.to_string())
        .or_else(|| {
            provider_cfg
                .and_then(|c| c.api_key_env.as_deref())
                .filter(|name| !name.is_empty())
                .and_then(|name| std::env::var(name).ok())
                .filter(|v| !v.trim().is_empty())
        })
        .unwrap_or_default();

    use hermes_agent::provider::{
        AnthropicProvider, GenericProvider, OpenAiProvider, OpenRouterProvider,
    };
    use hermes_agent::bedrock::{BedrockProvider, resolve_bedrock_region, bedrock_runtime_base_url};
    use hermes_agent::providers_extra::{
        CopilotProvider, KimiProvider, MiniMaxProvider, NousProvider, QwenProvider,
    };

    match runtime_provider.as_str() {
        "openai" => {
            let mut p = OpenAiProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "anthropic" => {
            let mut p = AnthropicProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "openrouter" => Arc::new(OpenRouterProvider::new(&api_key).with_model(&model_name)),
        "bedrock" => {
            let mut p = BedrockProvider::new()
                .with_region(resolve_bedrock_region())
                .with_model(&model_name);
            if let Some(url) = base_url
                .or_else(|| Some(bedrock_runtime_base_url(&resolve_bedrock_region())))
            {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "qwen" => {
            let mut p = QwenProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "kimi" => {
            let mut p = KimiProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "minimax" => {
            let mut p = MiniMaxProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        "copilot" => {
            let url = base_url.unwrap_or_else(|| "https://api.githubcopilot.com".to_string());
            Arc::new(CopilotProvider::new(url, &api_key).with_model(&model_name))
        }
        "nous" => {
            let mut p = NousProvider::new(&api_key).with_model(&model_name);
            if let Some(url) = base_url {
                p = p.with_base_url(url);
            }
            Arc::new(p)
        }
        _ => {
            let url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
            Arc::new(
                GenericProvider::new(url, &api_key, &model_name)
                    .with_provider_profile(&runtime_provider),
            )
        }
    }
}

/// Build the full tool registry with built-in Hermes tools.
fn build_tool_registry(config: &GatewayConfig) -> Arc<hermes_tools::ToolRegistry> {
    let registry = Arc::new(hermes_tools::ToolRegistry::new());

    let manager = BackendManager::new(config.terminal.clone());
    let terminal_backend = manager.terminal_backend();

    let home = config
        .home_dir
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(hermes_config::hermes_home);
    let skills_dir = home.join("skills");
    let skill_store = FileSkillStore::new(skills_dir);
    let skill_manager = SkillManager::new(Arc::new(skill_store));
    let skill_provider: Arc<dyn hermes_core::SkillProvider> = Arc::new(skill_manager);

    hermes_tools::register_builtin_tools(&registry, terminal_backend, skill_provider);
    registry
}

/// Bridge hermes_tools::ToolRegistry → AgentToolRegistry (for AgentLoop).
fn bridge_agent_tools(tools: &hermes_tools::ToolRegistry) -> Arc<AgentToolRegistry> {
    let mut agent_registry = AgentToolRegistry::new();
    for schema in tools.get_definitions() {
        let name = schema.name.clone();
        let tools_clone = tools.clone();
        agent_registry.register(
            name.clone(),
            schema,
            Arc::new(
                move |params: serde_json::Value| -> Result<String, hermes_core::ToolError> {
                    Ok(tools_clone.dispatch(&name, params))
                },
            ),
        );
    }
    Arc::new(agent_registry)
}

/// Build AgentLoop from Hermes config + model.
fn build_agent_loop(
    config: &GatewayConfig,
    model: &str,
    callbacks: AgentCallbacks,
) -> AgentLoop {
    let agent_config = build_agent_config(config, model);
    let provider = build_provider(config, model);
    let tools = build_tool_registry(config);
    let agent_tools = bridge_agent_tools(&tools);

    let agent = AgentLoop::new(agent_config, agent_tools, provider).with_callbacks(callbacks);
    attach_discovered_memory(agent)
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Send a chat message via the directly-integrated AgentLoop.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_chat(
    app: AppHandle,
    message: String,
    session_id: Option<String>,
    model: Option<String>,
    toolsets: Option<Vec<String>>,
    context_folder: Option<String>,
) -> Result<serde_json::Value, String> {
    let config = load_config(None).map_err(|e| format!("Failed to load Hermes config: {e}"))?;
    let model_name = model.unwrap_or_else(|| {
        config
            .model
            .clone()
            .unwrap_or_else(|| "gpt-4o".to_string())
    });

    ABORT_FLAG.store(false, Ordering::SeqCst);
    {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = session_id.clone();
    }

    // Build messages
    let mut messages = Vec::new();
    if let Some(ref folder) = context_folder {
        let trimmed = folder.trim();
        if !trimmed.is_empty() {
            messages.push(Message::system(&format!(
                "The working folder for this conversation is {}. When the user asks you to read, create, modify, or run project files, use the file, terminal, and code-execution tools with absolute paths under this folder.",
                trimmed
            )));
        }
    }
    messages.push(Message::user(&message));

    // Build AgentCallbacks for tool/reasoning events
    let app_tool_start = app.clone();
    let app_tool_complete = app.clone();
    let captured_id_start = session_id.clone();
    let captured_id_complete = session_id.clone();

    let callbacks = AgentCallbacks {
        on_tool_start: Some(Box::new(move |name: &str, args: &serde_json::Value| {
            let id = format!("tool-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0));
            let _ = app_tool_start.emit("agent-tool-start", json!({
                "id": id,
                "name": name,
                "args": args,
                "session_id": captured_id_start,
            }));
        })),
        on_tool_complete: Some(Box::new(move |name: &str, result: &str| {
            let id = format!("tool-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0));
            let _ = app_tool_complete.emit("agent-tool-complete", json!({
                "id": id,
                "name": name,
                "result": result,
                "session_id": captured_id_complete.clone(),
            }));
        })),
        ..Default::default()
    };

    // Build AgentLoop and run
    let agent = build_agent_loop(&config, &model_name, callbacks);
    let captured_session_id2 = session_id.clone();

    let app_delta = app.clone();
    let app_usage = app.clone();
    let on_chunk = Some(Box::new(move |chunk: StreamChunk| {
        if let Some(ref delta) = chunk.delta {
            if let Some(ref content) = delta.content {
                let _ = app_delta.emit("agent-delta", json!({
                    "text": content,
                    "session_id": captured_session_id2,
                }));
            }
        }
        if let Some(ref usage) = chunk.usage {
            let _ = app_usage.emit("agent-usage", json!({
                "prompt_tokens": usage.prompt_tokens,
                "completion_tokens": usage.completion_tokens,
                "total_tokens": usage.total_tokens,
                "session_id": captured_session_id2.clone(),
            }));
        }
    }) as Box<dyn Fn(StreamChunk) + Send + Sync>);

    // Get and optionally filter tool schemas
    let tools = build_tool_registry(&config);
    let tool_schemas: Vec<ToolSchema> = if let Some(ref ts) = toolsets {
        tools
            .get_definitions()
            .into_iter()
            .filter(|s| ts.iter().any(|t| s.name.starts_with(t)))
            .collect()
    } else {
        tools.get_definitions()
    };

    let app_err = app.clone();
    let captured_session_id3 = session_id.clone();

    let result = agent.run_stream(messages, Some(tool_schemas), on_chunk).await;

    match result {
        Ok(agent_result) => {
            let final_response = agent_result
                .messages
                .iter()
                .rev()
                .find(|m| m.role == hermes_core::MessageRole::Assistant)
                .and_then(|m| m.content.clone())
                .unwrap_or_default();

            let _ = app.emit("agent-done", json!({
                "response": final_response,
                "session_id": captured_session_id3.clone(),
                "message_count": 0i32,
            }));

            Ok(json!({
                "response": final_response,
                "session_id": captured_session_id3,
                "message_count": 0,
            }))
        }
        Err(err) => {
            let err_msg = err.to_string();
            let _ = app_err.emit("agent-error", json!({
                "message": err_msg,
                "session_id": captured_session_id3,
            }));
            Err(err_msg)
        }
    }
}

/// Abort current chat.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_abort_chat() -> Result<serde_json::Value, String> {
    ABORT_FLAG.store(true, Ordering::SeqCst);
    let session_id = CURRENT_SESSION_ID.lock().unwrap().clone();
    Ok(json!({
        "aborted": true,
        "session_id": session_id,
    }))
}

/// Clear cache — no-op with direct AgentLoop (each call is fresh).
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_clear_cache(_session_id: String) -> Result<serde_json::Value, String> {
    Ok(json!({"ok": true, "session_id": _session_id}))
}

/// Check Agent availability.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_check_available() -> Result<serde_json::Value, String> {
    let config = load_config(None).ok();
    let has_config = config.is_some();
    let has_providers = config
        .as_ref()
        .map(|c| !c.llm_providers.is_empty())
        .unwrap_or(false);

    Ok(json!({
        "available": has_config && has_providers,
        "ready": has_config && has_providers,
        "error": if !has_config {
            serde_json::Value::String("Hermes config not found".to_string())
        } else if !has_providers {
            serde_json::Value::String("No LLM providers configured in config.yaml".to_string())
        } else {
            serde_json::Value::Null
        },
    }))
}

/// Get custom models from Hermes config.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_get_models() -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::get_models()
}

/// Add a model to Hermes config.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_add_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::add_model(model)
}

/// Remove a model from Hermes config.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_remove_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::remove_model(model)
}

/// Set the default model in Hermes config.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_set_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::set_default_model(model)
}
