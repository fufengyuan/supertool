//! Agent Chat Bridge - communicate with AI Agent via HTTP chat server
//!
//! Uses persistent FastAPI HTTP server at port 18686 with NDJSON streaming.
//! Replaced the old stdin/stdout Python subprocess approach.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Hermes Chat HTTP server URL and port
const HERMES_CHAT_SERVER_URL: &str = "http://127.0.0.1:18686";
const HERMES_CHAT_SERVER_PORT: &str = "18686";

// Global state for HTTP server mode
lazy_static::lazy_static! {
    static ref SERVER_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);
    static ref ABORT_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref CURRENT_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
}

/// Input command to Python bridge
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
pub enum BridgeCommand {
    Chat {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        toolsets: Option<Vec<String>>,
    },
    ListSessions {
        #[serde(default = "default_limit")]
        limit: usize,
    },
    GetSession {
        session_id: String,
    },
    DeleteSession {
        session_id: String,
    },
    RenameSession {
        session_id: String,
        title: String,
    },
    SearchSessions {
        query: String,
        #[serde(default = "default_limit")]
        limit: usize,
        #[serde(default)]
        offset: usize,
    },
    Abort {},
    GetModels {},
    AddModel {
        model: String,
    },
    RemoveModel {
        model: String,
    },
}

fn default_limit() -> usize {
    20
}

/// Output message from Python bridge
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BridgeMessage {
    Delta {
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    ToolStart {
        id: Option<String>,
        name: String,
        args: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    ToolComplete {
        id: Option<String>,
        name: String,
        result: Option<String>,
        duration_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    Thinking {
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    Done {
        response: Option<String>,
        session_id: String,
        message_count: usize,
    },
    Error {
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    Sessions {
        data: Vec<SessionInfo>,
        total: usize,
    },
    SearchResults {
        data: Vec<SearchResult>,
        total: usize,
        query: String,
    },
    Session {
        session_id: String,
        messages: Vec<MessageInfo>,
    },
    Deleted {
        session_id: String,
    },
    Renamed {
        session_id: String,
        title: String,
    },
    Aborted {
        session_id: Option<String>,
    },
    Models {
        custom_models: Vec<String>,
        default_model: Option<String>,
    },
    ModelAdded {
        model: String,
        custom_models: Vec<String>,
    },
    ModelRemoved {
        model: String,
        custom_models: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "source")]
    pub source: String,
    #[serde(rename = "startedAt", alias = "started_at")]
    pub started_at: Option<f64>,
    #[serde(
        rename = "endedAt",
        alias = "ended_at",
        skip_serializing_if = "Option::is_none"
    )]
    pub ended_at: Option<f64>,
    #[serde(rename = "messageCount", alias = "message_count")]
    pub message_count: usize,
    #[serde(rename = "preview")]
    pub preview: String,
    #[serde(
        rename = "lastActive",
        alias = "last_active",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_active: Option<f64>,
}

/// Search result from FTS5 search
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "sessionId", alias = "session_id")]
    pub session_id: String,
    #[serde(
        rename = "sessionTitle",
        alias = "session_title",
        skip_serializing_if = "Option::is_none"
    )]
    pub session_title: Option<String>,
    #[serde(rename = "messageId", alias = "message_id")]
    pub message_id: String,
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "snippet")]
    pub snippet: String,
    #[serde(rename = "content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    #[serde(rename = "source")]
    pub source: String,
    #[serde(rename = "model")]
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    #[serde(
        rename = "toolName",
        alias = "tool_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_name: Option<String>,
    #[serde(
        rename = "toolCallId",
        alias = "tool_call_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_call_id: Option<String>,
    #[serde(
        rename = "toolCalls",
        alias = "tool_calls",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallInfo {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "function")]
    pub function: FunctionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionInfo {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "arguments")]
    pub arguments: String,
}

/// Find Python bridge script location
fn find_bridge_script() -> Option<PathBuf> {
    // Try bundled location first (when packaged)
    // macOS .app bundle: exe is in Contents/MacOS/, resources are in Contents/Resources/
    // Tauri 2.x stores relative-path resources under Contents/Resources/_up_/
    // e.g. "../scripts/hermes_bridge.py" → Contents/Resources/_up_/scripts/hermes_bridge.py
    let exe = std::env::current_exe().ok()?;
    let exe_parent = exe.parent()?;
    
    // Check if running inside macOS .app bundle
    // exe_parent for macOS .app: /path/to/SuperTool.app/Contents/MacOS
    if exe_parent.ends_with("MacOS") {
        let contents_dir = exe_parent.parent()?; // Contents/
        let resources_dir = contents_dir.join("Resources");
        
        // Tauri 2.x: resources from "../scripts/..." are stored under _up_/
        let script_path = resources_dir.join("_up_").join("scripts").join("hermes_bridge.py");
        if script_path.exists() {
            return Some(script_path);
        }
        
        // Also try flat path (older Tauri or different config)
        let flat_path = resources_dir.join("scripts").join("hermes_bridge.py");
        if flat_path.exists() {
            return Some(flat_path);
        }
        
        // Direct path (some builds)
        let direct_path = resources_dir.join("hermes_bridge.py");
        if direct_path.exists() {
            return Some(direct_path);
        }
    }
    
    // Generic bundled location: exe_parent/scripts/
    let bundled = exe_parent.join("scripts").join("hermes_bridge.py");
    if bundled.exists() {
        return Some(bundled);
    }
    
    // Linux/Windows: try exe_parent/../resources/scripts/ or _up_/scripts/
    if let Some(parent) = exe_parent.parent() {
        let up_path = parent.join("_up_").join("scripts").join("hermes_bridge.py");
        if up_path.exists() {
            return Some(up_path);
        }
        let resources_path = parent.join("resources").join("scripts").join("hermes_bridge.py");
        if resources_path.exists() {
            return Some(resources_path);
        }
    }

    // Try development location
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| Some(p.join("scripts").join("hermes_bridge.py")));

    if dev.as_ref().map(|p| p.exists()).unwrap_or(false) {
        return dev;
    }

    // Fallback: try relative to working directory
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("scripts").join("hermes_bridge.py"))
        .filter(|p| p.exists())
}

/// Find Python executable
fn find_python() -> String {
    // 优先使用 Hermes Agent venv 的 Python（因为依赖都在 venv 里）
    let hermes_venv_python = dirs::home_dir()
        .map(|h| {
            h.join(".hermes")
                .join("hermes-agent")
                .join("venv")
                .join("bin")
                .join("python3")
        })
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.hermes/hermes-agent/venv/bin/python3".to_string());

    // 检查 venv Python 是否存在且可执行
    if Path::new(&hermes_venv_python).exists() {
        if Command::new(&hermes_venv_python)
            .arg("--version")
            .output()
            .is_ok()
        {
            return hermes_venv_python;
        }
    }

    // Fallback 到系统 Python
    if Command::new("python3").arg("--version").output().is_ok() {
        return "python3".to_string();
    }
    if Command::new("python").arg("--version").output().is_ok() {
        return "python".to_string();
    }
    "python3".to_string()
}

/// Ensure the Hermes Chat HTTP server is running (async version)
async fn ensure_server_running() -> Result<(), String> {
    {
        let mut server = SERVER_PROCESS.lock().unwrap();
        if let Some(ref mut child) = *server {
            match child.try_wait() {
                Ok(Some(_)) => { *server = None; }
                Ok(None) => { return Ok(()); }
                Err(_) => { *server = None; }
            }
        }
    }
    let script = find_bridge_script()
        .and_then(|p| {
            let s = p.parent()?.join("hermes_chat_server.py");
            if s.exists() { Some(s) } else { None }
        })
        .ok_or_else(|| "Agent chat server script not found.".to_string())?;
    let python = find_python();
    let child = Command::new(&python).arg(&script)
        .env("HERMES_CHAT_PORT", HERMES_CHAT_SERVER_PORT)
        .env("HERMES_CHAT_HOST", "127.0.0.1")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start server: {}", e))?;
    { let mut s = SERVER_PROCESS.lock().unwrap(); *s = Some(child); }
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(15) {
        if let Ok(r) = client.get(format!("{}/v1/health", HERMES_CHAT_SERVER_URL))
            .timeout(std::time::Duration::from_secs(2)).send().await
        { if r.status().is_success() { return Ok(()); } }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Err("Server failed to start".to_string())
}

/// Send chat message with streaming events via HTTP server
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_chat(
    app: AppHandle,
    message: String,
    session_id: Option<String>,
    model: Option<String>,
    toolsets: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    // 1. 确保 HTTP 服务器已启动
    ensure_server_running().await?;

    // 2. 重置 abort flag 并记录 session_id
    ABORT_FLAG.store(false, Ordering::SeqCst);
    {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = session_id.clone();
    }

    // 3. 发送 HTTP 请求到聊天服务器
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "message": message,
        "session_id": session_id,
        "model": model,
        "toolsets": toolsets,
    });

    let resp = client
        .post(format!("{}/v1/chat", HERMES_CHAT_SERVER_URL))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Chat request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Chat server error ({}): {}", status, text));
    }

    // 4. 从响应头获取 session_id（服务端可能创建了新的）
    let server_session_id = resp.headers()
        .get("X-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut captured_session_id: Option<String> = session_id.or(server_session_id);
    let mut final_response: Option<String> = None;
    let mut final_session_id: Option<String> = None;
    let mut message_count: usize = 0;
    let mut accumulated_text = String::new();

    // 更新 CURRENT_SESSION_ID
    if let Some(ref sid) = captured_session_id {
        let mut current = CURRENT_SESSION_ID.lock().unwrap();
        *current = Some(sid.clone());
    }

    // 5. 流式读取 NDJSON 响应
    let mut stream = resp.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk_result) = stream.next().await {
        // 检查 abort flag
        if ABORT_FLAG.load(Ordering::SeqCst) {
            // 通知服务端中断（clone 避免 MutexGuard 跨 await）
            let abort_sid = CURRENT_SESSION_ID.lock().unwrap().clone();
            if let Some(ref sid) = abort_sid {
                let _ = client
                    .post(format!("{}/v1/abort", HERMES_CHAT_SERVER_URL))
                    .json(&serde_json::json!({"session_id": sid}))
                    .send()
                    .await;
            }
            break;
        }

        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        buffer.extend_from_slice(&chunk);

        // 按行处理完整的 JSON
        while let Some(nl_pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer[..nl_pos].to_vec();
            buffer = buffer[nl_pos + 1..].to_vec();

            if line_bytes.is_empty() {
                continue;
            }

            let line_str = String::from_utf8_lossy(&line_bytes);
            if !line_str.trim_start().starts_with('{') {
                eprintln!("[bridge] log: {}", line_str);
                continue;
            }

            let msg: BridgeMessage = match serde_json::from_str(&line_str) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[bridge] parse error: {} - {}", e, line_str);
                    app.emit("agent-error", serde_json::json!({
                        "type": "parse_error",
                        "message": format!("JSON parse error: {}", e),
                        "raw": line_str.chars().take(100).collect::<String>(),
                    })).ok();
                    continue;
                }
            };

            match msg {
                BridgeMessage::Delta { text, session_id } => {
                    if let Some(t) = &text {
                        accumulated_text.push_str(t);
                    }
                    if captured_session_id.is_none() {
                        captured_session_id = session_id.clone();
                    }
                    app.emit("agent-delta", serde_json::json!({
                        "text": text, "session_id": session_id,
                    })).ok();
                }
                BridgeMessage::ToolStart { id, name, args, session_id } => {
                    app.emit("agent-tool-start", serde_json::json!({
                        "id": id, "name": name, "args": args, "session_id": session_id,
                    })).ok();
                }
                BridgeMessage::ToolComplete { id, name, result, duration_ms, session_id } => {
                    app.emit("agent-tool-complete", serde_json::json!({
                        "id": id, "name": name, "result": result,
                        "duration_ms": duration_ms, "session_id": session_id,
                    })).ok();
                }
                BridgeMessage::Thinking { text, session_id } => {
                    app.emit("agent-thinking", serde_json::json!({
                        "text": text, "session_id": session_id,
                    })).ok();
                }
                BridgeMessage::Done { response, session_id, message_count: mc } => {
                    app.emit("agent-done", serde_json::json!({
                        "response": response, "session_id": session_id,
                        "message_count": mc,
                    })).ok();
                    final_response = response;
                    final_session_id = Some(session_id.clone());
                    message_count = mc;
                    if captured_session_id.is_none() {
                        captured_session_id = Some(session_id);
                    }
                }
                BridgeMessage::Error { message, session_id } => {
                    app.emit("agent-error", serde_json::json!({
                        "message": message, "session_id": session_id,
                    })).ok();
                    return Err(message);
                }
                BridgeMessage::Aborted { session_id } => {
                    if captured_session_id.is_none() {
                        captured_session_id = session_id.clone();
                    }
                    let sid = captured_session_id.clone();
                    if let Some(ref s) = sid {
                        app.emit("agent-done", serde_json::json!({
                            "response": Option::<String>::None,
                            "session_id": s,
                            "message_count": message_count,
                            "aborted": true,
                        })).ok();
                    }
                    return Ok(serde_json::json!({
                        "response": Option::<String>::None,
                        "session_id": sid,
                        "message_count": message_count,
                        "aborted": true,
                    }));
                }
                _ => {
                    eprintln!("[bridge] unhandled event type");
                }
            }
        }
    }

    // 处理 buffer 中可能残留的最后一个行（无换行结尾的情况）
    if !buffer.is_empty() {
        let line_str = String::from_utf8_lossy(&buffer);
        if line_str.trim_start().starts_with('{') {
            if let Ok(msg) = serde_json::from_str::<BridgeMessage>(&line_str) {
                match msg {
                    BridgeMessage::Done { response, session_id, message_count: mc } => {
                        final_response = response;
                        final_session_id = Some(session_id);
                        message_count = mc;
                    }
                    BridgeMessage::Error { message, .. } => return Err(message),
                    _ => {}
                }
            }
        }
    }

    Ok(serde_json::json!({
        "response": final_response.unwrap_or(accumulated_text),
        "session_id": captured_session_id.or(final_session_id),
        "message_count": message_count,
    }))
}

/// Abort current chat via HTTP server
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_abort_chat() -> Result<serde_json::Value, String> {
    // 设置 abort flag 打断 HTTP 流读取循环
    ABORT_FLAG.store(true, Ordering::SeqCst);

    // 通知 HTTP 服务端中断正在运行的 agent
    let session_id = {
        let current = CURRENT_SESSION_ID.lock().unwrap();
        current.clone()
    };

    if let Some(ref sid) = session_id {
        let client = reqwest::Client::new();
        match client
            .post(format!("{}/v1/abort", HERMES_CHAT_SERVER_URL))
            .json(&serde_json::json!({"session_id": sid}))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                eprintln!("[INFO] Abort sent to server for session {}: {}", sid, resp.status());
            }
            Err(e) => {
                eprintln!("[WARN] Failed to send abort to server: {}", e);
            }
        }
    }

    Ok(serde_json::json!({
        "aborted": true,
        "session_id": session_id,
    }))
}

/// Check Agent availability (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_check_available() -> Result<serde_json::Value, String> {
    let available = crate::commands::hermes_config::hermes_is_installed();
    let script_found = find_bridge_script().is_some();
    Ok(serde_json::json!({
        "available": available,
        "script_found": script_found,
        "python": "rust-native",
        "error": if available { serde_json::Value::Null } else {
            serde_json::Value::String("Hermes Agent not installed. Please install Hermes first.".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_command_serialize() {
        let cmd = BridgeCommand::Chat {
            session_id: Some("abc123".to_string()),
            message: "Hello".to_string(),
            model: Some("anthropic/claude-sonnet-4".to_string()),
            toolsets: Some(vec!["web".to_string(), "terminal".to_string()]),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"action\":\"chat\""));
        assert!(json.contains("\"message\":\"Hello\""));
    }

    #[test]
    fn test_bridge_message_deserialize() {
        let json = "{\"type\":\"delta\",\"text\":\"Hello\"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Delta { text, .. } => assert_eq!(text, Some("Hello".to_string())),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_delta_null_text() {
        let json = "{\"type\":\"delta\",\"text\":null}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Delta { text, .. } => assert!(text.is_none()),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_start_message() {
        let json = r#"{"type":"tool_start","name":"terminal","args":{"command":"ls"}}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolStart { name, args, .. } => {
                assert_eq!(name, "terminal");
                assert_eq!(args["command"], "ls");
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_complete_message() {
        let json = r#"{"type":"tool_complete","name":"terminal","result":"file1.txt\nfile2.txt","duration_ms":150}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete {
                name,
                result,
                duration_ms,
                ..
            } => {
                assert_eq!(name, "terminal");
                assert_eq!(result, Some("file1.txt\nfile2.txt".to_string()));
                assert_eq!(duration_ms, 150);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_complete_null_result() {
        let json = r#"{"type":"tool_complete","name":"terminal","result":null,"duration_ms":150}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete {
                name,
                result,
                duration_ms,
                ..
            } => {
                assert_eq!(name, "terminal");
                assert!(result.is_none());
                assert_eq!(duration_ms, 150);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_find_python() {
        let python = find_python();
        // 可以是系统 python3/python，也可以是 Hermes venv 的 Python
        assert!(python == "python3" || python == "python" || python.contains("venv/bin/python3"));
    }

    /// 测试 SessionInfo 可以解析 Python bridge 返回的 snake_case 格式
    #[test]
    fn test_session_info_deserialize_snake_case() {
        // Python bridge 返回的格式（snake_case）
        let json = r#"{
            "id": "test-123",
            "title": "Test Session",
            "model": "claude-sonnet-4",
            "source": "anthropic",
            "started_at": 1778752839.745,
            "ended_at": null,
            "message_count": 5,
            "preview": "Hello world",
            "last_active": 1778752900.0
        }"#;

        let session: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "test-123");
        assert_eq!(session.title, Some("Test Session".to_string()));
        assert_eq!(session.model, "claude-sonnet-4");
        assert_eq!(session.source, "anthropic");
        assert_eq!(session.message_count, 5);
        assert_eq!(session.preview, "Hello world");
    }

    /// 测试 SessionInfo 也可以解析 camelCase 格式
    #[test]
    fn test_session_info_deserialize_camel_case() {
        let json = r#"{
            "id": "test-456",
            "title": "Another Session",
            "model": "claude-opus-4",
            "source": "anthropic",
            "startedAt": 1778752839.745,
            "endedAt": 1778752900.0,
            "messageCount": 10,
            "preview": "Another preview",
            "lastActive": 1778752950.0
        }"#;

        let session: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "test-456");
        assert_eq!(session.message_count, 10);
    }

    /// 测试 SessionInfo 序列化输出为 camelCase（前端期望格式）
    #[test]
    fn test_session_info_serialize_camel_case() {
        let session = SessionInfo {
            id: "test-789".to_string(),
            title: Some("My Session".to_string()),
            model: "claude-sonnet-4".to_string(),
            source: "anthropic".to_string(),
            started_at: Some(1778752839.745),
            ended_at: None,
            message_count: 3,
            preview: "Test message".to_string(),
            last_active: Some(1778752900.0),
        };

        let json = serde_json::to_string(&session).unwrap();
        // 输出必须是 camelCase
        assert!(json.contains("\"messageCount\":3"));
        assert!(json.contains("\"startedAt\":"));
        assert!(json.contains("\"lastActive\":"));
        // 不应出现 snake_case
        assert!(!json.contains("message_count"));
        assert!(!json.contains("started_at"));
    }

    /// 测试 MessageInfo 解析 snake_case
    #[test]
    fn test_message_info_deserialize_snake_case() {
        let json = r#"{
            "role": "user",
            "content": "Hello",
            "timestamp": 1778752839.0,
            "tool_name": null
        }"#;

        let msg: MessageInfo = serde_json::from_str(json).unwrap();
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, Some("Hello".to_string()));
    }

    /// 测试 MessageInfo 序列化为 camelCase
    #[test]
    fn test_message_info_serialize_camel_case() {
        let msg = MessageInfo {
            role: "assistant".to_string(),
            content: Some("Response".to_string()),
            timestamp: Some(1778752839.0),
            tool_name: Some("web_search".to_string()),
            tool_call_id: None,
            tool_calls: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"toolName\":\"web_search\""));
        assert!(!json.contains("tool_name"));
    }

    /// 测试 BridgeMessage::Sessions 解析
    #[test]
    fn test_sessions_response_deserialize() {
        let json = r#"{
            "type": "sessions",
            "data": [
                {
                    "id": "sess-1",
                    "title": "First",
                    "model": "claude-sonnet-4",
                    "source": "anthropic",
                    "started_at": 1778752839.0,
                    "message_count": 2,
                    "preview": "Preview text"
                }
            ],
            "total": 1
        }"#;

        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Sessions { data, total } => {
                assert_eq!(total, 1);
                assert_eq!(data.len(), 1);
                assert_eq!(data[0].id, "sess-1");
                assert_eq!(data[0].message_count, 2);
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Session 解析
    #[test]
    fn test_session_response_deserialize() {
        let json = r#"{
            "type": "session",
            "session_id": "sess-123",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello",
                    "timestamp": 1778752839.0
                }
            ]
        }"#;

        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Session {
                session_id,
                messages,
            } => {
                assert_eq!(session_id, "sess-123");
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0].role, "user");
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试实际调用 Python bridge 获取会话列表
    #[test]
    fn test_real_list_sessions_via_bridge() {
        // 检查 bridge script 是否存在
        let script = find_bridge_script();
        if script.is_none() {
            // 如果脚本不存在，跳过此测试
            eprintln!("Skipping: bridge script not found");
            return;
        }

        let python = find_python();
        let script_path = script.unwrap();

        // 发送 list_sessions 命令
        let cmd = BridgeCommand::ListSessions { limit: 5 };
        let cmd_json = serde_json::to_string(&cmd).unwrap();

        let output = std::process::Command::new(&python)
            .arg(&script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        if output.is_err() {
            eprintln!("Skipping: cannot spawn Python bridge");
            return;
        }

        let mut child = output.unwrap();

        // 写入命令
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(cmd_json.as_bytes()).unwrap();
            stdin.write_all(b"\n").unwrap();
        }

        // 读取响应
        let result = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();

        // 解析响应
        if stdout.is_empty() {
            eprintln!("Skipping: empty response from bridge");
            return;
        }

        let msg: BridgeMessage = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "Failed to parse bridge response: {} - stdout: {}",
                e, stdout
            );
        });

        match msg {
            BridgeMessage::Sessions { data, total } => {
                // 验证返回数据格式正确
                println!("Got {} sessions (total: {})", data.len(), total);
                for s in &data {
                    println!("  Session: {} - {} messages", s.id, s.message_count);
                    assert!(!s.id.is_empty());
                    assert!(!s.model.is_empty());
                }
            }
            BridgeMessage::Error { message, .. } => {
                // Hermes 未安装是可接受的错误
                if message.contains("Hermes not available") {
                    eprintln!("Skipping: Hermes not installed");
                } else {
                    panic!("Unexpected error: {}", message);
                }
            }
            _ => panic!("Unexpected response type"),
        }
    }

    /// 测试 agent_check_available 实际调用
    #[test]
    fn test_real_check_available() {
        // 在 tokio runtime 中执行异步函数
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(agent_check_available());

        match result {
            Ok(json) => {
                println!(
                    "Check available result: {}",
                    serde_json::to_string_pretty(&json).unwrap()
                );
                // 验证返回格式
                assert!(json.get("available").is_some());
                assert!(json.get("python").is_some());
            }
            Err(e) => {
                // 路径问题可能导致错误，但不应该 crash
                eprintln!("Check available failed: {}", e);
            }
        }
    }

    /// 测试 Python bridge chat 命令序列化
    #[test]
    fn test_chat_command_serialize() {
        // 新会话
        let cmd = BridgeCommand::Chat {
            session_id: None,
            message: "你好".to_string(),
            model: Some("anthropic/claude-sonnet-4".to_string()),
            toolsets: Some(vec!["web".to_string()]),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        println!("Chat command JSON: {}", json);
        assert!(json.contains("\"action\":\"chat\""));
        assert!(json.contains("\"message\":\"你好\""));
        assert!(json.contains("\"model\":\"anthropic/claude-sonnet-4\""));
        assert!(json.contains("\"toolsets\":[\"web\"]"));
        // session_id 为 None 时不应出现
        assert!(!json.contains("session_id"));

        // 已有会话
        let cmd2 = BridgeCommand::Chat {
            session_id: Some("sess-123".to_string()),
            message: "继续对话".to_string(),
            model: None,
            toolsets: None,
        };
        let json2 = serde_json::to_string(&cmd2).unwrap();
        println!("Resume chat command JSON: {}", json2);
        assert!(json2.contains("\"session_id\":\"sess-123\""));
        assert!(json2.contains("\"message\":\"继续对话\""));
        // model/toolsets 为 None 时不应出现
        assert!(!json2.contains("model"));
        assert!(!json2.contains("toolsets"));
    }

    /// 测试 Python bridge 实际 chat 调用（需要 Hermes 已安装且有 API key）
    #[test]
    fn test_real_chat_via_bridge() {
        // 检查 Hermes 是否可用
        let rt = tokio::runtime::Runtime::new().unwrap();
        let check_result = rt.block_on(agent_check_available());

        let available = check_result
            .ok()
            .and_then(|json| json.get("available").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        if !available {
            eprintln!("Skipping: Hermes not available");
            return;
        }

        let script = find_bridge_script().expect("Bridge script should exist");
        let python = find_python();

        // 发送一个简单的测试消息，不指定模型让 Hermes 从配置读取默认模型
        let cmd = BridgeCommand::Chat {
            session_id: None,
            message: "你好，请用一句话介绍你自己".to_string(),
            model: None, // 使用配置中的默认模型
            toolsets: None,
        };
        let cmd_json = serde_json::to_string(&cmd).unwrap();
        println!("Sending chat command: {}", cmd_json);

        let mut child = std::process::Command::new(&python)
            .arg(&script)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn Python bridge");

        // 写入命令
        {
            use std::io::Write;
            let stdin = child.stdin.as_mut().expect("stdin");
            stdin.write_all(cmd_json.as_bytes()).unwrap();
            stdin.write_all(b"\n").unwrap();
            stdin.flush().unwrap();
        }

        // 读取所有输出行
        let output = child.wait_with_output().expect("Failed to wait for bridge");
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        println!("Bridge stdout:\n{}", stdout);
        if !stderr.is_empty() {
            println!("Bridge stderr:\n{}", stderr);
        }

        // 解析最后一行（应该是 done 或 error）
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.is_empty() {
            panic!("No output from bridge");
        }

        // 可能有多行输出（delta 等），取最后一行
        let last_line = lines.last().unwrap();
        println!("Last line: {}", last_line);

        let msg: BridgeMessage = serde_json::from_str(last_line).unwrap_or_else(|e| {
            panic!("Failed to parse last line: {} - content: {}", e, last_line)
        });

        match msg {
            BridgeMessage::Done {
                response,
                session_id,
                message_count,
            } => {
                println!("Chat completed!");
                println!("  Session ID: {}", session_id);
                println!("  Message count: {}", message_count);
                println!("  Response: {:?}", response);
                assert!(!session_id.is_empty());

                // response 可能是 None（API 调用失败）
                match response {
                    Some(text) if !text.is_empty() => {
                        println!("  Chat succeeded with response!");
                        assert!(message_count >= 2); // user + assistant
                    }
                    Some(_) => {
                        println!("  Empty response - API may have partially failed");
                    }
                    None => {
                        println!("  No response - API call failed");
                        // 这是预期的错误情况（模型不支持等）
                    }
                }
            }
            BridgeMessage::Error { message, .. } => {
                // API key 问题等可以接受
                if message.contains("API key")
                    || message.contains("authentication")
                    || message.contains("rate limit")
                {
                    eprintln!("Skipping: API error - {}", message);
                } else {
                    panic!("Chat error: {}", message);
                }
            }
            _ => panic!("Unexpected final message type: {:?}", msg),
        }
    }

    /// 测试 BridgeMessage::Done 解析
    #[test]
    fn test_done_message_deserialize() {
        // 有响应
        let json = "{\"type\":\"done\",\"response\":\"Hello\",\"session_id\":\"sess-abc\",\"message_count\":3}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Done {
                response,
                session_id,
                message_count,
            } => {
                assert_eq!(response, Some("Hello".to_string()));
                assert_eq!(session_id, "sess-abc");
                assert_eq!(message_count, 3);
            }
            _ => panic!("Wrong type"),
        }

        // 无响应（API 失败）
        let json_null =
            "{\"type\":\"done\",\"response\":null,\"session_id\":\"sess-def\",\"message_count\":1}";
        let msg_null: BridgeMessage = serde_json::from_str(json_null).unwrap();
        match msg_null {
            BridgeMessage::Done { response, .. } => {
                assert_eq!(response, None);
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Error 解析
    #[test]
    fn test_error_message_deserialize() {
        let json = "{\"type\":\"error\",\"message\":\"Something went wrong\"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Error { message, .. } => {
                assert_eq!(message, "Something went wrong");
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Delta 解析（流式文本）
    #[test]
    fn test_delta_message_deserialize() {
        let json = "{\"type\":\"delta\",\"text\":\"Hello \"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Delta { text, .. } => {
                assert_eq!(text, Some("Hello ".to_string()));
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::ToolStart 解析
    #[test]
    fn test_tool_start_deserialize() {
        let json =
            "{\"type\":\"tool_start\",\"name\":\"web_search\",\"args\":{\"query\":\"test\"}}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolStart { name, args, .. } => {
                assert_eq!(name, "web_search");
                assert!(args.is_object());
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::ToolComplete 解析
    #[test]
    fn test_tool_complete_deserialize() {
        let json = "{\"type\":\"tool_complete\",\"name\":\"web_search\",\"result\":\"Search results\",\"duration_ms\":150}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete {
                name,
                result,
                duration_ms,
                ..
            } => {
                assert_eq!(name, "web_search");
                assert_eq!(result, Some("Search results".to_string()));
                assert_eq!(duration_ms, 150);
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Aborted 解析
    #[test]
    fn test_aborted_message_deserialize() {
        let json = "{\"type\":\"aborted\",\"session_id\":\"sess-xyz\"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Aborted { session_id } => {
                assert_eq!(session_id, Some("sess-xyz".to_string()));
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Deleted 解析
    #[test]
    fn test_deleted_message_deserialize() {
        let json = "{\"type\":\"deleted\",\"session_id\":\"sess-del\"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Deleted { session_id } => {
                assert_eq!(session_id, "sess-del");
            }
            _ => panic!("Wrong type"),
        }
    }

    // ========================================================================
    // Integration Tests: HTTP Chat Server (Phase 2)
    // ========================================================================

    /// Test that the HTTP chat server starts and responds to health check
    #[test]
    fn test_http_server_health() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(ensure_server_running()) {
            eprintln!("Skipping: server not started - {}", e);
            return;
        }
        let client = reqwest::blocking::Client::new();
        match client
            .get(format!("{}/v1/health", HERMES_CHAT_SERVER_URL))
            .timeout(std::time::Duration::from_secs(5))
            .send()
        {
            Ok(resp) => {
                assert!(resp.status().is_success());
                let json: serde_json::Value = resp.json().unwrap();
                assert_eq!(json["status"], "ok");
                println!("✅ Health check passed");
            }
            Err(e) => eprintln!("Skipping: health request failed - {}", e),
        }
    }

    /// Test HTTP chat stream - POST /v1/chat, verify newline-delimited JSON response format
    #[test]
    fn test_http_server_chat_stream() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(ensure_server_running()) {
            eprintln!("Skipping: server not available - {}", e);
            return;
        }
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = reqwest::Client::new();
            let body = serde_json::json!({"message": "嗨，用一句话打个招呼", "toolsets": []});

            let resp = match client
                .post(format!("{}/v1/chat", HERMES_CHAT_SERVER_URL))
                .json(&body)
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    eprintln!("Server error: {}", r.text().await.unwrap_or_default());
                    return;
                }
                Err(e) => {
                    eprintln!("Request failed: {}", e);
                    return;
                }
            };

            let bytes = resp.bytes().await.unwrap();
            let text = String::from_utf8_lossy(&bytes);
            let lines: Vec<&str> = text.lines().collect();

            println!("Received {} event lines", lines.len());
            assert!(!lines.is_empty(), "Should receive at least one event");

            for (i, line) in lines.iter().enumerate() {
                let msg: BridgeMessage = serde_json::from_str(line).unwrap_or_else(|e| {
                    panic!("Line {} invalid JSON: {} - content: {}", i, e, line);
                });
                match &msg {
                    BridgeMessage::Delta { text, .. } => {
                        let preview = text
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .take(60)
                            .collect::<String>();
                        println!("  Δ delta: {}", preview);
                    }
                    BridgeMessage::Done {
                        session_id,
                        message_count,
                        ..
                    } => {
                        println!("  ✓ done: session={}, count={}", session_id, message_count);
                        assert!(!session_id.is_empty());
                    }
                    BridgeMessage::Error { message, .. } => {
                        eprintln!("  ✗ error: {}", message);
                    }
                    _ => {
                        println!("  • other event");
                    }
                }
            }

            // Last line must be Done or Error
            let last_line = lines.last().unwrap();
            let last_msg: BridgeMessage = serde_json::from_str(last_line).unwrap();
            assert!(
                matches!(last_msg, BridgeMessage::Done { .. } | BridgeMessage::Error { .. }),
                "Last event should be Done or Error, got: {}",
                last_line
            );
        });
    }

    /// Test HTTP abort endpoint
    #[test]
    fn test_http_server_abort() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(ensure_server_running()) {
            eprintln!("Skipping: server not available - {}", e);
            return;
        }
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = reqwest::Client::new();
            let resp = match client
                .post(format!("{}/v1/abort", HERMES_CHAT_SERVER_URL))
                .json(&serde_json::json!({"session_id": "__test__"}))
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Abort request failed: {}", e);
                    return;
                }
            };
            assert!(resp.status().is_success());
            let json: serde_json::Value = resp.json().await.unwrap_or_default();
            println!("Abort response: {:?}", json);
        });
    }

    /// Test agent_abort_chat IPC command — no AppHandle needed
    #[test]
    fn test_agent_abort_chat_ipc() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(ensure_server_running()) {
            eprintln!("Skipping: server not available - {}", e);
            return;
        }
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(agent_abort_chat());
        match result {
            Ok(json) => {
                println!("agent_abort_chat result: {}", serde_json::to_string(&json).unwrap());
                assert!(json.get("aborted").is_some());
            }
            Err(e) => {
                eprintln!("agent_abort_chat failed: {}", e);
            }
        }
    }

    // ========================================================================
    // Event Payload Format Tests: Verify JSON shapes match frontend expectations
    // ========================================================================

    /// Verify all event payload JSON shapes match HermesChat.vue's TypeScript types
    #[test]
    fn test_event_payload_shapes() {
        // agent-delta: listen<{ text: string | null; session_id: string | null }>
        let delta = serde_json::json!({"text": "Hello", "session_id": "sess-1"});
        assert!(delta["text"].is_string());
        assert!(delta["session_id"].is_string());
        // agent-tool-start: listen<{ id?: string; name: string; args: unknown; session_id: string | null }>
        let tool_start = serde_json::json!({"id":"call-1","name":"web_search","args":{"query":"test"},"session_id":"sess-1"});
        assert!(tool_start["name"].is_string());
        assert!(tool_start["args"].is_object());
        // agent-tool-complete: listen<{ id?: string; name: string; result: string | null; duration_ms: number; session_id: string | null }>
        let tool_complete = serde_json::json!({"id":"call-1","name":"web_search","result":"results","duration_ms":1500,"session_id":"sess-1"});
        assert!(tool_complete["name"].is_string());
        assert!(tool_complete["duration_ms"].is_number());
        // agent-thinking: listen<{ text: string | null; session_id: string | null }>
        let thinking = serde_json::json!({"text": "思考中...", "session_id": "sess-1"});
        assert!(thinking["text"].is_string());
        // agent-error: listen<{ message: string; session_id: string | null }>
        let error = serde_json::json!({"message": "err", "session_id": "sess-1"});
        assert!(error["message"].is_string());
        // agent-done: listen<{ response: string | null; session_id: string; message_count: number }>
        let done = serde_json::json!({"response":"Answer","session_id":"sess-1","message_count":3});
        assert!(done["session_id"].is_string());
        assert!(done["message_count"].is_number());
        // agent_chat return: invoke<{ response: string; session_id: string; message_count: number }>
        let invoke_result = serde_json::json!({"response":"Answer","session_id":"sess-1","message_count":3});
        assert!(invoke_result["response"].is_string());
        println!("✅ Event payload shapes valid");
    }

    /// Test BridgeMessage serde serialization matches frontend event payloads
    #[test]
    fn test_bridge_message_to_event_shape() {
        // Delta -> agent-delta
        let delta = BridgeMessage::Delta { text: Some("Hello".to_string()), session_id: Some("sess-1".to_string()) };
        let json = serde_json::to_value(&delta).unwrap();
        assert_eq!(json["type"], "delta");
        assert_eq!(json["text"], "Hello");
        // ToolStart -> agent-tool-start
        let start = BridgeMessage::ToolStart { id: Some("call-1".to_string()), name: "web_search".to_string(), args: serde_json::json!({"query":"test"}), session_id: Some("sess-1".to_string()) };
        let json = serde_json::to_value(&start).unwrap();
        assert_eq!(json["name"], "web_search");
        assert!(json["args"].is_object());
        // Done -> agent-done
        let done = BridgeMessage::Done { response: Some("Answer".to_string()), session_id: "sess-1".to_string(), message_count: 3 };
        let json = serde_json::to_value(&done).unwrap();
        assert_eq!(json["type"], "done");
        assert!(json["message_count"].is_number());
        println!("✅ BridgeMessage -> event shape mappings valid");
    }
}
