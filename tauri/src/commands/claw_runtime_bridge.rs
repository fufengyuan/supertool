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

use runtime::{
    ApiClient, ApiRequest, AssistantEvent, RuntimeError, TokenUsage, ToolError, ToolExecutor,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, TurnResult};
use tauri::{AppHandle, Emitter};

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
}

impl TauriApiClient {
    pub(crate) fn new(
        llm_client: Arc<LlmClient>,
        tool_defs: Vec<api::ToolDefinition>,
        reasoning_effort: Option<String>,
        app: AppHandle,
        session_id: String,
    ) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("TauriApiClient tokio runtime");
        Self { llm_client, rt, tool_defs, reasoning_effort, app, session_id }
    }
}

impl ApiClient for TauriApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
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
}

impl TauriToolExecutor {
    pub(crate) fn new(app: AppHandle, session_id: String) -> Self {
        Self { app, session_id }
    }
}

impl ToolExecutor for TauriToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        // Real-time: notify frontend that this tool started
        let tool_input: serde_json::Value =
            serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({ "raw": input }));
        let _ = self.app.emit("agent-tool-start", serde_json::json!({
            "name": tool_name,
            "args": tool_input,
            "session_id": &self.session_id,
        }));

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
