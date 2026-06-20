//! LLM streaming client module — thin wrapper over the `claw` crate.
//!
//! Preserves the same public API so the Tauri chat commands keep working
//! without changes.  Under the hood every call is delegated to the claw
//! crate (derived from claw-code).

use std::sync::Arc;

use serde::Serialize;

// ---------------------------------------------------------------------------
// Re-export types used by omp_chat.rs
// ---------------------------------------------------------------------------

/// A single message in the conversation history.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
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

/// Result of a single LLM API turn (one request-response cycle).
pub struct TurnResult {
    pub text: String,
    pub reasoning: String,
    pub tool_calls: Vec<(String, String, serde_json::Value)>, // (id, name, input)
    pub usage: Option<(u64, u64)>,
}

/// A complete tool call accumulated from streaming events.
#[derive(Debug)]
struct AccumulatingToolCall {
    id: String,
    name: String,
    input: String,
}

/// A lightweight LLM API client that sends streaming requests.
///
/// Wraps [`api::ProviderClient`] and delegates every call to it.
#[derive(Debug, Clone)]
pub struct LlmClient {
    inner: Arc<api::ProviderClient>,
    model: String,
}

impl LlmClient {
    /// Try to create a client from environment variables.
    ///
    /// Uses the model name from `ANTHROPIC_MODEL` / `OPENAI_MODEL` / `XAI_MODEL`
    /// (or a sensible default), then delegates to `ProviderClient::from_model`.
    ///
    /// **Proxy override**: when `OPENAI_BASE_URL` is set, the user is routing
    /// through an OpenAI-compatible proxy (e.g. token.example.com). Even if the
    /// model is `claude-sonnet-4-6` (which would normally be detected as `Anthropic`),
    /// we force the OpenAI-compatible client — the proxy speaks the OpenAI wire
    /// protocol and doesn't understand the Anthropic `/v1/messages` format.
    pub fn from_env() -> Result<Self, String> {
        let model = Self::resolve_model_from_env();
        Self::from_model(&model)
    }

    /// Create a client for the given model name (uses env api_key/base_url).
    /// Used to create the sub-agent client with a different (faster) model
    /// while sharing the same provider configuration.
    pub fn from_model(model: &str) -> Result<Self, String> {
        let client = if std::env::var_os("OPENAI_BASE_URL").is_some() {
            let forced = format!("openai/{}", model);
            api::ProviderClient::from_model(&forced)
                .map_err(|e| format!("Failed to create LLM client: {e}"))?
        } else {
            api::ProviderClient::from_model(model)
                .map_err(|e| format!("Failed to create LLM client: {e}"))?
        };
        Ok(Self {
            inner: Arc::new(client),
            model: model.to_string(),
        })
    }

    /// Return the provider kind (as a debug-friendly string).
    #[must_use]
    pub fn provider(&self) -> api::ProviderKind {
        self.inner.provider_kind()
    }

    /// Return the inner ProviderClient for direct access.
    #[must_use]
    pub fn inner(&self) -> &api::ProviderClient {
        &self.inner
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
                Ok(Some(api::StreamEvent::ContentBlockStart(
                    api::ContentBlockStartEvent {
                        content_block:
                            api::OutputContentBlock::Text { .. },
                        ..
                    },
                ))) => {
                    // Block start — no delta to emit yet.
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::TextDelta { text },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::TextDelta { text }));
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::ThinkingDelta { thinking },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::ThinkingDelta { thinking }));
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::InputJsonDelta { partial_json },
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
                Ok(Some(api::StreamEvent::ContentBlockStart(
                    api::ContentBlockStartEvent {
                        content_block:
                            api::OutputContentBlock::ToolUse {
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
                Ok(Some(api::StreamEvent::MessageDelta(
                    api::MessageDeltaEvent {
                        usage:
                            api::Usage {
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
                Ok(Some(api::StreamEvent::MessageStop(_))) => {
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

    /// Send a streaming request with a pre-built [`api::MessageRequest`].
    ///
    /// This is the structured equivalent of `send_streaming` — use it when you
    /// need multi-block content (Thinking with signature, ToolUse, ToolResult)
    /// matching the claw-code CLI's native message format.
    pub async fn send_streaming_request<F>(
        &self,
        request: &api::MessageRequest,
        on_event: F,
    ) -> Result<(), String>
    where
        F: Fn(Result<LlmStreamEvent, String>),
    {
        let mut stream = self
            .inner
            .stream_message(request)
            .await
            .map_err(|e| format!("Failed to start stream: {e}"))?;

        loop {
            match stream.next_event().await {
                Ok(Some(api::StreamEvent::ContentBlockStart(
                    api::ContentBlockStartEvent {
                        content_block:
                            api::OutputContentBlock::Text { .. },
                        ..
                    },
                ))) => {}
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::TextDelta { text },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::TextDelta { text }));
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::ThinkingDelta { thinking },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::ThinkingDelta { thinking }));
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::InputJsonDelta { partial_json },
                        ..
                    },
                ))) => {
                    on_event(Ok(LlmStreamEvent::ToolCall {
                        id: String::new(),
                        name: String::new(),
                        input: serde_json::from_str(&partial_json)
                            .unwrap_or_else(|_| serde_json::json!({ "raw": partial_json })),
                    }));
                }
                Ok(Some(api::StreamEvent::ContentBlockStart(
                    api::ContentBlockStartEvent {
                        content_block:
                            api::OutputContentBlock::ToolUse {
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
                Ok(Some(api::StreamEvent::MessageDelta(
                    api::MessageDeltaEvent {
                        usage:
                            api::Usage {
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
                Ok(Some(api::StreamEvent::MessageStop(_))) => {
                    on_event(Ok(LlmStreamEvent::Done));
                    return Ok(());
                }
                Ok(Some(_)) => {}
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

    /// Send a turn with optional tools and an event callback.
    ///
    /// Builds a `MessageRequest` with the given messages, system prompt, and
    /// tool definitions, streams the response, and returns a `TurnResult`
    /// (text, reasoning, tool_calls, usage).
    ///
    /// The optional `on_event` callback is called synchronously for each
    /// streaming event (`TextDelta`, `ThinkingDelta`, complete `ToolCall`,
    /// `Usage`, `Done`) and can be used to forward events to the GUI.
    pub async fn send_turn<F>(
        &self,
        messages: Vec<api::InputMessage>,
        system_prompt: Option<&str>,
        tools: Option<Vec<api::ToolDefinition>>,
        reasoning_effort: Option<String>,
        on_event: Option<F>,
    ) -> Result<TurnResult, String>
    where
        F: Fn(LlmStreamEvent),
    {
        // Match upstream: only include tools/tool_choice when non-empty
        let has_tools = tools.as_ref().is_some_and(|t| !t.is_empty());

        let request = api::MessageRequest {
            model: self.model.clone(),
            max_tokens: api::max_tokens_for_model(&self.model),
            messages,
            system: system_prompt.map(|s| s.to_string()),
            tools: if has_tools { tools } else { None },
            tool_choice: if has_tools { Some(api::ToolChoice::Auto) } else { None },
            stream: true,
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            reasoning_effort,
            extra_body: std::collections::BTreeMap::new(),
        };

        let mut stream = self
            .inner
            .stream_message(&request)
            .await
            .map_err(|e| format!("Failed to start stream: {e}"))?;

        let mut text = String::new();
        let mut reasoning = String::new();
        let mut pending_tool: Option<AccumulatingToolCall> = None;
        let mut tool_calls: Vec<(String, String, serde_json::Value)> = Vec::new();
        let mut usage: Option<(u64, u64)> = None;

        loop {
            match stream.next_event().await {
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::TextDelta { text: chunk },
                        ..
                    },
                ))) => {
                    text.push_str(&chunk);
                    if let Some(ref cb) = on_event {
                        cb(LlmStreamEvent::TextDelta { text: chunk });
                    }
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::ThinkingDelta { thinking },
                        ..
                    },
                ))) => {
                    reasoning.push_str(&thinking);
                    if let Some(ref cb) = on_event {
                        cb(LlmStreamEvent::ThinkingDelta { thinking });
                    }
                }
                Ok(Some(api::StreamEvent::ContentBlockStart(
                    api::ContentBlockStartEvent {
                        content_block: api::OutputContentBlock::ToolUse { ref id, ref name, .. },
                        ..
                    },
                ))) => {
                    // Start accumulating a new tool call
                    if pending_tool.is_some() {
                        // Flush previous incomplete tool
                        let prev = pending_tool.take().unwrap();
                        let input: serde_json::Value =
                            serde_json::from_str(&prev.input).unwrap_or(serde_json::json!({}));
                        tool_calls.push((prev.id, prev.name, input));
                    }
                    pending_tool = Some(AccumulatingToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input: String::new(),
                    });
                }
                Ok(Some(api::StreamEvent::ContentBlockDelta(
                    api::ContentBlockDeltaEvent {
                        delta: api::ContentBlockDelta::InputJsonDelta { partial_json },
                        ..
                    },
                ))) => {
                    if let Some(ref mut tool) = pending_tool {
                        tool.input.push_str(&partial_json);
                    }
                }
                Ok(Some(api::StreamEvent::MessageDelta(
                    api::MessageDeltaEvent {
                        usage:
                            api::Usage {
                                input_tokens,
                                output_tokens,
                                ..
                            },
                        ..
                    },
                ))) => {
                    usage = Some((u64::from(input_tokens), u64::from(output_tokens)));
                    if let Some(ref cb) = on_event {
                        cb(LlmStreamEvent::Usage {
                            input_tokens: u64::from(input_tokens),
                            output_tokens: u64::from(output_tokens),
                        });
                    }
                }
                Ok(Some(api::StreamEvent::MessageStop(_))) | Ok(None) => {
                    // Finalize any pending tool call
                    if let Some(tool) = pending_tool.take() {
                        let input: serde_json::Value =
                            serde_json::from_str(&tool.input).unwrap_or(serde_json::json!({}));
                        tool_calls.push((tool.id, tool.name, input));
                    }
                    if let Some(ref cb) = on_event {
                        cb(LlmStreamEvent::Done);
                    }
                    return Ok(TurnResult {
                        text,
                        reasoning,
                        tool_calls,
                        usage,
                    });
                }
                Ok(Some(_)) => {} // Ignore other events
                Err(e) => return Err(format!("Stream error: {e}")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a [`api::MessageRequest`] from our simple Message slice.
fn build_message_request(model: &str, messages: &[Message]) -> api::MessageRequest {
    api::MessageRequest {
        model: model.to_string(),
        max_tokens: 8192,
        messages: messages
            .iter()
            .map(|m| api::InputMessage {
                role: m.role.clone(),
                content: vec![api::InputContentBlock::Text {
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

/// Build a [`api::MessageRequest`] from structured InputMessages (CLI-compatible).
/// Use this instead of `build_message_request` when you need multi-block content
/// (Thinking blocks with signature, ToolUse, ToolResult — matching claw-code CLI).
pub fn build_chat_request(
    model: &str,
    system_prompt: Option<&str>,
    messages: Vec<api::InputMessage>,
    max_tokens: Option<u32>,
    reasoning_effort: Option<String>,
    tools: Option<Vec<api::ToolDefinition>>,
) -> api::MessageRequest {
    api::MessageRequest {
        model: model.to_string(),
        max_tokens: max_tokens.unwrap_or(8192),
        messages,
        system: system_prompt.map(|s| s.to_string()),
        tools,
        tool_choice: None,
        stream: true,
        temperature: None,
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        reasoning_effort,
        extra_body: std::collections::BTreeMap::new(),
    }
}
