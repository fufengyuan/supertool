use std::sync::Arc;
use supertool_omp::acp::AcpClient;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// OMP 聊天状态（单例，存在 app state 中）
pub struct OmpChatState {
    client: Mutex<Option<Arc<AcpClient>>>,
    session_id: Mutex<Option<String>>,
}

impl OmpChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session_id: Mutex::new(None),
        }
    }
}

/// 初始化 OMP 连接 + 创建 session
/// 发送事件: omp:ready
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_chat_init(
    app: AppHandle,
    state: tauri::State<'_, OmpChatState>,
    cwd: Option<String>,
) -> Result<(), String> {
    // 查找 omp 二进制
    let omp_path = find_omp().map_err(|e| format!("找不到 omp 命令: {e}"))?;

    let client = AcpClient::spawn(&omp_path, cwd.as_deref())
        .await
        .map_err(|e| format!("启动 omp acp 失败: {e}"))?;

    client.initialize().await.map_err(|e| format!("initialize 失败: {e}"))?;
    client.authenticate().await.map_err(|e| format!("authenticate 失败: {e}"))?;

    let workdir = cwd.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into())
    });

    let session_id = client
        .new_session(&workdir)
        .await
        .map_err(|e| format!("new_session 失败: {e}"))?;
    let sid = session_id.clone();

    // 订阅通知 → 转为 Chat GUI 事件
    let mut rx = client.subscribe();
    let app_emit = app.clone();
    let sid_for_task = session_id.clone();
    tokio::spawn(async move {
        while let Ok(notif) = rx.recv().await {
            if notif.session_id != sid_for_task {
                continue;
            }
            match notif.update {
                supertool_omp::acp::AcpSessionUpdate::MessageChunk(text) => {
                    let _ = app_emit.emit("agent-delta", serde_json::json!({
                        "text": text,
                        "session_id": sid_for_task,
                    }));
                }
                supertool_omp::acp::AcpSessionUpdate::ThoughtChunk(text) => {
                    let _ = app_emit.emit("agent-reasoning-delta", serde_json::json!({
                        "text": text,
                        "session_id": sid_for_task,
                    }));
                }
                supertool_omp::acp::AcpSessionUpdate::ToolCall { id, name, raw_input } => {
                    let _ = app_emit.emit("agent-tool-call", serde_json::json!({
                        "tool_call_id": id,
                        "name": name,
                        "arguments": raw_input,
                        "session_id": sid_for_task,
                    }));
                }
                supertool_omp::acp::AcpSessionUpdate::ToolCallResult { id, content, is_error } => {
                    let _ = app_emit.emit("agent-tool-result", serde_json::json!({
                        "tool_call_id": id,
                        "content": content,
                        "is_error": is_error,
                        "session_id": sid_for_task,
                    }));
                }
                _ => {}
            }
        }
    });

    // 存入 state
    {
        let mut c = state.client.lock().await;
        *c = Some(Arc::new(client));
    }
    {
        let mut s = state.session_id.lock().await;
        *s = Some(session_id);
    }

    let _ = app.emit("agent-session-created", serde_json::json!({
        "session_id": sid,
    }));

    Ok(())
}

/// 发送消息到 OMP（streaming 事件通过 agent-delta 等自动到达）
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_chat_send(
    app: AppHandle,
    state: tauri::State<'_, OmpChatState>,
    message: String,
) -> Result<(), String> {
    let (client, session_id) = {
        let c = state.client.lock().await;
        let s = state.session_id.lock().await;
        (c.clone(), s.clone())
    };

    let client = client.ok_or("OMP not initialized")?;
    let session_id = session_id.ok_or("No session")?;

    // 发送 prompt，流式通知已通过 subscriber 自动 emit
    let result = client
        .prompt(&session_id, &message)
        .await
        .map_err(|e| format!("prompt 失败: {e}"))?;

    // 发送完成事件 + 用量
    let _ = app.emit("agent-done", serde_json::json!({
        "session_id": session_id,
        "usage": {
            "input_tokens": result.input_tokens,
            "output_tokens": result.output_tokens,
        },
    }));

    Ok(())
}

/// 关闭 OMP session + 断开连接
#[tauri::command(rename_all = "camelCase")]
pub async fn omp_chat_close(
    state: tauri::State<'_, OmpChatState>,
) -> Result<(), String> {
    let (client, session_id) = {
        let c = state.client.lock().await;
        let s = state.session_id.lock().await;
        (c.clone(), s.clone())
    };

    if let Some(client) = client {
        if let Some(sid) = session_id {
            let _ = client.close_session(&sid).await;
        }
    }

    {
        let mut c = state.client.lock().await;
        *c = None;
    }
    {
        let mut s = state.session_id.lock().await;
        *s = None;
    }

    Ok(())
}

/// 查找 omp 二进制
fn find_omp() -> Result<String, String> {
    // 1) PATH 中的 omp
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let candidate = format!("{dir}/omp");
            if std::path::Path::new(&candidate).exists() {
                return Ok(candidate);
            }
        }
    }
    // 2) 常见的安装位置
    for candidate in &["/usr/local/bin/omp", "/opt/homebrew/bin/omp", "/home/linuxbrew/.linuxbrew/bin/omp"] {
        if std::path::Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }
    Err("omp not found in PATH".into())
}
