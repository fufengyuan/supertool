use std::sync::Arc;

use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// OMP 聊天状态（单例，存在 app state 中）
///
/// Replaced the old ACP subprocess approach with direct LLM API calls.
pub struct ClawChatState {
    client: Mutex<Option<Arc<LlmClient>>>,
    session_id: Mutex<Option<String>>,
    messages: Mutex<Vec<Message>>,
}

impl ClawChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session_id: Mutex::new(None),
            messages: Mutex::new(Vec::new()),
        }
    }
}

/// 初始化 LLM 客户端连接
///
/// 读取环境变量（ANTHROPIC_API_KEY 或 OPENAI_API_KEY），
/// 创建 LlmClient 并存入 state。
/// 不再需要查找 omp 二进制或启动子进程。
///
/// 发送事件: `agent-session-created`
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_init(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    _cwd: Option<String>,
) -> Result<(), String> {
    log::info!("[claw_chat] Initializing LLM client from environment");

    let client = LlmClient::from_env()?;

    // Generate a session ID (UI tracking; no subprocess session)
    let session_id = uuid::Uuid::new_v4().to_string();

    log::info!(
        "[claw_chat] LLM client initialized: provider={:?}, model={}",
        client.provider(),
        client.model(),
    );

    // Store in state
    {
        let mut c = state.client.lock().await;
        *c = Some(Arc::new(client));
    }
    {
        let mut s = state.session_id.lock().await;
        *s = Some(session_id.clone());
    }

    let _ = app.emit(
        "agent-session-created",
        serde_json::json!({
            "session_id": session_id,
        }),
    );

    Ok(())
}

/// 发送消息到 LLM（streaming via SSE → Tauri events）
///
/// 构建消息数组，调用 `LlmClient::send_streaming()`，
/// 实时 emit 事件给前端：
/// - `agent-delta` — 文本增量
/// - `agent-reasoning-delta` — 思考/推理增量
/// - `agent-done` — 完成（含用量）
/// - `agent-error` — 错误
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_send(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    message: String,
) -> Result<(), String> {
    let (client, session_id) = {
        let c = state.client.lock().await;
        let s = state.session_id.lock().await;
        (c.clone(), s.clone())
    };

    let client = client.ok_or("OMP not initialized")?;
    let session_id = session_id.clone().ok_or("No session")?;

    // Append user message to conversation history
    {
        let mut msgs = state.messages.lock().await;
        msgs.push(Message {
            role: "user".to_string(),
            content: message.clone(),
        });
    }

    // Snapshot current messages for this request
    let messages: Vec<Message> = {
        let msgs = state.messages.lock().await;
        msgs.clone()
    };

    log::info!(
        "[claw_chat] Sending streaming request ({} messages)",
        messages.len()
    );

    let app_clone = app.clone();
    let sid = session_id.clone();

    client
        .send_streaming(&messages, move |event| {
            match event {
                Ok(LlmStreamEvent::TextDelta { text }) => {
                    let _ = app_clone.emit(
                        "agent-delta",
                        serde_json::json!({
                            "text": text,
                            "session_id": sid.clone(),
                        }),
                    );
                }
                Ok(LlmStreamEvent::ThinkingDelta { thinking }) => {
                    let _ = app_clone.emit(
                        "agent-reasoning-delta",
                        serde_json::json!({
                            "text": thinking,
                            "session_id": sid.clone(),
                        }),
                    );
                }
                Ok(LlmStreamEvent::ToolCall {
                    id,
                    name,
                    input,
                }) => {
                    let _ = app_clone.emit(
                        "agent-tool-call",
                        serde_json::json!({
                            "tool_call_id": id,
                            "name": name,
                            "arguments": input,
                            "session_id": sid.clone(),
                        }),
                    );
                }
                Ok(LlmStreamEvent::Usage {
                    input_tokens,
                    output_tokens,
                }) => {
                    // Usage is emitted as part of agent-done
                    let _ = app_clone.emit(
                        "agent-done",
                        serde_json::json!({
                            "session_id": sid.clone(),
                            "usage": {
                                "input_tokens": input_tokens,
                                "output_tokens": output_tokens,
                            },
                        }),
                    );
                }
                Ok(LlmStreamEvent::Done) => {
                    log::info!("[claw_chat] Stream completed");
                }
                Err(err_msg) => {
                    log::error!("[claw_chat] Stream error: {}", err_msg);
                    let _ = app_clone.emit(
                        "agent-error",
                        serde_json::json!({
                            "message": err_msg,
                            "session_id": sid.clone(),
                        }),
                    );
                }
            }
        })
        .await
        .map_err(|e| {
            log::error!("[claw_chat] send_streaming failed: {}", e);
            let _ = app.emit(
                "agent-error",
                serde_json::json!({
                    "message": e,
                    "session_id": session_id.clone(),
                }),
            );
            format!("send failed: {e}")
        })?;

    Ok(())
}

/// 关闭会话（清理 state）
///
/// 旧版 ACP 需要关闭子进程；新版仅清理内存中的 state。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_close(
    state: tauri::State<'_, ClawChatState>,
) -> Result<(), String> {
    log::info!("[claw_chat] Closing session");

    {
        let mut c = state.client.lock().await;
        *c = None;
    }
    {
        let mut s = state.session_id.lock().await;
        *s = None;
    }
    {
        let mut msgs = state.messages.lock().await;
        msgs.clear();
    }

    Ok(())
}

/// 获取当前会话列表（兼容性保留；直接 LLM 模式下无子进程会话）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_list_sessions(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    let session_id = {
        let s = state.session_id.lock().await;
        s.clone()
    };

    Ok(serde_json::json!({
        "sessions": if session_id.is_some() {
            vec![serde_json::json!({"sessionId": session_id})]
        } else {
            Vec::<serde_json::Value>::new()
        }
    }))
}

/// 获取 LLM 客户端信息（无 omp 二进制信息）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_info() -> Result<serde_json::Value, String> {
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .map(|_| true)
        .unwrap_or(false);
    let openai_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|_| true)
        .unwrap_or(false);
    let model = std::env::var("ANTHROPIC_MODEL")
        .or_else(|_| std::env::var("OPENAI_MODEL"))
        .unwrap_or_else(|_| "auto".into());

    Ok(serde_json::json!({
        "mode": "direct-llm",
        "anthropic_configured": anthropic_key,
        "openai_configured": openai_key,
        "model": model,
        "provider": if anthropic_key { "anthropic" } else if openai_key { "openai" } else { "none" },
    }))
}

/// 读取 OMP models.yaml（提供商 + 模型配置）
///
/// 保持原样 — 与 OMP 配置集成，非 LLM 客户端功能。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_models_config() -> Result<serde_json::Value, String> {
    let omp_home = dirs::home_dir()
        .ok_or("Cannot find home dir")?
        .join(".omp")
        .join("agent");
    let yaml_path = omp_home.join("models.yaml");
    let json_path = omp_home.join("models.json");
    let content = std::fs::read_to_string(&yaml_path)
        .or_else(|_| std::fs::read_to_string(&json_path))
        .map_err(|e| format!("OMP config not found (run omp once): {e}"))?;
    if yaml_path.exists() {
        serde_yaml::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("parse models.yaml failed: {e}"))
    } else {
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("parse models.json failed: {e}"))
    }
}

/// 读取 OMP 历史会话统计（来自 history.db）
///
/// 保持原样 — 与 OMP 配置集成。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_stats() -> Result<serde_json::Value, String> {
    let omp_home = dirs::home_dir()
        .ok_or("Cannot find home dir")?
        .join(".omp")
        .join("agent");
    let db_path = omp_home.join("history.db");
    if !db_path.exists() {
        return Ok(serde_json::json!({"sessions": 0, "messages": 0}));
    }
    let conn =
        rusqlite::Connection::open(&db_path).map_err(|e| format!("open history.db: {e}"))?;
    let sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0);
    let messages: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |r| r.get(0))
        .unwrap_or(0);
    Ok(serde_json::json!({"sessions": sessions, "messages": messages}))
}
