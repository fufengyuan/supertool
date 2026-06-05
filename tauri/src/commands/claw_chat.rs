use std::path::PathBuf;
use std::sync::Arc;

use runtime::{
    ContentBlock, ConversationMessage, MessageRole, Session,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message, TurnResult};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Claw 聊天状态（单例，存在 app state 中）
pub struct ClawChatState {
    pub(crate) client: Mutex<Option<Arc<LlmClient>>>,
    pub(crate) session: Mutex<Option<Session>>,
    pub(crate) workspace: Mutex<Option<PathBuf>>,
}

impl ClawChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session: Mutex::new(None),
            workspace: Mutex::new(None),
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
                            serde_json::from_str(input).unwrap_or(serde_json::json!({}));
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
                    log::info!("[claw_chat] Loaded plugin: {}", manifest.name);
                    // Plugin tools would be executed via their commands
                    // For now, log the plugin discovery
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

/// Convert a TurnResult to a runtime ConversationMessage (assistant turn).
pub(crate) fn turn_result_to_assistant_message(result: &TurnResult) -> ConversationMessage {
    let mut blocks: Vec<ContentBlock> = Vec::new();
    if !result.reasoning.is_empty() {
        blocks.push(ContentBlock::Thinking {
            thinking: result.reasoning.clone(),
            signature: None,
        });
    }
    for (id, name, input) in &result.tool_calls {
        let input_str = serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string());
        blocks.push(ContentBlock::ToolUse {
            id: id.clone(),
            name: name.clone(),
            input: input_str,
        });
    }
    if !result.text.is_empty() || blocks.is_empty() {
        blocks.push(ContentBlock::Text {
            text: result.text.clone(),
        });
    }
    ConversationMessage::assistant(blocks)
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


/// Send a turn with automatic retry on transient streaming errors.
/// Retries once on stream errors (network timeout, stall) — excludes auth errors.
async fn send_turn_with_retry(
    client: &LlmClient,
    messages: Vec<api::InputMessage>,
    system_prompt: &str,
    tool_defs: &[api::ToolDefinition],
    app: &tauri::AppHandle,
    sid: &str,
    reasoning_effort: Option<String>,
    max_retries: usize,
) -> Result<TurnResult, String> {
    for attempt in 0..=max_retries {
        let app_clone = app.clone();
        let sid_clone = sid.to_string();
        let td = tool_defs.to_vec();

        match client
            .send_turn(
                messages.clone(),
                Some(system_prompt),
                Some(td),
                reasoning_effort.clone(),
                Some(move |event| match event {
                    LlmStreamEvent::TextDelta { text } => {
                        let _ = app_clone.emit(
                            "agent-delta",
                            serde_json::json!({
                                "text": text,
                                "session_id": sid_clone,
                            }),
                        );
                    }
                    LlmStreamEvent::ThinkingDelta { thinking } => {
                        let _ = app_clone.emit(
                            "agent-reasoning-delta",
                            serde_json::json!({
                                "text": thinking,
                                "session_id": sid_clone,
                            }),
                        );
                    }
                    LlmStreamEvent::ToolCall { id, name, input } => {
                        let _ = app_clone.emit(
                            "agent-tool-start",
                            serde_json::json!({
                                "id": id,
                                "name": name,
                                "args": input,
                                "session_id": sid_clone,
                            }),
                        );
                    }
                    LlmStreamEvent::Usage {
                        input_tokens,
                        output_tokens,
                    } => {
                        let _ = app_clone.emit(
                            "agent-usage",
                            serde_json::json!({
                                "prompt_tokens": input_tokens,
                                "completion_tokens": output_tokens,
                                "total_tokens": input_tokens + output_tokens,
                                "session_id": sid_clone,
                            }),
                        );
                    }
                    LlmStreamEvent::Done => {}
                }),
            )
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Don't retry auth errors or intentional cancellations
                if e.contains("401") || e.contains("Unauthorized") || e.contains("INVALID_API_KEY") {
                    return Err(e);
                }
                if attempt < max_retries {
                    log::warn!("[claw_chat] Stream error (attempt {}/{}): {}, retrying...",
                        attempt + 1, max_retries + 1, e);
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                } else {
                    let hint = if e.contains("timeout") || e.contains("timed out") {
                        " — 请求超时，请检查网络连接"
                    } else {
                        ""
                    };
                    return Err(format!("发送失败: {e}{hint}"));
                }
            }
        }
    }
    unreachable!()
}

/// 发送消息到 LLM — 完整工具循环（含 tools + execute_tool + loop）
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
                let _ = sess.save_to_path(path);
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

    log::info!(
        "[claw_chat] Agent config: max_iterations={}, max_retries={}, skill_bytes_cap={}, auto_compaction={}",
        max_iterations, max_retries, skill_bytes_cap / 1024, auto_compaction
    );

    for iteration in 0..max_iterations {
        log::info!("[claw_chat] Tool loop iteration {}", iteration + 1);

        // ── Auto-compaction: compress old messages if context is too large ──
        if auto_compaction {
            let mut s = state.session.lock().await;
            if let Some(ref mut sess) = *s {
                let compact_config = runtime::CompactionConfig::default();
                if runtime::should_compact(sess, compact_config) {
                    log::info!("[claw_chat] Auto-compacting session ({} messages)", sess.messages.len());
                    let result = runtime::compact_session(sess, runtime::CompactionConfig::default());
                    if result.removed_message_count > 0 {
                        log::info!("[claw_chat] Compaction: removed {} messages",
                            result.removed_message_count);
                        *sess = result.compacted_session;
                        if let Some(path) = sess.persistence_path() {
                            let _ = sess.save_to_path(path);
                        }
                    }
                }
            }
        }

        // Convert session messages to InputMessage format
        let input_messages = {
            let s = state.session.lock().await;
            match s.as_ref() {
                Some(sess) => session_to_input_messages(&sess.messages),
                None => return Err("No session".into()),
            }
        };

        log::info!(
            "[claw_chat] Sending request ({} messages, {} tools)",
            input_messages.len(),
            tool_defs.len()
        );

        // Call LLM with tools — context-window overflow recovery matches CLI:
        // On context_window error, progressively compact (preserve 4→2→1→0) and retry.
        let result = 'turn: {
            let max_compact_rounds = 4;
            let preserve_schedule = [4, 2, 1, 0];

            for compact_round in 0..=max_compact_rounds {
                // Wrap each LLM call with a 120-second timeout to prevent indefinite hangs
                let turn_result = match tokio::time::timeout(
                    std::time::Duration::from_secs(120),
                    send_turn_with_retry(
                        &client,
                        input_messages.clone(),
                        &system_prompt,
                        &tool_defs,
                        &app,
                        &sid,
                        reasoning_effort.clone(),
                        max_retries,
                    ),
                ).await {
                    Ok(result) => result,
                    Err(_) => {
                        log::error!("[claw_chat] LLM request timed out after 120s");
                        let _ = app.emit("agent-error", serde_json::json!({
                            "message": "LLM 请求超时（120秒），请检查网络连接后重试",
                            "session_id": sid,
                        }));
                        return Err("LLM 请求超时（120秒）".into());
                    }
                };

                match turn_result {
                    Ok(result) => break 'turn result,
                    Err(e) => {
                        let is_context_window = e.contains("context_window")
                            || e.contains("Context window")
                            || e.contains("no parseable body")
                            || e.contains("maximum context length");

                        if !is_context_window || compact_round >= max_compact_rounds {
                            // Not a context error, or exhausted all rounds — emit error and fail
                            let _ = app.emit("agent-error", serde_json::json!({
                                "message": format!("发送失败: {e}"),
                                "session_id": sid,
                            }));
                            return Err(format!("发送失败: {e}"));
                        }

                        // Progressive compaction: each round preserves fewer messages
                        let preserve = preserve_schedule[compact_round.min(3)];
                        log::warn!("[claw_chat] Context window overflow, auto-compacting (round {}/{}, preserving {} recent messages)",
                            compact_round + 1, max_compact_rounds, preserve);

                        let mut s = state.session.lock().await;
                        if let Some(ref mut sess) = *s {
                            let config = runtime::CompactionConfig {
                                preserve_recent_messages: preserve,
                                max_estimated_tokens: 0, // aggressive: always compact
                            };
                            let result = runtime::compact_session(sess, config);
                            if result.removed_message_count == 0 {
                                log::warn!("[claw_chat] No further compaction possible");
                                break;
                            }
                            log::info!("[claw_chat] Compaction round {}: removed {} messages",
                                compact_round + 1, result.removed_message_count);
                            *sess = result.compacted_session;
                            if let Some(path) = sess.persistence_path() {
                                let _ = sess.save_to_path(path);
                            }
                        }

                        // Re-build input messages from compacted session
                        drop(s); // release lock before next iteration
                    }
                }
            }
            return Err("Auto-compaction exhausted without resolving context overflow".into());
        };
        // Push assistant message (with tool_use blocks) to session
        let assistant_msg = turn_result_to_assistant_message(&result);
        {
            let mut s = state.session.lock().await;
            if let Some(ref mut sess) = *s {
                sess.push_message(assistant_msg)
                    .map_err(|e| format!("Failed to push assistant message: {e}"))?;
            }
        }

        log::info!(
            "[claw_chat] LLM responded: text={} chars, reasoning={} chars, tools={}",
            result.text.len(),
            result.reasoning.len(),
            result.tool_calls.len()
        );

        // If no tool calls, we're done
        if result.tool_calls.is_empty() {
            break;
        }

        // ── Execute tools ──
        for (tool_id, tool_name, tool_input) in &result.tool_calls {
            log::info!("[claw_chat] Executing tool: {} (id={})", tool_name, tool_id);

            // Set workspace directory before each tool call
            {
                let ws = state.workspace.lock().await;
                if let Some(ref workspace_path) = *ws {
                    let _ = std::env::set_current_dir(workspace_path);
                }
            }

            let _ = app.emit(
                "agent-tool-start",
                serde_json::json!({
                    "id": tool_id,
                    "name": tool_name,
                    "args": tool_input,
                    "session_id": sid,
                }),
            );

            // ── Pre-tool hook (matches upstream three-state: cancelled/failed/denied) ──
            let hook_runner = load_hook_runner();
            let hook_result = hook_runner.run_pre_tool_use(tool_name, &tool_input.to_string());

            // Apply hook-modified input if provided (upstream: pre_hook_result.updated_input())
            let effective_tool_input: serde_json::Value = if let Some(updated) = hook_result.updated_input() {
                log::info!("[claw_chat] Pre-hook modified input for {}: {}", tool_name, &updated[..updated.len().min(100)]);
                serde_json::from_str(updated).unwrap_or_else(|_| serde_json::json!({"raw": updated}))
            } else {
                tool_input.clone()
            };

            // Three-state check: cancelled / failed / denied (matches upstream conversation.rs:421-456)
            if hook_result.is_cancelled() || hook_result.is_failed() || hook_result.is_denied() {
                let reason = if !hook_result.messages().is_empty() {
                    hook_result.messages().join("\n")
                } else if hook_result.is_cancelled() {
                    format!("PreToolUse hook cancelled tool `{tool_name}`")
                } else if hook_result.is_failed() {
                    format!("PreToolUse hook failed for tool `{tool_name}`")
                } else {
                    format!("PreToolUse hook denied tool `{tool_name}`")
                };
                log::warn!("[claw_chat] Pre-tool hook rejected {}: {}", tool_name, reason);
                let tool_msg = ConversationMessage::tool_result(
                    tool_id, tool_name, reason, true);
                {
                    let mut s = state.session.lock().await;
                    if let Some(ref mut sess) = *s {
                        let _ = sess.push_message(tool_msg);
                    }
                }
                continue;
            }

            // Execute the tool — use spawn_blocking to avoid tokio runtime nesting.
            // upstream CLI runs tools in std::thread::spawn (no tokio runtime),
            // we must do the equivalent via spawn_blocking since claw_chat_send is async.
            let tn = tool_name.to_string();
            let ti = effective_tool_input.clone();
            let (mut output, mut is_error) = match tokio::task::spawn_blocking(move || {
                tools::execute_tool(&tn, &ti)
            })
            .await
            {
                Ok(Ok(output)) => {
                    log::info!(
                        "[claw_chat] Tool {} completed: {} chars",
                        tool_name,
                        output.len()
                    );
                    (output, false)
                }
                Ok(Err(e)) => {
                    log::error!("[claw_chat] Tool {} failed: {}", tool_name, e);
                    (e, true)
                }
                Err(join_err) => {
                    log::error!("[claw_chat] Tool {} panicked: {}", tool_name, join_err);
                    (format!("Tool panicked: {join_err}"), true)
                }
            };

            // ── Post-tool hook (matches upstream: can flip is_error + merge feedback) ──
            let mut post_hook_result = if is_error {
                hook_runner.run_post_tool_use_failure(tool_name, &effective_tool_input.to_string(), &output)
            } else {
                hook_runner.run_post_tool_use(tool_name, &effective_tool_input.to_string(), &output, false)
            };
            // If post-hook denies/fails/cancels, mark output as error (upstream conversation.rs:482-487)
            if post_hook_result.is_denied() || post_hook_result.is_failed() || post_hook_result.is_cancelled() {
                is_error = true;
            }
            // Merge hook feedback into output (upstream conversation.rs:488-494)
            if !post_hook_result.messages().is_empty() {
                let label = if is_error { "Hook feedback (error)" } else { "Hook feedback" };
                let mut sections = Vec::new();
                if !output.trim().is_empty() {
                    sections.push(output);
                }
                sections.push(format!("{label}: {}", post_hook_result.messages().join("\n")));
                output = sections.join("\n\n");
            }

            // Truncate very large tool outputs — UTF-8 safe boundary
            let truncated_output = if output.len() > tool_output_truncation {
                log::warn!(
                    "[claw_chat] Truncating tool output from {} to {}K chars",
                    output.len(), tool_output_truncation / 1024
                );
                let safe_end = output.floor_char_boundary(tool_output_truncation);
                format!("{}...\n\n[Output truncated — was {} chars total]", &output[..safe_end], output.len())
            } else {
                output
            };

            // Push tool result to session

            let tool_result_msg =
                ConversationMessage::tool_result(tool_id, tool_name, truncated_output, is_error);
            {
                let mut s = state.session.lock().await;
                if let Some(ref mut sess) = *s {
                    sess.push_message(tool_result_msg)
                        .map_err(|e| format!("Failed to push tool result: {e}"))?;
                }
            }

            let _ = app.emit(
                "agent-tool-complete",
                serde_json::json!({
                    "id": tool_id,
                    "name": tool_name,
                    "result": if is_error { "error" } else { "success" },
                    "duration_ms": 0,
                    "session_id": sid,
                }),
            );
        }

        // Continue loop — send tool results back to LLM
    }

    // ── Emit agent-done ──
    let _ = app.emit(
        "agent-done",
        serde_json::json!({
            "session_id": sid,
        }),
    );

    // ── Persist final session ──
    {
        let s = state.session.lock().await;
        if let Some(ref sess) = *s {
            let _ = sess.save_to_path(&session_path);
        }
    }

    log::info!("[claw_chat] Tool loop completed for session={}", sid);
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


