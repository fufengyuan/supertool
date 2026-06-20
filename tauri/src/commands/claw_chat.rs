use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use runtime::{
    ConfigLoader, ContentBlock, ConversationMessage, MessageRole, PermissionMode, PermissionPolicy,
    RuntimeConfig, RuntimeFeatureConfig, Session,
};
use serde::{Deserialize, Serialize};
use supertool_claw::llm::LlmClient;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// ── Plan Mode State ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct PlanModeState {
    pub enabled: bool,
    pub plan_file_path: Option<String>,
}

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
/// Goal struct — mirrors oh-my-pi's Goal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Goal {
    pub id: String,
    pub objective: String,
    /// "active" | "paused" | "budget-limited" | "complete" | "dropped"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<u32>,
    pub tokens_used: u32,
    pub time_used_seconds: u64,
}

/// GoalModeState — mirrors oh-my-pi's GoalModeState
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GoalModeState {
    pub enabled: bool,
    /// "active" | "exiting"
    pub mode: String,
    pub goal: Goal,
}

/// Claw 聊天状态（单例，存在 app state 中）
pub struct ClawChatState {
    pub(crate) client: Mutex<Option<Arc<LlmClient>>>,
    pub(crate) session: Mutex<Option<Session>>,
    pub(crate) workspace: Mutex<Option<PathBuf>>,
    pub(crate) plan_state: Arc<std::sync::Mutex<PlanModeState>>,
    /// Quick atomic check for goal mode active/inactive
    pub(crate) goal_mode: Arc<AtomicBool>,
    /// Full goal state (Goal + GoalModeState)
    pub(crate) goal_state: Arc<std::sync::Mutex<Option<GoalModeState>>>,
    /// Turns used in the current goal session (for turn budget only)
    pub(crate) goal_turns_used: Arc<std::sync::Mutex<u32>>,
    /// Maximum turns allowed before auto-pause (default 20)
    pub(crate) goal_max_turns: Arc<std::sync::Mutex<u32>>,
    /// Consecutive judge parse failures (auto-pause after 3)
    pub(crate) goal_consecutive_parse_failures: Arc<std::sync::Mutex<u32>>,
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
            plan_state: Arc::new(std::sync::Mutex::new(PlanModeState::default())),
            goal_mode: Arc::new(AtomicBool::new(false)),
            goal_state: Arc::new(std::sync::Mutex::new(None)),
            goal_turns_used: Arc::new(std::sync::Mutex::new(0)),
            goal_max_turns: Arc::new(std::sync::Mutex::new(20)),
            goal_consecutive_parse_failures: Arc::new(std::sync::Mutex::new(0)),
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

// ── Goal State Persistence (stores GoalModeState as JSON) ──────────────

fn goals_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("goals")
}

fn goal_state_path(session_id: &str) -> PathBuf {
    goals_dir().join(format!("{session_id}.json"))
}

fn load_goal_state(session_id: &str) -> Option<GoalModeState> {
    let path = goal_state_path(session_id);
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}

fn save_goal_state(session_id: &str, state: &GoalModeState) {
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
        description: Some(
            "Switch to plan mode: the agent can read files and explore the codebase but cannot make any changes. \
             The agent MUST create a PLAN.md file with the full analysis and plan. \
             Call ExitPlanMode when the plan is ready to be approved and executed."
                .into(),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "objective": {
                    "type": "string",
                    "description": "Optional goal or objective for the planning session"
                }
            },
            "required": [],
            "additionalProperties": false,
        }),
    });
    defs.push(api::ToolDefinition {
        name: "ExitPlanMode".into(),
        description: Some(
            "Exit plan mode and trigger the approval flow. Sets plan_state.enabled = false. \
             Call this when the plan file is complete and ready for review. \
             Pass `approved: true` to indicate the plan is ready for execution; \
             pass `approved: false` or omit to simply cancel plan mode."
                .into(),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "approved": {
                    "type": "boolean",
                    "description": "Whether the plan is approved for execution"
                }
            },
            "required": [],
            "additionalProperties": false,
        }),
    });
    // Add goal mode tool — hidden single `goal` tool (matches oh-my-pi)
    // The goal tool controls goal lifecycle: create, get, complete, resume, drop, budget
    defs.push(api::ToolDefinition {
        name: "goal".into(),
        description: Some("Manage the active goal-mode objective. Use a single `op` field: create (requires objective, optional token_budget), get (returns current goal state), complete (marks goal complete after verification), resume (re-activates paused goal), drop (discards goal without completing), budget (updates token budget).".into()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "op": {
                    "type": "string",
                    "enum": ["create", "get", "complete", "resume", "drop", "budget"],
                    "description": "Goal operation to perform"
                },
                "objective": {
                    "type": "string",
                    "description": "Goal objective (required for op=create)"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Token budget (optional for op=create, required for op=budget)"
                }
            },
            "required": ["op"],
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

<critical>
Plan mode active. You MUST perform READ-ONLY operations only.

You NEVER:
- Create, edit, or delete files (except the plan file)
- Run state-changing commands (git commit, npm install, etc.)
- Make any system changes

To implement: call `ExitPlanMode` with `approved: true` and a clear summary of the plan → user approves → full write access is restored. Call `ExitPlanMode` with `approved: false` to simply cancel plan mode.

You NEVER ask the user to exit plan mode for you; you MUST call `ExitPlanMode` yourself.
</critical>

## Plan File

You MUST create a plan at `PLAN.md` in the sessions directory (`~/.claw/sessions/PLAN.md`).

You MUST use `Write` for incremental updates; use `Edit` only for full replacement.

<caution>
The approval flow:
- **Approved** (`approved: true`): plan is complete → exit plan mode → full write access restored for implementation.
- **Cancelled** (`approved: false`): exit plan mode without approval → write access restored but plan is discarded.

You MUST make the plan file self-contained: include requirements, decisions, key findings, and remaining todos.
</caution>

## Planning Workflow

<procedure>
### Phase 1: Understand
You MUST focus on the request and associated code. Use read/search tools to explore the codebase thoroughly.

### Phase 2: Design
You MUST draft an approach based on exploration. Consider trade-offs briefly, then choose.

### Phase 3: Review
You MUST read critical files. You MUST verify plan matches original request.

### Phase 4: Update Plan
You MUST create/update `PLAN.md` with:
- Recommended approach only
- Paths of critical files to modify
- Verification section: how to test end-to-end

The plan MUST be scannable yet detailed enough to execute.
</procedure>

<directives>
- You MUST use read-only tools to gather information
- You MAY NOT ask the user to approve your plan via conversation text — you MUST use `ExitPlanMode`
</directives>

<critical>
Your turn ends ONLY by:
1. Using read-only tools to gather information, OR
2. Calling `ExitPlanMode` with `approved: true` when the plan is ready

You MUST keep going until the plan is complete.
</critical>"#.to_string(),
    );

    // Append goal mode guidance (matches oh-my-pi goal-mode-active.md)
    sections.push(
        r#"## Goal Mode

You have access to the `goal` tool for managing a persistent cross-turn goal:

- `goal({"op":"create","objective":"..."})` — Set a goal. Provide a clear objective. Optionally include `token_budget` (positive integer) to limit token usage.
- `goal({"op":"get"})` — Get the current goal state and remaining budget.
- `goal({"op":"resume"})` — Resume a paused goal.
- `goal({"op":"complete"})` — Mark the goal complete, but ONLY after you have verified every deliverable against current evidence.
- `goal({"op":"drop"})` — Discard the current goal without completing it.
- `goal({"op":"budget","token_budget":N})` — Update the token budget.

When goal mode is active, the objective persists across turns. Before completing, audit the current state against every concrete deliverable. Do not call `complete` merely because a budget is low or a turn is ending.

Budget exhaustion is not completion. If the work is unfinished, leave the goal active."#
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

    // ── Sub-Agent model (optional, for `Agent` tool delegation) ──
    // Expose the configured sub-agent model to claw-tools via env var so that
    // delegated sub-agents (spawned through the `Agent` tool) default to this
    // faster/cheaper model when the caller doesn't specify one explicitly.
    {
        let config = crate::commands::claw_config::read_claw_config()?;
        if config.sub_agent_model.is_empty() {
            unsafe { std::env::remove_var("CLAW_SUB_AGENT_MODEL") };
        } else {
            log::info!("[claw_chat] Sub-agent default model: {}", config.sub_agent_model);
            unsafe { std::env::set_var("CLAW_SUB_AGENT_MODEL", &config.sub_agent_model) };
        }
    }

    // ── Restore persisted goal state for this session ──
    if let Some(mut goal_state) = load_goal_state(&sid) {
        let was_active = goal_state.goal.status == "active" || goal_state.goal.status == "paused";
        log::info!(
            "[claw_chat] Restored goal state for session {}: status={}, objective={}",
            sid, &goal_state.goal.status, &goal_state.goal.objective
        );
        // Set goal_mode atomic for quick check
        state.goal_mode.store(was_active, Ordering::Relaxed);
        // If paused, disable but keep state
        if !was_active {
            goal_state.enabled = false;
        }
        {
            let mut gs = state.goal_state.lock().unwrap();
            *gs = Some(goal_state);
        }
        // Initialize turns_used for turn budget tracking
        {
            let mut tu = state.goal_turns_used.lock().unwrap();
            *tu = 0;
        }
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
    let goal_state_locked = state.goal_state.lock().unwrap().clone();
    let goal_text_value = goal_state_locked.as_ref().map(|gs| gs.goal.objective.clone()).unwrap_or_default();
    if goal_mode_active && !goal_text_value.is_empty() {
        // If goal is paused on user message, set status back to active for judge
        {
            let mut gs = state.goal_state.lock().unwrap();
            if let Some(ref mut gs_inner) = *gs {
                if gs_inner.goal.status == "paused" {
                    gs_inner.goal.status = "active".to_string();
                }
            }
        }
        let goal_section = format!(
            r#"## Active Goal

You are in **goal mode**. Your persistent goal is:

> {goal}

You have the `goal` tool available to manage this goal. Use `goal({{\"op\":\"get\"}})` to check state, and `goal({{\"op\":\"complete\"}})` only after verifying every deliverable against current evidence.

Every response must advance toward this goal. The objective persists across turns."#,
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
    let goal_max_turns = *state.goal_max_turns.lock().unwrap();
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
            let gs = state.goal_state.lock().unwrap();
            gs.as_ref().map(|g| g.goal.status.as_str() == "active").unwrap_or(false)
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

        // Check abort before starting a new turn (e.g. user clicked stop
        // while a goal loop was between iterations)
        {
            let abort = state.hook_abort.lock().await;
            if abort.is_aborted() {
                log::info!("[claw_chat] Abort signaled before turn — stopping");
                // Restore session so state isn't lost
                {
                    let mut s = state.session.lock().await;
                    *s = Some(taken_session);
                }
                return Err("Cancelled by user".to_string());
            }
        }

        // Clone values for each iteration (block_in_place move closure consumes them)
        let client_iter = client.clone();
        let tool_defs_iter = tool_defs.clone();
        let reasoning_iter = reasoning_effort.clone();
        let fc_iter = feature_config.clone();
        let sp_iter = system_prompt_sections.clone();
        let app_hook = app.clone();
        let sid_hook = sid.clone();
        let plan_state_hook = state.plan_state.clone();
        let goal_mode_hook = state.goal_mode.clone();
        let goal_state_hook = state.goal_state.clone();
        let loop_mode_hook = state.loop_mode.clone();
        let abort_signal_iter = hook_abort_signal.clone();

        let (summary, session) = tokio::task::block_in_place(move || {
            let api_client = crate::commands::claw_runtime_bridge::TauriApiClient::new(
                client_iter,
                tool_defs_iter,
                reasoning_iter,
                app_hook.clone(),
                sid_hook.clone(),
                Some(abort_signal_iter),
            );
            let tool_executor = crate::commands::claw_runtime_bridge::TauriToolExecutor::new(
                app_hook.clone(),
                sid_hook.clone(),
                plan_state_hook.clone(),
                goal_mode_hook.clone(),
                goal_state_hook.clone(),
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

        // ── Check abort after turn completes (all modes) ──
        // This catches abort signals sent during the LLM call / tool execution.
        // For non-goal mode this is the ONLY opportunity to stop; for goal mode
        // it runs before the inter-iteration check below.
        {
            let abort = state.hook_abort.lock().await;
            if abort.is_aborted() {
                log::info!("[claw_chat] Turn completed but abort was signaled — stopping");
                goal_paused = true;
                {
                    let mut gs = state.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.status = "paused".to_string();
                        gs_inner.enabled = false;
                    }
                }
                let _ = app.emit(
                    "agent-done",
                    serde_json::json!({
                        "session_id": sid,
                        "aborted": true,
                    }),
                );
                break;
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
                    let mut gs = state.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.status = "paused".to_string();
                        gs_inner.enabled = false;
                    }
                }
                break;
            }
        }

        // Check turn budget
        if goal_loop_iteration >= goal_max_turns {
            log::info!(
                "[claw_chat] Goal turn budget exhausted ({}/{}), pausing",
                goal_loop_iteration, goal_max_turns
            );
            goal_paused = true;
            {
                let mut gs = state.goal_state.lock().unwrap();
                if let Some(ref mut gs_inner) = *gs {
                    gs_inner.goal.status = "paused".to_string();
                    gs_inner.enabled = false;
                }
            }
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "paused",
                "reason": format!("Turn budget exhausted ({}/{})", goal_loop_iteration, goal_max_turns),
                "session_id": sid,
            }));
            break;
        }

        // Check token budget (if goal has one)
        {
            let gs = state.goal_state.lock().unwrap();
            if let Some(ref gs_inner) = *gs {
                if gs_inner.goal.token_budget.is_some()
                    && gs_inner.goal.tokens_used >= gs_inner.goal.token_budget.unwrap()
                    && gs_inner.goal.status == "active"
                {
                    let mut gs_mut = state.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs_mut {
                        gs_inner.goal.status = "budget-limited".to_string();
                        gs_inner.enabled = false;
                    }
                    log::info!(
                        "[claw_chat] Goal token budget exhausted (used={}), pausing",
                        gs_inner.goal.tokens_used
                    );
                    goal_paused = true;
                    let _ = app.emit("goal-status", serde_json::json!({
                        "status": "budget-limited",
                        "reason": format!("Token budget exhausted ({} tokens used)", gs_inner.goal.tokens_used),
                        "session_id": sid,
                    }));
                    break;
                }
            }
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
            let mut failures = state.goal_consecutive_parse_failures.lock().unwrap();
            *failures += 1;
            let count = *failures;
            log::warn!("[claw_chat] Goal judge parse failure #{count}/3");
            if count >= 3 {
                log::warn!("[claw_chat] Too many consecutive parse failures — auto-pausing goal");
                goal_paused = true;
                {
                    let mut gs = state.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.status = "paused".to_string();
                        gs_inner.enabled = false;
                    }
                }
                let pause_reason = format!("Judge output parse failed {count} times in a row — check goal_judge model config");
                // Reset counter
                *failures = 0;
                let _ = app.emit("goal-status", serde_json::json!({
                    "status": "paused",
                    "reason": pause_reason,
                    "session_id": sid,
                }));
                break;
            }
        } else {
            // Reset on successful parse
            let mut failures = state.goal_consecutive_parse_failures.lock().unwrap();
            *failures = 0;
        }

        log::info!(
            "[claw_chat] Goal judge verdict: {} — {}",
            verdict, reason
        );

        if verdict == "done" {
            goal_completed = true;
            {
                let mut gs = state.goal_state.lock().unwrap();
                if let Some(ref mut gs_inner) = *gs {
                    gs_inner.goal.status = "complete".to_string();
                    gs_inner.mode = "exiting".to_string();
                    gs_inner.enabled = false;
                }
            }
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "complete",
                "reason": reason,
                "session_id": sid,
                "turns_used": goal_loop_iteration + 1,
            }));
            // Persist goal state (save as GoalModeState format)
            {
                let gs = state.goal_state.lock().unwrap();
                if let Some(ref gs_inner) = *gs {
                    save_goal_state(&sid, gs_inner);
                }
            }
            // Update goal_mode atomic to false
            state.goal_mode.store(false, Ordering::Relaxed);
            break;
        }

        // ── User message preemption ──
        // If this turn was triggered by a user message (not a goal continuity),
        // auto-pause the goal since the user has changed focus.
        if goal_was_active_before {
            log::info!("[claw_chat] User preempted goal — pausing (verdict: continue)");
            goal_paused = true;
            {
                let mut gs = state.goal_state.lock().unwrap();
                if let Some(ref mut gs_inner) = *gs {
                    gs_inner.goal.status = "paused".to_string();
                    gs_inner.enabled = false;
                }
            }
            // Reset parse failure counter
            {
                let mut failures = state.goal_consecutive_parse_failures.lock().unwrap();
                *failures = 0;
            }
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "paused",
                "reason": format!("User sent a new message while goal was active. Judge says: {reason}"),
                "session_id": sid,
            }));
            break;
        }

        // ── Build continuation message with oh-my-pi style audit instructions ──
        goal_loop_iteration += 1;

        let tokens_used_str = {
            let gs = state.goal_state.lock().unwrap();
            gs.as_ref().map(|g| g.goal.tokens_used.to_string()).unwrap_or_default()
        };
        let token_budget_str = {
            let gs = state.goal_state.lock().unwrap();
            gs.as_ref().and_then(|g| g.goal.token_budget.map(|b| b.to_string())).unwrap_or_else(|| "none".to_string())
        };
        let remaining_tokens_str = {
            let gs = state.goal_state.lock().unwrap();
            gs.as_ref().map(|g| {
                g.goal.token_budget.map(|b| {
                    let remaining = if b > g.goal.tokens_used { b - g.goal.tokens_used } else { 0 };
                    remaining.to_string()
                }).unwrap_or_else(|| "unbounded".to_string())
            }).unwrap_or_else(|| "unbounded".to_string())
        };
        let time_used_str = {
            let gs = state.goal_state.lock().unwrap();
            gs.as_ref().map(|g| g.goal.time_used_seconds.to_string()).unwrap_or_default()
        };

        current_message = format!(
            r#"Continue work on the active goal.

<objective>
{goal}
</objective>

Budget:
- Tokens used: {tokens_used}
- Token budget: {token_budget}
- Tokens remaining: {remaining_tokens}
- Time used: {time_used} seconds

This is an autonomous continuation. The objective persists across turns; do not redefine success around a smaller, easier, or already-completed subset.

Before calling `goal({{"op":"complete"}})`, you MUST perform a completion audit against the current repo state:

1. **Restate the objective as concrete deliverables.** What files, behaviors, tests, gates, or artifacts must exist for the objective to be true? Write them down (in your reasoning).
2. **Map each deliverable to evidence.** For every requirement, identify the authoritative source that would prove it: a file's contents, a command's output, a test's pass status, a PR/issue state.
3. **Inspect the actual current state.** Read the files. Run the commands. Check the tests. Do not rely on memory of earlier work in this session — the repo may have changed.
4. **Match verification scope to claim scope.** A narrow check (one file passes its unit test) does not prove a broad claim (the feature works end-to-end).
5. **Treat uncertainty as not-yet-achieved.** Indirect evidence, partial coverage, missing artifacts, or "looks right" without inspection mean continue working. Gather stronger evidence or do more work.
6. **Budget exhaustion is not completion.** Do not call complete merely because tokens are nearly out. If the budget is tight and the work is unfinished, leave the goal active and stop the turn.

Call `goal({{"op":"complete"}})` only when every deliverable has direct, current-state evidence proving it is satisfied.

If the work is not done, just keep working. Do not narrate that you are continuing — execute."#,
            goal = goal_text_value,
            tokens_used = tokens_used_str,
            token_budget = token_budget_str,
            remaining_tokens = remaining_tokens_str,
            time_used = time_used_str,
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
    let ps = state.plan_state.lock().unwrap().clone();
    Ok(serde_json::json!({
        "active": ps.enabled,
        "planFilePath": ps.plan_file_path,
    }))
}

/// 手动切换 plan 模式（前端 UI 调用）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_set_plan_mode(
    state: tauri::State<'_, ClawChatState>,
    active: bool,
) -> Result<serde_json::Value, String> {
    {
        let mut ps = state.plan_state.lock().unwrap();
        ps.enabled = active;
    }
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
    let turns_used = *state.goal_turns_used.lock().unwrap();
    let max_turns = *state.goal_max_turns.lock().unwrap();
    let goal_state = state.goal_state.lock().unwrap().clone();
    let (goal_text, status, tokens_used, token_budget, time_used_seconds, mode) = match goal_state {
        Some(ref gs) => (
            gs.goal.objective.clone(),
            gs.goal.status.clone(),
            gs.goal.tokens_used,
            gs.goal.token_budget,
            gs.goal.time_used_seconds,
            gs.mode.clone(),
        ),
        None => (
            String::new(),
            "inactive".to_string(),
            0,
            None,
            0u64,
            "active".to_string(),
        ),
    };
    Ok(serde_json::json!({
        "active": active,
        "goalText": goal_text,
        "status": status,
        "turnsUsed": turns_used,
        "maxTurns": max_turns,
        "tokensUsed": tokens_used,
        "tokenBudget": token_budget,
        "timeUsedSeconds": time_used_seconds,
        "mode": mode,
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
        // Create or update goal state with oh-my-pi style GoalModeState
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        let goal_id = format!("goal-{}", now_ms);
        let mut gs = state.goal_state.lock().unwrap();
        *gs = Some(GoalModeState {
            enabled: active,
            mode: if active { "active".to_string() } else { "exiting".to_string() },
            goal: Goal {
                id: goal_id,
                objective: text.clone(),
                status: if active { "active".to_string() } else { "dropped".to_string() },
                token_budget: None,
                tokens_used: 0,
                time_used_seconds: 0,
            },
        });
        drop(gs);
        // Reset tracking state when setting a new goal
        {
            let mut tu = state.goal_turns_used.lock().unwrap();
            *tu = 0;
        }
        {
            let mut mt = state.goal_max_turns.lock().unwrap();
            *mt = 20;
        }
        {
            let mut cf = state.goal_consecutive_parse_failures.lock().unwrap();
            *cf = 0;
        }
        log::info!("[claw_chat] Goal mode set to active with text: {}", text);
    } else {
        if !active {
            // Turning off — update goal state
            let mut gs = state.goal_state.lock().unwrap();
            if let Some(ref mut gs_inner) = *gs {
                gs_inner.enabled = false;
                gs_inner.mode = "exiting".to_string();
                if gs_inner.goal.status == "active" {
                    gs_inner.goal.status = "paused".to_string();
                }
            }
        } else {
            // Turning on — if there's a paused goal, mark enabled
            let mut gs = state.goal_state.lock().unwrap();
            if let Some(ref mut gs_inner) = *gs {
                gs_inner.enabled = true;
                gs_inner.mode = "active".to_string();
                if gs_inner.goal.status == "paused" || gs_inner.goal.status == "budget-limited" {
                    gs_inner.goal.status = "active".to_string();
                }
            }
        }
        log::info!("[claw_chat] Goal mode set to {}", active);
    }

    // Emit goal-status event for frontend
    let curr_text = state.goal_state.lock().unwrap().as_ref().map(|g| g.goal.objective.clone()).unwrap_or_default();
    let _ = app.emit("goal-status", serde_json::json!({
        "status": if active { "active" } else { "inactive" },
        "goalText": curr_text,
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
                let mut gs = state.goal_state.lock().unwrap();
                if let Some(ref mut gs_inner) = *gs {
                    gs_inner.goal.status = "paused".to_string();
                    gs_inner.enabled = false;
                }
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
                let mut gs = state.goal_state.lock().unwrap();
                if let Some(ref mut gs_inner) = *gs {
                    gs_inner.goal.status = "active".to_string();
                    gs_inner.enabled = true;
                    gs_inner.mode = "active".to_string();
                }
            }
            log::info!("[claw_chat] Goal mode resumed");
            let _ = app.emit("goal-status", serde_json::json!({
                "status": "active",
                "session_id": serde_json::Value::Null,
            }));
        }
        "clear" | "inactive" | "done" => {
            state.goal_mode.store(false, Ordering::Relaxed);
            {
                let mut gs = state.goal_state.lock().unwrap();
                *gs = None;
            }
            {
                let mut tu = state.goal_turns_used.lock().unwrap();
                *tu = 0;
            }
            {
                let mut mt = state.goal_max_turns.lock().unwrap();
                *mt = 20;
            }
            {
                let mut cf = state.goal_consecutive_parse_failures.lock().unwrap();
                *cf = 0;
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
