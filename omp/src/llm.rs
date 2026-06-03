//! LLM streaming client module.
//!
//! Provides [`LlmClient`] that calls Anthropic Messages API (streaming)
//! or OpenAI Chat Completions API (streaming) directly via reqwest + SSE,
//! without any OMP subprocess or ACP protocol dependency.
//!
//! # Environment variables
//!
//! | Provider | API key          | Base URL override          |
//! |----------|------------------|----------------------------|
//! | Anthropic| `ANTHROPIC_API_KEY` | — (uses `https://api.anthropic.com`) |
//! | OpenAI   | `OPENAI_API_KEY`    | `OPENAI_BASE_URL` (default `https://api.openai.com/v1`) |

use std::sync::Arc;

use reqwest::Client as HttpClient;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which LLM provider to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
}

/// Static configuration for an [`LlmClient`].
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

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
// LlmClient
// ---------------------------------------------------------------------------

/// A lightweight LLM API client that sends streaming requests.
///
/// Supports both Anthropic Messages API (SSE) and OpenAI Chat Completions
/// API (SSE).  The provider is chosen at construction time based on which
/// environment variables are set.
#[derive(Debug, Clone)]
pub struct LlmClient {
    http: HttpClient,
    config: Arc<LlmConfig>,
}

impl LlmClient {
    /// Try to create a client from environment variables.
    ///
    /// Precedence: `ANTHROPIC_API_KEY` → Anthropic,
    /// `OPENAI_API_KEY` → OpenAI (with `OPENAI_BASE_URL` override).
    pub fn from_env() -> Result<Self, String> {
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            let key = key.trim().to_string();
            if key.is_empty() {
                return Err("ANTHROPIC_API_KEY is set but empty".into());
            }
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".into());
            return Ok(Self::new(LlmConfig {
                provider: LlmProvider::Anthropic,
                api_key: key,
                base_url: "https://api.anthropic.com".into(),
                model,
            }));
        }

        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            let key = key.trim().to_string();
            if key.is_empty() {
                return Err("OPENAI_API_KEY is set but empty".into());
            }
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into());
            let model = std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o".into());
            return Ok(Self::new(LlmConfig {
                provider: LlmProvider::OpenAI,
                api_key: key,
                base_url,
                model,
            }));
        }

        Err("No LLM API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY.".into())
    }

    /// Create a client from an explicit configuration.
    pub fn new(config: LlmConfig) -> Self {
        let http = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build reqwest Client");
        Self {
            http,
            config: Arc::new(config),
        }
    }

    /// Return the provider kind.
    pub fn provider(&self) -> LlmProvider {
        self.config.provider
    }

    /// Return the model name.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Send a streaming request and drive the given callback for each event.
    ///
    /// The `on_event` closure is called synchronously from within the HTTP
    /// response reading loop.  When the stream is exhausted (or an error
    /// occurs) `on_event` is called one last time with [`LlmStreamEvent::Done`]
    /// (or the error variant, if any).
    pub async fn send_streaming<F>(
        &self,
        messages: &[Message],
        on_event: F,
    ) -> Result<(), String>
    where
        F: Fn(Result<LlmStreamEvent, String>),
    {
        match self.config.provider {
            LlmProvider::Anthropic => {
                self.send_anthropic_stream(messages, on_event).await
            }
            LlmProvider::OpenAI => {
                self.send_openai_stream(messages, on_event).await
            }
        }
    }

    // ------------------------------------------------------------------
    // Anthropic Messages API (SSE)
    // ------------------------------------------------------------------

    async fn send_anthropic_stream<F>(
        &self,
        messages: &[Message],
        on_event: F,
    ) -> Result<(), String>
    where
        F: Fn(Result<LlmStreamEvent, String>),
    {
        let url = format!(
            "{}/v1/messages",
            self.config.base_url.trim_end_matches('/')
        );

        // Build message blocks from our simple Message type
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 8192,
            "stream": true,
            "messages": msgs,
        });

        log::info!(
            "[llm] Anthropic streaming request: model={}, messages={}",
            self.config.model,
            messages.len(),
        );

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".into());
            return Err(format!("Anthropic API error ({}): {}", status, text));
        }

        // Parse SSE stream
        let mut parser = SseParser::new();
        let mut usage = LlmUsage::default();

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream read error: {e}"))?;
            let events = parser.push(&chunk);
            for ev in events {
                match ev {
                    SseParsedEvent::ContentBlockDelta { text } => {
                        on_event(Ok(LlmStreamEvent::TextDelta { text }));
                    }
                    SseParsedEvent::ThinkingDelta { thinking } => {
                        on_event(Ok(LlmStreamEvent::ThinkingDelta { thinking }));
                    }
                    SseParsedEvent::Usage {
                        input_tokens,
                        output_tokens,
                    } => {
                        usage.input_tokens = input_tokens;
                        usage.output_tokens = output_tokens;
                    }
                    SseParsedEvent::Ping | SseParsedEvent::MessageStop => {
                        // no-op
                    }
                }
            }
        }

        on_event(Ok(LlmStreamEvent::Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        }));
        on_event(Ok(LlmStreamEvent::Done));

        log::info!(
            "[llm] Anthropic stream complete: {} in / {} out",
            usage.input_tokens,
            usage.output_tokens,
        );

        Ok(())
    }

    // ------------------------------------------------------------------
    // OpenAI Chat Completions API (SSE)
    // ------------------------------------------------------------------

    async fn send_openai_stream<F>(
        &self,
        messages: &[Message],
        on_event: F,
    ) -> Result<(), String>
    where
        F: Fn(Result<LlmStreamEvent, String>),
    {
        let base = self.config.base_url.trim_end_matches('/');
        let url = format!("{}/chat/completions", base);

        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": self.config.model,
            "stream": true,
            "messages": msgs,
        });

        log::info!(
            "[llm] OpenAI streaming request: model={}, messages={}",
            self.config.model,
            messages.len(),
        );

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".into());
            return Err(format!("OpenAI API error ({}): {}", status, text));
        }

        let mut parser = SseParser::new();
        let mut usage = LlmUsage::default();
        let _role_seen = false;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream read error: {e}"))?;
            let events = parser.push(&chunk);
            for ev in events {
                match ev {
                    SseParsedEvent::ContentBlockDelta { text } => {
                        // For OpenAI, the first delta text may be just role prefix;
                        // we skip empty texts.
                        if !text.is_empty() {
                            on_event(Ok(LlmStreamEvent::TextDelta { text }));
                        }
                    }
                    SseParsedEvent::Usage {
                        input_tokens,
                        output_tokens,
                    } => {
                        usage.input_tokens = input_tokens;
                        usage.output_tokens = output_tokens;
                    }
                    SseParsedEvent::ThinkingDelta { .. } => {
                        // OpenAI doesn't have a separate thinking channel
                        // via standard SSE deltas.
                    }
                    SseParsedEvent::Ping | SseParsedEvent::MessageStop => {}
                }
            }
        }

        on_event(Ok(LlmStreamEvent::Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        }));
        on_event(Ok(LlmStreamEvent::Done));

        log::info!(
            "[llm] OpenAI stream complete: {} in / {} out",
            usage.input_tokens,
            usage.output_tokens,
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SSE parser (minimal, inspired by claw-code)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct LlmUsage {
    input_tokens: u64,
    output_tokens: u64,
}

#[derive(Debug)]
enum SseParsedEvent {
    ContentBlockDelta { text: String },
    ThinkingDelta { thinking: String },
    Usage { input_tokens: u64, output_tokens: u64 },
    Ping,
    MessageStop,
}

/// Minimal SSE parser that handles both Anthropic and OpenAI line formats.
struct SseParser {
    buffer: Vec<u8>,
}

impl SseParser {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Push new bytes and return any complete events that were parsed.
    fn push(&mut self, chunk: &[u8]) -> Vec<SseParsedEvent> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        loop {
            // Find the next frame boundary (\n\n or \r\n\r\n)
            let end = self
                .buffer
                .windows(2)
                .position(|w| w == b"\n\n")
                .map(|p| p + 2)
                .or_else(|| {
                    self.buffer
                        .windows(4)
                        .position(|w| w == b"\r\n\r\n")
                        .map(|p| p + 4)
                });

            match end {
                None => break, // wait for more data
                Some(n) => {
                    let frame_bytes: Vec<u8> = self.buffer.drain(..n).collect();
                    let frame =
                        String::from_utf8_lossy(&frame_bytes[..frame_bytes.len() - n + n]);
                    // The frame includes the trailing \n\n; trim it
                    let frame = frame.trim();
                    if let Some(event) = Self::parse_frame(frame) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }

    /// Parse a single SSE frame (without the trailing `\n\n`).
    fn parse_frame(frame: &str) -> Option<SseParsedEvent> {
        let trimmed = frame.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') {
            return None;
        }

        let mut event_type: Option<&str> = None;
        let mut data_lines: Vec<&str> = Vec::new();

        for line in trimmed.lines() {
            if line.starts_with(':') {
                continue;
            }
            if let Some(name) = line.strip_prefix("event:") {
                event_type = Some(name.trim());
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                data_lines.push(data.trim_start());
            }
        }

        let et = event_type.unwrap_or("");
        let payload = data_lines.join("\n");

        // OpenAI: "data: [DONE]"
        if payload == "[DONE]" {
            return Some(SseParsedEvent::MessageStop);
        }

        // Anthropic ping
        if et == "ping" {
            return Some(SseParsedEvent::Ping);
        }

        if payload.is_empty() {
            return None;
        }

        // Try to parse as JSON
        let json: serde_json::Value = serde_json::from_str(&payload).ok()?;

        // Dispatch based on known event types
        match et {
            "content_block_delta" => {
                let delta = json.get("delta")?;
                match delta.get("type")?.as_str()? {
                    "text_delta" => {
                        let text = delta.get("text")?.as_str()?.to_string();
                        Some(SseParsedEvent::ContentBlockDelta { text })
                    }
                    "thinking_delta" => {
                        let thinking = delta.get("thinking")?.as_str()?.to_string();
                        Some(SseParsedEvent::ThinkingDelta { thinking })
                    }
                    "signature_delta" => {
                        // Not exposed; treat as no-op
                        None
                    }
                    _ => None,
                }
            }
            "message_delta" => {
                let usage = json.get("usage")?;
                let input_tokens = usage
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output_tokens = usage
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                Some(SseParsedEvent::Usage {
                    input_tokens,
                    output_tokens,
                })
            }
            "message_stop" => Some(SseParsedEvent::MessageStop),
            "content_block_start" | "content_block_stop" | "message_start" => {
                // Ignored — we don't need block-level bookkeeping
                None
            }
            // OpenAI-style: no event type, just `data: {...}`
            "" => {
                // OpenAI Chat Completions chunk
                let choices = json.get("choices")?.as_array()?;
                let first = choices.first()?;
                let delta = first.get("delta")?;

                // Check for text content
                if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                    return Some(SseParsedEvent::ContentBlockDelta {
                        text: content.to_string(),
                    });
                }

                // Check for usage in the final chunk (OpenAI puts usage at top-level)
                if let Some(usage_val) = json.get("usage") {
                    let input_tokens = usage_val
                        .get("prompt_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output_tokens = usage_val
                        .get("completion_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    return Some(SseParsedEvent::Usage {
                        input_tokens,
                        output_tokens,
                    });
                }

                // Check finish_reason (marks end of stream)
                if let Some(reason) = first.get("finish_reason") {
                    if !reason.is_null() {
                        return Some(SseParsedEvent::MessageStop);
                    }
                }

                None
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_parses_anthropic_text_delta() {
        let frame = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n"
        );
        let event = SseParser::parse_frame(frame);
        assert!(matches!(
            event,
            Some(SseParsedEvent::ContentBlockDelta { text }) if text == "Hello"
        ));
    }

    #[test]
    fn sse_parses_anthropic_thinking_delta() {
        let frame = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"step 1\"}}\n\n"
        );
        let event = SseParser::parse_frame(frame);
        assert!(matches!(
            event,
            Some(SseParsedEvent::ThinkingDelta { thinking }) if thinking == "step 1"
        ));
    }

    #[test]
    fn sse_parses_anthropic_message_delta_usage() {
        let frame = concat!(
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":10,\"output_tokens\":20}}\n\n"
        );
        let event = SseParser::parse_frame(frame);
        assert!(matches!(
            event,
            Some(SseParsedEvent::Usage { input_tokens: 10, output_tokens: 20 })
        ));
    }

    #[test]
    fn sse_parses_openai_chunk() {
        let frame = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":12345,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n";
        let event = SseParser::parse_frame(frame);
        assert!(matches!(
            event,
            Some(SseParsedEvent::ContentBlockDelta { text }) if text == "Hello"
        ));
    }

    #[test]
    fn sse_parses_openai_done() {
        let frame = "data: [DONE]\n\n";
        let event = SseParser::parse_frame(frame);
        assert!(matches!(event, Some(SseParsedEvent::MessageStop)));
    }

    #[test]
    fn sse_parses_openai_finish() {
        let frame = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":12345,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n";
        let event = SseParser::parse_frame(frame);
        assert!(matches!(event, Some(SseParsedEvent::MessageStop)));
    }

    #[test]
    fn sse_parses_openai_usage() {
        let frame = "data: {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":12345,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":20,\"total_tokens\":30}}\n\n";
        let event = SseParser::parse_frame(frame);
        assert!(matches!(
            event,
            Some(SseParsedEvent::Usage { input_tokens: 10, output_tokens: 20 })
        ));
    }

    #[test]
    fn sse_ignores_ping() {
        let frame = concat!(
            ": keepalive\n",
            "event: ping\n",
            "data: {\"type\":\"ping\"}\n\n"
        );
        let event = SseParser::parse_frame(frame);
        assert!(matches!(event, Some(SseParsedEvent::Ping)));
    }

    #[test]
    fn sse_ignores_content_block_start() {
        let frame = concat!(
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n"
        );
        let event = SseParser::parse_frame(frame);
        assert!(event.is_none());
    }

    #[test]
    fn sse_chunked_stream() {
        let mut parser = SseParser::new();
        let first = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel";
        let second = b"lo\"}}\n\n";

        let first_events = parser.push(first);
        assert!(first_events.is_empty());

        let second_events = parser.push(second);
        assert_eq!(second_events.len(), 1);
        assert!(matches!(
            &second_events[0],
            SseParsedEvent::ContentBlockDelta { text } if text == "Hello"
        ));
    }

    #[test]
    fn llm_client_from_env_no_key() {
        // Without any keys set, from_env should return an error
        let prev_anthropic = std::env::var("ANTHROPIC_API_KEY").ok();
        let prev_openai = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");

        let result = LlmClient::from_env();
        assert!(result.is_err());

        // Restore
        if let Some(k) = prev_anthropic {
            std::env::set_var("ANTHROPIC_API_KEY", k);
        }
        if let Some(k) = prev_openai {
            std::env::set_var("OPENAI_API_KEY", k);
        }
    }
}
