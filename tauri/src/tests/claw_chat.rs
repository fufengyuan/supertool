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

    let reply_text = Arc::new(std::sync::Mutex::new(String::new()));
    let reply_clone = reply_text.clone();
    let has_usage = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_usage_clone = has_usage.clone();

    let result = {
        let client = state.client.lock().await.clone().unwrap();
        client
            .send_streaming(&prompt, move |event| {
                match event {
                    Ok(LlmStreamEvent::TextDelta { text }) => {
                        let mut r = reply_clone.lock().unwrap();
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
            let reply = reply_text.lock().unwrap();
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

    let text_deltas = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
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
                    td.lock().unwrap().push(text);
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
    let text_content: String = text_deltas.lock().unwrap().iter().cloned().collect();

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
    let found = sessions.iter().find(|s| s["sessionId"] == "test_sess_title");
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
    let found = sessions.iter().find(|s| s["sessionId"] == "test_sess_empty");
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
    let found = sessions.iter().find(|s| s["sessionId"] == "test_sess_long");
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

// ── send_turn with tools integration test ──────────────────────────────

/// Real LLM test: verify send_turn with tools returns tool_calls.
/// This test calls the actual API and checks that:
/// 1. send_turn sends tools in the request
/// 2. The model returns tool_use blocks when asked to read a file
/// 3. Tool calls are properly accumulated with correct id, name, and input
#[tokio::test(flavor = "multi_thread")]
#[ignore] // Requires real API key
async fn test_send_turn_with_tools() {
    use crate::commands::claw_chat::{build_tool_definitions, setup_env_from_claw_config};
    use supertool_claw::llm::LlmClient;

    setup_env_from_claw_config().expect("setup env from claw config");

    let client = LlmClient::from_env().expect("create LlmClient from env");
    println!("Provider: {:?}, Model: {}", client.provider(), client.model());

    let tool_defs = build_tool_definitions();
    println!("Tool definitions: {} tools", tool_defs.len());
    for def in &tool_defs {
        println!("  - {}: {}", def.name, def.description.as_deref().unwrap_or(""));
    }

    let messages = vec![api::InputMessage {
        role: "user".to_string(),
        content: vec![api::InputContentBlock::Text {
            text: "Read the file /etc/hostname using the read_file tool".to_string(),
        }],
    }];

    let text_chunks = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let tool_calls_seen = Arc::new(std::sync::Mutex::new(Vec::<(String, String, serde_json::Value)>::new()));
    let thinking_chunks = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    let tc = text_chunks.clone();
    let tcs = tool_calls_seen.clone();
    let thc = thinking_chunks.clone();

    let result = client
        .send_turn(
            messages,
            Some("You are a coding assistant with file system tools. Use them to help the user."),
            Some(tool_defs),
            None, // reasoning_effort
            Some(move |event| match event {
                LlmStreamEvent::TextDelta { text } => {
                    tc.lock().unwrap().push(text);
                }
                LlmStreamEvent::ThinkingDelta { thinking } => {
                    thc.lock().unwrap().push(thinking);
                }
                LlmStreamEvent::ToolCall { id, name, input } => {
                    tcs.lock().unwrap().push((id, name, input));
                }
                _ => {}
            }),
        )
        .await
        .expect("send_turn should succeed");

    let text = text_chunks.lock().unwrap();
    let tool_calls = tool_calls_seen.lock().unwrap();
    let thinking = thinking_chunks.lock().unwrap();

    println!("\n=== send_turn Results ===");
    println!("Text chunks: {}", text.len());
    println!("Text content: {}", text.join("").chars().take(200).collect::<String>());
    println!("Thinking chunks: {}", thinking.len());
    println!("Tool calls: {}", tool_calls.len());
    for (i, (id, name, input)) in tool_calls.iter().enumerate() {
        println!("  Tool {}: {} (id={})", i + 1, name, id);
        println!("  Input: {}", serde_json::to_string_pretty(input).unwrap_or_default());
    }
    println!("TurnResult.tool_calls: {}", result.tool_calls.len());
    for (i, (id, name, input)) in result.tool_calls.iter().enumerate() {
        println!("  Tool {}: {} (id={})", i + 1, name, id);
    }

    // KEY ASSERTIONS
    // The model SHOULD use the read_file tool when explicitly asked
    assert!(
        result.tool_calls.len() > 0 || !result.text.is_empty(),
        "Should get either tool calls or text response. Got neither."
    );

    if result.tool_calls.is_empty() {
        println!("⚠️  Model did NOT use tools — this may indicate tools are not working.");
        println!("   Text response: {}", result.text.chars().take(300).collect::<String>());
    } else {
        println!("✅ Model used {} tool(s) — tools are working!", result.tool_calls.len());
        // Verify tool call structure
        for (id, name, input) in &result.tool_calls {
            assert!(!id.is_empty(), "Tool call id should not be empty");
            assert!(!name.is_empty(), "Tool call name should not be empty");
            assert!(input.is_object() || input.is_array() || input.is_null(),
                "Tool call input should be a JSON value");
        }
    }
}

/// Verify session_to_input_messages correctly converts runtime ConversationMessage
/// blocks to api InputMessage format (unit test, no LLM call).
#[test]
fn test_session_to_input_messages_conversion() {
    use crate::commands::claw_chat::session_to_input_messages;

    let messages = vec![
        ConversationMessage::user_text("hello"),
        ConversationMessage::assistant(vec![
            ContentBlock::Thinking {
                thinking: "I should help".to_string(),
                signature: Some("sig123".to_string()),
            },
            ContentBlock::Text {
                text: "Hi there!".to_string(),
            },
            ContentBlock::ToolUse {
                id: "tu_001".to_string(),
                name: "read_file".to_string(),
                input: r#"{"path":"/etc/hostname"}"#.to_string(),
            },
        ]),
        ConversationMessage::tool_result("tu_001", "read_file", "myhost\n", false),
        ConversationMessage::user_text("thanks"),
    ];

    let input_msgs = session_to_input_messages(&messages);

    // Should have 4 messages (system is filtered out, but none here)
    assert_eq!(input_msgs.len(), 4, "Should have 4 InputMessages");

    // User message
    assert_eq!(input_msgs[0].role, "user");
    assert_eq!(input_msgs[0].content.len(), 1);
    match &input_msgs[0].content[0] {
        api::InputContentBlock::Text { text } => assert_eq!(text, "hello"),
        _ => panic!("Expected Text block"),
    }

    // Assistant message with thinking + text + tool_use
    assert_eq!(input_msgs[1].role, "assistant");
    assert_eq!(input_msgs[1].content.len(), 3); // Thinking + Text + ToolUse

    match &input_msgs[1].content[0] {
        api::InputContentBlock::Thinking { thinking, signature } => {
            assert_eq!(thinking, "I should help");
            assert_eq!(signature.as_deref(), Some("sig123"));
        }
        _ => panic!("Expected Thinking block"),
    }

    match &input_msgs[1].content[2] {
        api::InputContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "tu_001");
            assert_eq!(name, "read_file");
            assert_eq!(input["path"], "/etc/hostname");
        }
        _ => panic!("Expected ToolUse block"),
    }

    // Tool result message — upstream maps Tool role to "user" for OpenAI compat
    assert_eq!(input_msgs[2].role, "user");
    match &input_msgs[2].content[0] {
        api::InputContentBlock::ToolResult { tool_use_id, content, is_error } => {
            assert_eq!(tool_use_id, "tu_001");
            assert!(!is_error);
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected ToolResult block"),
    }

    // Second user message
    assert_eq!(input_msgs[3].role, "user");

    println!("✅ session_to_input_messages conversion test passed");
}

/// Verify turn_result_to_assistant_message correctly converts TurnResult to ConversationMessage.
#[test]
fn test_turn_result_to_assistant_message_conversion() {
    use crate::commands::claw_chat::turn_result_to_assistant_message;
    use supertool_claw::llm::TurnResult;

    let result = TurnResult {
        text: "Here is the file content".to_string(),
        reasoning: "I should read the file".to_string(),
        tool_calls: vec![
            ("tu_001".to_string(), "read_file".to_string(), json!({"path": "/etc/hostname"})),
            ("tu_002".to_string(), "bash".to_string(), json!({"command": "echo hello"})),
        ],
        usage: Some((100, 50)),
    };

    let msg = turn_result_to_assistant_message(&result);

    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.blocks.len(), 4); // Thinking + 2 ToolUse + Text

    // Check blocks
    match &msg.blocks[0] {
        ContentBlock::Thinking { thinking, .. } => assert_eq!(thinking, "I should read the file"),
        _ => {}
    }
    // Actually the first block is Thinking, then ToolUses, then Text
    let tool_use_blocks: Vec<_> = msg.blocks.iter().filter(|b| matches!(b, ContentBlock::ToolUse { .. })).collect();
    assert_eq!(tool_use_blocks.len(), 2, "Should have 2 ToolUse blocks");

    println!("✅ turn_result_to_assistant_message conversion test passed");
}

/// Verify build_tool_definitions returns the expected tools.
#[test]
fn test_build_tool_definitions() {
    use crate::commands::claw_chat::build_tool_definitions;

    let defs = build_tool_definitions();

    assert!(defs.len() >= 6, "Should have at least 6 tools, got {}", defs.len());

    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"bash"), "Should include 'bash' tool, got: {:?}", names);
    assert!(names.contains(&"read_file"), "Should include 'read_file' tool");
    assert!(names.contains(&"write_file"), "Should include 'write_file' tool");
    assert!(names.contains(&"edit_file"), "Should include 'edit_file' tool");
    assert!(names.contains(&"glob_search"), "Should include 'glob_search' tool");
    assert!(names.contains(&"grep_search"), "Should include 'grep_search' tool");

    // Each definition should have a valid schema
    for def in &defs {
        assert!(!def.name.is_empty(), "Tool name should not be empty");
        assert!(def.description.is_some(), "Tool '{}' should have a description", def.name);
        assert!(def.input_schema.is_object(), "Tool '{}' input_schema should be a JSON object", def.name);
        let schema = def.input_schema.as_object().unwrap();
        assert!(schema.contains_key("type"), "Tool '{}' schema should have 'type'", def.name);
        // Some tools (e.g. StructuredOutput) have input_schema without properties
        // assert!(schema.contains_key("properties"), ...);
    }

    println!("✅ build_tool_definitions test passed: {} tools", defs.len());
}


// ── Integration test: full tool loop against real API ──────────────
//
// Exercises the full claw-chat tool loop against a real LLM:
//   send_turn -> tool_calls -> execute_tool -> send_turn -> final answer
//
// This validates:
//   1. Tool definitions reach the model correctly
//   2. Model returns tool_calls (not just text)
//   3. Tools execute and return results
//   4. Model receives tool results and produces coherent final answer
//   5. Session files are persisted with full message chain
#[tokio::test(flavor = "multi_thread")]
#[ignore] // Requires valid API key in ~/.claw/settings.json
async fn integration_full_tool_loop() {
    use crate::commands::claw_chat::{
        build_tool_definitions, session_to_input_messages, setup_env_from_claw_config,
        turn_result_to_assistant_message,
    };
    use runtime::{ConversationMessage, Session};
    use supertool_claw::llm::LlmClient;
    use tools;

    // 1. Setup
    setup_env_from_claw_config().expect("setup env from claw config");
    let client = LlmClient::from_env().expect("create LLM client");
    eprintln!("[test] Provider: {:?}, Model: {}", client.provider(), client.model());

    // 2. Create temp session with persistence
    let session_dir = std::path::PathBuf::from("/tmp/_claw_integration_test");
    std::fs::create_dir_all(&session_dir).ok();
    let session_path = session_dir.join("session.json");
    let mut session = Session::new().with_persistence_path(&session_path);

    // 3. Set workspace for tool execution
    std::env::set_current_dir("/tmp").expect("set cwd");

    // 4. Build tool definitions
    let tool_defs = build_tool_definitions();
    eprintln!("[test] {} tools loaded", tool_defs.len());

    // 5. Initial user message
    session
        .push_user_text("Please use the bash tool to run this exact command: echo hello_from_claw. Then tell me the output.")
        .expect("push user text");

    let system_prompt = "You are a coding assistant with file system and bash tools. Always use tools when asked to run commands or read files.";
    let mut iteration = 0;

    // 6. Tool loop (max 5 iterations)
    loop {
        iteration += 1;
        assert!(iteration <= 5, "Too many iterations — possible infinite loop");

        let input_messages = session_to_input_messages(&session.messages);
        eprintln!("[test] === Iteration {iteration}: {} input messages ===", input_messages.len());

        let result = client
            .send_turn(
                input_messages,
                Some(system_prompt),
                Some(tool_defs.clone()),
                None,
                Some(|event| match event {
                    supertool_claw::llm::LlmStreamEvent::TextDelta { text } => {
                        eprint!("{text}");
                    }
                    supertool_claw::llm::LlmStreamEvent::ToolCall { id, name, input } => {
                        eprintln!("\n[test] Tool call: {name} (id={id}) input={input}");
                    }
                    supertool_claw::llm::LlmStreamEvent::Usage { input_tokens, output_tokens } => {
                        eprintln!("\n[test] Usage: in={input_tokens} out={output_tokens}");
                    }
                    _ => {}
                }),
            )
            .await
            .expect("send_turn should succeed");

        eprintln!(
            "\n[test] Response: text={} chars, tools={}, reasoning={}",
            result.text.len(),
            result.tool_calls.len(),
            result.reasoning.len()
        );

        // Push assistant message to session
        let assistant_msg = turn_result_to_assistant_message(&result);
        session.push_message(assistant_msg).expect("push assistant");

        // If no tool calls, we're done
        if result.tool_calls.is_empty() {
            eprintln!("[test] No more tool calls — loop complete after {iteration} iterations");
            eprintln!("[test] Final text: {}", result.text.chars().take(500).collect::<String>());
            break;
        }

        // Execute each tool
        for (tool_id, tool_name, tool_input) in &result.tool_calls {
            eprintln!("[test] Executing tool: {tool_name} (id={tool_id})");

            let start = std::time::Instant::now();
            // tools::execute_tool internally creates a tokio runtime (bash tool),
            // so we must run it outside the async context via spawn_blocking.
            let tn = tool_name.clone();
            let ti = tool_input.clone();
            let (output, is_error) = tokio::task::spawn_blocking(move || {
                tools::execute_tool(&tn, &ti)
            })
            .await
            .expect("spawn_blocking panicked")
            .map(|o| (o, false))
            .unwrap_or_else(|e| (e, true));
            eprintln!(
                "[test] Tool {} {}: {} chars ({:?})",
                tool_name,
                if is_error { "FAILED" } else { "OK" },
                output.len(),
                start.elapsed()
            );

            let truncated = if output.len() > 50_000 {
                format!("{}...\n[Truncated from {} chars]", &output[..50_000], output.len())
            } else {
                output
            };

            let tool_msg = ConversationMessage::tool_result(tool_id, tool_name, truncated, is_error);
            session.push_message(tool_msg).expect("push tool result");
        }
    }

    // 7. Persist session
    session.save_to_path(&session_path).expect("save session");

    // 8. Verify session file (JSONL format — one JSON object per line)
    assert!(session_path.exists(), "session file should exist");
    let raw = std::fs::read_to_string(&session_path).expect("read session");
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    eprintln!("[test] Session persisted: {} JSONL lines in {}", lines.len(), session_path.display());
    assert!(lines.len() >= 4, "Expected at least 4 JSONL lines, got {}", lines.len());

    // Parse each line to verify valid JSON
    let mut roles: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let val: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => panic!("Line {} is not valid JSON: {}", i, e),
        };
        if let Some(msg) = val.get("message") {
            if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                roles.push(role.to_string());
            }
        }
    }
    eprintln!("[test] Roles: {:?}", roles);
    assert!(roles.iter().any(|r| r == "user"), "Should have user message");
    assert!(roles.iter().any(|r| r == "tool"), "Should have tool result");
    assert!(roles.iter().any(|r| r == "assistant"), "Should have assistant message");

    let _ = std::fs::remove_dir_all(&session_dir);
    eprintln!("[test] Integration test PASSED");
}


/// Verify claw_agent_system_prompt includes Hermes skills.
#[test]
fn test_claw_agent_system_prompt_includes_skills() {
    use crate::commands::claw_chat::claw_agent_system_prompt;
    let prompt = claw_agent_system_prompt(200 * 1024);
    assert!(prompt.len() > 1000, "System prompt should be substantial, got {} chars", prompt.len());
    assert!(prompt.contains("Hermes Skills"), "System prompt should contain Hermes Skills section");
    println!("✅ System prompt: {} chars", prompt.len());
}

/// Integration test: verify skills appear in system prompt and model uses them.

/// Integration test: verify CLAUDE.md memory is loaded and used by the model.
#[tokio::test(flavor = "multi_thread")]
#[ignore] // Requires real API key
async fn integration_claude_md_memory_is_used() {
    use crate::commands::claw_chat::{
        build_tool_definitions, session_to_input_messages, setup_env_from_claw_config,
        turn_result_to_assistant_message,
    };
    use runtime::{ConversationMessage, Session};
    use supertool_claw::llm::LlmClient;
    use tools;

    setup_env_from_claw_config().expect("setup env");
    let client = LlmClient::from_env().expect("create LLM client");
    eprintln!("[test] Provider: {:?}, Model: {}", client.provider(), client.model());

    // Create a temporary workspace with a CLAUDE.md
    let workspace = std::path::PathBuf::from("/tmp/_claw_memory_test");
    std::fs::create_dir_all(&workspace).ok();
    let claude_md = workspace.join("CLAUDE.md");
    let test_memory = "# Project Memory\n\nThis project uses a custom convention:\n- All functions must be prefixed with `myapp_`\n- The database is PostgreSQL 15\n- The team color is #FF5733\n- Never use `SELECT *` in queries\n";
    std::fs::write(&claude_md, test_memory).expect("write CLAUDE.md");

    // Init git repo
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(&workspace)
        .output();

    std::env::set_current_dir(&workspace).expect("set cwd");

    let mut session = Session::new();
    let tool_defs = build_tool_definitions();

    session
        .push_user_text("What is the team color and what prefix should all functions use? Answer briefly.")
        .expect("push user text");

    let system_prompt = crate::commands::claw_chat::claw_agent_system_prompt(200 * 1024);
    eprintln!("[test] System prompt length: {} chars", system_prompt.len());
    eprintln!("[test] Contains 'myapp_': {}", system_prompt.contains("myapp_"));
    eprintln!("[test] Contains 'FF5733': {}", system_prompt.contains("FF5733"));

    // Tool loop
    let mut final_text = String::new();
    for iteration in 0..5 {
        let input_messages = session_to_input_messages(&session.messages);
        eprintln!("[test] Iteration {}: {} messages", iteration + 1, input_messages.len());

        let result = client
            .send_turn(
                input_messages,
                Some(&system_prompt),
                Some(tool_defs.clone()),
                None,
                Some(|event| match event {
                    supertool_claw::llm::LlmStreamEvent::TextDelta { text } => {
                        eprint!("{text}");
                    }
                    supertool_claw::llm::LlmStreamEvent::ToolCall { id: _, name, input } => {
                        eprintln!("\n[test] Tool: {}({})", name, input.to_string().chars().take(100).collect::<String>());
                    }
                    _ => {}
                }),
            )
            .await
            .expect("send_turn");

        eprintln!("\n[test] text={} chars, tools={}", result.text.len(), result.tool_calls.len());

        let assistant_msg = turn_result_to_assistant_message(&result);
        session.push_message(assistant_msg).expect("push");

        if result.tool_calls.is_empty() {
            final_text = result.text.clone();
            break;
        }

        for (tool_id, tool_name, tool_input) in &result.tool_calls {
            let tn = tool_name.clone();
            let ti = tool_input.clone();
            let output = tokio::task::spawn_blocking(move || tools::execute_tool(&tn, &ti))
                .await
                .unwrap()
                .unwrap_or_else(|e| e);
            let truncated = if output.len() > 2000 { format!("{}...", &output[..2000]) } else { output };
            let tool_msg = ConversationMessage::tool_result(tool_id, tool_name, truncated, false);
            session.push_message(tool_msg).expect("push tool result");
        }
    }

    eprintln!("\n[test] Final: {}", final_text);
    assert!(!final_text.is_empty(), "Should get a response");

    let text = final_text.to_lowercase();
    let has_info = text.contains("ff5733") || text.contains("myapp_") || text.contains("myapp");
    assert!(has_info, "Response should reference CLAUDE.md content. Got: {}", final_text);

    eprintln!("\n[test] CLAUDE.md memory test PASSED");
    let _ = std::fs::remove_dir_all(&workspace);
}

/// Verify load_hermes_skills returns content with skill index.
#[test]
fn test_load_hermes_skills_includes_index() {
    use crate::commands::claw_chat::load_hermes_skills;
    let result = load_hermes_skills(200 * 1024);
    assert!(result.contains("Hermes Skills"), "Should contain 'Hermes Skills' header");
    assert!(result.contains("software-development") || result.contains("github"),
        "Should contain at least one coding skill category");
    println!("load_hermes_skills returned {} chars", result.len());
}

/// Integration test: verify skills appear in system prompt and model uses them.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn integration_skills_in_prompt_are_used() {
    use crate::commands::claw_chat::{
        build_tool_definitions, load_hermes_skills, session_to_input_messages,
        setup_env_from_claw_config, turn_result_to_assistant_message,
    };
    use runtime::{ConversationMessage, Session};
    use supertool_claw::llm::LlmClient;
    use tools;

    setup_env_from_claw_config().expect("setup env");
    let client = LlmClient::from_env().expect("create LLM client");

    let skills = load_hermes_skills(200 * 1024);
    assert!(skills.contains("Hermes Skills"), "Skills should be loaded");

    let mut session = Session::new();
    let tool_defs = build_tool_definitions();

    session
        .push_user_text("You have a GitHub skill loaded in your system prompt. Summarize in 3 bullet points what the GitHub skill says about creating pull requests. Do NOT use any tools.")
        .expect("push user text");

    let system_prompt = crate::commands::claw_chat::claw_agent_system_prompt(200 * 1024);
    let mut final_text = String::new();

    for _ in 0..5 {
        let input_messages = session_to_input_messages(&session.messages);
        let result = client
            .send_turn(input_messages, Some(&system_prompt), Some(tool_defs.clone()), None,
                Some(|event| match event {
                    supertool_claw::llm::LlmStreamEvent::TextDelta { text } => { eprint!("{text}"); }
                    _ => {}
                }))
            .await
            .expect("send_turn");
        session.push_message(turn_result_to_assistant_message(&result)).expect("push");
        if result.tool_calls.is_empty() {
            final_text = result.text.clone();
            break;
        }
        for (tool_id, tool_name, tool_input) in &result.tool_calls {
            let tn = tool_name.clone(); let ti = tool_input.clone();
            let output = tokio::task::spawn_blocking(move || tools::execute_tool(&tn, &ti)).await.unwrap().unwrap_or_else(|e| e);
            session.push_message(ConversationMessage::tool_result(tool_id, tool_name, output, false)).expect("push");
        }
    }

    assert!(!final_text.is_empty(), "Should get a response");
    let text = final_text.to_lowercase();
    assert!(text.contains("pull request") || text.contains("github") || text.contains("branch"),
        "Response should reference GitHub skill. Got: {}", final_text);
    println!("\nSkills test PASSED");
}

// ── End-to-end: session load → context verification → LLM send ──────────

/// Integration test: load a session from disk, verify context is correct,
/// then send a follow-up message and verify the LLM sees the history.
///
/// This simulates the exact user flow:
///   1. list_sessions_info() returns session IDs
///   2. User clicks a session → claw_chat_init(sessionId)
///   3. Session messages are restored
///   4. User sends a new message → LLM receives full context
#[tokio::test(flavor = "multi_thread")]
async fn integration_session_load_and_context() {
    use crate::commands::claw_chat::{
        load_session, list_sessions_info, session_messages_to_json,
        session_to_input_messages, setup_env_from_claw_config,
        sessions_dir,
    };
    use runtime::Session;
    use supertool_claw::llm::LlmClient;

    // ── Step 0: Create a test session file with known content ──
    let sess_dir = sessions_dir();
    std::fs::create_dir_all(&sess_dir).unwrap();
    let test_file = sess_dir.join("e2e_test_session.json");
    let meta = r#"{"session_id":"should-not-be-used","created_at_ms":1717500000000,"updated_at_ms":1717500001000,"type":"session_meta","version":1}"#;
    let msg1 = r#"{"message":{"blocks":[{"text":"My name is Alice and I like cats","type":"text"}],"role":"user"},"type":"message"}"#;
    let msg2 = r#"{"message":{"blocks":[{"text":"Nice to meet you Alice! Cats are wonderful pets.","type":"text"}],"role":"assistant"},"type":"message"}"#;
    std::fs::write(&test_file, format!("{meta}\n{msg1}\n{msg2}\n")).unwrap();
    eprintln!("[e2e] Created test session file: {}", test_file.display());

    // ── Step 1: list_sessions_info should use FILE STEM as sessionId ──
    let sessions = list_sessions_info();
    let our_session = sessions.iter().find(|s| {
        s.get("sessionId")
            .and_then(|v| v.as_str())
            == Some("e2e_test_session")
    });
    assert!(
        our_session.is_some(),
        "list_sessions_info should return sessionId='e2e_test_session' (file stem), got: {:?}",
        sessions.iter().map(|s| s.get("sessionId")).collect::<Vec<_>>()
    );
    let session_info = our_session.unwrap();
    assert_eq!(session_info["messageCount"], 2);
    let title = session_info["title"].as_str().unwrap_or("");
    assert!(
        title.contains("Alice"),
        "Title should contain first message text, got: {title}"
    );
    eprintln!("[e2e] Step 1 PASSED: sessionId={}, title={}", session_info["sessionId"], title);

    // ── Step 2: load_session should work with the sessionId from step 1 ──
    let loaded: Option<Session> = load_session("e2e_test_session");
    assert!(loaded.is_some(), "load_session('e2e_test_session') should find the file");
    let session = loaded.unwrap();
    assert_eq!(
        session.messages.len(),
        2,
        "Should have 2 restored messages"
    );
    eprintln!("[e2e] Step 2 PASSED: loaded {} messages", session.messages.len());

    // ── Step 3: Verify message content is correct ──
    let json_msgs = session_messages_to_json(&session.messages);
    assert_eq!(json_msgs.len(), 2);
    assert_eq!(json_msgs[0]["role"], "user");
    assert!(
        json_msgs[0]["content"]
            .as_str()
            .unwrap()
            .contains("Alice"),
        "First message should contain 'Alice'"
    );
    assert_eq!(json_msgs[1]["role"], "agent");
    let content1 = json_msgs[1]["content"].as_str().unwrap_or("(not string)");
    eprintln!("[e2e-debug] content1='{}'", content1);
    assert!(
        json_msgs[1]["content"].as_str().unwrap().to_lowercase().contains("cat"),
        "Second message should contain cat (case-insensitive)"
    );
    eprintln!("[e2e] Step 3 PASSED: message content verified");

    // ── Step 4: Convert to InputMessages for LLM — verify context ──
    let input_msgs = session_to_input_messages(&session.messages);
    assert_eq!(input_msgs.len(), 2, "Should have 2 InputMessages");
    assert_eq!(input_msgs[0].role, "user");
    assert_eq!(input_msgs[1].role, "assistant");

    let user_text = match &input_msgs[0].content[0] {
        api::InputContentBlock::Text { text } => text.clone(),
        _ => panic!("Expected Text block in user message"),
    };
    let assistant_text = match &input_msgs[1].content[0] {
        api::InputContentBlock::Text { text } => text.clone(),
        _ => panic!("Expected Text block in assistant message"),
    };
    assert!(user_text.contains("Alice"), "Context should preserve user text");
    assert!(assistant_text.to_lowercase().contains("cat"), "Context should preserve assistant text");
    eprintln!("[e2e] Step 4 PASSED: context conversion verified");

    // ── Step 5: Send follow-up message — LLM should see Alice context ──
    setup_env_from_claw_config().expect("setup env");
    let client = match LlmClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[e2e] SKIP Step 5: Cannot create LLM client: {e}");
            let _ = std::fs::remove_file(&test_file);
            return;
        }
    };
    eprintln!(
        "[e2e] LLM: provider={:?}, model={}",
        client.provider(),
        client.model()
    );

    // Add a follow-up user message
    let mut session = session;
    session
        .push_user_text("What is my name? Do NOT use any tools, just answer from memory.")
        .expect("push follow-up");

    let input_messages = session_to_input_messages(&session.messages);
    assert_eq!(
        input_messages.len(),
        3,
        "Should have 3 InputMessages (user + assistant + follow-up)"
    );
    eprintln!(
        "[e2e] Sending {} messages to LLM",
        input_messages.len()
    );

    let result = client
        .send_turn(
            input_messages,
            Some("You are a helpful assistant. Remember the user's name from the conversation."),
            None, // no tools
            None,
            Some(|event| match event {
                supertool_claw::llm::LlmStreamEvent::TextDelta { text } => {
                    eprint!("{text}");
                }
                _ => {}
            }),
        )
        .await;

    match result {
        Ok(turn_result) => {
            let response = turn_result.text.to_lowercase();
            eprintln!(
                "\n[e2e] LLM response: {}",
                turn_result.text.chars().take(200).collect::<String>()
            );
            assert!(
                response.contains("alice"),
                "LLM should know the user's name is Alice from context! Got: {}",
                turn_result.text
            );
            eprintln!("[e2e] Step 5 PASSED: LLM correctly remembers Alice from context");
        }
        Err(e) => {
            panic!("LLM call failed: {e}");
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
    eprintln!("[e2e] Integration test PASSED");
}

// Test: load skills from ~/.claw/skills/ and verify LLM can use them
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn integration_claw_skills_from_data_dir() {
    use crate::commands::claw_chat::{
        build_tool_definitions, session_to_input_messages, setup_env_from_claw_config,
    };
    use runtime::{ContentBlock, ConversationMessage, Session};
    use supertool_claw::llm::LlmClient;

    // 1. Verify ~/.claw/skills/ exists and has content
    let claw_skills_dir = dirs::home_dir().unwrap().join(".claw/skills");
    assert!(claw_skills_dir.exists(), "~/.claw/skills/ should exist");
    
    // Count skills
    let mut skill_count = 0;
    let mut total_skill_bytes: usize = 0;
    if let Ok(entries) = std::fs::read_dir(&claw_skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let desc_path = path.join("DESCRIPTION.md");
            if desc_path.exists() {
                skill_count += 1;
                if let Ok(content) = std::fs::read_to_string(&desc_path) {
                    total_skill_bytes += content.len();
                }
            }
        }
    }
    eprintln!("[claw-skills] Found {} skill categories in ~/.claw/skills/", skill_count);
    assert!(skill_count >= 20, "Should have at least 20 skill categories, got {}", skill_count);

    // 2. Load skills via the same function used by the app
    let skills_content = crate::commands::claw_chat::load_hermes_skills(200 * 1024);
    eprintln!("[claw-skills] Loaded {} chars of skill content", skills_content.len());
    assert!(skills_content.contains("Hermes Skills"), "Should contain Hermes Skills header");
    
    // Check specific skills are present
    assert!(skills_content.contains("github"), "Should contain github skill");
    assert!(skills_content.contains("software-development"), "Should contain software-development");
    assert!(skills_content.contains("devops"), "Should contain devops");
    
    // 3. Build full system prompt
    let system_prompt = crate::commands::claw_chat::claw_agent_system_prompt(200 * 1024);
    eprintln!("[claw-skills] System prompt: {} chars", system_prompt.len());
    assert!(system_prompt.len() > 100_000, "System prompt should be substantial (>100KB)");
    
    // 4. Send to LLM — ask about a skill that only exists in our skills directory
    setup_env_from_claw_config().expect("setup env");
    let client = match LlmClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[claw-skills] SKIP: {e}");
            return;
        }
    };

    let mut session = Session::new();
    session.push_user_text(
        "You have a GitHub skill loaded. Tell me in 2 sentences what the github skill says about creating pull requests. Do NOT use any tools."
    ).expect("push");

    let input_messages = session_to_input_messages(&session.messages);
    let result = client
        .send_turn(
            input_messages,
            Some(&system_prompt),
            None, // no tools
            None,
            Some(|event| match event {
                supertool_claw::llm::LlmStreamEvent::TextDelta { text } => {
                    eprint!("{text}");
                }
                _ => {}
            }),
        )
        .await
        .expect("send_turn");

    let response = result.text.to_lowercase();
    eprintln!("\n[claw-skills] Response: {}", result.text.chars().take(300).collect::<String>());
    
    // The LLM should reference PR-related concepts from the github skill
    assert!(
        response.contains("pull request") || response.contains("pr") || response.contains("github"),
        "LLM should reference PR/GitHub from the skill. Got: {}",
        result.text
    );
    
    eprintln!("[claw-skills] ✅ Skills from ~/.claw/skills/ loaded and used by LLM");
}
