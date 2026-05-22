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

/// Create a reqwest client that bypasses system proxy for localhost requests.
/// Prevents VPN/proxy tools (ClashX, V2Ray, etc.) from buffering NDJSON streams.
fn local_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("Failed to build reqwest client")
}

/// Kill any process listening on the given port.
/// Uses lsof (macOS/Linux) to find the PID, then kills it.
fn kill_process_on_port(port: &str) {
    let output = std::process::Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pid_str.is_empty() {
                // Kill PIDs using kill command (safe, no unsafe code)
                for pid in pid_str.lines() {
                    let pid = pid.trim();
                    if pid.is_empty() { continue; }
                    let _ = std::process::Command::new("kill")
                        .args([pid])
                        .output();
                }
                // Give processes a moment to die
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
    }
}

/// Find hermes_chat_server.py script location
fn find_chat_server_script() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_parent = exe.parent()?;
    
    // Check if running inside macOS .app bundle
    if exe_parent.ends_with("MacOS") {
        let contents_dir = exe_parent.parent()?;
        let resources_dir = contents_dir.join("Resources");
        
        // Tauri 2.x: resources from "../scripts/..." are stored under _up_/
        let script_path = resources_dir.join("_up_").join("scripts").join("hermes_chat_server.py");
        if script_path.exists() {
            return Some(script_path);
        }
        
        let flat_path = resources_dir.join("scripts").join("hermes_chat_server.py");
        if flat_path.exists() {
            return Some(flat_path);
        }
        
        let direct_path = resources_dir.join("hermes_chat_server.py");
        if direct_path.exists() {
            return Some(direct_path);
        }
    }
    
    // Generic bundled location: exe_parent/scripts/
    let bundled = exe_parent.join("scripts").join("hermes_chat_server.py");
    if bundled.exists() {
        return Some(bundled);
    }
    
    // Linux/Windows: try exe_parent/../resources/scripts/ or _up_/scripts/
    if let Some(parent) = exe_parent.parent() {
        let up_path = parent.join("_up_").join("scripts").join("hermes_chat_server.py");
        if up_path.exists() {
            return Some(up_path);
        }
        let resources_path = parent.join("resources").join("scripts").join("hermes_chat_server.py");
        if resources_path.exists() {
            return Some(resources_path);
        }
    }

    // Try development location
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| Some(p.join("scripts").join("hermes_chat_server.py")));

    if dev.as_ref().map(|p| p.exists()).unwrap_or(false) {
        return dev;
    }

    // Fallback: try relative to working directory
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("scripts").join("hermes_chat_server.py"))
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
    // Kill any existing process on the port to ensure we get a fresh server with latest code
    kill_process_on_port(HERMES_CHAT_SERVER_PORT);
    
    let script = find_chat_server_script()
        .ok_or_else(|| "Agent chat server script not found.".to_string())?;
    let python = find_python();
    let child = Command::new(&python).arg(&script)
        .env("HERMES_CHAT_PORT", HERMES_CHAT_SERVER_PORT)
        .env("HERMES_CHAT_HOST", "127.0.0.1")
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start server: {}", e))?;
    { let mut s = SERVER_PROCESS.lock().unwrap(); *s = Some(child); }
    let client = local_client();
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
    let client = local_client();
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
                        if captured_session_id.is_none() {
                            captured_session_id = session_id.clone();
                        }
                        app.emit("agent-delta", serde_json::json!({
                            "text": t, "session_id": session_id,
                        })).ok();
                    }
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
        let client = local_client();
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

/// Clear cached agent for a session (called when switching models)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_clear_cache(session_id: String) -> Result<serde_json::Value, String> {
    ensure_server_running().await?;

    let client = local_client();
    let resp = client
        .post(format!("{}/v1/clear_cache", HERMES_CHAT_SERVER_URL))
        .json(&serde_json::json!({"session_id": session_id}))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Clear cache request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Clear cache server error ({}): {}", status, text));
    }

    Ok(serde_json::json!({"ok": true, "session_id": session_id}))
}

/// Check Agent availability (pure Rust, no Python bridge)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_check_available() -> Result<serde_json::Value, String> {
    let available = crate::commands::hermes_config::hermes_is_installed();
    let script_found = find_chat_server_script().is_some();
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

/// Set the default model in Hermes config (persists to config.yaml)
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_set_model(model: String) -> Result<serde_json::Value, String> {
    crate::commands::hermes_config::set_default_model(model)
}
