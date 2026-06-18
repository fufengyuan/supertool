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

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use hermes_agent::agent_loop::ToolRegistry as AgentToolRegistry;
use hermes_agent::{
    attach_discovered_memory, AgentCallbacks, AgentConfig, AgentLoop,
};
use hermes_config::loader::load_config;
use hermes_config::GatewayConfig;
use hermes_core::{LlmProvider, Message, StreamChunk, ToolCall, ToolSchema};
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

/// Resolve the effective model name from config.yaml.
///
/// `GatewayConfig.model` is `Option<String>` — but the user's config.yaml may
/// have a nested `model:` section (OMP-style format) with `default`, `provider`,
/// `base_url`, `api_key` sub-keys. serde silently ignores nested maps for a
/// string field, so we fall back to raw-YAML parsing.
fn resolve_effective_model(config: &GatewayConfig, cli_model: Option<String>) -> String {
    if let Some(m) = cli_model {
        return m;
    }
    if let Some(m) = &config.model {
        return m.clone();
    }
    // Fallback: read raw YAML to extract nested model.default + model.provider
    if let Some(model_str) = resolve_nested_model_info().map(|m| m.model_str) {
        return model_str;
    }
    "gpt-4o".to_string()
}

/// Holds the full model info extracted from a nested `model:` YAML section.
#[derive(Clone)]
struct NestedModelInfo {
    model_str: String,
    base_url: Option<String>,
    api_key: Option<String>,
}

/// Read raw config.yaml and extract nested model section
/// (`model.default`, `model.provider`, `model.base_url`, `model.api_key`).
/// Retries on empty/parse error to handle concurrent file writes by other processes.
fn resolve_nested_model_info() -> Option<NestedModelInfo> {
    let path = hermes_config::paths::config_path();
    log::info!("[AgentChat] nested_model_info: path={:?}, exists={}", path, path.exists());
    if !path.exists() {
        return None;
    }
    for attempt in 0..3 {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[AgentChat] nested_model_info: read error (attempt {}): {}", attempt, e);
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                continue;
            }
        };
        let yaml: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[AgentChat] nested_model_info: yaml parse error (attempt {}): {}", attempt, e);
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                continue;
            }
        };
        let model_section = match yaml.get("model") {
            Some(s) => s,
            None => {
                log::warn!("[AgentChat] nested_model_info: no 'model' key in yaml");
                return None;
            }
        };
        if !model_section.is_mapping() {
            log::info!("[AgentChat] nested_model_info: model is a plain string, not nested");
            return None;
        }
        let default_model = match model_section.get("default").and_then(|v| v.as_str()) {
            Some(s) => s.trim().to_string(),
            None => {
                log::warn!("[AgentChat] nested_model_info: missing model.default");
                return None;
            }
        };
        let provider = model_section
            .get("provider")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string());

        let model_str = match &provider {
            Some(p) => format!("{}:{}", p, default_model),
            None => default_model,
        };

        let base_url = model_section
            .get("base_url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let api_key = model_section
            .get("api_key")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let result = NestedModelInfo {
            model_str,
            base_url,
            api_key,
        };
        log::info!("[AgentChat] nested_model_info: resolved model_str={}, base_url={:?}, has_key={}",
            result.model_str, result.base_url, result.api_key.is_some());
        return Some(result);
    }
    log::warn!("[AgentChat] nested_model_info: all 3 attempts failed");
    None
}

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

/// Look up provider config from `llm_providers` first, then fall back to
/// auth.json `credential_pool` entries, and finally to raw config.yaml's nested `model:` section.
fn find_provider_config(
    config: &GatewayConfig,
    provider_name: &str,
) -> Option<(String, Option<String>)> {
    let runtime_provider = normalize_provider(provider_name);

    // 1. Try config.yaml llm_providers
    let cfg = config
        .llm_providers
        .get(provider_name)
        .or_else(|| config.llm_providers.get(&runtime_provider));

    if let Some(cfg) = cfg {
        log::info!("[AgentChat] find_provider: found in llm_providers for '{}'", provider_name);
        let api_key = cfg
            .api_key
            .clone()
            .or_else(|| {
                cfg.api_key_env.as_deref().and_then(|name| {
                    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
                })
            })
            .unwrap_or_default();
        let base_url = cfg.base_url.clone();
        return Some((api_key, base_url));
    }

    // 2. Fallback: auth.json credential_pool
    if let Some(result) = load_credential_pool_entry(provider_name)
        .or_else(|| load_credential_pool_entry(&runtime_provider))
    {
        let (api_key, base_url) = result;
        log::info!("[AgentChat] find_provider: found in credential_pool for '{}': base_url={:?}, has_key={}",
            provider_name, base_url, !api_key.is_empty());
        if !api_key.is_empty() || base_url.is_some() {
            return Some((api_key, base_url));
        }
    } else {
        log::info!("[AgentChat] find_provider: not found in credential_pool for '{}'", provider_name);
    }

    // 3. Final fallback: raw config.yaml nested model: section (base_url + api_key)
    if let Some(nested) = resolve_nested_model_info() {
        log::info!("[AgentChat] find_provider: nested model fallback for '{}': base_url={:?}, has_key={}",
            provider_name, nested.base_url, nested.api_key.is_some());
        if nested.base_url.is_some() {
            let api_key = nested.api_key.unwrap_or_default();
            return Some((api_key, nested.base_url));
        }
    } else {
        log::info!("[AgentChat] find_provider: nested model fallback returned None for '{}'", provider_name);
    }

    log::warn!("[AgentChat] find_provider: all fallbacks exhausted for '{}'", provider_name);
    None
}

/// Read a single entry from auth.json credential_pool by provider name.
fn load_credential_pool_entry(provider_name: &str) -> Option<(String, Option<String>)> {
    let home = hermes_config::hermes_home();
    let auth_path = home.join("auth.json");
    if !auth_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(auth_path).ok()?;
    let auth: serde_json::Value = serde_json::from_str(&content).ok()?;
    let pool = auth.get("credential_pool")?.as_object()?;

    // Try exact match first, then partial match
    let keys: [&str; 2] = [provider_name, &format!("custom:{}", provider_name)];
    for key in keys {
        if let Some(entries) = pool.get(key) {
            if let Some(entry) = entries.as_array()?.first() {
                let api_key = entry
                    .get("api_key")
                    .and_then(|v| v.as_str())
                    .or_else(|| entry.get("access_token").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
                let base_url = entry
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                return Some((api_key, base_url));
            }
        }
    }
    None
}

/// Build all runtime provider configs from config.yaml + auth.json credential_pool.
fn build_runtime_providers(
    config: &GatewayConfig,
) -> HashMap<String, hermes_agent::agent_loop::RuntimeProviderConfig> {
    let mut providers: HashMap<String, hermes_agent::agent_loop::RuntimeProviderConfig> =
        HashMap::new();

    // From config.yaml llm_providers
    for (name, cfg) in &config.llm_providers {
        providers.insert(
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
        );
    }

    // Fallback: from auth.json credential_pool
    let home = config
        .home_dir
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(hermes_config::hermes_home);
    let auth_path = home.join("auth.json");
    if auth_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&auth_path) {
            if let Ok(auth) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(pool) = auth.get("credential_pool").and_then(|v| v.as_object()) {
                    for (key, entries) in pool {
                        if providers.contains_key(key) {
                            continue; // Don't overwrite config.yaml entries
                        }
                        if let Some(entry) = entries.as_array().and_then(|a| a.first()) {
                            let api_key = entry
                                .get("api_key")
                                .and_then(|v| v.as_str())
                                .or_else(|| {
                                    entry.get("access_token").and_then(|v| v.as_str())
                                })
                                .unwrap_or("")
                                .to_string();
                            let base_url = entry
                                .get("base_url")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string());
                            providers.insert(
                                key.clone(),
                                hermes_agent::agent_loop::RuntimeProviderConfig {
                                    api_key: Some(api_key),
                                    api_key_env: None,
                                    base_url,
                                    api_mode: None,
                                    command: None,
                                    args: vec![],
                                    oauth_token_url: None,
                                    oauth_client_id: None,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    providers
}

/// Build an AgentConfig from Hermes config.yaml + model string.
fn build_agent_config(config: &GatewayConfig, model: &str, session_id: Option<String>) -> AgentConfig {
    let (provider_name, model_name) = resolve_provider_and_model(config, model);
    let runtime_providers = build_runtime_providers(config);

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
        session_id,
        ..AgentConfig::default()
    }
}

/// Build an LLM provider from config + model string, with credential_pool fallback.
fn build_provider(config: &GatewayConfig, model: &str) -> Arc<dyn LlmProvider> {
    let (provider_name, model_name) = resolve_provider_and_model(config, model);
    let runtime_provider = normalize_provider(&provider_name);

    let (api_key, base_url) = find_provider_config(config, &provider_name)
        .or_else(|| find_provider_config(config, &runtime_provider))
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
    session_id: Option<String>,
    callbacks: AgentCallbacks,
) -> AgentLoop {
    let agent_config = build_agent_config(config, model, session_id);
    let provider = build_provider(config, model);
    let tools = build_tool_registry(config);
    let agent_tools = bridge_agent_tools(&tools);

    let agent = AgentLoop::new(agent_config, agent_tools, provider).with_callbacks(callbacks);
    attach_discovered_memory(agent)
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Convert a DB HermesMessage to a core Message for AgentLoop context.
fn db_message_to_core(msg: &supertool_core::db::agent::HermesMessage) -> Message {
    match msg.role.as_str() {
        "user" => Message::user(msg.content.clone().unwrap_or_default()),
        "assistant" => {
            if let Some(ref tc) = msg.tool_calls {
                if let Ok(calls) = serde_json::from_str::<Vec<ToolCall>>(tc) {
                    let mut m = Message::assistant_with_tool_calls(msg.content.clone(), calls);
                    if let Some(ref r) = msg.reasoning_content {
                        m.reasoning_content = Some(r.clone());
                    }
                    return m;
                }
            }
            let mut m = Message::assistant(msg.content.clone().unwrap_or_default());
            if let Some(ref r) = msg.reasoning_content {
                m.reasoning_content = Some(r.clone());
            }
            m
        }
        "tool" => Message::tool_result(
            msg.tool_call_id.clone().unwrap_or_default(),
            msg.content.clone().unwrap_or_default(),
        ),
        "system" => Message::system(msg.content.clone().unwrap_or_default()),
        _ => Message::user(msg.content.clone().unwrap_or_default()),
    }
}

/// Send a chat message via the directly-integrated AgentLoop.
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_chat(
    app: AppHandle,
    message: String,
    session_id: Option<String>,
    model: Option<String>,
    toolsets: Option<Vec<String>>,
    context_folder: Option<String>,
    goal: Option<String>,
) -> Result<serde_json::Value, String> {
    let config = load_config(None).map_err(|e| format!("Failed to load Hermes config: {e}"))?;
    let model_name = resolve_effective_model(&config, model);
    log::info!("[AgentChat] resolved model_name='{}'", model_name);

    ABORT_FLAG.store(false, Ordering::SeqCst);
    // Generate a session_id if this is a new conversation
    let effective_session_id = session_id.clone()
        .or_else(|| Some(uuid::Uuid::new_v4().to_string()));
    {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = effective_session_id.clone();
    }

    // Build messages — load session history from state.db if resuming
    let is_new_session = session_id.is_none();
    let mut messages: Vec<Message> = if let Some(ref sid) = effective_session_id {
        if is_new_session {
            // New session: create in state.db
            let _ = supertool_core::db::agent::create_hermes_session(sid, "tauri", &model_name);
            Vec::new()
        } else {
            // Existing session: load all previous messages from state.db
            match supertool_core::db::agent::list_hermes_messages(sid) {
                Ok(db_msgs) => {
                    log::info!("[AgentChat] loaded {} messages from session {}", db_msgs.len(), sid);
                    db_msgs.iter().map(db_message_to_core).collect()
                }
                Err(e) => {
                    log::warn!("[AgentChat] failed to load session history: {}", e);
                    Vec::new()
                }
            }
        }
    } else {
        Vec::new()
    };
    if let Some(ref folder) = context_folder {
        let trimmed = folder.trim();
        if !trimmed.is_empty() {
            messages.push(Message::system(&format!(
                "The working folder for this conversation is {}. When the user asks you to read, create, modify, or run project files, use the file, terminal, and code-execution tools with absolute paths under this folder.",
                trimmed
            )));
        }
    }
    // Inject goal as a persistent system message
    if let Some(ref goal_text) = goal {
        let trimmed = goal_text.trim();
        if !trimmed.is_empty() {
            messages.push(Message::system(&format!(
                "You are working toward the following goal: {}\n\
                 - Work toward this goal step by step across multiple turns.\n\
                 - After each response, evaluate whether you have achieved the goal.\n\
                 - If you believe the goal is achieved, end your response with GOAL_COMPLETED.",
                trimmed
            )));
        }
    }
    messages.push(Message::user(&message));
    // Persist user message to state.db immediately
    if let Some(ref sid) = effective_session_id {
        let _ = supertool_core::db::agent::insert_hermes_message(
            sid, "user", Some(&message), None, None, None,
        );
    }

    // Build AgentCallbacks for tool/reasoning events
    let sid_tool_start = effective_session_id.clone();
    let sid_tool_complete = effective_session_id.clone();
    let sid_delta = effective_session_id.clone();
    let sid_usage = effective_session_id.clone();
    let sid_done = effective_session_id.clone();

    let app_tool = app.clone();
    let app_tool_complete = app.clone();
    let app_delta = app.clone();
    let app_usage = app.clone();
    let app_done = app.clone();

    let callbacks = AgentCallbacks {
        on_tool_start: Some(Box::new(move |name: &str, args: &serde_json::Value| {
            let id = format!("tool-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0));
            let _ = app_tool.emit("agent-tool-start", json!({
                "id": id,
                "name": name,
                "args": args,
                "session_id": sid_tool_start,
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
                "session_id": sid_tool_complete,
            }));
        })),
        ..Default::default()
    };

    // Build AgentLoop and run
    let agent = build_agent_loop(&config, &model_name, effective_session_id.clone(), callbacks);
    let on_chunk = Some(Box::new(move |chunk: StreamChunk| {
        if let Some(ref delta) = chunk.delta {
            if let Some(ref content) = delta.content {
                let _ = app_delta.emit("agent-delta", json!({
                    "text": content,
                    "session_id": sid_delta,
                }));
            }
        }
        if let Some(ref usage) = chunk.usage {
            let _ = app_usage.emit("agent-usage", json!({
                "prompt_tokens": usage.prompt_tokens,
                "completion_tokens": usage.completion_tokens,
                "total_tokens": usage.total_tokens,
                "session_id": sid_usage.clone(),
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

            let _ = app_done.emit("agent-done", json!({
                "response": final_response,
                "session_id": sid_done,
                "message_count": 0i32,
            }));

            // Persist all messages to state.db (batch replace — mirrors CLI)
            if let Some(ref sid) = effective_session_id {
                // Delete old messages, then re-insert all AgentResult messages
                let _ = supertool_core::db::agent::delete_hermes_messages(sid);
                for msg in &agent_result.messages {
                    let role = format!("{:?}", msg.role).to_ascii_lowercase();
                    let tc_json = msg.tool_calls.as_ref()
                        .and_then(|tc| serde_json::to_string(tc).ok());
                    let _ = supertool_core::db::agent::insert_hermes_message(
                        sid, &role, msg.content.as_deref(),
                        msg.tool_call_id.as_deref(), tc_json.as_deref(),
                        msg.reasoning_content.as_deref(),
                    );
                }
            }

            Ok(json!({
                "response": final_response,
                "session_id": sid_done,
                "message_count": 0,
            }))
        }
        Err(err) => {
            let err_msg = err.to_string();
            let _ = app_done.emit("agent-error", json!({
                "message": err_msg,
                "session_id": sid_done,
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

    // Check auth.json credential_pool as well (not just llm_providers)
    let has_credentials = has_providers_in_config_or_auth(&config);

    Ok(json!({
        "available": has_config && has_credentials,
        "ready": has_config && has_credentials,
        "error": if !has_config {
            serde_json::Value::String("Hermes config not found".to_string())
        } else if !has_credentials {
            serde_json::Value::String("No LLM providers or credentials configured".to_string())
        } else {
            serde_json::Value::Null
        },
    }))
}

fn has_providers_in_config_or_auth(config: &Option<GatewayConfig>) -> bool {
    if let Some(c) = config {
        if !c.llm_providers.is_empty() {
            return true;
        }
    }
    // Check auth.json credential_pool
    let home = config
        .as_ref()
        .and_then(|c| c.home_dir.clone())
        .map(PathBuf::from)
        .unwrap_or_else(hermes_config::hermes_home);
    let auth_path = home.join("auth.json");
    if !auth_path.exists() {
        return false;
    }
    if let Ok(content) = std::fs::read_to_string(auth_path) {
        if let Ok(auth) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(pool) = auth.get("credential_pool").and_then(|v| v.as_object()) {
                return pool.iter().any(|(_, entries)| {
                    entries.as_array().is_some_and(|a| !a.is_empty())
                });
            }
        }
    }
    false
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
