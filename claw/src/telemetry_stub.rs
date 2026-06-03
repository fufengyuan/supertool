//! Minimal stubs replacing claw-code's `telemetry` crate.
//! Real implementations exist in the upstream `claw-code` repository;
//! these stubs exist only to let the LLM client code compile as a
//! standalone crate.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::error::ApiError;
use crate::types::MessageRequest;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

// ---------------------------------------------------------------------------
// ClientIdentity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ClientIdentity {
    app_name: String,
    app_version: String,
}

impl ClientIdentity {
    #[must_use]
    pub fn new(app_name: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
        }
    }

    #[must_use]
    pub fn with_runtime(self, _name: &str) -> Self {
        self
    }

    #[must_use]
    pub fn user_agent(&self) -> String {
        format!("{}/{}", self.app_name, self.app_version)
    }
}

// ---------------------------------------------------------------------------
// AnthropicRequestProfile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AnthropicRequestProfile {
    pub client_identity: ClientIdentity,
    betas: Vec<String>,
    extra_body: BTreeMap<String, Value>,
}

impl AnthropicRequestProfile {
    #[must_use]
    pub fn new(client_identity: ClientIdentity) -> Self {
        Self {
            client_identity,
            betas: Vec::new(),
            extra_body: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_beta(mut self, beta: impl Into<String>) -> Self {
        self.betas.push(beta.into());
        self
    }

    #[must_use]
    pub fn with_extra_body(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra_body.insert(key.into(), value);
        self
    }

    #[must_use]
    pub fn header_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        pairs.push((
            "anthropic-version".to_string(),
            DEFAULT_ANTHROPIC_VERSION.to_string(),
        ));
        pairs.push(("User-Agent".to_string(), self.client_identity.user_agent()));
        for beta in &self.betas {
            // Anthropic expects beta headers as a comma-separated list,
            // but the upstream telemetry crate sends them individually.
            pairs.push(("anthropic-beta".to_string(), beta.clone()));
        }
        pairs
    }

    pub fn render_json_body(&self, request: &MessageRequest) -> Result<Value, ApiError> {
        let mut value = serde_json::to_value(request).map_err(|e| ApiError::Json {
            provider: "Anthropic".to_string(),
            model: request.model.clone(),
            body_snippet: String::new(),
            source: e,
        })?;
        if let Some(obj) = value.as_object_mut() {
            for (key, val) in &self.extra_body {
                obj.insert(key.clone(), val.clone());
            }
        }
        Ok(value)
    }
}

impl Default for AnthropicRequestProfile {
    fn default() -> Self {
        Self::new(ClientIdentity::new("llm-client", "0.1.0"))
    }
}

// ---------------------------------------------------------------------------
// AnalyticsEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AnalyticsEvent {
    pub namespace: String,
    pub action: String,
    pub properties: BTreeMap<String, Value>,
}

impl AnalyticsEvent {
    #[must_use]
    pub fn new(namespace: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            action: action.into(),
            properties: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

// ---------------------------------------------------------------------------
// SessionTracer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SessionTracer;

impl SessionTracer {
    #[must_use]
    pub fn new(_name: &str, _sink: std::sync::Arc<MemoryTelemetrySink>) -> Self {
        Self
    }

    pub fn record_analytics(&self, _event: AnalyticsEvent) {}

    pub fn record_request_profile(&self, _profile: &AnthropicRequestProfile) {}

    pub fn record_event(&self, _event: &str) {}

    pub fn record_http_request_started(
        &self,
        _attempt: u32,
        _method: &str,
        _path: &str,
        _properties: Map<String, Value>,
    ) {
    }

    pub fn record_http_request_succeeded(
        &self,
        _attempt: u32,
        _method: &str,
        _path: &str,
        _status: u16,
        _request_id: Option<String>,
        _properties: Map<String, Value>,
    ) {
    }

    pub fn record_http_request_failed(
        &self,
        _attempt: u32,
        _method: &str,
        _path: &str,
        _error: String,
        _retryable: bool,
        _properties: Map<String, Value>,
    ) {
    }
}

impl Default for SessionTracer {
    fn default() -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// TelemetryEvent (minimal — used by tests)
// ---------------------------------------------------------------------------

use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    HttpRequestStarted {
        session_id: String,
        attempt: u32,
        method: String,
        path: String,
    },
    HttpRequestSucceeded {
        session_id: String,
        attempt: u32,
        method: String,
        path: String,
        status: u16,
        request_id: Option<String>,
    },
    HttpRequestFailed {
        session_id: String,
        attempt: u32,
        method: String,
        path: String,
        error: String,
        retryable: bool,
    },
    Analytics(AnalyticsEvent),
    SessionTrace(SessionTraceRecord),
}

#[derive(Debug, Clone)]
pub struct SessionTraceRecord {
    pub name: String,
}

pub trait TelemetrySink: Send + Sync {
    fn record(&self, event: TelemetryEvent);
}

#[derive(Default)]
pub struct MemoryTelemetrySink {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl MemoryTelemetrySink {
    #[must_use]
    pub fn events(&self) -> Vec<TelemetryEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl TelemetrySink for MemoryTelemetrySink {
    fn record(&self, event: TelemetryEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }
}
