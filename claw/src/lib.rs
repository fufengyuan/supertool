// Stub modules that replace claw-code's runtime/telemetry dependencies
pub mod runtime_stub;
pub mod telemetry_stub;

// --- Redirect claw-code imports to our stubs ---
pub mod runtime {
    pub use crate::runtime_stub::*;
}
pub mod telemetry {
    pub use crate::telemetry_stub::*;
}

// Now copy claw-code's modules (with `use runtime::` and `use telemetry::` resolved)
mod error;
mod http_client;
pub mod prompt_cache;
mod sse;
mod types;
pub mod providers;
mod client;

pub use client::{
    oauth_token_is_expired, read_base_url, read_xai_base_url, resolve_saved_oauth_token,
    resolve_startup_auth_source, MessageStream, OAuthTokenSet, ProviderClient,
};
pub use error::ApiError;
pub use http_client::{build_http_client, build_http_client_or_default, build_http_client_with, ProxyConfig};
pub use providers::anthropic::{AnthropicClient, AnthropicClient as ApiClient, AuthSource};
pub use providers::openai_compat::{
    build_chat_completion_request, check_request_body_size, estimate_request_body_size,
    flatten_tool_result_content, is_reasoning_model, model_rejects_is_error_field,
    model_requires_reasoning_content_in_history, translate_message, OpenAiCompatClient,
    OpenAiCompatConfig,
};
pub use providers::{
    detect_provider_kind, max_tokens_for_model, max_tokens_for_model_with_override,
    model_family_identity_for, model_family_identity_for_kind, provider_diagnostics_for_model,
    resolve_model_alias, ProviderDiagnostics, ProviderKind,
};
pub use sse::{parse_frame, SseParser};
pub use types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent, ContentBlockStopEvent,
    InputContentBlock, InputMessage, MessageDelta, MessageDeltaEvent, MessageRequest,
    MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock, StreamEvent,
    ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};
