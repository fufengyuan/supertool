//! ACP (Agent Client Protocol) client for omp subprocess communication.
//!
//! NDJSON over stdin/stdout:
//!   Request:  `{"id":N,"method":"M","params":P}\n`
//!   Response: `{"id":N,"result":R}\n` | `{"id":N,"error":{...}}\n`
//!   Push:     `{"method":"session/notification","params":...}\n`

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, broadcast, oneshot};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum AcpError {
    Io(String),
    Protocol(String),
    Rpc(String),
    Timeout,
}

impl std::fmt::Display for AcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcpError::Io(m) => write!(f, "IO: {m}"),
            AcpError::Protocol(m) => write!(f, "Protocol: {m}"),
            AcpError::Rpc(m) => write!(f, "RPC: {m}"),
            AcpError::Timeout => write!(f, "Timeout"),
        }
    }
}

impl std::error::Error for AcpError {}

impl From<std::io::Error> for AcpError {
    fn from(e: std::io::Error) -> Self { AcpError::Io(e.to_string()) }
}

const ACP_PROTOCOL_VERSION: &str = "2025-03-26";

// ---------------------------------------------------------------------------
// Notification types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum AcpSessionUpdate {
    MessageChunk(String),
    ThoughtChunk(String),
    UserMessageChunk(String),
    ToolCall { id: String, name: String, raw_input: serde_json::Value },
    ToolCallUpdate { id: String, status: String, raw_output: serde_json::Value },
    ToolCallResult { id: String, content: String, is_error: bool },
    PlanUpdate(serde_json::Value),
    ConfigOptionUpdate,
    ModeUpdate,
}

#[derive(Clone, Debug)]
pub struct AcpNotification {
    pub session_id: String,
    pub update: AcpSessionUpdate,
}

#[derive(Clone, Debug)]
pub struct AcpPromptResponse {
    pub message_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

// ---------------------------------------------------------------------------
// NDJSON parser
// ---------------------------------------------------------------------------

fn parse_notif(json: &serde_json::Value) -> Result<AcpNotification, AcpError> {
    let params = json.get("params").ok_or_else(|| AcpError::Protocol("no params".into()))?;
    let sid = params.get("sessionId").and_then(|v| v.as_str()).ok_or_else(|| AcpError::Protocol("no sessionId".into()))?.to_string();
    let u = params.get("update").ok_or_else(|| AcpError::Protocol("no update".into()))?;
    let kind = u.get("sessionUpdate").and_then(|v| v.as_str()).unwrap_or("");
    fn txt(v: &serde_json::Value) -> String {
        v.get("content").and_then(|c| c.get("text")).and_then(|t| t.as_str()).unwrap_or("").to_string()
    }
    let up = match kind {
        "agent_message_chunk" => AcpSessionUpdate::MessageChunk(txt(u)),
        "agent_thought_chunk" => AcpSessionUpdate::ThoughtChunk(txt(u)),
        "user_message_chunk" => AcpSessionUpdate::UserMessageChunk(txt(u)),
        "tool_call" => AcpSessionUpdate::ToolCall {
            id: u.get("toolCallId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: u.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            raw_input: u.get("rawInput").cloned().unwrap_or(serde_json::Value::Null),
        },
        "tool_call_update" => {
            let id = u.get("toolCallId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let status = u.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let raw = u.get("rawOutput").cloned().unwrap_or(serde_json::Value::Null);
            match status.as_str() {
                "completed" | "failed" => AcpSessionUpdate::ToolCallResult { id, content: raw.as_str().unwrap_or("").to_string(), is_error: status == "failed" },
                _ => AcpSessionUpdate::ToolCallUpdate { id, status, raw_output: raw },
            }
        }
        "config_option_update" => AcpSessionUpdate::ConfigOptionUpdate,
        "mode_update" => AcpSessionUpdate::ModeUpdate,
        "plan" => AcpSessionUpdate::PlanUpdate(u.clone()),
        _ => return Err(AcpError::Protocol(format!("unknown update {kind}"))),
    };
    Ok(AcpNotification { session_id: sid, update: up })
}

// ---------------------------------------------------------------------------
// AcpClient
// ---------------------------------------------------------------------------

pub struct AcpClient {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    notif_tx: broadcast::Sender<AcpNotification>,
    next_id: AtomicU64,
    _child: Option<Child>,
}

impl AcpClient {
    /// 启动 `omp acp` 子进程
    pub async fn spawn(omp_path: &str, cwd: Option<&str>) -> Result<Self, AcpError> {
        let mut cmd = Command::new(omp_path);
        cmd.arg("acp");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.stdin(std::process::Stdio::piped());
        if let Some(dir) = cwd { cmd.current_dir(dir); }
        let mut child = cmd.spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| AcpError::Io("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| AcpError::Io("no stdout".into()))?;
        let stderr = child.stderr.take().ok_or_else(|| AcpError::Io("no stderr".into()))?;

        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (notif_tx, _) = broadcast::channel(1024);
        let nt = notif_tx.clone();
        let p = pending.clone();

        // stdout reader → 分派响应和通知
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let t = line.trim();
                        if t.is_empty() { continue; }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(t) {
                            if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
                                let mut map = p.lock().await;
                                if let Some(s) = map.remove(&id) { let _ = s.send(json); }
                            } else if let Ok(n) = parse_notif(&json) {
                                let _ = nt.send(n);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // stderr → log
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => log::info!("[omp acp] {}", line.trim()),
                    Err(_) => break,
                }
            }
        });

        Ok(Self { stdin: Arc::new(Mutex::new(stdin)), pending, notif_tx, next_id: AtomicU64::new(1), _child: Some(child) })
    }

    /// 返回通知广播接收器
    pub fn subscribe(&self) -> broadcast::Receiver<AcpNotification> {
        self.notif_tx.subscribe()
    }

    /// RPC 调用
    async fn call_raw(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, AcpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        { let mut m = self.pending.lock().await; m.insert(id, tx); }
        let req = serde_json::json!({"id": id, "method": method, "params": params});
        {
            let mut s = self.stdin.lock().await;
            let line = serde_json::to_string(&req).map_err(|e| AcpError::Protocol(e.to_string()))?;
            s.write_all(line.as_bytes()).await?;
            s.write_all(b"\n").await?;
            s.flush().await?;
        }
        let resp = rx.await.map_err(|_| AcpError::Timeout)?;
        if let Some(err) = resp.get("error") {
            let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
            return Err(AcpError::Rpc(msg.to_string()));
        }
        resp.get("result").cloned().ok_or_else(|| AcpError::Protocol("no result".into()))
    }

    pub async fn initialize(&self) -> Result<(), AcpError> {
        self.call_raw("initialize", serde_json::json!({
            "protocolVersion": ACP_PROTOCOL_VERSION,
            "clientInfo": { "name": "supertool", "version": "1.0" },
            "clientCapabilities": {},
        })).await?;
        Ok(())
    }

    pub async fn authenticate(&self) -> Result<(), AcpError> {
        self.call_raw("authenticate", serde_json::json!({"methodId": "agent"})).await?;
        Ok(())
    }

    pub async fn new_session(&self, cwd: &str) -> Result<String, AcpError> {
        let r = self.call_raw("session/new", serde_json::json!({"cwd": cwd, "mcpServers": []})).await?;
        r.get("sessionId").and_then(|v| v.as_str()).map(String::from).ok_or_else(|| AcpError::Protocol("no sessionId".into()))
    }

    /// 发送 prompt（通知通过 broadcast channel 流式到达）
    pub async fn prompt(&self, session_id: &str, text: &str) -> Result<AcpPromptResponse, AcpError> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let r = self.call_raw("session/prompt", serde_json::json!({
            "sessionId": session_id,
            "messageId": msg_id,
            "prompt": [{ "type": "text", "text": text }],
        })).await?;
        let mid = r.get("userMessageId").and_then(|v| v.as_str()).unwrap_or(&msg_id).to_string();
        let usage = r.get("usage").unwrap_or(&serde_json::Value::Null);
        Ok(AcpPromptResponse {
            message_id: mid,
            input_tokens: usage.get("inputTokens").and_then(|v| v.as_u64()).unwrap_or(0),
            output_tokens: usage.get("outputTokens").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    pub async fn close_session(&self, session_id: &str) -> Result<(), AcpError> {
        self.call_raw("session/close", serde_json::json!({"sessionId": session_id})).await?;
        Ok(())
    }

    #[allow(unused)]
    pub async fn list_sessions(&self, cwd: Option<&str>) -> Result<serde_json::Value, AcpError> {
        let mut p = serde_json::json!({});
        if let Some(d) = cwd { p["cwd"] = serde_json::Value::String(d.to_string()); }
        self.call_raw("sessions/list", p).await
    }
}
