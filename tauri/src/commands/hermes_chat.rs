//! Agent Chat via Hermes HTTP API
//!
//! Uses Hermes Gateway's built-in HTTP API server (port 8642).
//! Replaces the Python bridge for simpler architecture.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::commands::hermes_config::check_api_server_config;

/// Hermes HTTP API server URL
const HERMES_API_URL: &str = "http://localhost:8642";

// Global state
lazy_static::lazy_static! {
    static ref ABORT_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref CURRENT_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
    static ref CURRENT_RUN_ID: Mutex<Option<String>> = Mutex::new(None);
}

/// Create a reqwest client that bypasses system proxy for localhost requests.
fn local_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("Failed to build reqwest client")
}

/// Get the API key from Hermes config
fn get_api_key() -> Result<String, String> {
    let (enabled, has_key, key) = check_api_server_config();
    if enabled && has_key && !key.is_empty() {
        return Ok(key);
    }
    Err("Hermes API server not configured. Run 'hermes gateway restart' after setting API_SERVER_KEY in ~/.hermes/.env".to_string())
}

/// Check if Hermes API server is running
async fn check_api_server_health() -> bool {
    let client = local_client();
    client
        .get(format!("{}/health", HERMES_API_URL))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// SSE event types from Hermes API
#[derive(Debug, Deserialize)]
struct ToolProgressEvent {
    tool: String,
    #[serde(default)]
    emoji: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    #[serde(rename = "toolCallId")]
    tool_call_id: Option<String>,
    status: String, // "running" or "completed"
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    object: Option<String>,
    choices: Vec<ChoiceDelta>,
}

#[derive(Debug, Deserialize)]
struct ChoiceDelta {
    index: u32,
    delta: DeltaContent,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    #[serde(default)]
    content: Option<String>,
}

/// Parse SSE line and extract event type and data
fn parse_sse_line(line: &str) -> Option<(Option<String>, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return None; // Comment or empty line
    }
    
    if let Some(data) = line.strip_prefix("data: ") {
        return Some((None, data.to_string()));
    }
    if let Some(event) = line.strip_prefix("event: ") {
        return Some((Some(event.to_string()), String::new()));
    }
    None
}

/// Send chat message via Hermes HTTP API with SSE streaming
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_chat(
    app: AppHandle,
    message: String,
    session_id: Option<String>,
    model: Option<String>,
    toolsets: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    // 1. Check API server health
    if !check_api_server_health().await {
        return Err("Hermes API server not running. Make sure gateway is started with API_SERVER_ENABLED=true".to_string());
    }

    // 2. Get API key
    let api_key = get_api_key()?;

    // 3. Reset abort flag and record session_id
    ABORT_FLAG.store(false, Ordering::SeqCst);
    {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = session_id.clone();
    }

    // 4. Build OpenAI-compatible request
    let client = local_client();
    let request_body = serde_json::json!({
        "model": model.unwrap_or_else(|| "hermes-agent".to_string()),
        "messages": [{"role": "user", "content": message}],
        "stream": true,
    });

    // 5. Send request with headers
    let mut req = client
        .post(format!("{}/v1/chat/completions", HERMES_API_URL))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Add session_id header if provided (for session continuity)
    if let Some(ref sid) = session_id {
        req = req.header("X-Hermes-Session-Id", sid);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Chat request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, text));
    }

    // 6. Extract session_id from response header
    let response_session_id = resp.headers()
        .get("X-Hermes-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let captured_session_id = session_id.or(response_session_id);
    if let Some(ref sid) = captured_session_id {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = Some(sid.clone());
    }

    // 7. Stream SSE response
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut accumulated_text = String::new();
    let mut final_response: Option<String> = None;
    let mut current_event_type: Option<String> = None;

    // Track tool calls for status updates
    let mut running_tools: std::collections::HashMap<String, (String, Option<serde_json::Value>)> = std::collections::HashMap::new();

    while let Some(chunk_result) = stream.next().await {
        // Check abort flag
        if ABORT_FLAG.load(Ordering::SeqCst) {
            // Try to stop the run if we have a run_id
            let run_id = CURRENT_RUN_ID.lock().unwrap().clone();
            if let Some(ref rid) = run_id {
                let _ = client
                    .post(format!("{}/v1/runs/{}/stop", HERMES_API_URL, rid))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send()
                    .await;
            }
            break;
        }

        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        // Process complete SSE events (ended by \n\n)
        while let Some(end_pos) = buffer.find("\n\n") {
            let event_block = buffer[..end_pos].to_string();
            buffer = buffer[end_pos + 2..].to_string();

            // Parse event block
            let mut event_type: Option<String> = None;
            let mut event_data: Option<String> = None;

            for line in event_block.lines() {
                if let Some((etype, data)) = parse_sse_line(line) {
                    if etype.is_some() {
                        event_type = etype;
                    } else if !data.is_empty() {
                        event_data = Some(data);
                    }
                }
            }

            // Handle event
            if let Some(data) = event_data {
                match event_type.as_deref() {
                    Some("hermes.tool.progress") => {
                        // Tool progress event
                        if let Ok(progress) = serde_json::from_str::<ToolProgressEvent>(&data) {
                            let tool_name = progress.tool.clone();
                            let tool_id = progress.tool_call_id.clone().unwrap_or_else(|| format!("tool-{}", running_tools.len()));
                            
                            if progress.status == "running" {
                                // Tool started
                                running_tools.insert(tool_id.clone(), (tool_name.clone(), None));
                                app.emit("agent-tool-start", serde_json::json!({
                                    "id": tool_id,
                                    "name": tool_name,
                                    "args": serde_json::Value::Null, // Hermes API doesn't send args
                                    "session_id": captured_session_id,
                                    "label": progress.label,
                                    "emoji": progress.emoji,
                                })).ok();
                            } else if progress.status == "completed" {
                                // Tool completed
                                if let Some((name, _)) = running_tools.remove(&tool_id) {
                                    app.emit("agent-tool-complete", serde_json::json!({
                                        "id": tool_id,
                                        "name": name,
                                        "result": Option::<String>::None, // Hermes API doesn't send result
                                        "duration_ms": 0,
                                        "session_id": captured_session_id,
                                    })).ok();
                                }
                            }
                        }
                    }
                    _ => {
                        // Default: chat completion chunk
                        if data == "[DONE]" {
                            // Stream finished
                            final_response = Some(accumulated_text.clone());
                            app.emit("agent-done", serde_json::json!({
                                "response": accumulated_text.clone(),
                                "session_id": captured_session_id,
                                "message_count": 0,
                            })).ok();
                        } else if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(&data) {
                            // Extract content from delta
                            for choice in &chunk.choices {
                                if let Some(content) = &choice.delta.content {
                                    accumulated_text.push_str(content);
                                    app.emit("agent-delta", serde_json::json!({
                                        "text": content,
                                        "session_id": captured_session_id,
                                    })).ok();
                                }
                                if choice.finish_reason.as_deref() == Some("stop") {
                                    final_response = Some(accumulated_text.clone());
                                    app.emit("agent-done", serde_json::json!({
                                        "response": accumulated_text.clone(),
                                        "session_id": captured_session_id,
                                        "message_count": 0,
                                    })).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({
        "response": final_response.unwrap_or(accumulated_text),
        "session_id": captured_session_id,
        "message_count": 0,
    }))
}

/// Abort current chat
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_abort_chat() -> Result<serde_json::Value, String> {
    ABORT_FLAG.store(true, Ordering::SeqCst);

    let session_id = CURRENT_SESSION_ID.lock().unwrap().clone();
    let run_id = CURRENT_RUN_ID.lock().unwrap().clone();

    // Try to stop via Hermes API if we have a run_id
    if let Some(ref rid) = run_id {
        let api_key = get_api_key().ok();
        if let Some(key) = api_key {
            let client = local_client();
            let _ = client
                .post(format!("{}/v1/runs/{}/stop", HERMES_API_URL, rid))
                .header("Authorization", format!("Bearer {}", key))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await;
        }
    }

    Ok(serde_json::json!({
        "aborted": true,
        "session_id": session_id,
    }))
}

/// Clear cache - no longer needed with Hermes HTTP API (agent cache is per-request)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_clear_cache(_session_id: String) -> Result<serde_json::Value, String> {
    // With Hermes HTTP API, there's no persistent agent cache to clear
    // Each request creates a fresh agent context (session continuity via X-Hermes-Session-Id)
    Ok(serde_json::json!({"ok": true, "session_id": _session_id}))
}

/// Check Agent availability (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_check_available() -> Result<serde_json::Value, String> {
    let installed = crate::commands::hermes_config::hermes_is_installed();
    let (api_enabled, has_key, _) = check_api_server_config();
    let api_running = check_api_server_health().await;

    Ok(serde_json::json!({
        "available": installed,
        "api_enabled": api_enabled,
        "api_key_configured": has_key,
        "api_running": api_running,
        "ready": installed && api_enabled && has_key && api_running,
        "error": if !installed {
            serde_json::Value::String("Hermes Agent not installed".to_string())
        } else if !api_enabled {
            serde_json::Value::String("API server not enabled. Add API_SERVER_ENABLED=true to ~/.hermes/.env".to_string())
        } else if !has_key {
            serde_json::Value::String("API key not configured. Add API_SERVER_KEY=xxx to ~/.hermes/.env".to_string())
        } else if !api_running {
            serde_json::Value::String("API server not running. Run 'hermes gateway restart'".to_string())
        } else {
            serde_json::Value::Null
        },
    }))
}

/// Get custom models from Hermes config (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_get_models() -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::get_models()
}

/// Add a model to Hermes config (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_add_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::add_model(model)
}

/// Remove a model from Hermes config (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_remove_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::remove_model(model)
}

/// Set the default model in Hermes config (persists to config.yaml)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_set_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::set_default_model(model)
}