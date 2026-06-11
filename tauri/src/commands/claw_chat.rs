use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use runtime::{
    ConfigLoader, ContentBlock, ConversationMessage, MessageRole, PermissionMode, PermissionPolicy,
    RuntimeConfig, RuntimeFeatureConfig, Session,
};
use serde::{Deserialize, Serialize};
use supertool_claw::llm::LlmClient;
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
    pub(crate) plan_mode: Arc<AtomicBool>,
    pub(crate) goal_mode: Arc<AtomicBool>,
    pub(crate) goal_text: Arc<std::sync::Mutex<String>>,
    /// Goal lifecycle status: "inactive" | "active" | "paused" | "done" | "cleared"
    pub(crate) goal_status: Arc<std::sync::Mutex<String>>,
    /// Turns used in the current goal session
    pub(crate) goal_turns_used: Arc<AtomicU32>,
    /// Maximum turns allowed before auto-pause (default 20)
    pub(crate) goal_max_turns: Arc<AtomicU32>,
    /// Last judge verdict: "continue" | "done" | None
    pub(crate) goal_last_verdict: Arc<std::sync::Mutex<Option<String>>>,
    /// Last judge reason text
    pub(crate) goal_last_reason: Arc<std::sync::Mutex<Option<String>>>,
    /// Consecutive judge parse failures (auto-pause after 3)
    pub(crate) goal_consecutive_parse_failures: Arc<AtomicU32>,
    pub(crate) loop_mode: Arc<AtomicBool>,
    /// Abort signal for the current conversation — shared with ConversationRuntime.
    pub(crate) hook_abort: Mutex<runtime::HookAbortSignal>,
}

impl ClawChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session: Mutex::new(None),
            workspace: Mutex::new(None),
            plan_mode: Arc::new(AtomicBool::new(false)),
            goal_mode: Arc::new(AtomicBool::new(false)),
            goal_text: Arc::new(std::sync::Mutex::new(String::new())),
            goal_status: Arc::new(std::sync::Mutex::new("inactive".to_string())),
            goal_turns_used: Arc::new(AtomicU32::new(0)),
            goal_max_turns: Arc::new(AtomicU32::new(20)),
            goal_last_verdict: Arc::new(std::sync::Mutex::new(None)),
            goal_last_reason: Arc::new(std::sync::Mutex::new(None)),
            goal_consecutive_parse_failures: Arc::new(AtomicU32::new(0)),
            loop_mode: Arc::new(AtomicBool::new(false)), // default: off (opt-in via /loop)
            hook_abort: Mutex::new(runtime::HookAbortSignal::new()),
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
            // Read the first 2 lines (meta record + first message) for session info
            // Uses BufReader to avoid loading entire file into memory
            if let Ok(file) = std::fs::File::open(&path) {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                let mut lines = reader.lines();
                let first_line = lines.next().and_then(|l| l.ok()).unwrap_or_default();
                let second_line = lines.next().and_then(|l| l.ok()).unwrap_or_default();

                #[allow(unused_assignments)]
                let mut created_at_ms: u64 = 0;
                #[allow(unused_assignments)]
                let mut title: Option<String> = None;
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&first_line) {
                    created_at_ms = meta
                        .get("created_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let updated_at_ms = meta
                        .get("updated_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    // Extract title from first message
                    if !second_line.is_empty() {
                        title = serde_json::from_str::<serde_json::Value>(&second_line)
                            .ok()
                            .and_then(|v| {
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
                                    let safe_end = c.floor_char_boundary(60);
                                    format!("{}...", &c[..safe_end])
                                } else {
                                    c
                                }
                            });
                    }
                    // Count remaining lines (message count = total - 1 meta line)
                    let mut message_count: usize = 0;
                    let mut model: Option<String> = None;
                    for line_result in lines {
                        if let Ok(line) = line_result {
                            message_count += 1;
                            // Check first assistant message for model name
                            if model.is_none() {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                                    if let Some(msg) = v.get("message") {
                                        if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                                            model = v.get("model").and_then(|m| m.as_str()).map(String::from);
                                        }
                                    }
                                }
                            }
                        }
                    }
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
/// Includes text, thinking, tool_use, and tool_result blocks.
pub(crate) fn session_messages_to_json(messages: &[ConversationMessage]) -> Vec<serde_json::Value> {
    let mut result = Vec::new();
    for cm in messages {
        let role = match cm.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "agent",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        };

        // Extract text from Text blocks only (should be one block per message
        // since ConversationRuntime merges all TextDelta events).
        let text: String = cm
            .blocks
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .concat();

        // Merge all Thinking blocks into a single reasoning string
        let reasoning: String = cm
            .blocks
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Thinking { thinking, .. } => Some(thinking.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .concat();

        // Only emit if there's text content (tool-only messages are handled separately)
        if !text.is_empty() {
            result.push(serde_json::json!({
                "role": role,
                "content": text,
            }));
        }

        // Emit merged thinking content as separate reasoning message
        if !reasoning.is_empty() {
            result.push(serde_json::json!({
                "role": "agent",
                "kind": "reasoning",
                "text": reasoning,
            }));
        }

        // Emit tool_use blocks as separate messages
        for block in &cm.blocks {
            if let ContentBlock::ToolUse { id, name, input } = block {
                result.push(serde_json::json!({
                    "role": "agent",
                    "kind": "tool_call",
                    "callId": id,
                    "name": name,
                    "args": input,
                }));
            }
        }

        // Emit tool_result blocks as separate messages
        for block in &cm.blocks {
            if let ContentBlock::ToolResult { tool_use_id, tool_name, output, is_error } = block {
                result.push(serde_json::json!({
                    "role": "agent",
                    "kind": "tool_result",
                    "callId": tool_use_id,
                    "name": tool_name,
                    "content": output,
                    "isError": is_error,
                }));
            }
        }
    }
    result
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

// ── Goal Mode (Judge Loop) ─────────────────────────────────────────────

/// Judge whether a goal has been achieved by examining the last assistant response.
///
/// Makes a non-streaming LLM call using the existing ProviderClient. Returns
/// `("continue", reason)` if the goal is not yet complete, `("done", reason)` if
/// it is complete. On any error (network, parse, etc.) returns `("continue", error_msg)`
/// to fail-open — the agent keeps working rather than prematurely stopping.
/// Judge whether the agent's last response completes the goal.
/// Returns `(verdict, reason, parse_failed)`.
/// - `verdict`: "done" | "continue" | "skipped"
/// - `parse_failed`: true only when the LLM call succeeded but output was unusable
async fn judge_goal(
    client: Arc<LlmClient>,
    goal_text: &str,
    last_response: &str,
) -> (String, String, bool) {
    if goal_text.trim().is_empty() {
        return ("skipped".to_string(), "empty goal".to_string(), false);
    }
    if last_response.trim().is_empty() {
        return ("continue".to_string(), "empty response (nothing to evaluate)".to_string(), false);
    }

    let system_prompt = "\
You are a strict judge evaluating whether an autonomous agent has achieved a user's stated goal.
You receive the goal text and the agent's most recent response. Your only job is to decide whether
the goal is fully satisfied based on that response.

A goal is DONE only when:
- The response explicitly confirms the goal was completed, OR
- The response clearly shows the final deliverable was produced, OR
- The response explains the goal is unachievable / blocked / needs user input (treat this as DONE with reason describing the block).

Otherwise the goal is NOT done — CONTINUE.

Reply ONLY with a single JSON object on one line:
{\"done\": <true|false>, \"reason\": \"<one-sentence rationale>\"}";

    let user_prompt = format!(
        "Goal:\n{goal}\n\nAgent's most recent response:\n{response}\n\nIs the goal satisfied?",
        goal = goal_text,
        response = last_response,
    );

    let request = api::MessageRequest {
        model: client.model().to_string(),
        max_tokens: 4096,
        messages: vec![
            api::InputMessage {
                role: "user".to_string(),
                content: vec![api::InputContentBlock::Text { text: user_prompt }],
            },
        ],
        system: Some(system_prompt.to_string()),
        tools: None,
        tool_choice: None,
        stream: false,
        temperature: Some(0.0),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        reasoning_effort: None,
        extra_body: std::collections::BTreeMap::new(),
    };

    match client.inner().send_message(&request).await {
        Ok(response) => {
            let response_text: String = response
                .content
                .iter()
                .filter_map(|block| match block {
                    api::OutputContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .concat();

            let trimmed = response_text.trim();

            if trimmed.is_empty() {
                return ("continue".to_string(), "judge returned empty response".to_string(), true);
            }

            // Parse JSON: try full string first, then handle markdown fences, then search inside prose
            let mut text = trimmed;

            // Strip markdown code fences (```json ... ```)
            if text.starts_with("```") {
                text = text.trim_start_matches('`');
                let nl = text.find('\n').unwrap_or(0);
                if nl > 0 {
                    text = text[nl + 1..].trim();
                }
                text = text.trim_end_matches('`').trim();
            }

            // First try: parse the whole blob
            let mut data: Option<serde_json::Value> = None;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
                data = Some(v);
            } else {
                // Second try: pull the first { ... } object out of the text
                if let Some(start) = text.find('{') {
                    if let Some(end) = text[start..].rfind('}') {
                        let candidate = &text[start..=start + end];
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(candidate) {
                            data = Some(v);
                        }
                    }
                }
            }

            match data {
                Some(json) => {
                    let done_val = json.get("done");
                    let done = match done_val {
                        Some(v) if v.is_boolean() => v.as_bool().unwrap_or(false),
                        Some(v) if v.is_string() => {
                            matches!(v.as_str().unwrap_or("").to_lowercase().as_str(), "true" | "yes" | "1" | "done")
                        }
                        _ => false,
                    };
                    let reason = json
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("no reason provided")
                        .to_string();
                    if done {
                        ("done".to_string(), reason, false)
                    } else {
                        ("continue".to_string(), reason, false)
                    }
                }
                None => {
                    log::warn!("[judge_goal] Failed to parse judge response as JSON: raw={}", _truncate(&response_text, 200));
                    ("continue".to_string(), format!("judge reply was not JSON: {}", _truncate(&response_text, 200)), true)
                }
            }
        }
        Err(e) => {
            log::warn!("[judge_goal] Judge LLM call failed: {e}");
            ("continue".to_string(), format!("judge error: {e}"), false)
        }
    }
}

/// Truncate text for logging, adding ellipsis if truncated.
fn _truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        text.to_string()
    } else {
        format!("{}… [truncated]", &text[..limit])
    }
}

/// Extract the last assistant text response from a session (for judge evaluation).
fn get_last_assistant_text(session: &Session) -> String {
    session
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role == MessageRole::Assistant)
        .map(|msg| {
            msg.blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .concat()
        })
        .unwrap_or_default()
}

// ── Goal State Persistence ─────────────────────────────────────────────

fn goals_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("goals")
}

fn goal_state_path(session_id: &str) -> PathBuf {
    goals_dir().join(format!("{session_id}.json"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoalState {
    goal_text: String,
    status: String, // "inactive" | "active" | "paused" | "done" | "cleared"
    turns_used: u32,
    max_turns: u32,
    last_verdict: Option<String>,
    last_reason: Option<String>,
}

fn load_goal_state(session_id: &str) -> Option<GoalState> {
    let path = goal_state_path(session_id);
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}

fn save_goal_state(session_id: &str, state: &GoalState) {
    let path = goal_state_path(session_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(content) = serde_json::to_string_pretty(state) {
        std::fs::write(&path, content).ok();
    }
}

#[allow(dead_code)]
fn clear_goal_state(session_id: &str) {
    let path = goal_state_path(session_id);
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }
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

/// Legacy wrapper — returns tool definitions from the registry,
/// plus plan mode tools (EnterPlanMode / ExitPlanMode).
pub(crate) fn build_tool_definitions() -> Vec<api::ToolDefinition> {
    let registry = build_tool_registry();
    let mut defs = registry.definitions(None);
    // Add plan mode tools — the LLM can call these to toggle plan mode
    defs.push(api::ToolDefinition {
        name: "EnterPlanMode".into(),
        description: Some("Switch to plan-only mode: the agent can read files and explore the codebase but cannot make any changes. Use this before writing code to create a thorough plan first. Call ExitPlanMode to leave this mode.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        }),
    });
    defs.push(api::ToolDefinition {
        name: "ExitPlanMode".into(),
        description: Some("Exit plan-only mode and restore the ability to write/change files. Must be called after EnterPlanMode when ready to implement the plan.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        }),
    });
    // Add goal mode tools — the LLM can set a persistent cross-turn goal
    defs.push(api::ToolDefinition {
        name: "EnterGoalMode".into(),
        description: Some("Switch to goal mode: set a persistent cross-turn goal that the agent works toward across multiple turns. Provide a clear goal text describing what to achieve. Call ExitGoalMode when the goal is completed or no longer needed.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "goal": {
                    "type": "string",
                    "description": "The goal text describing what to achieve"
                }
            },
            "required": ["goal"],
            "additionalProperties": false,
        }),
    });
    defs.push(api::ToolDefinition {
        name: "ExitGoalMode".into(),
        description: Some("Exit goal mode and resume normal chat. Call this when the goal is achieved, abandoned, or no longer needed.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        }),
    });
    // Add loop mode tools — controls iteration limit
    defs.push(api::ToolDefinition {
        name: "EnterLoopMode".into(),
        description: Some("Enable unlimited iteration mode: the agent will automatically continue working without any turn limit. Use this for long-running autonomous tasks. Call ExitLoopMode to restore the configured iteration limit.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        }),
    });
    defs.push(api::ToolDefinition {
        name: "ExitLoopMode".into(),
        description: Some("Disable unlimited iteration mode and restore the configured iteration limit. Call this when you want to bound the agent's autonomy to the configured limit.".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false,
        }),
    });
    defs
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


/// Build runtime plugin state (minimal version without MCP).
/// Mirrors upstream `build_runtime_plugin_state` / `build_runtime_plugin_state_with_loader`
/// in rusty-claude-cli/src/main.rs lines 11916-11945.
///
/// Loads ConfigLoader → RuntimeConfig → feature_config (hooks, permission_rules, etc.)
/// and builds the tool registry with a permission enforcer.
fn build_runtime_state() -> Result<(RuntimeFeatureConfig, tools::GlobalToolRegistry), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config: RuntimeConfig =
        loader.load().map_err(|e| format!("ConfigLoader failed: {e}"))?;
    let feature_config = runtime_config.feature_config().clone();

    // Build tool registry with enforcer
    let mut registry = tools::GlobalToolRegistry::builtin();
    // Set permission enforcer from feature_config's permission_rules
    let enforcer = runtime::permission_enforcer::PermissionEnforcer::new(
        PermissionPolicy::new(PermissionMode::Allow)
            .with_permission_rules(feature_config.permission_rules()),
    );
    registry.set_enforcer(enforcer);

    Ok((feature_config, registry))
}


/// System prompt for the Claw agent — mirrors upstream CLI's build_system_prompt.
/// Calls runtime::load_system_prompt() to get the full config-based prompt with
/// project context, then appends Hermes skills for SuperTool-specific knowledge.
pub(crate) fn claw_agent_system_prompt(skill_bytes_cap: usize) -> Vec<String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

    // Call the same load_system_prompt that the original claw CLI uses.
    // This reads ~/.claw/settings.json (and other config files via ConfigLoader walk-up),
    // discovers project context (git status, CLAUDE.md, AGENTS.md, rules, instructions),
    // and builds the rich multi-section system prompt with tool instructions.
    let mut sections: Vec<String> = match runtime::load_system_prompt(
        cwd,
        chrono::Utc::now().format("%Y-%m-%d").to_string(),
        std::env::consts::OS,
        "26.5",
        api::model_family_identity_for(
            &crate::commands::claw_config::read_claw_config()
                .unwrap_or_default()
                .model,
        ),
    ) {
        Ok(sections) => sections,
        Err(e) => {
            log::warn!(
                "[claw_chat] load_system_prompt failed (falling back to minimal prompt): {e}"
            );
            // Fallback: a minimal prompt when config/config files are unavailable
            vec![
                r#"You are Claw Code, an expert software engineering agent. You have access to tools
for reading, writing, searching, and exploring code. When given a task:

1. Read relevant files to understand context
2. Search for what you need
3. Make precise edits
4. Verify with terminal commands
5. Report findings clearly

Use your tools proactively — do not just describe what you would do."#
                    .to_string(),
            ]
        }
    };

    // Append Hermes skills as an additional section (SuperTool-specific)
    let skills_section = load_hermes_skills(skill_bytes_cap);
    if !skills_section.is_empty() {
        sections.push(skills_section);
    }

    // Append plan mode guidance
    sections.push(
        r#"## Plan Mode

You have access to EnterPlanMode and ExitPlanMode tools. Use them to follow a plan-first workflow:

1. **Before writing code**: Call EnterPlanMode to switch to read-only mode. This blocks write/edit/bash tools.
2. **Explore and plan**: Read files, search the codebase, and present your analysis/plan to the user.
3. **Ready to implement**: Call ExitPlanMode to restore write access, then make your changes.

Always plan before implementing complex changes. If unsure, start with EnterPlanMode."#
            .to_string(),
    );

    // Append goal mode guidance
    sections.push(
        r#"## Goal Mode

You have access to EnterGoalMode and ExitGoalMode tools. Use them to lock onto a persistent cross-turn goal:

1. **Set a goal**: Call EnterGoalMode with a clear goal text describing what to achieve.
2. **Work persistently**: Stay focused on the goal across turns until it is complete.
3. **Complete or abandon**: Call ExitGoalMode when the goal is done or no longer needed.

When goal mode is active, all responses should advance toward the stated goal."#
            .to_string(),
    );

    // Append loop mode guidance
    sections.push(
        r#"## Loop Mode

You have access to EnterLoopMode and ExitLoopMode tools to control auto-continuation:

- **EnterLoopMode** (default): Unlimited iterations — the agent auto-continues working until the task is done, with no turn limit.
- **ExitLoopMode**: Restore the configured iteration limit, so the agent stops after the configured number of tool turns.

Use ExitLoopMode when you want to bound the agent's autonomy to the configured limit."#
            .to_string(),
    );

    sections
}

/// 从 ~/.claw/settings.json 读取 API key 和 base URL，设置到进程环境变量
pub(crate) fn setup_env_from_claw_config() -> Result<(), String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    if config.api_key.is_empty() {
        log::info!("[claw_chat] No ~/.claw/settings.json api_key — falling back to env vars");
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
            " — 请检查 ~/.claw/settings.json 中的 API key 是否有效（当前可能是脱敏后的值）"
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

    // ── Restore persisted goal state for this session ──
    if let Some(goal_state) = load_goal_state(&sid) {
        let was_active = goal_state.status == "active" || goal_state.status == "paused";
        log::info!(
            "[claw_chat] Restored goal state for session {}: status={}, turns_used={}",
            sid, &goal_state.status, goal_state.turns_used
        );
        *state.goal_text.lock().unwrap() = goal_state.goal_text;
        state.goal_turns_used.store(goal_state.turns_used, Ordering::Relaxed);
        state.goal_max_turns.store(goal_state.max_turns, Ordering::Relaxed);
        {
            let mut s = state.goal_status.lock().unwrap();
            *s = goal_state.status;
        }
        {
            let mut v = state.goal_last_verdict.lock().unwrap();
            *v = goal_state.last_verdict;
        }
        {
            let mut r = state.goal_last_reason.lock().unwrap();
            *r = goal_state.last_reason;
        }
        state.goal_mode.store(was_active, Ordering::Relaxed);
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
) -> Result<serde_json::Value, String> {
    let (client, session_path_buf) = {
        let c = state.client.lock().await;
        let s = state.session.lock().await;
        let client = c.clone().ok_or("Claw not initialized — call claw_chat_init first")?;
        let path = s
            .as_ref()
            .and_then(|sess| sess.persistence_path().map(|p| p.to_path_buf()));
        (client, path)
    };
    let session_path = session_path_buf.ok_or("No session path set — call claw_chat_init first")?;

    // NOTE: Don't push user message here — ConversationRuntime::run_turn()
    // handles pushing the user message to the session internally.
    // Pushing here would cause duplicate messages.

    // ── Build tool definitions ──
    // ── Read agent behavior settings from config ──
    let agent_config = crate::commands::claw_config::read_claw_config().unwrap_or_default();
    let max_iterations = agent_config.max_iterations as usize;
    let skill_bytes_cap = agent_config.skill_bytes_cap as usize;
    let reasoning_effort = if agent_config.reasoning_effort.is_empty() {
        None
    } else {
        Some(agent_config.reasoning_effort.clone())
    };

    // Resolve active model config
    let active_model = crate::commands::claw_config::resolve_active_model(&agent_config);
    let compaction_threshold = active_model.compaction_threshold;

    // ── Build runtime state (feature_config + tool registry) ──
    // Mirrors upstream: build_runtime_plugin_state() / build_runtime_with_plugin_state()
    let (feature_config, tool_registry) = build_runtime_state()?;
    let tool_defs = tool_registry.definitions(None);
    let mut system_prompt_sections = claw_agent_system_prompt(skill_bytes_cap);
    let sid = session_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    log::info!(
        "[claw_chat] Starting tool loop with {} tools, session={}",
        tool_defs.len(), sid
    );

    // ── Check goal mode state ──
    let goal_mode_active = state.goal_mode.load(Ordering::Relaxed);
    let goal_text_value = state.goal_text.lock().map(|g| g.clone()).unwrap_or_default();
    if goal_mode_active && !goal_text_value.is_empty() {
        // If goal is paused on user message, set status back to active for judge
        {
            let mut status = state.goal_status.lock().unwrap();
            if *status == "paused" {
                *status = "active".to_string();
            }
        }
        let goal_section = format!(
            r#"## Active Goal

You are in **goal mode**. Your persistent goal is:

> {goal}

Every response must advance toward this goal. Once the goal is complete, call `ExitGoalMode` to resume normal chat."#,
            goal = goal_text_value
        );
        system_prompt_sections.push(goal_section);
        log::info!("[claw_chat] Goal mode active: {}", goal_text_value);
    }

    // Max iterations for a single run_turn (from config, loop mode doesn't affect this)
    let max_iters = max_iterations;
    let loop_mode_active = state.loop_mode.load(Ordering::Relaxed);

    // ── Goal loop ──
    // We run turns in a loop when goal mode is active.
    // The first iteration uses the user's `message`; subsequent iterations use
    // a continuation prompt. Between iterations we make async judge calls.
    let mut current_message = message;
    let mut goal_loop_iteration = 0u32;
    let goal_max_turns = state.goal_max_turns.load(Ordering::Relaxed);
    let mut goal_completed = false;
    let mut goal_paused = false;
    let mut final_auto_compaction: Option<usize> = None;

    // ── User message preemption ──
    // If a goal is already active when the user sends a new message,
    // the user's input preempts the goal. We let the judge evaluate
    // after this turn — if the user's message happens to complete the
    // goal, mark done; otherwise auto-pause the goal.
    let goal_was_active_before = goal_mode_active
        && !goal_text_value.is_empty()
        && {
            let status = state.goal_status.lock().unwrap();
            status.as_str() == "active"
        };
    if goal_was_active_before {
        log::info!("[claw_chat] User message preempts active goal — will auto-pause after judge");
    }

    loop {
        // ── Take session and run via ConversationRuntime ──
        // ConversationRuntime is !Send (contains Box<dyn HookProgressReporter>),
        // so all runtime creation + run_turn + into_session must happen inside
        // block_in_place, which runs synchronously without Send requirements.
        let taken_session = {
            let mut s = state.session.lock().await;
            s.take().ok_or("No session — call claw_chat_init first")?
        };

        log::info!(
            "[claw_chat] Starting ConversationRuntime::run_turn(), session={}, iteration={}",
            sid, goal_loop_iteration
        );

        // Create fresh abort signal for this turn
        let hook_abort_signal = runtime::HookAbortSignal::new();
        {
            let mut abort = state.hook_abort.lock().await;
            *abort = hook_abort_signal.clone();
        }

        // Clone values for each iteration (block_in_place move closure consumes them)
        let client_iter = client.clone();
        let tool_defs_iter = tool_defs.clone();
        let reasoning_iter = reasoning_effort.clone();
        let fc_iter = feature_config.clone();
        let sp_iter = system_prompt_sections.clone();
        let app_hook = app.clone();
        let sid_hook = sid.clone();
        let plan_mode_hook = state.plan_mode.clone();
        let goal_mode_hook = state.goal_mode.clone();
        let goal_text_hook = state.goal_text.clone();
        let loop_mode_hook = state.loop_mode.clone();

        let (summary, session) = tokio::task::block_in_place(move || {
            let api_client = crate::commands::claw_runtime_bridge::TauriApiClient::new(
                client_iter,
                tool_defs_iter,
                reasoning_iter,
                app_hook.clone(),
                sid_hook.clone(),
            );
            let tool_executor = crate::commands::claw_runtime_bridge::TauriToolExecutor::new(
                app_hook.clone(),
                sid_hook.clone(),
                plan_mode_hook.clone(),
                goal_mode_hook.clone(),
                goal_text_hook.clone(),
                loop_mode_hook.clone(),
            );
            // Permission policy with rules from config — mirrors upstream permission_policy()
            let permission_policy = PermissionPolicy::new(PermissionMode::Allow)
                .with_permission_rules(fc_iter.permission_rules());

            let mut rt = runtime::ConversationRuntime::new_with_features(
                taken_session,
                api_client,
                tool_executor,
                permission_policy,
                sp_iter,
                &fc_iter,
            )
            .with_max_iterations(max_iters)
            .with_hook_abort_signal(hook_abort_signal)
            .with_hook_progress_reporter(Box::new(
                crate::commands::claw_runtime_bridge::TauriHookReporter::new(app_hook, sid_hook),
            ))
            // Auto-compaction: trigger based on active model's compaction_threshold
            .with_auto_compaction_input_tokens_threshold(compaction_threshold);

            let result = rt.run_turn(current_message, None);
            let session = rt.into_session();

            match result {
                Ok(s) => (Ok(s), session),
                Err(e) => (Err(format!("Conversation failed: {e}")), session),
            }
        });

        let summary = match summary {
            Ok(s) => s,
            Err(e) => {
                // Restore session even on failure — don't lose user's conversation
                log::error!("[claw_chat] run_turn failed: {e}");
                {
                    let mut s = state.session.lock().await;
                    *s = Some(session);
                }
                return Err(e);
            }
        };

        log::info!(
            "[claw_chat] run_turn completed: {} iterations, {} tools",
            summary.iterations,
            summary.tool_results.len()
        );

        // ── Emit results to frontend ──
        let emit = crate::commands::claw_runtime_bridge::turn_summary_to_emit(&summary);
        final_auto_compaction = emit.auto_compaction_removed;

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

        // ── Goal mode: judge loop ──
        if !goal_mode_active || goal_text_value.is_empty() {
            // Not in goal mode — single turn only
            break;
        }

        // Check abort signal between iterations
        {
            let abort = state.hook_abort.lock().await;
            if abort.is_aborted() {
                log::info!("[claw_chat] Goal loop aborted by user");
                goal_paused = true;
                {
                    let mut status = state.goal_status.lock().unwrap();
                    *status = "paused".to_string();
                }
                break;
            }
        }

        // Check budget
        if goal_loop_iteration >= goal_max_turns {
            log::info!(
                "[claw_chat] Goal turn budget exhausted ({}/{}), pausing",
                goal_loop_iteration, goal_max_turns
            );
            goal_paused = true;
            {
                let mut status = state.goal_status.lock().unwrap();
                *status = "paused".to_string();
            }
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "paused",
                "reason": format!("Turn budget exhausted ({}/{})", goal_loop_iteration, goal_max_turns),
                "session_id": sid,
            }));
            break;
        }

        // ── Judge (async, outside block_in_place) ──
        // Get the last assistant response from the session
        let last_response = {
            let s = state.session.lock().await;
            s.as_ref()
                .map(|sess| get_last_assistant_text(sess))
                .unwrap_or_default()
        };

        let (verdict, reason, parse_failed) = judge_goal(
            client.clone(),
            &goal_text_value,
            &last_response,
        )
        .await;

        // Track consecutive parse failures
        if parse_failed {
            let failures = state.goal_consecutive_parse_failures.fetch_add(1, Ordering::Relaxed) + 1;
            log::warn!("[claw_chat] Goal judge parse failure #{failures}/3");
            if failures >= 3 {
                log::warn!("[claw_chat] Too many consecutive parse failures — auto-pausing goal");
                goal_paused = true;
                {
                    let mut status = state.goal_status.lock().unwrap();
                    *status = "paused".to_string();
                }
                let pause_reason = format!("Judge output parse failed {failures} times in a row — check goal_judge model config");
                // Reset counter
                state.goal_consecutive_parse_failures.store(0, Ordering::Relaxed);
                let _ = app.emit("goal-status", serde_json::json!({
                    "status": "paused",
                    "reason": pause_reason,
                    "session_id": sid,
                }));
                break;
            }
        } else {
            // Reset on successful parse
            state.goal_consecutive_parse_failures.store(0, Ordering::Relaxed);
        }

        // Store verdict
        {
            let mut v = state.goal_last_verdict.lock().unwrap();
            *v = Some(verdict.clone());
        }
        {
            let mut r = state.goal_last_reason.lock().unwrap();
            *r = Some(reason.clone());
        }

        log::info!(
            "[claw_chat] Goal judge verdict: {} — {}",
            verdict, reason
        );

        if verdict == "done" {
            goal_completed = true;
            {
                let mut status = state.goal_status.lock().unwrap();
                *status = "done".to_string();
            }
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "done",
                "reason": reason,
                "session_id": sid,
                "turns_used": goal_loop_iteration + 1,
            }));
            // Persist goal state
            save_goal_state(&sid, &GoalState {
                goal_text: goal_text_value.clone(),
                status: "done".to_string(),
                turns_used: goal_loop_iteration + 1,
                max_turns: goal_max_turns,
                last_verdict: Some(verdict),
                last_reason: Some(reason),
            });
            break;
        }

        // ── User message preemption ──
        // If this turn was triggered by a user message (not a goal continuity),
        // auto-pause the goal since the user has changed focus.
        if goal_was_active_before {
            log::info!("[claw_chat] User preempted goal — pausing (verdict: continue)");
            goal_paused = true;
            {
                let mut status = state.goal_status.lock().unwrap();
                *status = "paused".to_string();
            }
            // Reset parse failure counter
            state.goal_consecutive_parse_failures.store(0, Ordering::Relaxed);
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "paused",
                "reason": format!("User sent a new message while goal was active. Judge says: {reason}"),
                "session_id": sid,
            }));
            break;
        }

        // ── Build continuation message and loop ──
        goal_loop_iteration += 1;
        current_message = format!(
            "[Continuing toward your standing goal]\nGoal: {goal}\n\nContinue working toward this goal. Take the next concrete step. If you believe the goal is complete, state so explicitly and stop. If you are blocked and need input from the user, say so clearly and stop.",
            goal = goal_text_value,
        );

        let _ = app.emit("goal-status", serde_json::json!({
            "status": "continuing",
            "reason": reason,
            "session_id": sid,
            "turns_used": goal_loop_iteration,
            "max_turns": goal_max_turns,
        }));

        // Small yield to allow event loop to process
        tokio::task::yield_now().await;
    }

    log::info!("[claw_chat] Turn completed for session={}", sid);

    // ── Return session metadata ──
    let message_count = {
        let s = state.session.lock().await;
        s.as_ref().map(|sess| sess.messages.len()).unwrap_or(0)
    };

    Ok(serde_json::json!({
        "sessionId": sid,
        "messageCount": message_count,
        "autoCompaction": final_auto_compaction,
        "goalCompleted": goal_completed,
        "goalPaused": goal_paused,
        "goalTurnsUsed": goal_loop_iteration,
        "goalMaxTurns": goal_max_turns,
    }))
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
    let abort = state.hook_abort.lock().await;
    abort.abort();
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
        "configSource": "~/.claw/settings.json",
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
        "source": "~/.claw/settings.json",
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

/// 获取当前 plan 模式状态
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_get_plan_mode(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    let active = state.plan_mode.load(Ordering::Relaxed);
    Ok(serde_json::json!({
        "active": active,
    }))
}

/// 手动切换 plan 模式（前端 UI 调用）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_set_plan_mode(
    state: tauri::State<'_, ClawChatState>,
    active: bool,
) -> Result<serde_json::Value, String> {
    state.plan_mode.store(active, Ordering::Relaxed);
    log::info!("[claw_chat] Plan mode set to {}", active);
    Ok(serde_json::json!({
        "success": true,
        "active": active,
    }))
}

/// 获取当前 goal 模式状态（含完整跟踪信息）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_get_goal_mode(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    let active = state.goal_mode.load(Ordering::Relaxed);
    let goal_text = state.goal_text.lock().unwrap().clone();
    let status = state.goal_status.lock().unwrap().clone();
    let turns_used = state.goal_turns_used.load(Ordering::Relaxed);
    let max_turns = state.goal_max_turns.load(Ordering::Relaxed);
    let last_verdict = state.goal_last_verdict.lock().unwrap().clone();
    let last_reason = state.goal_last_reason.lock().unwrap().clone();
    Ok(serde_json::json!({
        "active": active,
        "goalText": goal_text,
        "status": status,
        "turnsUsed": turns_used,
        "maxTurns": max_turns,
        "lastVerdict": last_verdict,
        "lastReason": last_reason,
    }))
}

/// 手动切换 goal 模式（前端 UI 调用）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_set_goal_mode(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    active: bool,
    goal_text: Option<String>,
) -> Result<serde_json::Value, String> {
    state.goal_mode.store(active, Ordering::Relaxed);
    if let Some(text) = goal_text {
        *state.goal_text.lock().unwrap() = text.clone();
        // Reset tracking state when setting a new goal
        state.goal_turns_used.store(0, Ordering::Relaxed);
        state.goal_max_turns.store(20, Ordering::Relaxed);
        {
            let mut v = state.goal_last_verdict.lock().unwrap();
            *v = None;
        }
        {
            let mut r = state.goal_last_reason.lock().unwrap();
            *r = None;
        }
        {
            let mut s = state.goal_status.lock().unwrap();
            *s = if active { "active".to_string() } else { "inactive".to_string() };
        }
        log::info!("[claw_chat] Goal mode set to active with text: {}", text);
    } else {
        {
            let mut status = state.goal_status.lock().unwrap();
            *status = if active { "active".to_string() } else { "inactive".to_string() };
        }
        log::info!("[claw_chat] Goal mode set to {}", active);
    }

    // Emit goal-status event for frontend
    let _ = app.emit("goal-status", serde_json::json!({
        "status": if active { "active" } else { "inactive" },
        "goalText": state.goal_text.lock().unwrap().clone(),
        "session_id": serde_json::Value::Null,
    }));

    Ok(serde_json::json!({
        "success": true,
        "active": active,
    }))
}

/// 控制 goal 生命周期：pause / resume / clear
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_set_goal_status(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    status: String,
) -> Result<serde_json::Value, String> {
    match status.as_str() {
        "pause" | "paused" => {
            state.goal_mode.store(false, Ordering::Relaxed);
            {
                let mut s = state.goal_status.lock().unwrap();
                *s = "paused".to_string();
            }
            log::info!("[claw_chat] Goal mode paused");
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "paused",
                "session_id": serde_json::Value::Null,
            }));
        }
        "resume" | "active" => {
            state.goal_mode.store(true, Ordering::Relaxed);
            {
                let mut s = state.goal_status.lock().unwrap();
                *s = "active".to_string();
            }
            log::info!("[claw_chat] Goal mode resumed");
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "active",
                "session_id": serde_json::Value::Null,
            }));
        }
        "clear" | "inactive" | "done" => {
            state.goal_mode.store(false, Ordering::Relaxed);
            *state.goal_text.lock().unwrap() = String::new();
            state.goal_turns_used.store(0, Ordering::Relaxed);
            state.goal_max_turns.store(20, Ordering::Relaxed);
            {
                let mut v = state.goal_last_verdict.lock().unwrap();
                *v = None;
            }
            {
                let mut r = state.goal_last_reason.lock().unwrap();
                *r = None;
            }
            {
                let mut s = state.goal_status.lock().unwrap();
                *s = "cleared".to_string();
            }
            log::info!("[claw_chat] Goal mode cleared");
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "cleared",
                "session_id": serde_json::Value::Null,
            }));
        }
        _ => return Err(format!("Invalid goal status: {status}. Use: pause, resume, clear")),
    }

    Ok(serde_json::json!({
        "success": true,
        "status": status,
    }))
}

/// 获取当前 loop 模式状态
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_get_loop_mode(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    let active = state.loop_mode.load(Ordering::Relaxed);
    Ok(serde_json::json!({
        "active": active,
    }))
}

/// 手动切换 loop 模式（前端 UI 调用）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_set_loop_mode(
    state: tauri::State<'_, ClawChatState>,
    active: bool,
) -> Result<serde_json::Value, String> {
    state.loop_mode.store(active, Ordering::Relaxed);
    log::info!("[claw_chat] Loop mode set to {}", active);
    Ok(serde_json::json!({
        "success": true,
        "active": active,
    }))
}
