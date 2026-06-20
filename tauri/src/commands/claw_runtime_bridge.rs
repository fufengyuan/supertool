//! Bridge adapters that let SuperTool's LlmClient and tools::execute_tool
//! implement the upstream `ApiClient` and `ToolExecutor` traits required by
//! [`ConversationRuntime`].
//!
//! Upstream trait signatures (all sync):
//!   - ApiClient::stream(&mut self, ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError>
//!   - ToolExecutor::execute(&mut self, &str, &str)   -> Result<String, ToolError>
//!
//! LlmClient::send_turn() is async, so each adapter owns a dedicated
//! tokio::runtime::Runtime for the sync → async bridge.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use runtime::{
    ApiClient, ApiRequest, AssistantEvent, HookAbortSignal, RuntimeError, TokenUsage, ToolError, ToolExecutor,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, TurnResult};
use tauri::{AppHandle, Emitter};

use crate::commands::claw_chat::{GoalModeState, PlanModeState};

/// Adapter that implements the upstream [`ApiClient`] trait by wrapping
/// SuperTool's async [`LlmClient`].
///
/// Each instance owns a dedicated current-thread tokio runtime so that the
/// sync `stream()` method can `block_on` the async `send_turn()` inside a
/// synchronous context (the ConversationRuntime tool loop).
///
/// Real-time streaming: the `on_event` callback inside `stream()` emits
/// Tauri events (`agent-delta`, `agent-reasoning-delta`) as the LLM
/// produces tokens, rather than batching everything until the turn ends.
pub(crate) struct TauriApiClient {
    llm_client: Arc<LlmClient>,
    rt: tokio::runtime::Runtime,
    tool_defs: Vec<api::ToolDefinition>,
    reasoning_effort: Option<String>,
    app: AppHandle,
    session_id: String,
    abort_signal: Option<HookAbortSignal>,
}

impl TauriApiClient {
    pub(crate) fn new(
        llm_client: Arc<LlmClient>,
        tool_defs: Vec<api::ToolDefinition>,
        reasoning_effort: Option<String>,
        app: AppHandle,
        session_id: String,
        abort_signal: Option<HookAbortSignal>,
    ) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("TauriApiClient tokio runtime");
        Self { llm_client, rt, tool_defs, reasoning_effort, app, session_id, abort_signal }
    }
}

impl ApiClient for TauriApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        // Check abort signal before making the LLM call — short-circuit if
        // the user has already requested stop (e.g. from a previous iteration).
        if let Some(ref signal) = self.abort_signal {
            if signal.is_aborted() {
                log::info!("[TauriApiClient] Abort signaled — skipping LLM call");
                return Ok(Vec::new());
            }
        }

        let messages = super::claw_chat::session_to_input_messages(&request.messages);
        let system = request.system_prompt.join("\n");
        let td = self.tool_defs.clone();
        let re = self.reasoning_effort.clone();

        self.rt.block_on(async {
            // Collect LlmStreamEvent deltas into AssistantEvent Vec
            let events: std::cell::RefCell<Vec<AssistantEvent>> = std::cell::RefCell::new(Vec::new());

            let result: TurnResult = self
                .llm_client
                .send_turn(
                    messages,
                    Some(&system),
                    Some(td),
                    re,
                    Some(|event| {
                        let mut events = events.borrow_mut();
                        match event {
                            LlmStreamEvent::TextDelta { text } => {
                                events.push(AssistantEvent::TextDelta(text));
                            }
                            LlmStreamEvent::ThinkingDelta { thinking } => {
                                events.push(AssistantEvent::Thinking {
                                    thinking,
                                    signature: None,
                                });
                            }
                            LlmStreamEvent::ToolCall { id, name, input } => {
                                let input_str = serde_json::to_string(&input)
                                    .unwrap_or_else(|_| "{}".to_string());
                                events.push(AssistantEvent::ToolUse {
                                    id,
                                    name,
                                    input: input_str,
                                });
                            }
                            LlmStreamEvent::Usage {
                                input_tokens,
                                output_tokens,
                            } => {
                                events.push(AssistantEvent::Usage(TokenUsage {
                                    input_tokens: input_tokens as u32,
                                    output_tokens: output_tokens as u32,
                                    cache_creation_input_tokens: 0,
                                    cache_read_input_tokens: 0,
                                }));
                            }
                            LlmStreamEvent::Done => {
                                events.push(AssistantEvent::MessageStop);
                            }
                        }
                    }),
                )
                .await
                .map_err(|e| RuntimeError::new(format!("LLM stream failed: {e}")))?;

            // Per-iteration: emit complete assistant text and reasoning for this
            // LLM turn (one call within the tool loop), rather than per-token.
            // The frontend receives one agent-delta per LLM iteration.
            if !result.text.is_empty() {
                let _ = self.app.emit("agent-delta", serde_json::json!({
                    "text": &result.text,
                    "session_id": &self.session_id,
                }));
            }
            if !result.reasoning.is_empty() {
                let _ = self.app.emit("agent-reasoning-delta", serde_json::json!({
                    "text": &result.reasoning,
                    "session_id": &self.session_id,
                }));
            }

            // send_turn accumulates tool_calls internally but does NOT forward
            // ToolCall events to the on_event callback. We must emit ToolUse
            // events ourselves from the TurnResult so ConversationRuntime's
            // build_assistant_message() can create ToolUse content blocks.
            // Without this, the tool loop breaks after 1 iteration (0 tools).
            for (id, name, input) in &result.tool_calls {
                let input_str = serde_json::to_string(input)
                    .unwrap_or_else(|_| "{}".to_string());
                events.borrow_mut().push(AssistantEvent::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input_str,
                });
            }

            let mut final_events = events.into_inner();
            // Ensure MessageStop is present (LLM stream may not emit it explicitly)
            if !final_events.last().is_some_and(|e| matches!(e, AssistantEvent::MessageStop)) {
                final_events.push(AssistantEvent::MessageStop);
            }

            Ok(final_events)
        })
    }
}

/// Adapter that implements the upstream [`ToolExecutor`] trait by calling
/// SuperTool's synchronous `tools::execute_tool`.
///
/// Real-time tool event emission: emits `agent-tool-start` before executing
/// and `agent-tool-complete` after, so the frontend shows tool lifecycle
/// incrementally rather than batching everything until the turn ends.
pub(crate) struct TauriToolExecutor {
    app: AppHandle,
    session_id: String,
    plan_state: Arc<std::sync::Mutex<PlanModeState>>,
    goal_mode: Arc<AtomicBool>,
    goal_state: Arc<std::sync::Mutex<Option<GoalModeState>>>,
    loop_mode: Arc<AtomicBool>,
}

impl TauriToolExecutor {
    pub(crate) fn new(
        app: AppHandle,
        session_id: String,
        plan_state: Arc<std::sync::Mutex<PlanModeState>>,
        goal_mode: Arc<AtomicBool>,
        goal_state: Arc<std::sync::Mutex<Option<GoalModeState>>>,
        loop_mode: Arc<AtomicBool>,
    ) -> Self {
        Self { app, session_id, plan_state, goal_mode, goal_state, loop_mode }
    }
}

/// Tools that modify the workspace — blocked when plan mode is active
const WRITE_TOOLS: &[&str] = &[
    "Write", "Edit", "write_file", "edit_file", "Bash", "bash",
];

impl ToolExecutor for TauriToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        // Handle plan mode tools internally
        if tool_name == "EnterPlanMode" {
            // Extract optional objective from input
            let objective = serde_json::from_str::<serde_json::Value>(input)
                .ok()
                .and_then(|v| v.get("objective").and_then(|g| g.as_str().map(|s| s.to_string())));
            // Create plan file path (PLAN.md in sessions dir)
            let plan_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("~"))
                .join(".claw")
                .join("sessions");
            let plan_path = plan_dir.join("PLAN.md");
            if let Some(parent) = plan_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // Initialize empty plan file if it doesn't exist
            if !plan_path.exists() {
                let _ = std::fs::write(&plan_path, format!("# Plan\n\n{}", objective.as_deref().unwrap_or("")));
            }
            {
                let mut ps = self.plan_state.lock().unwrap();
                ps.enabled = true;
                ps.plan_file_path = Some(plan_path.to_string_lossy().to_string());
            }
            log::info!("[PlanMode] Entered plan mode (plan_path: {})", plan_path.display());
            return Ok(format!(
                r#"{{"success":true,"message":"Plan mode enabled. Plan file created at {plan}. You can explore the codebase with read-only tools. Write the plan to PLAN.md. Call ExitPlanMode with approved:true when the plan is ready."}}"#,
                plan = plan_path.display()
            ));
        }
        if tool_name == "ExitPlanMode" {
            // Extract optional approved flag
            let approved = serde_json::from_str::<serde_json::Value>(input)
                .ok()
                .and_then(|v| v.get("approved").and_then(|a| a.as_bool()))
                .unwrap_or(false);
            let _plan_path = {
                let ps = self.plan_state.lock().unwrap();
                ps.plan_file_path.clone()
            };
            {
                let mut ps = self.plan_state.lock().unwrap();
                ps.enabled = false;
                ps.plan_file_path = None;
            }
            if approved {
                log::info!("[PlanMode] Plan approved → exited plan mode, write access restored");
                return Ok(r#"{"success":true,"message":"Plan approved! Exiting plan mode. Write access restored — you can now implement the plan."}"#.into());
            } else {
                log::info!("[PlanMode] Plan cancelled → exited plan mode, write access restored");
                return Ok(r#"{"success":true,"message":"Plan mode cancelled. Exiting plan mode. Write access restored."}"#.into());
            }
        }

        // Handle goal mode tools internally — the hidden `goal` tool
        if tool_name == "goal" {
            let input_value: serde_json::Value = serde_json::from_str(input)
                .unwrap_or_else(|_| serde_json::json!({}));
            let op = input_value.get("op")
                .and_then(|v| v.as_str())
                .unwrap_or("get")
                .to_string();

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            match op.as_str() {
                "create" => {
                    let objective = input_value.get("objective")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if objective.trim().is_empty() {
                        return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"objective is required for op=create"}"#.into());
                    }
                    let token_budget = input_value.get("token_budget")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);

                    let goal_id = format!("goal-{}", now_ms);
                    let goal = super::claw_chat::Goal {
                        id: goal_id,
                        objective: objective.clone(),
                        status: "active".to_string(),
                        token_budget,
                        tokens_used: 0,
                        time_used_seconds: 0,
                    };
                    let state = super::claw_chat::GoalModeState {
                        enabled: true,
                        mode: "active".to_string(),
                        goal,
                    };

                    self.goal_mode.store(true, Ordering::Relaxed);
                    {
                        let mut gs = self.goal_state.lock().unwrap();
                        *gs = Some(state.clone());
                    }
                    log::info!("[GoalTool] Created goal: {} (budget={:?})", objective, token_budget);
                    return Ok(format!(r#"{{"goal":{},"remainingTokens":{},"completionBudgetReport":null}}"#,
                        serde_json::to_string(&state.goal).unwrap_or_default(),
                        token_budget.map(|b| b.to_string()).unwrap_or_else(|| "null".to_string())
                    ));
                }
                "get" => {
                    let gs = self.goal_state.lock().unwrap();
                    let response = if let Some(ref gs_inner) = *gs {
                        let remaining = gs_inner.goal.token_budget.map(|b| {
                            let r = if b > gs_inner.goal.tokens_used { b - gs_inner.goal.tokens_used } else { 0 };
                            r.to_string()
                        }).unwrap_or_else(|| "null".to_string());
                        format!(r#"{{"goal":{},"remainingTokens":{},"completionBudgetReport":null}}"#,
                            serde_json::to_string(&gs_inner.goal).unwrap_or_default(),
                            remaining
                        )
                    } else {
                        r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null}"#.to_string()
                    };
                    return Ok(response);
                }
                "complete" => {
                    let mut gs = self.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.status = "complete".to_string();
                        gs_inner.enabled = false;
                        gs_inner.mode = "exiting".to_string();
                        self.goal_mode.store(false, Ordering::Relaxed);

                        let budget_report = if gs_inner.goal.token_budget.is_some() || gs_inner.goal.time_used_seconds > 0 {
                            let mut parts = vec![];
                            if let Some(budget) = gs_inner.goal.token_budget {
                                parts.push(format!("tokens used: {} of {}", gs_inner.goal.tokens_used, budget));
                            }
                            if gs_inner.goal.time_used_seconds > 0 {
                                parts.push(format!("time used: {} seconds", gs_inner.goal.time_used_seconds));
                            }
                            Some(format!("Goal achieved. Report final budget usage to the user: {}.", parts.join("; ")))
                        } else {
                            None
                        };

                        log::info!("[GoalTool] Completed goal: {}", gs_inner.goal.objective);
                        return Ok(format!(r#"{{"goal":{},"remainingTokens":null,"completionBudgetReport":{}}}"#,
                            serde_json::to_string(&gs_inner.goal).unwrap_or_default(),
                            budget_report.map(|r| format!("\"{}\"", r.replace('"', "\\\""))).unwrap_or_else(|| "null".to_string())
                        ));
                    }
                    return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"No active goal to complete"}"#.into());
                }
                "resume" => {
                    let mut gs = self.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        if gs_inner.goal.status == "complete" {
                            return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"Goal is already complete"}"#.into());
                        }
                        gs_inner.enabled = true;
                        gs_inner.mode = "active".to_string();
                        gs_inner.goal.status = "active".to_string();
                        self.goal_mode.store(true, Ordering::Relaxed);
                        let remaining = gs_inner.goal.token_budget.map(|b| {
                            let r = if b > gs_inner.goal.tokens_used { b - gs_inner.goal.tokens_used } else { 0 };
                            r.to_string()
                        }).unwrap_or_else(|| "null".to_string());
                        log::info!("[GoalTool] Resumed goal: {}", gs_inner.goal.objective);
                        return Ok(format!(r#"{{"goal":{},"remainingTokens":{},"completionBudgetReport":null}}"#,
                            serde_json::to_string(&gs_inner.goal).unwrap_or_default(),
                            remaining
                        ));
                    }
                    return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"No paused goal to resume"}"#.into());
                }
                "drop" => {
                    let mut gs = self.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.status = "dropped".to_string();
                        gs_inner.enabled = false;
                        self.goal_mode.store(false, Ordering::Relaxed);
                        log::info!("[GoalTool] Dropped goal: {}", gs_inner.goal.objective);
                        return Ok(format!(r#"{{"goal":{},"remainingTokens":null,"completionBudgetReport":null}}"#,
                            serde_json::to_string(&gs_inner.goal).unwrap_or_default()
                        ));
                    }
                    return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"No active goal to drop"}"#.into());
                }
                "budget" => {
                    let token_budget = input_value.get("token_budget")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    let mut gs = self.goal_state.lock().unwrap();
                    if let Some(ref mut gs_inner) = *gs {
                        gs_inner.goal.token_budget = token_budget;
                        log::info!("[GoalTool] Updated goal budget to {:?}", token_budget);
                        return Ok(format!(r#"{{"goal":{},"remainingTokens":{},"completionBudgetReport":null}}"#,
                            serde_json::to_string(&gs_inner.goal).unwrap_or_default(),
                            token_budget.map(|b| b.to_string()).unwrap_or_else(|| "null".to_string())
                        ));
                    }
                    return Ok(r#"{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"No active goal"}"#.into());
                }
                _ => {
                    return Ok(format!(r#"{{"goal":null,"remainingTokens":null,"completionBudgetReport":null,"error":"Unknown goal op: {op}"}}"#));
                }
            }
        }

        // Handle loop mode tools internally
        if tool_name == "EnterLoopMode" {
            self.loop_mode.store(true, Ordering::Relaxed);
            log::info!("[LoopMode] Entered loop mode");
            return Ok(r#"{"success":true,"message":"Loop mode enabled. Your next prompt will auto-resubmit after each turn. Use /loop again or Esc to disable."}"#.into());
        }
        if tool_name == "ExitLoopMode" {
            self.loop_mode.store(false, Ordering::Relaxed);
            log::info!("[LoopMode] Exited loop mode");
            return Ok(r#"{"success":true,"message":"Loop mode disabled. Prompt will no longer auto-resubmit."}"#.into());
        }

        // Real-time: notify frontend that this tool started
        let tool_input: serde_json::Value =
            serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({ "raw": input }));
        let _ = self.app.emit("agent-tool-start", serde_json::json!({
            "name": tool_name,
            "args": tool_input,
            "session_id": &self.session_id,
        }));

        // Plan mode check: block write tools when plan mode is active,
        // UNLESS the tool targets the plan file path.
        {
            let ps = self.plan_state.lock().unwrap();
            if ps.enabled {
                let is_write = WRITE_TOOLS.iter().any(|w| w.eq_ignore_ascii_case(tool_name));
                if is_write {
                    // Check if this tool targets the plan file (whitelist)
                    let input_value: serde_json::Value =
                        serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({}));
                    let target_path = input_value
                        .get("path")
                        .or_else(|| input_value.get("file_path"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let is_plan_file = ps
                        .plan_file_path
                        .as_ref()
                        .map(|pfp| {
                            let canon_pfp = std::path::Path::new(pfp);
                            let canon_target = std::path::Path::new(target_path);
                            canon_pfp == canon_target
                                || target_path == "PLAN.md"
                                || target_path.contains("/PLAN.md")
                                || target_path.contains("\\PLAN.md")
                        })
                        .unwrap_or(false);
                    if !is_plan_file {
                        let _ = self.app.emit("agent-tool-complete", serde_json::json!({
                            "name": tool_name,
                            "result": "error",
                            "isError": true,
                            "session_id": &self.session_id,
                        }));
                        return Err(ToolError::new(format!(
                            "Plan mode is active — `{tool_name}` is blocked. \
                             You can only write to the plan file (PLAN.md). \
                             Call ExitPlanMode with approved:true to approve the plan and restore write access."
                        )));
                    }
                }
            }
        }

        // Execute the tool
        let input_value: serde_json::Value =
            serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({ "raw": input }));
        let result = tools::execute_tool(tool_name, &input_value);

        // Real-time: notify frontend that this tool completed
        let is_error = result.is_err();
        let _ = self.app.emit("agent-tool-complete", serde_json::json!({
            "name": tool_name,
            "result": if is_error { "error" } else { "success" },
            "isError": is_error,
            "session_id": &self.session_id,
        }));

        result.map_err(ToolError::new)
    }
}

/// Reports hook lifecycle events to the frontend via Tauri IPC events.
/// Implements upstream `HookProgressReporter` so the GUI shows pre/post hook activity.
pub(crate) struct TauriHookReporter {
    app: AppHandle,
    session_id: String,
}

impl TauriHookReporter {
    pub(crate) fn new(app: AppHandle, session_id: String) -> Self {
        Self { app, session_id }
    }
}

impl runtime::HookProgressReporter for TauriHookReporter {
    fn on_event(&mut self, event: &runtime::HookProgressEvent) {
        use runtime::HookProgressEvent;
        let (phase, hook, tool_name) = match event {
            HookProgressEvent::Started { event, tool_name, .. } => {
                ("started", event.as_str(), tool_name.clone())
            }
            HookProgressEvent::Completed { event, tool_name, .. } => {
                ("completed", event.as_str(), tool_name.clone())
            }
            HookProgressEvent::Cancelled { event, tool_name, .. } => {
                ("completed", event.as_str(), tool_name.clone())
            }
        };
        let _ = self.app.emit("agent-hook-progress", serde_json::json!({
            "phase": phase,
            "hook": hook,
            "tool_name": tool_name,
            "session_id": self.session_id,
        }));
    }
}

/// Result of a conversation turn, ready for frontend emission.
pub(crate) struct TurnEmit {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub auto_compaction_removed: Option<usize>,
}

/// Convert upstream [`TurnSummary`] into a flat [`TurnEmit`] for Tauri event dispatch.
pub(crate) fn turn_summary_to_emit(summary: &runtime::TurnSummary) -> TurnEmit {
    TurnEmit {
        input_tokens: summary.usage.input_tokens as u64,
        output_tokens: summary.usage.output_tokens as u64,
        auto_compaction_removed: summary.auto_compaction.map(|a| a.removed_message_count),
    }
}
