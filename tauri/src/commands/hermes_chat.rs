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
}

fn default_limit() -> usize {
    20
}

/// Output message from Python bridge
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BridgeMessage {
    Delta { text: Option<String> },
    ToolStart { name: String, args: serde_json::Value },
    ToolComplete { name: String, result: Option<String>, duration_ms: f64 },
    Thinking { text: Option<String> },
    Done { response: Option<String>, session_id: String, message_count: usize },
    Error { message: String },
    Sessions { data: Vec<SessionInfo>, total: usize },
    SearchResults { data: Vec<SearchResult>, total: usize, query: String },
    Session { session_id: String, messages: Vec<MessageInfo> },
    Deleted { session_id: String },
    Renamed { session_id: String, title: String },
    Aborted { session_id: Option<String> },
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
    #[serde(rename = "endedAt", alias = "ended_at", skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<f64>,
    #[serde(rename = "messageCount", alias = "message_count")]
    pub message_count: usize,
    #[serde(rename = "preview")]
    pub preview: String,
    #[serde(rename = "lastActive", alias = "last_active", skip_serializing_if = "Option::is_none")]
    pub last_active: Option<f64>,
}

/// Search result from FTS5 search
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "sessionId", alias = "session_id")]
    pub session_id: String,
    #[serde(rename = "sessionTitle", alias = "session_title", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "toolName", alias = "tool_name", skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(rename = "toolCallId", alias = "tool_call_id", skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(rename = "toolCalls", alias = "tool_calls", skip_serializing_if = "Option::is_none")]
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
    // 优先使用 Hermes Agent venv 的 Python（因为依赖都在 venv 里）
    let hermes_venv_python = dirs::home_dir()
        .map(|h| h.join(".hermes").join("hermes-agent").join("venv").join("bin").join("python3"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.hermes/hermes-agent/venv/bin/python3".to_string());
    
    // 检查 venv Python 是否存在且可执行
    if Path::new(&hermes_venv_python).exists() {
        if Command::new(&hermes_venv_python).arg("--version").output().is_ok() {
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
                if let Some(t) = &text {
                    accumulated_text.push_str(t);
                }
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
                final_response = response;
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

/// Rename session
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_rename_session(session_id: String, title: String) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::RenameSession { session_id, title };
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

        if let BridgeMessage::Renamed { session_id, title } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({ "session_id": session_id, "title": title }));
        } else if let BridgeMessage::Error { message } = msg {
            child.wait().ok();
            return Err(message);
        }
    }

    child.wait().ok();
    Err("No response from bridge".to_string())
}

/// Search sessions by content
#[tauri::command(rename_all = "camelCase")]
pub async fn agent_search_sessions(query: String, limit: Option<usize>, offset: Option<usize>) -> Result<serde_json::Value, String> {
    let (_, mut child, _) = start_bridge_process()?;

    let cmd = BridgeCommand::SearchSessions {
        query,
        limit: limit.unwrap_or(20),
        offset: offset.unwrap_or(0),
    };
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

        if let BridgeMessage::SearchResults { data, total, query } = msg {
            child.wait().ok();
            return Ok(serde_json::json!({
                "results": data,
                "total": total,
                "query": query,
            }));
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
            },
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_complete_message() {
        let json = r#"{"type":"tool_complete","name":"terminal","result":"file1.txt\nfile2.txt","duration_ms":150}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete { name, result, duration_ms } => {
                assert_eq!(name, "terminal");
                assert_eq!(result, Some("file1.txt\nfile2.txt".to_string()));
                assert_eq!(duration_ms, 150.0);
            },
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_tool_complete_null_result() {
        let json = r#"{"type":"tool_complete","name":"terminal","result":null,"duration_ms":150}"#;
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete { name, result, duration_ms } => {
                assert_eq!(name, "terminal");
                assert!(result.is_none());
                assert_eq!(duration_ms, 150.0);
            },
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
        
        let msg: BridgeMessage = serde_json::from_str(last_line)
            .unwrap_or_else(|e| panic!("Failed to parse last line: {} - content: {}", e, last_line));
        
        match msg {
            BridgeMessage::Done { response, session_id, message_count } => {
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
                    },
                    Some(_) => {
                        println!("  Empty response - API may have partially failed");
                    },
                    None => {
                        println!("  No response - API call failed");
                        // 这是预期的错误情况（模型不支持等）
                    },
                }
            },
            BridgeMessage::Error { message } => {
                // API key 问题等可以接受
                if message.contains("API key") || message.contains("authentication") || message.contains("rate limit") {
                    eprintln!("Skipping: API error - {}", message);
                } else {
                    panic!("Chat error: {}", message);
                }
            },
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
            BridgeMessage::Done { response, session_id, message_count } => {
                assert_eq!(response, Some("Hello".to_string()));
                assert_eq!(session_id, "sess-abc");
                assert_eq!(message_count, 3);
            },
            _ => panic!("Wrong type"),
        }
        
        // 无响应（API 失败）
        let json_null = "{\"type\":\"done\",\"response\":null,\"session_id\":\"sess-def\",\"message_count\":1}";
        let msg_null: BridgeMessage = serde_json::from_str(json_null).unwrap();
        match msg_null {
            BridgeMessage::Done { response, .. } => {
                assert_eq!(response, None);
            },
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::Error 解析
    #[test]
    fn test_error_message_deserialize() {
        let json = "{\"type\":\"error\",\"message\":\"Something went wrong\"}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::Error { message } => {
                assert_eq!(message, "Something went wrong");
            },
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
            },
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::ToolStart 解析
    #[test]
    fn test_tool_start_deserialize() {
        let json = "{\"type\":\"tool_start\",\"name\":\"web_search\",\"args\":{\"query\":\"test\"}}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolStart { name, args } => {
                assert_eq!(name, "web_search");
                assert!(args.is_object());
            },
            _ => panic!("Wrong type"),
        }
    }

    /// 测试 BridgeMessage::ToolComplete 解析
    #[test]
    fn test_tool_complete_deserialize() {
        let json = "{\"type\":\"tool_complete\",\"name\":\"web_search\",\"result\":\"Search results\",\"duration_ms\":150.5}";
        let msg: BridgeMessage = serde_json::from_str(json).unwrap();
        match msg {
            BridgeMessage::ToolComplete { name, result, duration_ms } => {
                assert_eq!(name, "web_search");
                assert_eq!(result, Some("Search results".to_string()));
                assert_eq!(duration_ms, 150.5);
            },
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
            },
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
            },
            _ => panic!("Wrong type"),
        }
    }
}