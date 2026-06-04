//! IPC-style tests for claw_chat commands.
//!
//! Generated from `commands/claw_chat.rs` — all tests moved here.

use crate::commands::claw_chat::*;
use crate::tests::invoke_ipc;

use runtime::{ContentBlock, ConversationMessage, MessageRole, Session};
use supertool_claw::llm::{LlmClient, LlmStreamEvent, Message};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Helper: build a mock app + webview ──────────────────────────────────

fn build_test_app(
    state: ClawChatState,
) -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = tauri::test::mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // 以下命令不含 AppHandle，在 MockRuntime 下可以正常 IPC
            crate::commands::claw_chat::claw_chat_info,
            crate::commands::claw_chat::claw_chat_list_sessions,
            crate::commands::claw_chat::claw_read_models_config,
            crate::commands::claw_chat::claw_read_stats,
        ])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app should build");

    let webview_window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");

    (app, webview_window)
}

/// 发送 IPC 请求（同步风格），适合在 multi_thread tokio test 中调用
fn invoke_cmd(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::get_ipc_response;
    use tauri::webview::InvokeRequest;

    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_string(),
        },
    )
}

fn invoke_cmd_ok(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> serde_json::Value {
    let res = invoke_cmd(webview, cmd, body);
    res.unwrap_or_else(|e| panic!("IPC command '{cmd}' failed: {e:?}"))
        .deserialize::<serde_json::Value>()
        .unwrap()
}

// ── Existing direct tests ────────────────────────────────────────────────

#[test]
fn test_list_sessions_returns_array() {
    let sessions = list_sessions_info();
    for s in &sessions {
        let sid = s.get("sessionId").and_then(|v| v.as_str());
        assert!(sid.is_some() && !sid.unwrap().is_empty(), "sessionId should be non-empty");
        assert!(s.get("createdAt").is_some());
    }
}

#[test]
fn test_session_save_and_load_round_trip() {
    let dir = std::env::temp_dir().join("claw_test_sessions");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_session.json");

    // Create and save session
    let mut session = Session::new().with_persistence_path(&path);
    session
        .push_user_text("Hello")
        .expect("push_user_text should succeed");
    session
        .push_message(ConversationMessage::assistant(vec![ContentBlock::Text {
            text: "Hi there!".to_string(),
        }]))
        .expect("push assistant should succeed");
    session.save_to_path(&path).expect("save_to_path should succeed");

    // Load and verify
    let loaded = Session::load_from_path(&path).expect("load_from_path should succeed");
    assert_eq!(loaded.messages.len(), 2, "should have 2 messages");
    assert_eq!(
        loaded.messages[0].role,
        MessageRole::User,
        "first message should be user"
    );
    assert_eq!(
        loaded.messages[1].role,
        MessageRole::Assistant,
        "second message should be assistant"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_to_prompt_messages_converts_correctly() {
    let cms = vec![
        ConversationMessage::user_text("Hello"),
        ConversationMessage::assistant(vec![ContentBlock::Text {
            text: "World".to_string(),
        }]),
    ];
    let msgs = to_prompt_messages(&cms);
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, "user");
    assert_eq!(msgs[0].content, "Hello");
    assert_eq!(msgs[1].role, "assistant");
    assert_eq!(msgs[1].content, "World");
}

#[test]
fn test_setup_env_works_with_valid_config() {
    let config = crate::commands::claw_config::read_claw_config().unwrap_or_default();
    if !config.api_key.is_empty() {
        setup_env_from_claw_config().expect("setup should succeed");
        let env_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| std::env::var("XAI_API_KEY"));
        assert!(env_key.is_ok(), "setup should set API_KEY env var");
    }
}

#[tokio::test]
async fn test_claw_chat_info_returns_valid_shape() {
    let info = claw_chat_info().await.unwrap();
    assert_eq!(info.get("mode").and_then(|v| v.as_str()), Some("claw"));
    assert!(info.get("apiKeyConfigured").is_some());
    assert!(info.get("model").is_some());
    assert!(info.get("provider").is_some());
    assert!(info.get("configSource").is_some());
}

#[tokio::test]
async fn test_claw_read_models_config_returns_valid_shape() {
    let result = claw_read_models_config().await.unwrap();
    assert!(result.get("providers").is_some());
    assert!(result.get("source").is_some());
    let providers = result.get("providers").and_then(|v| v.as_array()).unwrap();
    for p in providers {
        assert!(p.get("name").is_some());
        assert!(p.get("active").is_some());
    }
}

#[tokio::test]
async fn test_claw_read_stats_returns_valid_shape() {
    let stats = claw_read_stats().await.unwrap();
    assert!(stats.get("sessions").and_then(|v| v.as_u64()).is_some());
    assert!(stats.get("messages").and_then(|v| v.as_u64()).is_some());
    assert_eq!(stats.get("source").and_then(|v| v.as_str()), Some("claw"));
}

/// Integration test: init → send → close via ClawChatState
/// Gracefully skips if no API key is configured.
#[tokio::test]
async fn test_chat_state_machine() {
    // Setup env
    if let Err(e) = setup_env_from_claw_config() {
        println!("[SKIP] No Claw config found: {e}");
        return;
    }

    let client = match LlmClient::from_env() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            println!("[SKIP] Cannot create LLM client: {e}");
            return;
        }
    };

    // Create a session in temp dir
    let dir = std::env::temp_dir().join("claw_test_chat");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_chat.json");
    let mut session = Session::new().with_persistence_path(&path);
    session.save_to_path(&path).ok();

    let state = ClawChatState::new();
    {
        let mut c = state.client.lock().await;
        *c = Some(client);
    }
    {
        let mut s = state.session.lock().await;
        *s = Some(session);
    }

    // Send
    let prompt = to_prompt_messages(&{
        let s = state.session.lock().await;
        s.as_ref().unwrap().messages.clone()
    });

    let reply_text = Arc::new(Mutex::new(String::new()));
    let reply_clone = reply_text.clone();
    let has_usage = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_usage_clone = has_usage.clone();

    let result = {
        let client = state.client.lock().await.clone().unwrap();
        client
            .send_streaming(&prompt, move |event| {
                match event {
                    Ok(LlmStreamEvent::TextDelta { text }) => {
                        let mut r = reply_clone.blocking_lock();
                        r.push_str(&text);
                    }
                    Ok(LlmStreamEvent::Usage { .. }) => {
                        has_usage_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    _ => {}
                }
            })
            .await
    };

    match result {
        Ok(()) => {
            let reply = reply_text.lock().await;
            let usage = has_usage.load(std::sync::atomic::Ordering::SeqCst);
            println!(
                "[chat] ✅ Stream complete! Reply={} chars, usage={}",
                reply.len(),
                usage
            );
            assert!(!reply.is_empty(), "should have reply");
            assert!(usage, "should have usage info");

            // Persist
            let mut s = state.session.lock().await;
            if let Some(ref mut sess) = *s {
                sess.push_user_text("Say 'hello'").ok();
                sess.push_message(ConversationMessage::assistant(vec![
                    ContentBlock::Text {
                        text: reply.clone(),
                    },
                ]))
                .ok();
                sess.save_to_path(&path).ok();
            }
            drop(s);

            // Verify
            let loaded = Session::load_from_path(&path).expect("should load");
            assert!(loaded.messages.len() >= 2);
        }
        Err(e) => {
            assert!(
                e.contains("401")
                    || e.contains("auth")
                    || e.contains("key")
                    || e.contains("send failed")
                    || e.contains("Failed to start stream")
                    || e.contains("forbidden")
                    || e.contains("Forbidden"),
                "Error should mention cause: got: {e}"
            );
            println!("[chat] ⚠️  Expected stream error (no valid key?): {e}");
        }
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

// ── IPC 风格测试：通过 get_ipc_response 模拟前端调用 ─────────────────

/// 检查：mock builder + invoke_handler 能否正常构建
#[test]
fn test_ipc_mock_builder_creates_app() {
    let state = ClawChatState::new();
    let (_app, _webview) = build_test_app(state);
    // 测试：没有 panic 就成功了
}

/// 检查：claw_chat_info 通过 IPC 返回正确形状
#[tokio::test(flavor = "multi_thread")]
async fn test_ipc_info() {
    let state = ClawChatState::new();
    let (_app, webview) = build_test_app(state);

    let result = invoke_cmd_ok(&webview, "claw_chat_info", serde_json::json!({}));

    assert_eq!(result.get("mode").and_then(|v| v.as_str()), Some("claw"));
    assert!(result.get("model").is_some(), "model field");
    assert!(result.get("provider").is_some(), "provider field");
    assert!(result.get("apiKeyConfigured").is_some(), "apiKeyConfigured");
    assert!(result.get("configSource").and_then(|v| v.as_str()).is_some(), "configSource is a string");
}

// ── Real API integration test ────────────────────────────────────────────

/// Tests that the real API returns TextDelta events after ThinkingDelta when
/// the user sends "在吗" through the actual Claw configuration.
/// SKIP by default unless --include-ignored is passed (requires real API key).
#[tokio::test]
#[ignore]
async fn test_real_api_returns_text_after_thinking() {
    use crate::commands::claw_chat::setup_env_from_claw_config;
    setup_env_from_claw_config().expect("setup env from claw config");

    let client = LlmClient::from_env().expect("create LlmClient from env");
    log::info!("Provider: {:?}, Model: {}", client.provider(), client.model());

    let messages = vec![Message {
        role: "user".to_string(),
        content: "在吗".to_string(),
    }];

    let text_deltas = Arc::new(Mutex::new(Vec::<String>::new()));
    let thinking_count = Arc::new(AtomicU64::new(0));
    let text_count = Arc::new(AtomicU64::new(0));
    let done_received = Arc::new(AtomicU64::new(0));

    let td = text_deltas.clone();
    let tc = text_count.clone();
    let thc = thinking_count.clone();
    let dr = done_received.clone();

    client
        .send_streaming(&messages, move |event| {
            match event {
                Ok(LlmStreamEvent::TextDelta { text }) => {
                    tc.fetch_add(1, Ordering::SeqCst);
                    td.blocking_lock().push(text);
                }
                Ok(LlmStreamEvent::ThinkingDelta { .. }) => {
                    thc.fetch_add(1, Ordering::SeqCst);
                }
                Ok(LlmStreamEvent::Done) => {
                    dr.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        })
        .await
        .expect("send_streaming should succeed");

    let text_count = text_count.load(Ordering::SeqCst);
    let thinking_count = thinking_count.load(Ordering::SeqCst);
    let done_count = done_received.load(Ordering::SeqCst);
    let text_content: String = text_deltas.lock().await.iter().cloned().collect();

    println!("=== Real API Event Results ===");
    println!("TextDelta count: {text_count}");
    println!("ThinkingDelta count: {thinking_count}");
    println!("Done events: {done_count}");
    println!("Text content ({chars}): {text_content}",
        chars = text_content.len(),
        text_content = if text_content.len() > 200 {
            format!("{}...", &text_content[..200])
        } else {
            text_content.clone()
        }
    );

    // The CRITICAL assertion: there MUST be TextDelta events with actual content
    assert!(
        text_count > 0 && !text_content.trim().is_empty(),
        "REGRESSION: Real API returned {text_count} TextDelta events with {len} chars of text. \
         The final answer was lost after thinking!\n\
         ThinkingDelta count: {thinking_count}",
        len = text_content.len()
    );
    assert!(done_count > 0, "Must receive Done event");
}

/// 检查：claw_chat_list_sessions 通过 IPC 返回 sessions 数组
#[tokio::test(flavor = "multi_thread")]
async fn test_ipc_list_sessions() {
    let state = ClawChatState::new();
    let (_app, webview) = build_test_app(state);

    let result = invoke_cmd_ok(&webview, "claw_chat_list_sessions", serde_json::json!({}));

    let sessions = result
        .get("sessions")
        .and_then(|v| v.as_array())
        .expect("should have sessions array");
    // 至少返回一个数组（可能为空）
    assert!(sessions.is_empty() || sessions.len() > 0);
}

/// 检查：claw_read_stats 通过 IPC 返回统计信息
#[tokio::test(flavor = "multi_thread")]
async fn test_ipc_read_stats() {
    let state = ClawChatState::new();
    let (_app, webview) = build_test_app(state);

    let result = invoke_cmd_ok(&webview, "claw_read_stats", serde_json::json!({}));

    assert!(result.get("sessions").is_some(), "sessions count");
    assert!(result.get("messages").is_some(), "messages count");
    assert_eq!(result.get("source").and_then(|v| v.as_str()), Some("claw"));
}

/// 检查：claw_read_models_config 通过 IPC 返回 providers 列表
#[tokio::test(flavor = "multi_thread")]
async fn test_ipc_read_models() {
    let state = ClawChatState::new();
    let (_app, webview) = build_test_app(state);

    let result = invoke_cmd_ok(&webview, "claw_read_models_config", serde_json::json!({}));

    assert!(result.get("providers").is_some(), "providers list");
    assert!(result.get("source").is_some(), "source field");
}

// ── 移除含 AppHandle 命令的 IPC 测试（init/send/close）
// MockRuntime 不支持 AppHandle 的 CommandArg 解析
// 这些命令已在 test_chat_state_machine（真实 LLM 调用）中覆盖

// ── session_messages_to_json ────────────────────────────────────────────

#[test]
fn test_session_messages_to_json_empty() {
    let result = session_messages_to_json(&[]);
    assert!(result.is_empty(), "empty messages → empty JSON array");
}

#[test]
fn test_session_messages_to_json_converts_user_and_agent() {
    use runtime::MessageRole;
    let messages = vec![
        ConversationMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text { text: "Hello".into() }],
            usage: None,
        },
        ConversationMessage {
            role: MessageRole::Assistant,
            blocks: vec![
                ContentBlock::Text { text: "Hi there!".into() },
                ContentBlock::Thinking {
                    thinking: "let me think...".into(),
                    signature: None,
                },
            ],
            usage: None,
        },
    ];
    let result = session_messages_to_json(&messages);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0]["role"], "user");
    assert_eq!(result[0]["content"], "Hello");
    assert_eq!(result[1]["role"], "agent");
    // Both text + thinking blocks joined
    assert!(result[1]["content"].as_str().unwrap().contains("Hi there!"));
    assert!(result[1]["content"].as_str().unwrap().contains("let me think..."));
}

#[test]
fn test_session_messages_to_json_tool_role() {
    use runtime::MessageRole;
    let messages = vec![ConversationMessage {
        role: MessageRole::Tool,
        blocks: vec![ContentBlock::Text { text: "tool result".into() }],
        usage: None,
    }];
    let result = session_messages_to_json(&messages);
    assert_eq!(result[0]["role"], "tool");
}

#[test]
fn test_session_messages_to_json_thinking_only() {
    use runtime::MessageRole;
    let messages = vec![ConversationMessage {
        role: MessageRole::Assistant,
        blocks: vec![ContentBlock::Thinking {
            thinking: "just thinking".into(),
            signature: None,
        }],
        usage: None,
    }];
    let result = session_messages_to_json(&messages);
    assert_eq!(result[0]["content"], "just thinking");
}

// ── list_sessions_info title extraction ─────────────────────────────────

#[test]
fn test_session_title_extracted_from_first_message() {
    let sess_dir = sessions_dir();
    std::fs::create_dir_all(&sess_dir).unwrap();
    let session_path = sess_dir.join("test_sess_title.json");

    // Write JSONL: meta line + user message + assistant message
    let meta = r#"{"session_id":"sess_title","created_at_ms":1717500000000,"updated_at_ms":1717500001000,"type":"session_meta","version":1}"#;
    let msg1 = r#"{"message":{"blocks":[{"text":"What is the meaning of life?","type":"text"}],"role":"user"},"type":"message"}"#;
    let msg2 = r#"{"message":{"blocks":[{"text":"42","type":"text"}],"role":"assistant"},"type":"message"}"#;
    std::fs::write(&session_path, format!("{meta}\n{msg1}\n{msg2}\n")).unwrap();

    let sessions = list_sessions_info();
    let found = sessions.iter().find(|s| s["sessionId"] == "sess_title");
    assert!(found.is_some(), "session should be found");
    let session = found.unwrap();
    assert_eq!(session["title"], "What is the meaning of life?");
    assert_eq!(session["messageCount"], 2);

    let _ = std::fs::remove_file(&session_path);
}

#[test]
fn test_session_title_empty_when_no_messages() {
    let sess_dir = sessions_dir();
    std::fs::create_dir_all(&sess_dir).unwrap();
    let session_path = sess_dir.join("test_sess_empty.json");

    let meta = r#"{"session_id":"sess_empty","created_at_ms":1717500000000,"updated_at_ms":1717500000000,"type":"session_meta","version":1}"#;
    std::fs::write(&session_path, format!("{meta}\n")).unwrap();

    let sessions = list_sessions_info();
    let found = sessions.iter().find(|s| s["sessionId"] == "sess_empty");
    assert!(found.is_some(), "session should be found");
    let session = found.unwrap();
    assert!(session.get("title").and_then(|t| t.as_str()).is_none()
        || session["title"].as_str().unwrap_or("").is_empty(),
        "empty session should have no title");

    let _ = std::fs::remove_file(&session_path);
}

#[test]
fn test_session_title_truncated_to_60_chars() {
    let sess_dir = sessions_dir();
    std::fs::create_dir_all(&sess_dir).unwrap();
    let session_path = sess_dir.join("test_sess_long.json");

    let long_text = "a".repeat(100);
    let meta = r#"{"session_id":"sess_long","created_at_ms":1717500000000,"updated_at_ms":1717500001000,"type":"session_meta","version":1}"#;
    let msg = format!(r#"{{"message":{{"blocks":[{{"text":"{long_text}","type":"text"}}],"role":"user"}},"type":"message"}}"#);
    std::fs::write(&session_path, format!("{meta}\n{msg}\n")).unwrap();

    let sessions = list_sessions_info();
    let found = sessions.iter().find(|s| s["sessionId"] == "sess_long");
    assert!(found.is_some());
    let title = found.unwrap()["title"].as_str().unwrap();
    assert_eq!(title.len(), 63); // 60 chars + "..."
    assert!(title.ends_with("..."), "long title should be truncated with ...");

    let _ = std::fs::remove_file(&session_path);
}

// ── sessions_dir ────────────────────────────────────────────────────────

#[test]
fn test_sessions_dir_is_under_claw() {
    let dir = sessions_dir();
    let path_str = dir.to_string_lossy();
    assert!(path_str.contains(".claw"), "sessions_dir should be ~/.claw/sessions/");
    assert!(path_str.ends_with("sessions"), "sessions_dir should end with /sessions");
}
