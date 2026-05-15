//! Agent Chat Bridge - communicate with AI Agent via Python bridge script
//!
//! Uses stdin/stdout JSON protocol for bidirectional communication.
//! Supports streaming text deltas via Tauri events.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// Global process state - use lazy_static for HashMap initialization
lazy_static::lazy_static! {
    static ref PROCESS_COUNTER: AtomicU64 = AtomicU64::new(0);
    static ref PROCESSES: Mutex<HashMap<u64, Arc<Mutex<Option<Child>>>>> = Mutex::new(HashMap::new());
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
    Abort {},
}

fn default_limit() -> usize {
    20
}

/// Output message from Python bridge
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BridgeMessage {
    Delta { text: String },
    ToolStart { name: String, args: serde_json::Value },
    ToolComplete { name: String, result: String, duration_ms: f64 },
    Thinking { text: String },
    Done { response: String, session_id: String, message_count: usize },
    Error { message: String },
    Sessions { data: Vec<SessionInfo>, total: usize },
    Session { session_id: String, messages: Vec<MessageInfo> },
    Deleted { session_id: String },
    Aborted { session_id: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "title", alias = "title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "source")]
    pub source: String,
    #[serde(rename = "startedAt", alias = "started_at", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<f64>,
    #[serde(rename = "endedAt", alias = "ended_at", skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<f64>,
    #[serde(rename = "messageCount", alias = "message_count")]
    pub message_count: usize,
    #[serde(rename = "preview")]
    pub preview: String,
    #[serde(rename = "lastActive", alias = "last_active", skip_serializing_if = "Option::is_none")]
    pub last_active: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "timestamp", alias = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    #[serde(rename = "toolName", alias = "tool_name", skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

/// Find Python bridge script location
fn find_bridge_script() -> Option<PathBuf> {
    // Try bundled location first (when packaged)
    let bundled = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("scripts").join("hermes_bridge.py")));
    
    if bundled.as_ref().map(|p| p.exists()).unwrap_or(false) {
        return bundled;
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
    // Try python3 first
    if Command::new("python3").arg("--version").output().is_ok() {
        return "python3".to_string();
    }
    // Fallback to python
    if Command::new("python").arg("--version").output().is_ok() {
        return "python".to_string();
    }
    // Default
    "python3".to_string()
}

/// Start a new bridge process
fn start_bridge_process() -> Result<(u64, Child, Arc<AtomicBool>), String> {
    let script = find_bridge_script()
        .ok_or_else(|| "Agent bridge script not found. Please ensure scripts/hermes_bridge.py exists.")?;

    let python = find_python();

    let child = Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python bridge: {}", e))?;

    let process_id = PROCESS_COUNTER.fetch_add(1, Ordering::SeqCst);
    let abort_flag = Arc::new(AtomicBool::new(false));

    // Store process reference
    {
        let mut processes = PROCESSES.lock().unwrap();
        processes.insert(process_id, Arc::new(Mutex::new(Some(child))));
    }
    {
        let mut flags = ABORT_FLAGS.lock().unwrap();
        flags.insert(process_id, abort_flag.clone());
    }

    // Get child back (hacky but works)
    let child = {
        let mut processes = PROCESSES.lock().unwrap();
        processes.get_mut(&process_id)
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
        let stdin = child.stdin.as_mut().ok_or_else(|| "stdin not available".to_string())?;
        stdin.write_all(cmd_json.as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    // Read streaming output
    let stdout = child.stdout.take().ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    let mut final_response: Option<String> = None;
    let mut final_session_id: Option<String> = None;
    let mut final_message_count: usize = 0;
    let mut accumulated_text = String::new();

    for line in reader.lines() {
        if abort_flag.load(Ordering::SeqCst) {
            break;
        }

        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse bridge message: {} - line: {}", e, line))?;

        match msg {
            BridgeMessage::Delta { text } => {
                accumulated_text.push_str(&text);
                // Emit event for frontend streaming display
                app.emit("agent-delta", &text).ok();
            }
            BridgeMessage::ToolStart { name, args } => {
                app.emit("agent-tool-start", serde_json::json!({
                    "name": name,
                    "args": args
                })).ok();
            }
            BridgeMessage::ToolComplete { name, result, duration_ms } => {
                app.emit("agent-tool-complete", serde_json::json!({
                    "name": name,
                    "result": result,
                    "duration_ms": duration_ms
                })).ok();
            }
            BridgeMessage::Thinking { text } => {
                app.emit("agent-thinking", &text).ok();
            }
            BridgeMessage::Done { response, session_id, message_count } => {
                final_response = Some(response);
                final_session_id = Some(session_id);
                final_message_count = message_count;
            }
            BridgeMessage::Error { message } => {
                app.emit("agent-error", &message).ok();
                return Err(message);
            }
            BridgeMessage::Aborted { .. } => {
                return Err("Chat aborted by user".to_string());
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

    // Return result
    Ok(serde_json::json!({
        "response": final_response.unwrap_or(accumulated_text),
        "session_id": final_session_id,
        "message_count": final_message_count,
    }))
}

/// List Agent sessions
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_list_sessions(limit: Option<usize>) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::ListSessions { limit: limit.unwrap_or(20) };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| "stdin not available".to_string())?;
        stdin.write_all(cmd_json.as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child.stdout.take().ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Sessions { data, total } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({
                "sessions": data,
                "total": total,
            }));
        } else if let BridgeMessage::Error { message } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Get session messages
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_get_session(session_id: String) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::GetSession { session_id };
    let cmd_json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| "stdin not available".to_string())?;
        stdin.write_all(cmd_json.as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child.stdout.take().ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Session { session_id, messages } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({
                "session_id": session_id,
                "messages": messages,
            }));
        } else if let BridgeMessage::Error { message } = msg {
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
        let stdin = child.stdin.as_mut().ok_or_else(|| "stdin not available".to_string())?;
        stdin.write_all(cmd_json.as_bytes()).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
    }

    let stdout = child.stdout.take().ok_or_else(|| "stdout not available".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.is_empty() {
            continue;
        }

        let msg: BridgeMessage = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse: {}", e))?;

        if let BridgeMessage::Deleted { session_id } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({ "deleted": session_id }));
        } else if let BridgeMessage::Error { message } = msg {
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

            // Also try to kill the process directly for immediate termination
            let process = {
                let processes = PROCESSES.lock().unwrap();
                processes.get(&pid).cloned()
            };

            if let Some(arc_child) = process {
                if let Some(mut child) = arc_child.lock().unwrap().take() {
                    // Kill the Python bridge process
                    child.kill().ok();
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
    let check_script = format!(r#"
import sys
sys.path.insert(0, "{}")
try:
    from run_agent import AIAgent
    print("OK")
except ImportError as e:
    print(f"ERROR: {{e}}")
"#, hermes_path);

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
            BridgeMessage::Delta { text } => assert_eq!(text, "Hello"),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_find_python() {
        let python = find_python();
        assert!(python == "python3" || python == "python");
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
            },
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
            BridgeMessage::Session { session_id, messages } => {
                assert_eq!(session_id, "sess-123");
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0].role, "user");
            },
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
            panic!("Failed to parse bridge response: {} - stdout: {}", e, stdout);
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
            },
            BridgeMessage::Error { message } => {
                // Hermes 未安装是可接受的错误
                if message.contains("Hermes not available") {
                    eprintln!("Skipping: Hermes not installed");
                } else {
                    panic!("Unexpected error: {}", message);
                }
            },
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
                println!("Check available result: {}", serde_json::to_string_pretty(&json).unwrap());
                // 验证返回格式
                assert!(json.get("available").is_some());
                assert!(json.get("python").is_some());
            },
            Err(e) => {
                // 路径问题可能导致错误，但不应该 crash
                eprintln!("Check available failed: {}", e);
            }
        }
    }
}