use std::path::PathBuf;
use std::sync::Arc;

use runtime::{
    ContentBlock, ConversationMessage, MessageRole, Session,
};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// Claw 聊天状态（单例，存在 app state 中）
pub struct ClawChatState {
    pub(crate) client: Mutex<Option<Arc<LlmClient>>>,
    pub(crate) session: Mutex<Option<Session>>,
}

impl ClawChatState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            session: Mutex::new(None),
        }
    }
}

// ── Session persistence (uses claw-code's Session API) ──────────────────

fn sessions_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claw")
        .join("sessions")
}

fn session_path(id: &str) -> PathBuf {
    sessions_dir().join(format!("{id}.json"))
}

/// Load a persisted session by ID.
fn load_session(id: &str) -> Option<Session> {
    let path = session_path(id);
    Session::load_from_path(&path).ok()
}

/// List all persisted sessions by reading their JSONL meta record (first line).
pub(crate) fn list_sessions_info() -> Vec<serde_json::Value> {
    let dir = sessions_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut sessions: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            // Read the first line (meta record) for session info
            if let Ok(content) = std::fs::read_to_string(&path) {
                let first_line = content.lines().next().unwrap_or("");
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(first_line) {
                    let session_id = meta
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let created_at_ms = meta
                        .get("created_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let _updated_at_ms = meta
                        .get("updated_at_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    // Read the first message (after meta line) for a title preview
                    let preview: Option<String> = content
                        .lines()
                        .nth(1)
                        .and_then(|line| {
                            serde_json::from_str::<serde_json::Value>(line).ok()
                        })
                        .and_then(|v| {
                            v.get("content")
                                .and_then(|c| c.as_str())
                                .map(|c| c.trim().to_string())
                        })
                        .filter(|c| !c.is_empty())
                        .map(|c| {
                            if c.len() > 60 {
                                format!("{}...", &c[..60])
                            } else {
                                c
                            }
                        });
                    // Count message lines in the JSONL file
                    let message_count = content.lines().skip(1).count();
                    sessions.push(serde_json::json!({
                        "sessionId": session_id,
                        "createdAt": format_ts(created_at_ms),
                        "messageCount": message_count,
                        "title": preview,
                    }));
                }
            }
        }
    }
    // Newest first by created_at_ms
    sessions.sort_by(|a, b| {
        let ta = a.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        let tb = b.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        tb.cmp(ta)
    });
    sessions
}

/// Convert session messages to a simplified JSON array for the front-end.
fn session_messages_to_json(messages: &[ConversationMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|cm| {
            let role = match cm.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "agent",
                MessageRole::System => "system",
                MessageRole::Tool => "tool",
            };
            let text = cm
                .blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.clone()),
                    ContentBlock::Thinking { thinking, .. } => Some(thinking.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::json!({
                "role": role,
                "content": text,
            })
        })
        .collect()
}

/// Format a Unix-epoch millis timestamp to RFC 3339 for the frontend.
fn format_ts(ms: u64) -> String {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_millis_opt(ms as i64)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

// ── Config ───────────────────────────────────────────────────────────────

/// 从 ~/.claw/config.json 读取 API key 和 base URL，设置到进程环境变量
pub(crate) fn setup_env_from_claw_config() -> Result<(), String> {
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

// ── ConversationMessage ↔ LlmClient::Message conversion ─────────────────

/// Convert claw-code's `ConversationMessage` to the flat `Message` format
/// that `LlmClient::send_streaming` expects.
pub(crate) fn to_prompt_messages(session_messages: &[ConversationMessage]) -> Vec<Message> {
    session_messages
        .iter()
        .map(|cm| {
            let role = match cm.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
            }
            .to_string();

            // Flatten blocks into a single text string
            let content: String = cm
                .blocks
                .iter()
                .map(|b| match b {
                    ContentBlock::Text { text } => text.clone(),
                    ContentBlock::Thinking { thinking, .. } => thinking.clone(),
                    ContentBlock::ToolUse { id, name, input } => {
                        format!("[ToolUse: {name}({id})] {input}")
                    }
                    ContentBlock::ToolResult {
                        tool_name, output, ..
                    } => format!("[ToolResult: {tool_name}] {output}"),
                })
                .collect::<Vec<_>>()
                .join("\n");

            Message { role, content }
        })
        .collect()
}

// ── Tauri Commands ──────────────────────────────────────────────────────

/// 初始化 LLM 客户端和 Session。如果提供了 session_id，则从磁盘恢复。
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_init(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    session_id: Option<String>,
    _cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[claw_chat] Initializing LLM client from Claw config");

    setup_env_from_claw_config()?;

    // ── Session ──
    let (sid, restored_count) = if let Some(ref existing) = session_id {
        if let Some(loaded) = load_session(existing) {
            let count = loaded.messages.len();
            log::info!("[claw_chat] Restored session {} ({} messages)", existing, count);
            {
                let mut s = state.session.lock().await;
                *s = Some(loaded);
            }
            (existing.clone(), count)
        } else {
            log::info!("[claw_chat] Session {} not found on disk, creating new", existing);
            let new_id = uuid::Uuid::new_v4().to_string();
            let path = session_path(&new_id);
            std::fs::create_dir_all(path.parent().unwrap()).ok();
            // Don't save to disk yet — defer until first message
            let session = Session::new()
                .with_persistence_path(&path);
            {
                let mut s = state.session.lock().await;
                *s = Some(session);
            }
            (new_id, 0)
        }
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        let path = session_path(&new_id);
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        // Don't save to disk yet — defer until first message
        let session = Session::new()
            .with_persistence_path(&path);
        {
            let mut s = state.session.lock().await;
            *s = Some(session);
        }
        (new_id, 0)
    };

    // ── LLM Client ──
    let client = LlmClient::from_env().map_err(|e| format!("LLM init failed: {e}"))?;
    log::info!(
        "[claw_chat] LLM client initialized: provider={:?}, model={}",
        client.provider(),
        client.model(),
    );
    {
        let mut c = state.client.lock().await;
        *c = Some(Arc::new(client));
    }

    let _ = app.emit(
        "agent-session-created",
        serde_json::json!({
            "session_id": sid,
            "restored": restored_count > 0,
            "message_count": restored_count,
        }),
    );

    let restored_messages = if restored_count > 0 {
        let s = state.session.lock().await;
        s.as_ref()
            .map(|sess| session_messages_to_json(&sess.messages))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(serde_json::json!({
        "sessionId": sid,
        "restored": restored_count > 0,
        "messageCount": restored_count,
        "messages": restored_messages,
    }))
}

/// 发送消息到 LLM（streaming via SSE → Tauri events）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_send(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    message: String,
) -> Result<(), String> {
    let (client, session_path_buf) = {
        let c = state.client.lock().await;
        let s = state.session.lock().await;
        let client = c.clone().ok_or("Claw not initialized")?;
        let path = s
            .as_ref()
            .and_then(|sess| sess.persistence_path().map(|p| p.to_path_buf()));
        (client, path)
    };
    let session_path =
        session_path_buf.ok_or("No session path set — call claw_chat_init first")?;

    // ── Push user message & persist ──
    {
        let mut s = state.session.lock().await;
        if let Some(ref mut sess) = *s {
            sess.push_user_text(&message)
                .map_err(|e| format!("Failed to push user message: {e}"))?;
            // Persist immediately so user message is saved even if stream fails
            if let Some(path) = sess.persistence_path() {
                let _ = sess.save_to_path(path);
            }
        }
    }

    // ── Convert session messages to prompt format ──
    let prompt_messages: Vec<Message> = {
        let s = state.session.lock().await;
        match s.as_ref() {
            Some(sess) => to_prompt_messages(&sess.messages),
            None => return Err("No session".into()),
        }
    };

    log::info!(
        "[claw_chat] Sending streaming request ({} messages)",
        prompt_messages.len()
    );

    // ── Stream to Tauri events ──
    let app_clone = app.clone();
    let sid = session_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let has_received_usage = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_received_usage_clone = has_received_usage.clone();
    let assistant_reply = Arc::new(Mutex::new(String::new()));
    let assistant_reply_clone = assistant_reply.clone();

    let sid_stream = sid.clone();
    let sid_err = sid.clone();
    client
        .send_streaming(&prompt_messages, move |event| {
            match event {
                Ok(LlmStreamEvent::TextDelta { text }) => {
                    let mut reply = assistant_reply_clone.blocking_lock();
                    reply.push_str(&text);
                    let _ = app_clone.emit(
                        "agent-delta",
                        serde_json::json!({
                            "text": text,
                            "session_id": sid_stream.clone(),
                        }),
                    );
                }
                Ok(LlmStreamEvent::ThinkingDelta { thinking }) => {
                    let _ = app_clone.emit(
                        "agent-reasoning-delta",
                        serde_json::json!({
                            "text": thinking,
                            "session_id": sid_stream.clone(),
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
                            "session_id": sid_stream.clone(),
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
                            "session_id": sid_stream.clone(),
                        }),
                    );
                    let _ = app_clone.emit(
                        "agent-done",
                        serde_json::json!({
                            "session_id": sid_stream.clone(),
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
                                "session_id": sid_stream.clone(),
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
                    "session_id": sid_err.clone(),
                }),
            );
            format!("send failed: {e}")
        })?;

    // ── Persist assistant reply to Session ──
    let reply_text = assistant_reply.lock().await;
    if !reply_text.is_empty() {
        let assistant_msg =
            ConversationMessage::assistant(vec![ContentBlock::Text {
                text: reply_text.clone(),
            }]);
        let mut s = state.session.lock().await;
        if let Some(ref mut sess) = *s {
            sess.push_message(assistant_msg)
                .map_err(|e| format!("Failed to push assistant message: {e}"))?;
            // Full snapshot after each turn
            let _ = sess.save_to_path(&session_path);
        }
    }

    Ok(())
}

/// 关闭会话（仅断开 LLM 连接，保留 session 持久化）
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_close(
    state: tauri::State<'_, ClawChatState>,
) -> Result<(), String> {
    log::info!("[claw_chat] Closing session (preserving messages)");

    // Final save
    {
        let s = state.session.lock().await;
        if let Some(ref sess) = *s {
            if let Some(path) = sess.persistence_path() {
                let _ = sess.save_to_path(path);
            }
        }
    }

    // Drop LLM client, keep session
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
    let mut sessions = list_sessions_info();

    // 如果有活跃会话且不在磁盘列表中，加上
    let active_sid = {
        let s = state.session.lock().await;
        s.as_ref().map(|sess| sess.session_id.clone())
    };
    if let Some(ref sid) = active_sid.as_deref() {
        if !sessions.iter().any(|s| {
            s.get("sessionId").and_then(|v| v.as_str()) == Some(sid)
        }) {
            let msg_count = {
                let s = state.session.lock().await;
                s.as_ref().map(|sess| sess.messages.len()).unwrap_or(0)
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
    let base_url = if config.base_url.is_empty() {
        None
    } else {
        Some(config.base_url)
    };

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
    let sessions = list_sessions_info();
    let total_messages: usize = sessions
        .iter()
        .filter_map(|s| s.get("messageCount").and_then(|v| v.as_u64()))
        .sum::<u64>() as usize;

    Ok(serde_json::json!({
        "sessions": sessions.len(),
        "messages": total_messages,
        "source": "claw",
    }))
}


