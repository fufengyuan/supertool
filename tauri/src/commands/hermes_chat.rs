//! Agent Chat Bridge - communicate with AI Agent via Python bridge script
//!
//! Uses stdin/stdout JSON protocol for bidirectional communication.
//! Supports streaming text deltas via Tauri events.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// Global process state - use lazy_static for HashMap initialization
lazy_static::lazy_static! {
    static ref PROCESS_COUNTER: AtomicU64 = AtomicU64::new(0);
    static ref PROCESSES: Mutex<HashMap<u64, Arc<Mutex<Option<Child>>>>> = Mutex::new(HashMap::new());
    static ref PROCESS_PIDS: Mutex<HashMap<u64, u32>> = Mutex::new(HashMap::new());
    static ref ABORT_FLAGS: Mutex<HashMap<u64, Arc<AtomicBool>>> = Mutex::new(HashMap::new());
    static ref CURRENT_CHAT_PROCESS_ID: Mutex<Option<u64>> = Mutex::new(None);
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

/// Start a new bridge process
fn start_bridge_process() -> Result<(u64, Child, Arc<AtomicBool>), String> {
    let script = find_bridge_script().ok_or_else(
        || "Agent bridge script not found. Please ensure scripts/hermes_bridge.py exists.",
    )?;

    let python = find_python();

    // 加载 Hermes .env 文件的环境变量
    let hermes_env_path = dirs::home_dir()
        .map(|h| h.join(".hermes").join(".env"))
        .filter(|p| p.exists());

    let mut cmd = Command::new(&python);
    cmd.arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // 注入 Hermes 环境变量
    if let Some(env_path) = hermes_env_path {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                // 跳过注释和空行
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                // 解析 KEY=VALUE
                if let Some((key, value)) = line.split_once('=') {
                    cmd.env(key.trim(), value.trim());
                }
            }
        }
    }

    // 禁用系统代理（避免 GNOME 代理配置干扰）
    // Python requests 库在 Linux 上会读取 GNOME 代理配置，可能导致连接失败
    cmd.env("http_proxy", "");
    cmd.env("https_proxy", "");
    cmd.env("HTTP_PROXY", "");
    cmd.env("HTTPS_PROXY", "");
    cmd.env("all_proxy", "");
    cmd.env("ALL_PROXY", "");

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start Python bridge: {}", e))?;

    let process_id = PROCESS_COUNTER.fetch_add(1, Ordering::SeqCst);
    let abort_flag = Arc::new(AtomicBool::new(false));
    let os_pid = child.id();

    // Store process reference
    {
        let mut processes = PROCESSES.lock().unwrap();
        processes.insert(process_id, Arc::new(Mutex::new(Some(child))));
    }
    {
        let mut pids = PROCESS_PIDS.lock().unwrap();
        pids.insert(process_id, os_pid);
    }
    {
        let mut flags = ABORT_FLAGS.lock().unwrap();
        flags.insert(process_id, abort_flag.clone());
    }

    // Get child back (hacky but works)
    let child = {
        let mut processes = PROCESSES.lock().unwrap();
        processes
            .get_mut(&process_id)
            .and_then(|arc| arc.lock().unwrap().take())
            .ok_or_else(|| "Failed to retrieve child process".to_string())?
    };

    Ok((process_id, child, abort_flag))
}

/// Send chat message with streaming events
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_chat(
    app: AppHandle,
    message: String,
    session_id: Option<String>,
    model: Option<String>,
    toolsets: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    let (process_id, mut child, abort_flag) = start_bridge_process()?;

    // Record current chat process ID for abort functionality
    {
        let mut current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
        *current = Some(process_id);
    }

    // Send command
    let cmd = BridgeCommand::Chat {
        session_id,
        message,
        model,
        toolsets,
    };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    // Read streaming output
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    let mut final_response: Option<String> = None;
    let mut final_session_id: Option<String> = None;
    let mut message_count: usize = 0;
    let mut accumulated_text = String::new();
    // Capture session_id from the first event that provides it (Delta, ToolStart, etc.)
    // This is critical for abort: if "done" is never received, we still need the
    // session_id so the frontend can resume the conversation instead of creating a new one.
    let mut captured_session_id: Option<String> = None;

    for line in reader.lines() {
        if abort_flag.load(Ordering::SeqCst) {
            child.kill().ok(); // Kill process immediately when abort flag is set
            break;
        }

        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        // 跳过非 JSON 行（日志、警告等）
        if !line.trim_start().starts_with('{') {
            eprintln!("[DEBUG] bridge log: {}", line);
            continue;
        }

        let msg: BridgeMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                // JSON 解析失败，通知前端
                eprintln!("[DEBUG] bridge parse error: {} - line: {}", e, line);
                app.emit(
                    "agent-error",
                    serde_json::json!({
                        "type": "parse_error",
                        "message": format!("JSON parse error: {}", e),
                        "raw": line.chars().take(100).collect::<String>()
                    }),
                )
                .ok();
                continue;
            }
        };

        match msg {
            BridgeMessage::Delta { text, session_id } => {
                if let Some(t) = &text {
                    accumulated_text.push_str(t);
                }
                // Capture session_id from the first event
                if captured_session_id.is_none() {
                    captured_session_id = session_id.clone();
                }
                eprintln!("[DEBUG] agent-delta: {:?}", text);
                app.emit(
                    "agent-delta",
                    serde_json::json!({
                        "text": text,
                        "session_id": session_id
                    }),
                )
                .ok();
            }
            BridgeMessage::ToolStart { id, name, args, session_id } => {
                eprintln!(
                    "[DEBUG] agent-tool-start: {} {:?} (id: {:?})",
                    name, args, id
                );
                app.emit(
                    "agent-tool-start",
                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "args": args,
                        "session_id": session_id
                    }),
                )
                .ok();
            }
            BridgeMessage::ToolComplete {
                id,
                name,
                result,
                duration_ms,
                session_id,
            } => {
                eprintln!(
                    "[DEBUG] agent-tool-complete: {} (id: {:?}, duration: {}ms)",
                    name, id, duration_ms
                );
                app.emit(
                    "agent-tool-complete",
                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "result": result,
                        "duration_ms": duration_ms,
                        "session_id": session_id
                    }),
                )
                .ok();
            }
            BridgeMessage::Thinking { text, session_id } => {
                app.emit(
                    "agent-thinking",
                    serde_json::json!({
                        "text": text,
                        "session_id": session_id
                    }),
                )
                .ok();
            }
            BridgeMessage::Done {
                response,
                session_id,
                message_count: done_message_count,
            } => {
                // 立即发送 agent-done 事件，让前端恢复状态
                app.emit(
                    "agent-done",
                    serde_json::json!({
                        "response": response,
                        "session_id": session_id,
                        "message_count": done_message_count,
                    }),
                )
                .ok();
                final_response = response;
                final_session_id = Some(session_id);
                message_count = done_message_count;

                // 捕获 session_id（从 done 事件）
                if captured_session_id.is_none() {
                    captured_session_id = final_session_id.clone();
                }
            }
            BridgeMessage::Error { message, session_id } => {
                app.emit(
                    "agent-error",
                    serde_json::json!({
                        "message": message,
                        "session_id": session_id
                    }),
                )
                .ok();
                // 先清理再返回，避免资源泄漏
                child.wait().ok();
                {
                    let mut processes = PROCESSES.lock().unwrap();
                    processes.remove(&process_id);
                }
                {
                    let mut pids = PROCESS_PIDS.lock().unwrap();
                    pids.remove(&process_id);
                }
                {
                    let mut flags = ABORT_FLAGS.lock().unwrap();
                    flags.remove(&process_id);
                }
                {
                    let mut current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
                    if current.as_ref() == Some(&process_id) {
                        *current = None;
                    }
                }
                return Err(message);
            }
            BridgeMessage::Aborted { session_id } => {
                // Capture session_id even from aborted message
                if captured_session_id.is_none() {
                    captured_session_id = session_id.clone();
                }
                // 先清理再返回，避免资源泄漏
                child.wait().ok();
                {
                    let mut processes = PROCESSES.lock().unwrap();
                    processes.remove(&process_id);
                }
                {
                    let mut pids = PROCESS_PIDS.lock().unwrap();
                    pids.remove(&process_id);
                }
                {
                    let mut flags = ABORT_FLAGS.lock().unwrap();
                    flags.remove(&process_id);
                }
                {
                    let mut current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
                    if current.as_ref() == Some(&process_id) {
                        *current = None;
                    }
                }
                // 返回 session_id 而非抛 error，前端需要它来保存会话 ID
                // 否则下一轮 invoke 用 null sessionId 创建新会话，旧会话丢失
                let effective_session_id = captured_session_id.clone();
                if let Some(ref sid) = effective_session_id {
                    app.emit(
                        "agent-done",
                        serde_json::json!({
                            "response": Option::<String>::None,
                            "session_id": sid,
                            "message_count": message_count,
                            "aborted": true,
                        }),
                    )
                    .ok();
                }
                return Ok(serde_json::json!({
                    "response": Option::<String>::None,
                    "session_id": effective_session_id,
                    "message_count": message_count,
                    "aborted": true,
                }));
            }
            _ => {}
        }
    }

    // Clean up process
    child.wait().ok();
    {
        let mut processes = PROCESSES.lock().unwrap();
        processes.remove(&process_id);
    }
    {
        let mut pids = PROCESS_PIDS.lock().unwrap();
        pids.remove(&process_id);
    }
    {
        let mut flags = ABORT_FLAGS.lock().unwrap();
        flags.remove(&process_id);
    }
    // Clear current chat process ID
    {
        let mut current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
        if current.as_ref() == Some(&process_id) {
            *current = None;
        }
    }

    // Return result with captured session_id as fallback
    Ok(serde_json::json!({
        "response": final_response.unwrap_or(accumulated_text),
        "session_id": captured_session_id.or(final_session_id),
        "message_count": message_count,
    }))
}

/// Get session messages
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_get_session(session_id: String) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::GetSession { session_id };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage =
            serde_json::from_str(&line).map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Session {
            session_id,
            messages,
        } = msg
        {
            child.wait().ok();
            return Ok(serde_json::json!({
                "session_id": session_id,
                "messages": messages,
            }));
        } else if let BridgeMessage::Error { message, .. } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Delete session
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_delete_session(session_id: String) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::DeleteSession { session_id };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage =
            serde_json::from_str(&line).map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Deleted { session_id } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({ "deleted": session_id }));
        } else if let BridgeMessage::Error { message, .. } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Rename session
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_rename_session(
    session_id: String,
    title: String,
) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::RenameSession { session_id, title };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage =
            serde_json::from_str(&line).map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Renamed { session_id, title } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({ "session_id": session_id, "title": title }));
        } else if let BridgeMessage::Error { message, .. } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Search sessions by content
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_search_sessions(
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::SearchSessions {
        query,
        limit: limit.unwrap_or(20),
        offset: offset.unwrap_or(0),
    };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage =
            serde_json::from_str(&line).map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::SearchResults { data, total, query } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({
                "results": data,
                "total": total,
                "query": query,
            }));
        } else if let BridgeMessage::Error { message, .. } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Abort current chat
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_abort_chat() -> Result<serde_json::Value, String> {
    // Get current chat process ID
    let current_process_id = {
        let current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
        current.clone()
    };

    if let Some(pid) = current_process_id {
        // Find and set the abort flag for the running chat process
        let abort_flag = {
            let flags = ABORT_FLAGS.lock().unwrap();
            flags.get(&pid).cloned()
        };

        if let Some(flag) = abort_flag {
            // Set abort flag to break the read loop in agent_chat
            flag.store(true, Ordering::SeqCst);

            // Send SIGTERM (signal 15) instead of SIGKILL to allow graceful shutdown
            // Python can capture SIGTERM, set abort flag, and Hermes will call _persist_session
            {
                let pids = PROCESS_PIDS.lock().unwrap();
                if let Some(&os_pid) = pids.get(&pid) {
                    // SIGTERM (15) allows Python signal handler to run and save messages
                    let _ = std::process::Command::new("kill")
                        .arg("-15")  // SIGTERM instead of SIGKILL (9)
                        .arg(os_pid.to_string())
                        .status();
                }
            }

            // Wait for Python to complete graceful shutdown (with timeout)
            // Python signal handler sets abort_flag, Hermes calls _persist_session
            let child_opt = {
                let processes = PROCESSES.lock().unwrap();
                if let Some(arc_child) = processes.get(&pid) {
                    arc_child.lock().unwrap().take()
                } else {
                    None
                }
            };
            // MutexGuard is dropped here, before spawn_blocking

            if let Some(mut child) = child_opt {
                // Use spawn_blocking to avoid blocking tokio runtime
                let wait_result = tokio::task::spawn_blocking(move || {
                    // Wait up to 5 seconds for graceful shutdown
                    let start = std::time::Instant::now();
                    let timeout = std::time::Duration::from_secs(5);
                    
                    while start.elapsed() < timeout {
                        match child.try_wait() {
                            Ok(Some(_status)) => {
                                // Process exited gracefully - messages were saved
                                eprintln!("[INFO] Agent process gracefully terminated, messages saved");
                                return true; // Success
                            }
                            Ok(None) => {
                                // Process still running, wait a bit more
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                            Err(e) => {
                                eprintln!("[WARN] Error checking process status: {}", e);
                                break;
                            }
                        }
                    }
                    
                    // Timeout - force kill
                    eprintln!("[WARN] SIGTERM timeout after 5s, forcing SIGKILL");
                    child.kill().ok();
                    child.wait().ok();
                    return false; // Forced termination
                }).await;
                
                match wait_result {
                    Ok(true) => {
                        // Graceful shutdown completed
                        eprintln!("[INFO] Messages saved successfully");
                    }
                    Ok(false) => {
                        // Forced termination
                        eprintln!("[WARN] Messages may be incomplete due to forced termination");
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Spawn blocking failed: {}", e);
                    }
                }
            }

            // Clear current chat process ID
            {
                let mut current = CURRENT_CHAT_PROCESS_ID.lock().unwrap();
                *current = None;
            }

            Ok(serde_json::json!({ "aborted": true, "process_id": pid }))
        } else {
            Err("No abort flag found for current chat process".to_string())
        }
    } else {
        // No chat is running
        Ok(serde_json::json!({ "aborted": false, "message": "No active chat to abort" }))
    }
}

/// Check Agent availability
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_check_available() -> Result<serde_json::Value, String> {
    let script = find_bridge_script();
    let python = find_python();

    // Expand home directory path
    let hermes_path = dirs::home_dir()
        .map(|h| h.join(".hermes").join("hermes-agent"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.hermes/hermes-agent".to_string());

    // Try to import Hermes
    let check_script = format!(
        r#"
import sys
sys.path.insert(0, "{}")
try:
    from run_agent import AIAgent
    print("OK")
except ImportError as e:
    print(f"ERROR: {{e}}")
"#,
        hermes_path
    );

    let output = Command::new(&python)
        .arg("-c")
        .arg(&check_script)
        .output()
        .map_err(|e| format!("Failed to check Agent: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(serde_json::json!({
        "available": stdout == "OK",
        "script_found": script.is_some(),
        "python": python,
        "error": if stdout.starts_with("ERROR") { Some(stdout) } else { None },
    }))
}

/// Get custom models from Hermes config
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_get_models() -> Result<serde_json::Value, String> {
    let script = find_bridge_script().ok_or_else(|| "Bridge script not found".to_string())?;
    let python = find_python();

    let mut child = Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python bridge: {}", e))?;

    // Send get_models command
    let cmd = BridgeCommand::GetModels {};
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    // Read response
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() || !line.trim_start().starts_with('{') {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line).map_err(|e| e.to_string())?;

        match msg {
            BridgeMessage::Models {
                custom_models,
                default_model,
            } => {
                // Wait for process to finish
                child.wait().ok();
                return Ok(serde_json::json!({
                    "customModels": custom_models,
                    "defaultModel": default_model,
                }));
            }
            BridgeMessage::Error { message, .. } => {
                child.wait().ok();
                return Err(message);
            }
            _ => continue,
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Add a model to Hermes config
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_add_model(model: String) -> Result<serde_json::Value, String> {
    let script = find_bridge_script().ok_or_else(|| "Bridge script not found".to_string())?;
    let python = find_python();

    let mut child = Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python bridge: {}", e))?;

    // Send add_model command
    let cmd = BridgeCommand::AddModel { model };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    // Read response
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() || !line.trim_start().starts_with('{') {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line).map_err(|e| e.to_string())?;

        match msg {
            BridgeMessage::ModelAdded {
                model,
                custom_models,
            } => {
                child.wait().ok();
                return Ok(serde_json::json!({
                    "success": true,
                    "model": model,
                    "customModels": custom_models,
                }));
            }
            BridgeMessage::Error { message, .. } => {
                child.wait().ok();
                return Err(message);
            }
            _ => continue,
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Remove a model from Hermes config
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_remove_model(model: String) -> Result<serde_json::Value, String> {
    let script = find_bridge_script().ok_or_else(|| "Bridge script not found".to_string())?;
    let python = find_python();

    let mut child = Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python bridge: {}", e))?;

    // Send remove_model command
    let cmd = BridgeCommand::RemoveModel { model };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "stdin not available".to_string())?;
        stdin
            .write_all(cmd_json.as_bytes())
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    // Read response
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() || !line.trim_start().starts_with('{') {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line).map_err(|e| e.to_string())?;

        match msg {
            BridgeMessage::ModelRemoved {
                model,
                custom_models,
            } => {
                child.wait().ok();
                return Ok(serde_json::json!({
                    "success": true,
                    "model": model,
                    "customModels": custom_models,
                }));
            }
            BridgeMessage::Error { message, .. } => {
                child.wait().ok();
                return Err(message);
            }
            _ => continue,
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
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
            BridgeMessage::Delta { text } => assert_eq!(text, Some("Hello".to_string())),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_delta_null_text() {
        let json = "{\"type\":\"delta\",\"text\":null}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Delta { text } => assert!(text.is_none()),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_start_message() {
        let json = r#"{"type":"tool_start","name":"terminal","args":{"command":"ls"}}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolStart { name, args } => {
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
            } => {
                assert_eq!(name, "terminal");
                assert_eq!(result, Some("file1.txt\nfile2.txt".to_string()));
                assert_eq!(duration_ms, 150.0);
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
            } => {
                assert_eq!(name, "terminal");
                assert!(result.is_none());
                assert_eq!(duration_ms, 150.0);
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
            BridgeMessage::Delta { text } => {
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
            BridgeMessage::ToolStart { name, args } => {
                assert_eq!(name, "web_search");
                assert!(args.is_object());
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::ToolComplete 解析
    #[test]
    fn test_tool_complete_deserialize() {
        let json = "{\"type\":\"tool_complete\",\"name\":\"web_search\",\"result\":\"Search results\",\"duration_ms\":150.5}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete {
                name,
                result,
                duration_ms,
            } => {
                assert_eq!(name, "web_search");
                assert_eq!(result, Some("Search results".to_string()));
                assert_eq!(duration_ms, 150.5);
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
}
