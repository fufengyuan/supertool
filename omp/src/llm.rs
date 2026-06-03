//! LLM streaming client module — thin wrapper over the `llm-client` crate.
//!
//! Preserves the same public API as the old hand-written `llm.rs` so the
//! `omp_chat` command keeps working without changes.  Under the hood every
//! call is delegated to the upstream-derived `llm_client` crate.

use std::sync::Arc;

use serde::Serialize;

// ---------------------------------------------------------------------------
// Re-export types used by omp_chat.rs
// ---------------------------------------------------------------------------

/// A single message in the conversation history.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Events emitted during an LLM streaming response.
#[derive(Debug, Clone)]
pub enum LlmStreamEvent {
    /// A text delta from the assistant.
    TextDelta { text: String },
    /// A thinking / reasoning delta (Anthropic extended thinking).
    ThinkingDelta { thinking: String },
    /// A tool-call block (only emitted when tools are configured).
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Final usage information.
    Usage {
        input_tokens: u64,
        output_tokens: u64,
    },
    /// Stream is finished.
    Done,
}

// ---------------------------------------------------------------------------
// LlmClient — thin wrapper
// ---------------------------------------------------------------------------

/// A lightweight LLM API client that sends streaming requests.
///
/// Wraps [`llm_client::ProviderClient`] and delegates every call to it.
#[derive(Debug, Clone)]
pub struct LlmClient {
    inner: Arc<llm_client::ProviderClient>,
    model: String,
}

impl LlmClient {
    /// Try to create a client from environment variables.
    ///
    /// Uses the model name from `ANTHROPIC_MODEL` / `OPENAI_MODEL` / `XAI_MODEL`
    /// (or a sensible default), then delegates to `ProviderClient::from_model`.
    pub fn from_env() -> Result<Self, String> {
        let model = Self::resolve_model_from_env();
        let client = llm_client::ProviderClient::from_model(&model)
            .map_err(|e| format!("Failed to create LLM client: {e}"))?;
        Ok(Self {
            inner: Arc::new(client),
            model,
        })
    }

    /// Return the provider kind (as a debug-friendly string).
    #[must_use]
    pub fn provider(&self) -> llm_client::ProviderKind {
        self.inner.provider_kind()
    }

    /// Return the model name.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Send a streaming request and drive the given callback for each event.
    ///
    /// The `on_event` closure is called synchronously from within the HTTP
    /// response reading loop.  When the stream is exhausted (or an error
    /// occurs) `on_event` is called one last time with [`LlmStreamEvent::Done`].
    pub async fn send_streaming<F>(
        &self,
        messages: &[Message],
        on_event: F,
    ) -> Result<(), String>
    where
        F: Fn(Result<LlmStreamEvent, String>),
    {
        let request = build_message_request(&self.model, messages);
        let mut stream = self
            .inner
            .stream_message(&request)
            .await
            .map_err(|e| format!("Failed to start stream: {e}"))?;

        loop {
            match stream.next_event().await {
                Ok(Some(llm_client::StreamEvent::ContentBlockStart(
                    llm_client::ContentBlockStartEvent {
                        content_block:
                            llm_client::OutputContentBlock::Text { .. },
                        ..
                    },
                ))) => {
                    // Block start — no delta to emit yet.
                }
                Ok(Some(llm_client::StreamEvent::ContentBlockDelta(
                    llm_client::ContentBlockDeltaEvent {
                        delta: llm_client::ContentBlockDelta::TextDelta { text },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::TextDelta { text }));
                }
                Ok(Some(llm_client::StreamEvent::ContentBlockDelta(
                    llm_client::ContentBlockDeltaEvent {
                        delta: llm_client::ContentBlockDelta::ThinkingDelta { thinking },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::ThinkingDelta { thinking }));
                }
                Ok(Some(llm_client::StreamEvent::ContentBlockDelta(
                    llm_client::ContentBlockDeltaEvent {
                        delta: llm_client::ContentBlockDelta::InputJsonDelta { partial_json },
                        ..
                    },
                ))) => {
                    // Tool-call arguments delta — we accumulate these in a
                    // simple way: emit a ToolCall event per chunk.  The real
                    // upstream client handles full tool-call lifecycle.
                    on_event(Ok(LlmStreamEvent::ToolCall {
                        id: String::new(),
                        name: String::new(),
                        input: serde_json::from_str(&partial_json)
                            .unwrap_or_else(|_| serde_json::json!({ "raw": partial_json })),
                    }));
                }
                Ok(Some(llm_client::StreamEvent::ContentBlockStart(
                    llm_client::ContentBlockStartEvent {
                        content_block:
                            llm_client::OutputContentBlock::ToolUse {
                                ref id,
                                ref name,
                                ..
                            },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input: serde_json::json!({}),
                    }));
                }
                Ok(Some(llm_client::StreamEvent::MessageDelta(
                    llm_client::MessageDeltaEvent {
                        usage:
                            llm_client::Usage {
                                input_tokens,
                                output_tokens,
                                ..
                            },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::Usage {
                        input_tokens: u64::from(input_tokens),
                        output_tokens: u64::from(output_tokens),
                    }));
                }
                Ok(Some(llm_client::StreamEvent::MessageStop(_))) => {
                    on_event(Ok(LlmStreamEvent::Done));
                    return Ok(());
                }
                Ok(Some(_)) => {
                    // Ignore other events (ContentBlockStop, MessageStart, etc.)
                }
                Ok(None) => {
                    on_event(Ok(LlmStreamEvent::Done));
                    return Ok(());
                }
                Err(e) => {
                    let msg = e.to_string();
                    on_event(Err(msg.clone()));
                    return Err(msg);
                }
            }
        }
    }

    /// Resolve the model name from environment variables.
    fn resolve_model_from_env() -> String {
        std::env::var("ANTHROPIC_MODEL")
            .or_else(|_| std::env::var("OPENAI_MODEL"))
            .or_else(|_| std::env::var("XAI_MODEL"))
            .unwrap_or_else(|_| {
                // Try to auto-detect from available credentials
                if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                    "claude-sonnet-4-6".to_string()
                } else if std::env::var("OPENAI_API_KEY").is_ok() {
                    "openai/gpt-4.1-mini".to_string()
                } else {
                    "claude-sonnet-4-6".to_string()
                }
            })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a [`llm_client::MessageRequest`] from our simple Message slice.
fn build_message_request(model: &str, messages: &[Message]) -> llm_client::MessageRequest {
    llm_client::MessageRequest {
        model: model.to_string(),
        max_tokens: 8192,
        messages: messages
            .iter()
            .map(|m| llm_client::InputMessage {
                role: m.role.clone(),
                content: vec![llm_client::InputContentBlock::Text {
                    text: m.content.clone(),
                }],
            })
            .collect(),
        system: None,
        tools: None,
        tool_choice: None,
        stream: true,
        temperature: None,
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        reasoning_effort: None,
        extra_body: std::collections::BTreeMap::new(),
    }
}
