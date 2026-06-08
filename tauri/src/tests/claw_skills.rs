//! IPC-style tests for claw_skills commands.

use crate::commands::claw_skills::*;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
#[test]
fn test_list_skills_returns_array() {
    let skills = claw_list_skills().unwrap_or_default();
    for s in &skills {
        assert!(!s.name.is_empty());
        assert!(!s.category.is_empty());
    }
        // Verify camelCase serialization
    if let Some(s) = skills.first() {
        let json = serde_json::to_value(s).unwrap();
        assert!(json.get("name").is_some());
        assert!(json.get("category").is_some());
        assert!(json.get("description").is_some());
        assert!(json.get("source").is_some());
    }
}
#[test]
fn test_parse_frontmatter_without_markers() {
    let content = "# MySkill\n\nThis is a description.\n";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "MySkill", "should extract heading as name");
    assert!(!desc.is_empty(), "should extract description");
}
#[test]
fn test_parse_frontmatter_with_yaml() {
    let content = "---\nname: \"FormalSkill\"\ndescription: \"A formally described skill\"\n---\nBody text here\n";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "FormalSkill");
    assert_eq!(desc, "A formally described skill");
}
#[test]
fn test_parse_frontmatter_empty() {
    let (name, desc) = parse_skill_frontmatter("");
    assert!(name.is_empty());
    assert!(desc.is_empty());
}
#[test]
fn test_trim_to_120_short() {
    let result = trim_to_120("short text");
    assert_eq!(result, "short text");
}
#[test]
fn test_trim_to_120_long() {
    let long = "a".repeat(200);
    let result = trim_to_120(&long);
    assert!(result.len() <= 120);
    assert!(result.ends_with("..."));
}
    // ── IPC 风格测试 ─────────────────────────────────────────────────
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::claw_skills::claw_list_skills,
            crate::commands::claw_skills::claw_get_skill_content,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
fn invoke_ok<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> R {
    let res = get_ipc_response(
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
    );
    res.unwrap_or_else(|e| panic!("IPC '{cmd}' failed: {e:?}"))
        .deserialize::<R>()
        .unwrap()
}
#[test]
fn test_ipc_list_skills() {
    let (_app, ww) = build_test_app();
    let result: Vec<serde_json::Value> =
        invoke_ok(&ww, "claw_list_skills", serde_json::json!({}));
    for s in &result {
        assert!(
            s.get("name").and_then(|v| v.as_str()).is_some(),
            "skill: name"
        );
        assert!(
            s.get("category").and_then(|v| v.as_str()).is_some(),
            "skill: category"
        );
        assert!(
            s.get("description").and_then(|v| v.as_str()).is_some(),
            "skill: description"
        );
        assert!(
            s.get("path").and_then(|v| v.as_str()).is_some(),
            "skill: path"
        );
        assert!(
            s.get("source").and_then(|v| v.as_str()).is_some(),
            "skill: source"
        );
    }
}
#[test]
fn test_ipc_get_skill_content_nonexistent() {
    let (_app, ww) = build_test_app();
    let result: String = invoke_ok(
        &ww,
        "claw_get_skill_content",
        serde_json::json!({"path": "/nonexistent/skill"}),
    );
    assert!(
        result.is_empty(),
        "nonexistent skill should return empty string"
    );
}
