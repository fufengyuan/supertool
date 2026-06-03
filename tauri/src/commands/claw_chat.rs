use std::path::PathBuf;
use std::sync::Arc;

use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Claw 聊天状态（单例，存在 app state 中）
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

// ── Session persistence ──────────────────────────────────────────────────

fn sessions_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("sessions")
}

fn session_file(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.json"))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSession {
    session_id: String,
    created_at: String,
    messages: Vec<Message>,
}

fn save_session(id: &str, messages: &[Message]) {
    let dir = sessions_dir();
    let _ = std::fs::create_dir_all(&dir);
    let file = session_file(id);
    let now = chrono::Utc::now().to_rfc3339();
    let session = PersistedSession {
        session_id: id.to_string(),
        created_at: now,
        messages: messages.to_vec(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let _ = std::fs::write(&file, json);
    }
}

fn load_session(id: &str) -> Option<PersistedSession> {
    let file = session_file(id);
    let content = std::fs::read_to_string(&file).ok()?;
    serde_json::from_str(&content).ok()
}

fn list_session_files() -> Vec<serde_json::Value> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut sessions: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<PersistedSession>(&content) {
                        sessions.push(serde_json::json!({
                            "sessionId": session.session_id,
                            "createdAt": session.created_at,
                            "messageCount": session.messages.len(),
                        }));
                    }
                }
            }
        }
    }
    sessions.sort_by(|a, b| {
        let ta = a.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        let tb = b.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        tb.cmp(ta) // newest first
    });
    sessions
}

// ── Config ───────────────────────────────────────────────────────────────

/// 从 ~/.claw/config.json 读取 API key 和 base URL，设置到进程环境变量
fn setup_env_from_claw_config() -> Result<(), String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    if config.api_key.is_empty() {
        log::info!("[claw_chat] No ~/.claw/config.json api_key — falling back to env vars");
        return Ok(());
    }

    log::info!(
        "[claw_chat] Read Claw config: model={}, has_api_key={}, base_url={}",
        config.model,
        !config.api_key.is_empty(),
        config.base_url
    );

    let has_base_url = !config.base_url.is_empty();

    if has_base_url {
        unsafe { std::env::set_var("OPENAI_API_KEY", &config.api_key); }
        unsafe { std::env::set_var("OPENAI_BASE_URL", &config.base_url); }
        unsafe { std::env::set_var("OPENAI_MODEL", &config.model); }
        log::info!("[claw_chat] Base URL set → using OpenAI-compatible client");
    } else {
        let canonical = config.model.to_ascii_lowercase();
        if canonical.starts_with("claude") || canonical.starts_with("anthropic/") {
            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("ANTHROPIC_MODEL", &config.model); }
        } else if canonical.starts_with("openai/") || canonical.starts_with("gpt-") {
            unsafe { std::env::set_var("OPENAI_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("OPENAI_MODEL", &config.model); }
        } else if canonical.starts_with("grok") {
            unsafe { std::env::set_var("XAI_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("XAI_MODEL", &config.model); }
        } else {
            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("ANTHROPIC_MODEL", &config.model); }
        }
    }

    Ok(())
}

// ── Commands ─────────────────────────────────────────────────────────────

/// 初始化 LLM 客户端。如果提供了 session_id，则从磁盘恢复历史消息。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_init(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    session_id: Option<String>,
    _cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[claw_chat] Initializing LLM client from Claw config");

    setup_env_from_claw_config()?;
    let client = LlmClient::from_env()?;

    // 决定 session ID：恢复旧的 or 创建新的
    let sid = if let Some(ref existing) = session_id {
        if load_session(existing).is_some() {
            log::info!("[claw_chat] Restoring session {}", existing);
            existing.clone()
        } else {
            log::info!("[claw_chat] Session {} not found, creating new", existing);
            uuid::Uuid::new_v4().to_string()
        }
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    log::info!(
        "[claw_chat] LLM client initialized: provider={:?}, model={}",
        client.provider(),
        client.model(),
    );

    // Store client
    {
        let mut c = state.client.lock().await;
        *c = Some(Arc::new(client));
    }

    // Restore messages from disk if available
    let restored_count;
    {
        let mut s = state.session_id.lock().await;
        *s = Some(sid.clone());
    }
    {
        let mut msgs = state.messages.lock().await;
        if let Some(persisted) = load_session(&sid) {
            restored_count = persisted.messages.len();
            *msgs = persisted.messages;
        } else {
            restored_count = 0;
            msgs.clear();
        }
    }

    let _ = app.emit(
        "agent-session-created",
        serde_json::json!({
            "session_id": sid,
            "restored": restored_count > 0,
            "message_count": restored_count,
        }),
    );

    Ok(serde_json::json!({
        "sessionId": sid,
        "restored": restored_count > 0,
        "messageCount": restored_count,
    }))
}

/// 发送消息到 LLM（streaming via SSE → Tauri events）
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

    let client = client.ok_or("Claw not initialized")?;
    let session_id = session_id.clone().ok_or("No session")?;

    // 保存用户消息到历史
    {
        let mut msgs = state.messages.lock().await;
        msgs.push(Message {
            role: "user".to_string(),
            content: message.clone(),
        });
    }

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
    // Collect assistant reply during streaming, save after completion
    let has_received_usage = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_received_usage_clone = has_received_usage.clone();
    let assistant_reply = Arc::new(Mutex::new(String::new()));
    let assistant_reply_clone = assistant_reply.clone();

    client
        .send_streaming(&messages, move |event| {
            match event {
                Ok(LlmStreamEvent::TextDelta { text }) => {
                    // Accumulate for persistence
                    {
                        let mut reply = assistant_reply_clone.blocking_lock();
                        reply.push_str(&text);
                    }
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
                Ok(LlmStreamEvent::ToolCall { id, name, input }) => {
                    let _ = app_clone.emit(
                        "agent-tool-start",
                        serde_json::json!({
                            "id": id,
                            "name": name,
                            "args": input,
                            "session_id": sid.clone(),
                        }),
                    );
                }
                Ok(LlmStreamEvent::Usage {
                    input_tokens,
                    output_tokens,
                }) => {
                    has_received_usage_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    let _ = app_clone.emit(
                        "agent-usage",
                        serde_json::json!({
                            "prompt_tokens": input_tokens,
                            "completion_tokens": output_tokens,
                            "total_tokens": input_tokens + output_tokens,
                            "session_id": sid.clone(),
                        }),
                    );
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
                    if !has_received_usage_clone.load(std::sync::atomic::Ordering::SeqCst) {
                        let _ = app_clone.emit(
                            "agent-done",
                            serde_json::json!({
                                "session_id": sid.clone(),
                            }),
                        );
                    }
                    log::info!("[claw_chat] Stream completed");
                }
                Err(err_msg) => {
                    log::error!("[claw_chat] Stream error: {}", err_msg);
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

    // Stream 完成后：将 assistant 回复保存到 messages 并持久化到磁盘
    let reply_text = assistant_reply.lock().await;
    if !reply_text.is_empty() {
        let mut msgs = state.messages.lock().await;
        msgs.push(Message {
            role: "assistant".to_string(),
            content: reply_text.clone(),
        });
        save_session(&session_id, &msgs);
    }

    Ok(())
}

/// 关闭会话（仅断开 LLM 连接，保留消息历史）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_close(
    state: tauri::State<'_, ClawChatState>,
) -> Result<(), String> {
    log::info!("[claw_chat] Closing session (preserving messages)");

    // 先持久化当前消息
    {
        let sid = state.session_id.lock().await.clone();
        let msgs = state.messages.lock().await;
        if let Some(ref id) = sid {
            save_session(id, &msgs);
        }
    }

    // 断开 LLM 连接但保留 session_id 和 messages
    {
        let mut c = state.client.lock().await;
        *c = None;
    }

    Ok(())
}

/// 获取当前会话列表（从磁盘读取所有持久化的会话）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_list_sessions(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    // 合并磁盘上的历史会话 + 当前活跃会话
    let mut sessions = list_session_files();

    // 如果有活跃会话且不在磁盘列表中，加上
    let active_sid = {
        let s = state.session_id.lock().await;
        s.clone()
    };
    if let Some(ref sid) = active_sid {
        if !sessions.iter().any(|s| s.get("sessionId").and_then(|v| v.as_str()) == Some(sid)) {
            let msg_count = {
                let msgs = state.messages.lock().await;
                msgs.len()
            };
            sessions.push(serde_json::json!({
                "sessionId": sid,
                "createdAt": chrono::Utc::now().to_rfc3339(),
                "messageCount": msg_count,
                "active": true,
            }));
        }
    }

    Ok(serde_json::json!({ "sessions": sessions }))
}

/// 获取 Claw 客户端信息
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_info() -> Result<serde_json::Value, String> {
    let config = crate::commands::claw_config::read_claw_config()?;
    let api_key_found = !config.api_key.is_empty();
    let model = config.model.clone();
    let provider = config.provider.clone();
    let base_url = if config.base_url.is_empty() { None } else { Some(config.base_url) };

    Ok(serde_json::json!({
        "mode": "claw",
        "apiKeyConfigured": api_key_found,
        "baseUrl": base_url,
        "model": model,
        "provider": provider,
        "configSource": "~/.claw/config.json",
    }))
}

/// 读取模型配置
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_models_config() -> Result<serde_json::Value, String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    let mut providers: Vec<serde_json::Value> = Vec::new();

    if !config.api_key.is_empty() || !config.base_url.is_empty() {
        let provider_name = if config.provider.is_empty() {
            if config.model.starts_with("claude") || config.model.starts_with("anthropic/") {
                "Anthropic"
            } else if config.model.starts_with("gpt") || config.model.starts_with("openai/") {
                "OpenAI"
            } else if config.model.starts_with("grok") {
                "xAI"
            } else if config.model.starts_with("qwen") {
                "DashScope"
            } else {
                "Custom"
            }
        } else {
            &config.provider
        };

        providers.push(serde_json::json!({
            "name": provider_name,
            "models": if !config.model.is_empty() {
                vec![serde_json::json!({"id": &config.model, "active": true})]
            } else {
                Vec::<serde_json::Value>::new()
            },
            "apiKey": !config.api_key.is_empty(),
            "baseUrl": &config.base_url,
            "active": true,
        }));
    }

    Ok(serde_json::json!({
        "providers": providers,
        "source": "~/.claw/config.json",
    }))
}

/// 读取聊天统计
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_stats() -> Result<serde_json::Value, String> {
    let sessions = list_session_files();
    let total_messages: usize = sessions.iter()
        .filter_map(|s| s.get("messageCount").and_then(|v| v.as_u64()).map(|n| n as usize))
        .sum();

    Ok(serde_json::json!({
        "sessions": sessions.len(),
        "messages": total_messages,
        "source": "claw",
    }))
}
