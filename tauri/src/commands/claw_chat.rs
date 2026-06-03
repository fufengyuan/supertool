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

/// 从 SuperTool Claw 配置读取 API key 和 base URL，设置到进程环境变量
///
/// 配置来源：`~/.supertool/claw-config.json`（前端 Settings 页面配置）
/// 如果没有配置，回退到环境变量（兼容直接设置 ANTHROPIC_API_KEY 的场景）。
fn setup_env_from_claw_config() -> Result<(), String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    // 如果没有配置 api_key，回退到环境变量（让用户直接 export 也能工作）
    if config.api_key.is_empty() {
        log::info!("[claw_chat] No claw-config.json api_key — falling back to env vars");
        return Ok(());
    }

    log::info!(
        "[claw_chat] Read Claw config: model={}, has_api_key={}, base_url={}",
        config.model,
        !config.api_key.is_empty(),
        config.base_url,
    );

    // 根据配置设置环境变量
    // 优先级：有 base_url → OpenAI-compatible
    //         无 base_url → 按模型名前缀路由
    let has_base_url = !config.base_url.is_empty();

    if has_base_url {
        // 有 base_url → 走 OpenAI-compatible（兼容 OpenRouter / 代理 / 本地模型等）
        unsafe { std::env::set_var("OPENAI_API_KEY", &config.api_key); }
        unsafe { std::env::set_var("OPENAI_BASE_URL", &config.base_url); }
        unsafe { std::env::set_var("OPENAI_MODEL", &config.model); }
        log::info!(
            "[claw_chat] Base URL set → using OpenAI-compatible client"
        );
    } else {
        // 无 base_url → 按模型名前缀路由
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
            // 未知前缀 → Anthropic 兜底
            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &config.api_key); }
            unsafe { std::env::set_var("ANTHROPIC_MODEL", &config.model); }
        }
    }

    Ok(())
}

/// 初始化 LLM 客户端连接
///
/// 从 Hermes config.yaml 读取 api_key + base_url，设置到进程环境变量后
/// 创建 LlmClient。不再依赖系统环境变量。
///
/// 发送事件: `agent-session-created`
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_chat_init(
    app: AppHandle,
    state: tauri::State<'_, ClawChatState>,
    _cwd: Option<String>,
) -> Result<(), String> {
    log::info!("[claw_chat] Initializing LLM client from Claw config");

    // 从 SuperTool Claw 配置读取 API key/base URL 并设置环境变量
    setup_env_from_claw_config()?;

    let client = LlmClient::from_env()?;

    // Generate a session ID (UI tracking)
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
/// - `agent-tool-start` — 工具调用开始
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
    let has_received_usage = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_received_usage_clone = has_received_usage.clone();

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
                    // 先发 usage 事件
                    let _ = app_clone.emit(
                        "agent-usage",
                        serde_json::json!({
                            "prompt_tokens": input_tokens,
                            "completion_tokens": output_tokens,
                            "total_tokens": input_tokens + output_tokens,
                            "session_id": sid.clone(),
                        }),
                    );
                    // 再发 done（信号流结束）
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
                    // 如果 Usage 已经发过 done 了，这里不再重复发
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
                    // Error will be caught by map_err below — no double emit
                }
            }
        })
        .await
        .map_err(|e| {
            log::error!("[claw_chat] send_streaming failed: {}", e);
            // 发送流结束事件，让前端结束 loading
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

/// 获取当前会话列表
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

/// 获取 Claw 客户端信息（从 SuperTool claw-config.json）
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
        "configSource": "~/.supertool/claw-config.json",
    }))
}

/// 读取模型配置（从 Claw 自己的配置）
/// 返回当前配置的提供商和模型信息
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_models_config() -> Result<serde_json::Value, String> {
    let config = crate::commands::claw_config::read_claw_config()?;

    let mut providers: Vec<serde_json::Value> = Vec::new();

    // 从 claw-config.json 构建当前配置的 provider
    if !config.api_key.is_empty() || !config.base_url.is_empty() {
        let provider_name = if config.provider.is_empty() {
            // 根据模型名推断 provider
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
        "source": "~/.supertool/claw-config.json",
    }))
}

/// 读取聊天统计（从 Hermes 状态）
/// Claw 模式没有持久化会话，返回简单统计
#[tauri::command(rename_all = "camelCase")]
pub async fn claw_read_stats(
    state: tauri::State<'_, ClawChatState>,
) -> Result<serde_json::Value, String> {
    let messages_count = {
        let msgs = state.messages.lock().await;
        msgs.len()
    };
    let session_active = {
        let s = state.session_id.lock().await;
        s.is_some()
    };

    Ok(serde_json::json!({
        "sessions": if session_active { 1 } else { 0 },
        "messages": messages_count,
        "source": "claw",
    }))
}
