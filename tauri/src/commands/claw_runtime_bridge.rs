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
use std::cell::RefCell;

use runtime::{
    ApiClient, ApiRequest, AssistantEvent, RuntimeError, TokenUsage, ToolError, ToolExecutor,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, TurnResult};

/// Adapter that implements the upstream [`ApiClient`] trait by wrapping
/// SuperTool's async [`LlmClient`].
///
/// Each instance owns a dedicated current-thread tokio runtime so that the
/// sync `stream()` method can `block_on` the async `send_turn()` inside a
/// synchronous context (the ConversationRuntime tool loop).
pub(crate) struct TauriApiClient {
    llm_client: Arc<LlmClient>,
    rt: tokio::runtime::Runtime,
    tool_defs: Vec<api::ToolDefinition>,
    reasoning_effort: Option<String>,
    model: String,
}

impl TauriApiClient {
    pub(crate) fn new(
        llm_client: Arc<LlmClient>,
        tool_defs: Vec<api::ToolDefinition>,
        reasoning_effort: Option<String>,
        model: String,
    ) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("TauriApiClient tokio runtime");
        Self { llm_client, rt, tool_defs, reasoning_effort, model }
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

            let _result: TurnResult = self
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

            let mut final_events = events.into_inner();
            // Ensure MessageStop is present
            if !final_events.iter().any(|e| matches!(e, AssistantEvent::MessageStop)) {
                final_events.push(AssistantEvent::MessageStop);
            }

            Ok(final_events)
        })
    }
}

/// Adapter that implements the upstream [`ToolExecutor`] trait by calling
/// SuperTool's synchronous `tools::execute_tool`.
#[derive(Default)]
pub(crate) struct TauriToolExecutor;

impl ToolExecutor for TauriToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        let input_value: serde_json::Value =
            serde_json::from_str(input).unwrap_or_else(|_| serde_json::json!({ "raw": input }));
        tools::execute_tool(tool_name, &input_value).map_err(|e| ToolError::new(e))
    }
}

/// Result of a conversation turn, ready for frontend emission.
pub(crate) struct TurnEmit {
    pub assistant_text: String,
    pub tool_calls: Vec<(String, String, String)>, // (id, name, input)
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub auto_compaction_removed: Option<usize>,
}

/// Convert upstream [`TurnSummary`] into a flat [`TurnEmit`] for Tauri event dispatch.
pub(crate) fn turn_summary_to_emit(summary: &runtime::TurnSummary) -> TurnEmit {
    let mut assistant_text = String::new();
    let mut tool_calls = Vec::new();

    for msg in &summary.assistant_messages {
        for block in &msg.blocks {
            match block {
                runtime::ContentBlock::Text { text } => assistant_text.push_str(text),
                runtime::ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push((id.clone(), name.clone(), input.clone()));
                }
                runtime::ContentBlock::Thinking { thinking, .. } => {
                    assistant_text.push_str(thinking);
                }
                _ => {}
            }
        }
    }

    TurnEmit {
        assistant_text,
        tool_calls,
        input_tokens: summary.usage.input_tokens as u64,
        output_tokens: summary.usage.output_tokens as u64,
        auto_compaction_removed: summary.auto_compaction.map(|a| a.removed_message_count),
    }
}
