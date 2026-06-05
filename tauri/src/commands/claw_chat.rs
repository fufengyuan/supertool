use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use runtime::{
    ContentBlock, ConversationMessage, MessageRole, Session,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message, TurnResult};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// ── Cost estimation (matches upstream pricing_for_model) ───────────────────

/// Per-million-token pricing by model family.
#[derive(Debug, Clone, Copy)]
struct ModelPricing {
    input_per_million: f64,
    output_per_million: f64,
}

impl ModelPricing {
    const fn haiku() -> Self {
        Self { input_per_million: 1.0, output_per_million: 5.0 }
    }
    const fn sonnet() -> Self {
        Self { input_per_million: 15.0, output_per_million: 75.0 }
    }
    const fn opus() -> Self {
        Self { input_per_million: 15.0, output_per_million: 75.0 }
    }
    const fn default_sonnet() -> Self {
        Self::sonnet()
    }
}

fn pricing_for_model(model: &str) -> ModelPricing {
    let n = model.to_ascii_lowercase();
    if n.contains("haiku") { ModelPricing::haiku() }
    else if n.contains("opus") { ModelPricing::opus() }
    else if n.contains("sonnet") || n.contains("claude") { ModelPricing::sonnet() }
    else { ModelPricing::default_sonnet() }
}

fn estimate_cost(input_tokens: u64, output_tokens: u64, model: &str) -> f64 {
    let p = pricing_for_model(model);
    (input_tokens as f64 / 1_000_000.0) * p.input_per_million
        + (output_tokens as f64 / 1_000_000.0) * p.output_per_million
}

/// Cumulative token usage tracker — mirrors upstream UsageTracker in runtime/src/usage.rs.
/// Tracks input/output tokens across all API calls in a turn for cost estimation and
/// auto-compaction threshold checks.
/// Claw 聊天状态（单例，存在 app state 中）
pub struct ClawChatState {
    pub(crate) client: Mutex<Option<Arc<LlmClient>>>,
    pub(crate) session: Mutex<Option<Session>>,
    pub(crate) workspace: Mutex<Option<PathBuf>>,
    /// Abort signal for the current tool loop — shared with hook runner.
    pub(crate) hook_abort: Arc<AtomicBool>,
}

impl ClawChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session: Mutex::new(None),
            workspace: Mutex::new(None),
            hook_abort: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Session persistence (uses claw-code's Session API) ──────────────────

pub(crate) fn sessions_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("sessions")
}

fn session_path(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.json"))
}

/// Load a persisted session by ID.
pub(crate) fn load_session(id: &str) -> Option<Session> {
    let path = session_path(id);
    log::info!("[claw_chat] load_session({}): looking for {}", id, path.display());
    if !path.exists() {
        log::warn!("[claw_chat] load_session({}): file not found at {}", id, path.display());
        return None;
    }
    match Session::load_from_path(&path) {
        Ok(session) => {
            log::info!("[claw_chat] load_session({}): OK, {} messages", id, session.messages.len());
            Some(session)
        }
        Err(e) => {
            log::error!("[claw_chat] load_session({}): FAILED to parse: {}", id, e);
            None
        }
    }
}

/// List all persisted sessions by reading their JSONL meta record (first line).
pub(crate) fn list_sessions_info() -> Vec<serde_json::Value> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut sessions: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            // CRITICAL: Use the FILE NAME (stem) as sessionId, NOT the session_id field
            // inside the JSON. The claw-code CLI may save files with names that differ
            // from the internal session_id. load_session(id) looks for {id}.json, so
            // the ID returned here MUST match the filename.
            let file_stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if file_stem.is_empty() {
                continue;
            }
            // Read the first line (meta record) for session info
            if let Ok(content) = std::fs::read_to_string(&path) {
                let first_line = content.lines().next().unwrap_or("");
                let mut created_at_ms: u64 = 0;
                let mut title: Option<String> = None;
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(first_line) {
                    created_at_ms = meta
                        .get("created_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let updated_at_ms = meta
                        .get("updated_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    // Read the first message (after meta line) for a title preview
                    title = content
                        .lines()
                        .nth(1)
                        .and_then(|line| {
                            serde_json::from_str::<serde_json::Value>(line).ok()
                        })
                        .and_then(|v| {
                            // Try the actual session JSONL format first:
                            // {"message":{"blocks":[{"text":"...","type":"text"}],"role":"user"},"type":"message"}
                            v.get("message")
                                .and_then(|msg| msg.get("blocks"))
                                .and_then(|blocks| blocks.as_array())
                                .and_then(|arr| arr.first())
                                .and_then(|block| block.get("text"))
                                .and_then(|t| t.as_str())
                                .map(|t| t.trim().to_string())
                        })
                        .filter(|c| !c.is_empty())
                        .map(|c| {
                            if c.len() > 60 {
                                format!("{}...", &c[..60])
                            } else {
                                c
                            }
                        });
                    // Count message lines in the JSONL file
                    let message_count = content.lines().skip(1).count();
                    // Extract model from first assistant message (if any)
                    let model = content.lines().skip(1)
                        .find_map(|line| {
                            let v: serde_json::Value = serde_json::from_str(line).ok()?;
                            let msg = v.get("message")?;
                            if msg.get("role")?.as_str()? == "assistant" { Some(v) } else { None }
                        })
                        .and_then(|v| v.get("model").and_then(|m| m.as_str()).map(String::from));
                    sessions.push(serde_json::json!({
                        "sessionId": file_stem,
                        "createdAt": format_ts(created_at_ms),
                        "updatedAt": format_ts(updated_at_ms),
                        "messageCount": message_count,
                        "title": title,
                        "model": model,
                    }));
                }
            }
        }
    }
    // Newest first by created_at_ms
    sessions.sort_by(|a, b| {
        let ta = a.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        let tb = b.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        tb.cmp(ta)
    });
    sessions
}

/// Convert session messages to a simplified JSON array for the front-end.
pub(crate) fn session_messages_to_json(messages: &[ConversationMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|cm| {
            let role = match cm.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "agent",
                MessageRole::System => "system",
                MessageRole::Tool => "tool",
            };
            let text = cm
                .blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.clone()),
                    ContentBlock::Thinking { thinking, .. } => Some(thinking.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::json!({
                "role": role,
                "content": text,
            })
        })
        .collect()
}

/// Format a Unix-epoch millis timestamp to RFC 3339 for the frontend.
fn format_ts(ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_millis_opt(ms as i64)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

// ── Session → InputMessage conversion (for tool-loop requests) ──────────

/// Convert runtime `ConversationMessage` to `api::InputMessage` for the tool loop.
/// Handles all block types: Text, Thinking, ToolUse, ToolResult.
pub(crate) fn session_to_input_messages(messages: &[ConversationMessage]) -> Vec<api::InputMessage> {
    // Match upstream convert_messages(): System|User|Tool → "user", Assistant → "assistant".
    // OpenAI-compatible APIs may not support "tool" role; upstream maps Tool to "user".
    messages
        .iter()
        .filter_map(|cm| {
            let role = match cm.role {
                MessageRole::System | MessageRole::User | MessageRole::Tool => "user",
                MessageRole::Assistant => "assistant",
            };
            let content: Vec<api::InputContentBlock> = cm
                .blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text } if text.is_empty() => None,
                    ContentBlock::Text { text } => Some(api::InputContentBlock::Text {
                        text: text.clone(),
                    }),
                    ContentBlock::Thinking { thinking, signature } => {
                        Some(api::InputContentBlock::Thinking {
                            thinking: thinking.clone(),
                            signature: signature.clone(),
                        })
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        let input_value: serde_json::Value =
                            serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({"raw": input}));
                        Some(api::InputContentBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input_value,
                        })
                    }
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output,
                        is_error,
                        ..
                    } => Some(api::InputContentBlock::ToolResult {
                        tool_use_id: tool_use_id.clone(),
                        content: vec![api::ToolResultContentBlock::Text {
                            text: output.clone(),
                        }],
                        is_error: *is_error,
                    }),
                })
                .collect();
            if content.is_empty() {
                return None;
            }
            Some(api::InputMessage {
                role: role.to_string(),
                content,
            })
        })
        .collect()
}

/// Build tool definitions for the LLM request from claw-tools `mvp_tool_specs()`.
/// Build a unified tool registry with builtin + plugin tools and permission enforcer.
///
/// This replaces the old `build_tool_definitions()` which only returned builtin tools.
/// The `GlobalToolRegistry` handles:
/// - Builtin tools (bash, read_file, write_file, edit_file, glob_search, grep_search)
/// - Plugin tools (from ~/.claw/plugins/)
/// - Permission enforcement (via PermissionEnforcer)
pub(crate) fn build_tool_registry() -> tools::GlobalToolRegistry {
    // Load plugin tools from ~/.claw/plugins/
    let plugin_tools = load_claw_plugin_tools();
    log::info!(
        "[claw_chat] Building tool registry: {} plugin tools loaded",
        plugin_tools.len()
    );

    // Build registry with builtin + plugin tools
    let mut registry = tools::GlobalToolRegistry::with_plugin_tools(plugin_tools)
        .unwrap_or_else(|e| {
            log::warn!("[claw_chat] Failed to load plugin tools: {}", e);
            tools::GlobalToolRegistry::builtin()
        });

    // Set up permission enforcer (auto-approve all in GUI mode)
    let enforcer = runtime::permission_enforcer::PermissionEnforcer::new(
        runtime::PermissionPolicy::new(runtime::PermissionMode::Allow),
    );
    registry.set_enforcer(enforcer);

    registry
}

/// Load plugin tools from ~/.claw/plugins/installed/
fn load_claw_plugin_tools() -> Vec<plugins::PluginTool> {
    let plugins_dir = dirs::home_dir()
        .map(|h| h.join(".claw/plugins/installed"))
        .unwrap_or_default();

    if !plugins_dir.exists() {
        return Vec::new();
    }

    let mut all_tools = Vec::new();

    // Read the installed plugins registry
    let registry_path = plugins_dir.parent().unwrap_or(&plugins_dir).join("installed.json");
    let enabled_map: std::collections::BTreeMap<String, bool> =
        if let Ok(content) = std::fs::read_to_string(&registry_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::BTreeMap::new()
        };

    // Scan installed plugin directories
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let plugin_dir = entry.path();
            if !plugin_dir.is_dir() {
                continue;
            }

            // Check if plugin is enabled
            let dir_name = plugin_dir.file_name().unwrap_or_default().to_string_lossy();
            if let Some(&false) = enabled_map.get(dir_name.as_ref()) {
                continue;
            }

            // Load plugin manifest
            let manifest_path = plugin_dir.join(".claude-plugin/plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            match plugins::load_plugin_from_directory(&plugin_dir) {
                Ok(manifest) => {
                    log::info!(
                        "[claw_chat] Loaded plugin: {} ({} tools)",
                        manifest.name,
                        manifest.tools.len()
                    );
                    // Convert PluginToolManifest → PluginTool (matches upstream aggregated_tools)
                    for tool_manifest in &manifest.tools {
                        let definition = plugins::PluginToolDefinition {
                            name: tool_manifest.name.clone(),
                            description: Some(tool_manifest.description.clone()),
                            input_schema: tool_manifest.input_schema.clone(),
                        };
                        let plugin_tool = plugins::PluginTool::new(
                            dir_name.to_string(),          // plugin_id
                            manifest.name.clone(),         // plugin_name
                            definition,
                            tool_manifest.command.clone(),
                            tool_manifest.args.clone(),
                            tool_manifest.required_permission,
                            Some(plugin_dir.clone()),      // root
                        );
                        all_tools.push(plugin_tool);
                    }
                }
                Err(e) => {
                    log::debug!("[claw_chat] Skipping plugin {}: {}", dir_name, e);
                }
            }
        }
    }

    all_tools
}

/// Legacy wrapper — returns tool definitions from the registry.
pub(crate) fn build_tool_definitions() -> Vec<api::ToolDefinition> {
    let registry = build_tool_registry();
    registry.definitions(None)
}

/// Load Hermes skills from ~/.hermes/skills/ and return a formatted section.
///
/// Strategy:
/// - Load all DESCRIPTION.md files for a high-level skill index (~200 bytes each)
/// - Load full SKILL.md for coding-relevant skills only (capped at 120KB total)
/// - Return as a single section to append to the system prompt
pub(crate) fn load_hermes_skills(skill_bytes_cap: usize) -> String {
    let skills_dir = dirs::home_dir()
        .map(|h| h.join(".hermes/skills"))
        .unwrap_or_default();

    if !skills_dir.exists() {
        return String::new();
    }

    let mut sections: Vec<String> = Vec::new();

    // Coding-relevant skill categories to load in full
    let full_load_categories = [
        "github",
        "coding-ultimate-rules",
        "dev",
        "devops",
        "software-development",
    ];

    // 1. Build skill index from DESCRIPTION.md files
    let mut index_lines: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let category = path.file_name().unwrap_or_default().to_string_lossy();
            let desc_path = path.join("DESCRIPTION.md");
            if desc_path.exists() {
                if let Ok(desc) = std::fs::read_to_string(&desc_path) {
                    let brief: String = desc.lines().take(3).collect::<Vec<_>>().join(" ");
                    // Unicode-safe truncation (matches upstream floor_char_boundary)
                    let brief = if brief.len() > 200 {
                        let safe_end = brief.floor_char_boundary(200);
                        format!("{}...", &brief[..safe_end])
                    } else {
                        brief
                    };
                    index_lines.push(format!("- **{category}**: {brief}"));
                }
            } else {
                index_lines.push(format!("- **{category}**"));
            }
        }
    }

    if !index_lines.is_empty() {
        sections.push(format!(
            "# Available Hermes Skills (Index)\nThese skills are loaded from ~/.hermes/skills/ and can provide specialized knowledge:\n{}",
            index_lines.join("\n")
        ));
    }

    // 2. Load full SKILL.md for coding-relevant categories
    let mut total_skill_bytes: usize = 0;

    'skill_categories: for category in &full_load_categories {
        let cat_dir = skills_dir.join(category);
        if !cat_dir.exists() {
            continue;
        }

        // Find all SKILL.md files in this category (recursive, max depth 2)
        if let Ok(walker) = std::fs::read_dir(&cat_dir) {
            for sub in walker.flatten() {
                let sub_path = sub.path();
                if !sub_path.is_dir() {
                    continue;
                }
                // Look for SKILL.md in immediate subdirectory
                let skill_file = sub_path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&skill_file) {
                        let skill_name = sub_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let content_len = content.len();
                        if total_skill_bytes + content_len > skill_bytes_cap {
                            log::info!(
                                "[claw_chat] Skill cap reached ({}KB), skipping remaining skills",
                                total_skill_bytes / 1024
                            );
                            break 'skill_categories;
                        }
                        total_skill_bytes += content_len;
                        sections.push(format!(
                            "# Skill: {category}/{skill_name}\n{content}"
                        ));
                    }
                }
            }
        }
    }

    log::info!(
        "[claw_chat] Loaded Hermes skills: {} sections, {}KB total",
        sections.len(),
        total_skill_bytes / 1024
    );

    if sections.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n# Hermes Skills\nThe following specialized skills are available from the Hermes Agent system:\n\n{}",
            sections.join("\n\n---\n\n")
        )
    }
}


/// Load a HookRunner from the upstream settings.json via ConfigLoader.
/// Falls back to default (empty) config if settings.json is missing or has no hooks.
pub(crate) fn load_hook_runner() -> runtime::HookRunner {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    match runtime::ConfigLoader::default_for(cwd).load() {
        Ok(config) => runtime::HookRunner::from_feature_config(config.feature_config()),
        Err(e) => {
            log::debug!("[claw_chat] Could not load hook config from settings.json: {}", e);
            runtime::HookRunner::new(runtime::RuntimeHookConfig::default())
        }
    }
}

/// System prompt for the Claw agent — uses the real load_system_prompt from runtime.
pub(crate) fn claw_agent_system_prompt(skill_bytes_cap: usize) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let model = std::env::var("ANTHROPIC_MODEL")
        .or_else(|_| std::env::var("OPENAI_MODEL"))
        .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());

    let base_prompt = match runtime::load_system_prompt(
        &cwd,
        chrono::Utc::now().format("%Y-%m-%d").to_string(),
        std::env::consts::OS.to_string(),
        "26.5".to_string(),
        api::model_family_identity_for(&model),
    ) {
        Ok(sections) => sections.join("\n\n"),
        Err(e) => {
            log::warn!("[claw_chat] Failed to load system prompt: {}, using fallback", e);
            "You are an expert software engineer and coding assistant. You have access to tools for reading files, writing files, editing files, running shell commands, and searching code. Always use your tools to help the user.".to_string()
        }
    };

    // Append Hermes skills
    let skills_section = load_hermes_skills(skill_bytes_cap);
    if skills_section.is_empty() {
        base_prompt
    } else {
        format!("{base_prompt}\n{skills_section}")
    }
}

// ── Config ───────────────────────────────────────────────────────────────

/// 从 ~/.claw/config.json 读取 API key 和 base URL，设置到进程环境变量
pub(crate) fn setup_env_from_claw_config() -> Result<(), String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    if config.api_key.is_empty() {
        log::info!("[claw_chat] No ~/.claw/config.json api_key — falling back to env vars");
        return Ok(());
    }

    log::info!(
        "[claw_chat] Read Claw config: model={}, has_api_key={}, base_url={}",
        config.model,
        !config.api_key.is_empty(),
        config.base_url
    );

    let has_base_url = !config.base_url.is_empty();

    if has_base_url {
        unsafe { std::env::set_var("OPENAI_API_KEY", &config.api_key); }
        unsafe { std::env::set_var("OPENAI_BASE_URL", &config.base_url); }
        unsafe { std::env::set_var("OPENAI_MODEL", &config.model); }
        log::info!("[claw_chat] Base URL set → using OpenAI-compatible client");
    } else {
        let canonical = config.model.to_ascii_lowercase();
        if canonical.starts_with("claude") || canonical.starts_with("anthropic/") {
            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("ANTHROPIC_MODEL", &config.model); }
        } else if canonical.starts_with("openai/") || canonical.starts_with("gpt-") {
            unsafe { std::env::set_var("OPENAI_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("OPENAI_MODEL", &config.model); }
        } else if canonical.starts_with("grok") {
            unsafe { std::env::set_var("XAI_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("XAI_MODEL", &config.model); }
        } else {
            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("ANTHROPIC_MODEL", &config.model); }
        }
    }

    Ok(())
}

// ── ConversationMessage ↔ LlmClient::Message conversion ─────────────────

/// Convert claw-code's `ConversationMessage` to the flat `Message` format
/// that `LlmClient::send_streaming` expects.
pub(crate) fn to_prompt_messages(session_messages: &[ConversationMessage]) -> Vec<Message> {
    session_messages
        .iter()
        .map(|cm| {
            let role = match cm.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            }
            .to_string();

            // Flatten blocks into a single text string
            let content: String = cm
                .blocks
                .iter()
                .map(|b| match b {
                    ContentBlock::Text { text } => text.clone(),
                    ContentBlock::Thinking { thinking, .. } => thinking.clone(),
                    ContentBlock::ToolUse { id, name, input } => {
                        format!("[ToolUse: {name}({id})] {input}")
                    }
                    ContentBlock::ToolResult {
                        tool_name, output, ..
                    } => format!("[ToolResult: {tool_name}] {output}"),
                })
                .collect::<Vec<_>>()
                .join("\n");

            Message { role, content }
        })
        .collect()
}

// ── Tauri Commands ──────────────────────────────────────────────────────

/// 初始化 LLM 客户端和 Session。如果提供了 session_id，则从磁盘恢复。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_init(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    session_id: Option<String>,
    cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[claw_chat] Initializing LLM client from Claw config");

    // Set workspace directory for tool execution
    if let Some(ref workspace_str) = cwd {
        let workspace_path = std::path::PathBuf::from(workspace_str);
        if workspace_path.exists() {
            log::info!("[claw_chat] Setting workspace: {}", workspace_str);
            let _ = std::env::set_current_dir(&workspace_path);
            {
                let mut ws = state.workspace.lock().await;
                *ws = Some(workspace_path);
            }
        }
    }

    setup_env_from_claw_config()?;

    // ── Session ──
    log::info!("[claw_chat] claw_chat_init: session_id={:?}", session_id);
    let (sid, restored_count) = if let Some(ref existing) = session_id {
        if let Some(loaded) = load_session(existing) {
            let count = loaded.messages.len();
            log::info!("[claw_chat] Restored session {} ({} messages)", existing, count);
            {
                let mut s = state.session.lock().await;
                *s = Some(loaded);
            }
            (existing.clone(), count)
        } else {
            log::info!("[claw_chat] Session {} not found on disk, creating new", existing);
            let new_id = uuid::Uuid::new_v4().to_string();
            let path = session_path(&new_id);
            std::fs::create_dir_all(path.parent().unwrap()).ok();
            // Don't save to disk yet — defer until first message
            let session = Session::new()
                .with_persistence_path(&path);
            {
                let mut s = state.session.lock().await;
                *s = Some(session);
            }
            (new_id, 0)
        }
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        let path = session_path(&new_id);
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        // Don't save to disk yet — defer until first message
        let session = Session::new()
            .with_persistence_path(&path);
        {
            let mut s = state.session.lock().await;
            *s = Some(session);
        }
        (new_id, 0)
    };

    // ── LLM Client ──
    let client = LlmClient::from_env().map_err(|e| {
        let hint = if e.contains("401") || e.contains("Unauthorized") || e.contains("INVALID_API_KEY") {
            " — 请检查 ~/.claw/config.json 中的 API key 是否有效（当前可能是脱敏后的值）"
        } else if e.contains("timeout") || e.contains("timed out") {
            " — API 请求超时，请检查中转站地址是否可达"
        } else {
            ""
        };
        format!("LLM 客户端初始化失败: {e}{hint}")
    })?;
    log::info!(
        "[claw_chat] LLM client initialized: provider={:?}, model={}",
        client.provider(),
        client.model(),
    );
    {
        let mut c = state.client.lock().await;
        *c = Some(Arc::new(client));
    }

    let _ = app.emit(
        "agent-session-created",
        serde_json::json!({
            "session_id": sid,
            "restored": restored_count > 0,
            "message_count": restored_count,
        }),
    );

    let restored_messages = if restored_count > 0 {
        let s = state.session.lock().await;
        s.as_ref()
            .map(|sess| session_messages_to_json(&sess.messages))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    log::info!(
        "[claw_chat] claw_chat_init returning: sid={}, restored={}, message_count={}, restored_messages={}",
        sid, restored_count > 0, restored_count, restored_messages.len()
    );

    Ok(serde_json::json!({
        "sessionId": sid,
        "restored": restored_count > 0,
        "messageCount": restored_count,
        "messages": restored_messages,
    }))
}


#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_send(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    message: String,
) -> Result<(), String> {
    let (client, session_path_buf) = {
        let c = state.client.lock().await;
        let s = state.session.lock().await;
        let client = c.clone().ok_or("Claw not initialized")?;
        let path = s
            .as_ref()
            .and_then(|sess| sess.persistence_path().map(|p| p.to_path_buf()));
        (client, path)
    };
    let session_path = session_path_buf.ok_or("No session path set — call claw_chat_init first")?;

    // ── Push user message & persist ──
    {
        let mut s = state.session.lock().await;
        if let Some(ref mut sess) = *s {
            sess.push_user_text(&message)
                .map_err(|e| format!("Failed to push user message: {e}"))?;
            if let Some(path) = sess.persistence_path() {
                sess.save_to_path(path)
                    .map_err(|e| format!("Failed to save session: {e}"))?;
            }
        }
    }

    // ── Build tool definitions ──
    // ── Read agent behavior settings from config ──
    let agent_config = crate::commands::claw_config::read_claw_config().unwrap_or_default();
    let max_iterations = agent_config.max_iterations as usize;
    let max_retries = agent_config.max_retries as usize;
    let skill_bytes_cap = agent_config.skill_bytes_cap as usize;
    let tool_output_truncation = agent_config.tool_output_truncation as usize;
    let reasoning_effort = if agent_config.reasoning_effort.is_empty() {
        None
    } else {
        Some(agent_config.reasoning_effort)
    };
    let auto_compaction = agent_config.auto_compaction;

    // ── Build tool definitions ──
    let tool_defs = build_tool_definitions();
    let system_prompt = claw_agent_system_prompt(skill_bytes_cap);
    let sid = session_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    log::info!(
        "[claw_chat] Starting tool loop with {} tools, session={}",
        tool_defs.len(), sid
    );

    // ── Take session and run via ConversationRuntime ──
    // ConversationRuntime is !Send (contains Box<dyn HookProgressReporter>),
    // so all runtime creation + run_turn + into_session must happen inside
    // block_in_place, which runs synchronously without Send requirements.
    let taken_session = {
        let mut s = state.session.lock().await;
        s.take().ok_or("No session — call claw_chat_init first")?
    };
    let sid = session_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    log::info!(
        "[claw_chat] Starting ConversationRuntime::run_turn(), session={}, max_iterations={}",
        sid, max_iterations
    );

    let model_name = agent_config.model.clone();
    let max_iters = max_iterations;

    // block_in_place runs synchronously — !Send types stay in this stack frame
    let (summary, session) = tokio::task::block_in_place(move || {
        let api_client = crate::commands::claw_runtime_bridge::TauriApiClient::new(
            client,
            tool_defs,
            reasoning_effort,
            model_name.clone(),
        );
        let tool_executor = crate::commands::claw_runtime_bridge::TauriToolExecutor::default();
        let permission_policy = runtime::PermissionPolicy::new(runtime::PermissionMode::Allow);

        let mut rt = runtime::ConversationRuntime::new(
            taken_session,
            api_client,
            tool_executor,
            permission_policy,
            vec![system_prompt], // Vec<String> as expected by upstream
        )
        .with_max_iterations(max_iters);

        let result = rt.run_turn(message, None);
        let session = rt.into_session();

        match result {
            Ok(s) => (Ok(s), session),
            Err(e) => (Err(format!("Conversation failed: {e}")), session),
        }
    });

    let summary = summary?;
    log::info!(
        "[claw_chat] run_turn completed: {} iterations, {} tools",
        summary.iterations,
        summary.tool_results.len()
    );

    // ── Emit results to frontend (batch mode — previous real-time streaming removed) ──
    let emit = crate::commands::claw_runtime_bridge::turn_summary_to_emit(&summary);

    if !emit.assistant_text.is_empty() {
        let _ = app.emit(
            "agent-delta",
            serde_json::json!({
                "text": emit.assistant_text,
                "session_id": sid,
            }),
        );
    }
    for (tool_id, tool_name, tool_input) in &emit.tool_calls {
        let _ = app.emit(
            "agent-tool-start",
            serde_json::json!({
                "id": tool_id,
                "name": tool_name,
                "args": serde_json::from_str(tool_input).unwrap_or(serde_json::json!({})),
                "session_id": sid,
            }),
        );
        let _ = app.emit(
            "agent-tool-complete",
            serde_json::json!({
                "id": tool_id,
                "name": tool_name,
                "result": "success",
                "session_id": sid,
            }),
        );
    }

    let cost = estimate_cost(emit.input_tokens, emit.output_tokens, &agent_config.model);
    let total_tokens = emit.input_tokens + emit.output_tokens;
    if total_tokens > 0 {
        let _ = app.emit(
            "agent-usage",
            serde_json::json!({
                "prompt_tokens": emit.input_tokens,
                "completion_tokens": emit.output_tokens,
                "total_tokens": total_tokens,
                "cost": cost,
                "session_id": sid,
            }),
        );
    }
    let _ = app.emit(
        "agent-done",
        serde_json::json!({
            "session_id": sid,
            "usage": {
                "prompt_tokens": emit.input_tokens,
                "completion_tokens": emit.output_tokens,
                "total_tokens": total_tokens,
                "cost": cost,
            },
            "auto_compaction": emit.auto_compaction_removed,
        }),
    );

    // ── Save session ──
    {
        let mut s = state.session.lock().await;
        *s = Some(session);
    }
    {
        let s = state.session.lock().await;
        if let Some(ref sess) = *s {
            if let Some(path) = sess.persistence_path() {
                sess.save_to_path(path)
                    .map_err(|e| format!("Failed to save session: {e}"))?;
            }
        }
    }

    log::info!("[claw_chat] Turn completed for session={}", sid);
    Ok(())
}

/// 关闭会话（仅断开 LLM 连接，保留 session 持久化）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_close(
    state: tauri::State<'_, ClawChatState>,
) -> Result<(), String> {
    log::info!("[claw_chat] Closing session (preserving messages)");

    // Final save
    {
        let s = state.session.lock().await;
        if let Some(ref sess) = *s {
            if let Some(path) = sess.persistence_path() {
                let _ = sess.save_to_path(path);
            }
        }
    }

    // Drop LLM client, keep session
    {
        let mut c = state.client.lock().await;
        *c = None;
    }

    Ok(())
}

/// Abort the current tool loop — sets the hook abort signal so the loop
/// exits gracefully after the current tool call completes.
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_abort(
    state: tauri::State<'_, ClawChatState>,
) -> Result<(), String> {
    log::info!("[claw_chat] Abort signal requested");
    state.hook_abort.store(true, Ordering::SeqCst);
    Ok(())
}

/// 获取当前会话列表（从磁盘读取所有持久化的会话）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_list_sessions(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    log::info!("[claw_chat] claw_chat_list_sessions called");
    let mut sessions = list_sessions_info();
    log::info!("[claw_chat] list_sessions_info returned {} sessions", sessions.len());

    // 如果有活跃会话且不在磁盘列表中，加上
    let active_sid = {
        let s = state.session.lock().await;
        s.as_ref().map(|sess| sess.session_id.clone())
    };
    if let Some(ref sid) = active_sid.as_deref() {
        if !sessions.iter().any(|s| {
            s.get("sessionId").and_then(|v| v.as_str()) == Some(sid)
        }) {
            let msg_count = {
                let s = state.session.lock().await;
                s.as_ref().map(|sess| sess.messages.len()).unwrap_or(0)
            };
            sessions.push(serde_json::json!({
                "sessionId": sid,
                "createdAt": chrono::Utc::now().to_rfc3339(),
                "messageCount": msg_count,
                "active": true,
            }));
        }
    }

    Ok(serde_json::json!({ "sessions": sessions }))
}

/// 获取 Claw 客户端信息
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_info() -> Result<serde_json::Value, String> {
    let config = crate::commands::claw_config::read_claw_config()?;
    let api_key_found = !config.api_key.is_empty();
    let model = config.model.clone();
    let provider = config.provider.clone();
    let base_url = if config.base_url.is_empty() {
        None
    } else {
        Some(config.base_url)
    };

    Ok(serde_json::json!({
        "mode": "claw",
        "apiKeyConfigured": api_key_found,
        "baseUrl": base_url,
        "model": model,
        "provider": provider,
        "configSource": "~/.claw/config.json",
    }))
}

/// 读取模型配置
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_models_config() -> Result<serde_json::Value, String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    let mut providers: Vec<serde_json::Value> = Vec::new();

    if !config.api_key.is_empty() || !config.base_url.is_empty() {
        let provider_name = if config.provider.is_empty() {
            if config.model.starts_with("claude") || config.model.starts_with("anthropic/") {
                "Anthropic"
            } else if config.model.starts_with("gpt") || config.model.starts_with("openai/") {
                "OpenAI"
            } else if config.model.starts_with("grok") {
                "xAI"
            } else if config.model.starts_with("qwen") {
                "DashScope"
            } else {
                "Custom"
            }
        } else {
            &config.provider
        };

        providers.push(serde_json::json!({
            "name": provider_name,
            "models": if !config.model.is_empty() {
                vec![serde_json::json!({"id": &config.model, "active": true})]
            } else {
                Vec::<serde_json::Value>::new()
            },
            "apiKey": !config.api_key.is_empty(),
            "baseUrl": &config.base_url,
            "active": true,
        }));
    }

    Ok(serde_json::json!({
        "providers": providers,
        "source": "~/.claw/config.json",
    }))
}

/// 读取聊天统计
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_stats() -> Result<serde_json::Value, String> {
    let sessions = list_sessions_info();
    let total_messages: usize = sessions
        .iter()
        .filter_map(|s| s.get("messageCount").and_then(|v| v.as_u64()))
        .sum::<u64>() as usize;

    Ok(serde_json::json!({
        "sessions": sessions.len(),
        "messages": total_messages,
        "source": "claw",
    }))
}


