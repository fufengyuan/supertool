//! IPC-style tests for hermes_skills commands.

use crate::commands::hermes_skills::*;
use std::path::PathBuf;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::hermes_skills::list_installed_skills,
            crate::commands::hermes_skills::list_bundled_skills,
            crate::commands::hermes_skills::get_skill_content,
            crate::commands::hermes_skills::install_skill,
            crate::commands::hermes_skills::uninstall_skill,
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
    // ── Pure-logic: trim_to_120 ──────────────────────────────────────────────
#[test]
fn test_trim_to_120_short_string() {
    assert_eq!(trim_to_120("hello"), "hello");
}
#[test]
fn test_trim_to_120_exact_boundary() {
    let s = "a".repeat(120);
    assert_eq!(trim_to_120(&s), s);
}
#[test]
fn test_trim_to_120_overflow() {
    let s = "a".repeat(130);
    let result = trim_to_120(&s);
    assert_eq!(result.len(), 120);
    assert!(result.ends_with("..."));
}
#[test]
fn test_trim_to_120_empty() {
    assert_eq!(trim_to_120(""), "");
}
#[test]
fn test_trim_to_120_multi_byte_chars() {
        // Use 2-byte unicode chars, fewer than the 120-byte boundary
    let s = "é".repeat(50); // 50 chars × 2 bytes = 100 bytes → no truncation
    let result = trim_to_120(&s);
    assert_eq!(result.len(), 100);
    assert_eq!(result, s);
}
    // ── Pure-logic: parse_skill_frontmatter ──────────────────────────────────
#[test]
fn test_parse_frontmatter_with_valid_yaml() {
    let content = r#"---
name: "test-skill"
description: "A test skill"
---
# Content"#;
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "test-skill");
    assert_eq!(desc, "A test skill");
}
#[test]
fn test_parse_frontmatter_with_single_quotes() {
    let content = r#"---
name: 'another-skill'
description: 'Another description'
---"#;
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "another-skill");
    assert_eq!(desc, "Another description");
}
#[test]
fn test_parse_frontmatter_without_quotes() {
    let content = r#"---
name: bare-name
description: bare description
---"#;
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "bare-name");
    assert_eq!(desc, "bare description");
}
#[test]
fn test_parse_frontmatter_no_yaml_delimiters() {
    let content = "# My Skill\n\nThis is a skill description.";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "My Skill");
    assert_eq!(desc, "This is a skill description.");
}
#[test]
fn test_parse_frontmatter_empty() {
    let (name, desc) = parse_skill_frontmatter("");
    assert_eq!(name, "");
    assert_eq!(desc, "");
}
#[test]
fn test_parse_frontmatter_only_yaml_no_content() {
    let content = "---\nname: lonely\n---";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "lonely");
    assert_eq!(desc, "");
}
#[test]
fn test_parse_frontmatter_missing_fields() {
    let content = r#"---
other: value
---"#;
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "");
    assert_eq!(desc, "");
}
#[test]
fn test_parse_frontmatter_long_description_is_truncated() {
    let long_desc = "a".repeat(200);
    let content = format!(
        r#"---
name: "skill"
description: "{}"
---"#,
        long_desc
    );
    let (name, desc) = parse_skill_frontmatter(&content);
    assert_eq!(name, "skill");
    assert_eq!(desc.len(), 120);
    assert!(desc.ends_with("..."));
}
#[test]
fn test_parse_frontmatter_fallback_no_heading() {
    let content = "Just a plain text without markdown heading.";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "");
    assert_eq!(desc, "Just a plain text without markdown heading.");
}
#[test]
fn test_parse_frontmatter_fallback_empty_lines_first() {
    let content = "\n\n\n# Heading\n\nSome description here.";
    let (name, desc) = parse_skill_frontmatter(content);
    assert_eq!(name, "Heading");
    assert_eq!(desc, "Some description here.");
}
    // ── Pure-logic: SkillInfo struct ─────────────────────────────────────────
#[test]
fn test_skill_info_serialization() {
    let skill = SkillInfo {
        name: "test".into(),
        category: "devops".into(),
        description: "A test skill".into(),
        path: "/tmp/test".into(),
        source: "installed".into(),
    };
    let json = serde_json::to_string(&skill).unwrap();
    assert!(json.contains("\"name\":\"test\""));
    assert!(json.contains("\"source\":\"installed\""));
}
#[test]
fn test_skill_info_deserialization() {
    let json = r#"{"name":"test","category":"devops","description":"desc","path":"/p","source":"bundled"}"#;
    let skill: SkillInfo = serde_json::from_str(json).unwrap();
    assert_eq!(skill.name, "test");
    assert_eq!(skill.source, "bundled");
}
    // ── Pure-logic: SkillCliResult struct ────────────────────────────────────
#[test]
fn test_skill_cli_result_success() {
    let result = SkillCliResult {
        success: true,
        error: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"success\":true"));
        // error: None serializes as null by default (no skip_serializing_if)
    assert!(json.contains("\"error\":null"));
}
#[test]
fn test_skill_cli_result_failure() {
    let result = SkillCliResult {
        success: false,
        error: Some("command not found".into()),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"success\":false"));
    assert!(json.contains("\"error\":\"command not found\""));
}
    // ── Pure-logic: Directory resolution ─────────────────────────────────────
#[test]
fn test_installed_skills_dir_ends_correctly() {
    let path = installed_skills_dir();
    let path_str = path.to_string_lossy();
    assert!(path_str.ends_with(".hermes/skills"));
}
#[test]
fn test_bundled_skills_dir_ends_correctly() {
    let path = bundled_skills_dir();
    let path_str = path.to_string_lossy();
    assert!(path_str.ends_with(".hermes/hermes-agent/skills"));
}
    // ── Pure-logic: scan_skills_dir with temp dirs ───────────────────────────
#[test]
fn test_scan_skills_dir_non_existent() {
    let bogus = PathBuf::from("/tmp/__nonexistent_skills_test__");
    let skills = scan_skills_dir(&bogus, "installed");
    assert!(skills.is_empty());
}
#[test]
fn test_scan_skills_dir_empty_dir() {
    let dir = std::env::temp_dir().join(format!("skills_test_empty_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let skills = scan_skills_dir(&dir, "installed");
    assert!(skills.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test]
fn test_scan_skills_dir_with_valid_skills() {
    let base = std::env::temp_dir().join(format!("skills_test_valid_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
        // Create: devops/git-skill/SKILL.md
    let devops_dir = base.join("devops/git-skill");
    std::fs::create_dir_all(&devops_dir).unwrap();
    std::fs::write(
        devops_dir.join("SKILL.md"),
        "---\nname: \"Git Helper\"\ndescription: \"Git operations\"\n---\n# Content",
    )
    .unwrap();
        // Create: mlops/train-skill/SKILL.md
    let mlops_dir = base.join("mlops/train-skill");
    std::fs::create_dir_all(&mlops_dir).unwrap();
    std::fs::write(
        mlops_dir.join("SKILL.md"),
        "---\nname: 'Trainer'\ndescription: 'ML training'\n---",
    )
    .unwrap();
        // Create: empty/ dir with no SKILL.md (should be skipped)
    let empty_cat = base.join("empty");
    std::fs::create_dir_all(empty_cat.join("no-skill")).unwrap();
    let skills = scan_skills_dir(&base, "installed");
    assert_eq!(skills.len(), 2);
        // Verify sorting: devops < mlops
    assert_eq!(skills[0].category, "devops");
    assert_eq!(skills[0].name, "Git Helper");
    assert_eq!(skills[0].source, "installed");
    assert_eq!(skills[1].category, "mlops");
    assert_eq!(skills[1].name, "Trainer");
    let _ = std::fs::remove_dir_all(&base);
}
#[test]
fn test_scan_skills_dir_skips_non_directories() {
    let base = std::env::temp_dir().join(format!("skills_test_files_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
        // Place a file directly in the base (should be skipped, not a dir)
    std::fs::write(base.join("somefile.md"), "content").unwrap();
    let skills = scan_skills_dir(&base, "installed");
    assert!(skills.is_empty());
    let _ = std::fs::remove_dir_all(&base);
}
#[test]
fn test_scan_skills_dir_fallback_when_no_frontmatter() {
    let base = std::env::temp_dir().join(format!("skills_test_nofm_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cat_dir = base.join("tools/my-tool");
    std::fs::create_dir_all(&cat_dir).unwrap();
    std::fs::write(cat_dir.join("SKILL.md"), "# My Tool\n\nUseful description.").unwrap();
    let skills = scan_skills_dir(&base, "bundled");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "My Tool");
    assert_eq!(skills[0].description, "Useful description.");
    assert_eq!(skills[0].source, "bundled");
    let _ = std::fs::remove_dir_all(&base);
}
    // ── Pure-logic: hermes_cli_path ──────────────────────────────────────────
#[test]
fn test_hermes_cli_path_returns_string() {
    let path = hermes_cli_path();
    assert!(!path.is_empty());
}
    // ── IPC: list_installed_skills ───────────────────────────────────────────
#[test]
fn test_ipc_list_installed_skills() {
    let (_app, ww) = build_test_app();
    let result: Vec<serde_json::Value> =
        invoke_ok(&ww, "list_installed_skills", serde_json::json!({}));
        // Should at least be an array (possibly empty if no skills installed)
    for skill in &result {
        assert!(
            skill.get("name").and_then(|v| v.as_str()).is_some(),
            "skill: name"
        );
        assert!(
            skill.get("category").and_then(|v| v.as_str()).is_some(),
            "skill: category"
        );
        assert!(
            skill.get("source").and_then(|v| v.as_str()).is_some(),
            "skill: source"
        );
        assert!(
            skill.get("path").and_then(|v| v.as_str()).is_some(),
            "skill: path"
        );
    }
}
    // ── IPC: list_bundled_skills ─────────────────────────────────────────────
#[test]
fn test_ipc_list_bundled_skills() {
    let (_app, ww) = build_test_app();
    let result: Vec<serde_json::Value> =
        invoke_ok(&ww, "list_bundled_skills", serde_json::json!({}));
        // Should at least be an array (possibly empty if no bundles exist)
    for skill in &result {
        assert!(
            skill.get("name").and_then(|v| v.as_str()).is_some(),
            "skill: name"
        );
        assert!(
            skill.get("category").and_then(|v| v.as_str()).is_some(),
            "skill: category"
        );
        assert!(
            skill.get("source").and_then(|v| v.as_str()).is_some(),
            "skill: source"
        );
    }
}
    // ── IPC: get_skill_content ───────────────────────────────────────────────
#[test]
fn test_ipc_get_skill_content() {
    let (_app, ww) = build_test_app();
        // Create a temp skill directory with a SKILL.md
    let tmp = std::env::temp_dir().join(format!("ipc_skill_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("SKILL.md"), "# Test Skill\n\nHello world!").unwrap();
    let content: String = invoke_ok(
        &ww,
        "get_skill_content",
        serde_json::json!({ "path": tmp.to_string_lossy() }),
    );
    assert_eq!(content, "# Test Skill\n\nHello world!");
    let _ = std::fs::remove_dir_all(&tmp);
}
#[test]
fn test_ipc_get_skill_content_nonexistent() {
    let (_app, ww) = build_test_app();
    let content: String = invoke_ok(
        &ww,
        "get_skill_content",
        serde_json::json!({ "path": "/tmp/__no_such_skill_test__" }),
    );
    assert_eq!(content, "");
}
    // ── IPC: install_skill ───────────────────────────────────────────────────
#[test]
fn test_ipc_install_skill() {
    let (_app, ww) = build_test_app();
    let result: SkillCliResult = invoke_ok(
        &ww,
        "install_skill",
        serde_json::json!({ "identifier": "test-skill" }),
    );
        // May succeed or fail depending on whether `hermes` CLI is available;
        // but must return a valid SkillCliResult
    assert!(
        result.success || result.error.is_some(),
        "expected either success or error"
    );
}
    // ── IPC: uninstall_skill ─────────────────────────────────────────────────
#[test]
fn test_ipc_uninstall_skill() {
    let (_app, ww) = build_test_app();
    let result: SkillCliResult = invoke_ok(
        &ww,
        "uninstall_skill",
        serde_json::json!({ "identifier": "test-skill" }),
    );
        // May succeed or fail depending on whether `hermes` CLI is available
    assert!(
        result.success || result.error.is_some(),
        "expected either success or error"
    );
}
