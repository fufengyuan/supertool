//! IPC-style tests for hermes_memory commands.

use crate::commands::hermes_memory::*;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
};
use serde_json::json;
    /// Serial helper so parallel test runs don't clobber each other's env vars.
static TEST_MUTEX: Mutex<()> = Mutex::new(());
    /// Helper: set env var inside unsafe block.
unsafe fn set_env_var(key: &str, val: &str) {
    unsafe { std::env::set_var(key, val) }
}
use std::fs;
unsafe fn remove_env_var(key: &str) {
    unsafe { std::env::remove_var(key) }
}
    /// Lock the test mutex, recovering from any previous poison.
fn lock_test() -> std::sync::MutexGuard<'static, ()> {
    TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}
fn with_temp_home<F>(f: F)
where
    F: FnOnce(&std::path::Path),
{
    let _lock = lock_test();
    let dir = std::env::temp_dir().join(format!("hermes_memory_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    unsafe { set_env_var("HERMES_HOME", &dir.to_string_lossy()); }
    f(&dir);
    let _ = fs::remove_dir_all(&dir);
}
    // ── hermes_home ───────────────────────────────────────
#[test]
fn should_use_hermes_home_env_when_set() {
    with_temp_home(|tmp| {
        let home = hermes_home();
        assert_eq!(home, tmp);
    });
}
#[test]
fn should_fallback_to_home_dir_when_no_env() {
    let _lock = lock_test();
    unsafe { remove_env_var("HERMES_HOME"); }
    let home = hermes_home();
    let expected = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".hermes");
    assert_eq!(home, expected);
}
    // ── read_file_safe ────────────────────────────────────
#[test]
fn should_return_empty_for_missing_file() {
    with_temp_home(|tmp| {
        let missing = tmp.join("nonexistent.txt");
        let (content, exists, modified) = read_file_safe(&missing);
        assert_eq!(content, "");
        assert!(!exists);
        assert!(modified.is_none());
    });
}
#[test]
fn should_read_existing_file_content() {
    with_temp_home(|tmp| {
        let file = tmp.join("test_read.md");
        fs::write(&file, "hello world").unwrap();
        let (content, exists, modified) = read_file_safe(&file);
        assert_eq!(content, "hello world");
        assert!(exists);
        assert!(modified.is_some());
    });
}
#[test]
fn should_read_empty_file() {
    with_temp_home(|tmp| {
        let file = tmp.join("empty.md");
        fs::write(&file, "").unwrap();
        let (content, exists, _) = read_file_safe(&file);
        assert_eq!(content, "");
        assert!(exists);
    });
}
#[test]
fn should_read_large_file() {
    with_temp_home(|tmp| {
        let file = tmp.join("large.md");
        let large = "A".repeat(10_000);
        fs::write(&file, &large).unwrap();
        let (content, exists, _) = read_file_safe(&file);
        assert!(exists);
        assert_eq!(content.len(), 10_000);
        assert_eq!(content, large);
    });
}
    // ── write_file_atomic ──────────────────────────────────
#[test]
fn should_write_file_atomically() {
    with_temp_home(|tmp| {
        let file = tmp.join("atomic.md");
        write_file_atomic(&file, "atomic content").unwrap();
        assert!(file.exists());
        let read = fs::read_to_string(&file).unwrap();
        assert_eq!(read, "atomic content");
            // temp file should be gone
        assert!(!file.with_extension("md.tmp").exists());
    });
}
#[test]
fn should_create_parent_dir_when_writing() {
    with_temp_home(|tmp| {
        let nested = tmp.join("sub/deep/file.md");
        write_file_atomic(&nested, "nested").unwrap();
        assert!(nested.exists());
        assert_eq!(fs::read_to_string(&nested).unwrap(), "nested");
    });
}
#[test]
fn should_overwrite_existing_file() {
    with_temp_home(|tmp| {
        let file = tmp.join("overwrite.md");
        fs::write(&file, "old").unwrap();
        write_file_atomic(&file, "new").unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "new");
    });
}
    // ── parse_memory_entries ──────────────────────────────
#[test]
fn should_parse_no_entries_from_empty_string() {
    let entries = parse_memory_entries("");
    assert!(entries.is_empty());
}
#[test]
fn should_parse_no_entries_from_whitespace() {
    let entries = parse_memory_entries("   \n  \t  ");
    assert!(entries.is_empty());
}
#[test]
fn should_parse_single_entry() {
    let entries = parse_memory_entries("hello world");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].index, 0);
    assert_eq!(entries[0].content, "hello world");
}
#[test]
fn should_parse_multiple_entries() {
    let entries = parse_memory_entries("entry one\n§\nentry two\n§\nentry three");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].content, "entry one");
    assert_eq!(entries[1].content, "entry two");
    assert_eq!(entries[2].content, "entry three");
}
#[test]
fn should_trim_entry_content() {
    let entries = parse_memory_entries("  spaced  \n§\n  padded  ");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].content, "spaced");
    assert_eq!(entries[1].content, "padded");
}
#[test]
fn should_skip_empty_entries() {
    let entries = parse_memory_entries("first\n§\n\n§\nthird");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].content, "first");
    assert_eq!(entries[1].content, "third");
}
#[test]
fn should_assign_sequential_indices() {
    let entries = parse_memory_entries("a\n§\nb\n§\nc");
    assert_eq!(entries.len(), 3);
    for (i, e) in entries.iter().enumerate() {
        assert_eq!(e.index, i as i32);
    }
}
#[test]
fn should_handle_only_delimiter() {
    let entries = parse_memory_entries("§");
    assert_eq!(entries.len(), 1);
        // "§" itself isn't trimmed to empty since it's a non-whitespace char
    assert_eq!(entries[0].content, "§");
}
#[test]
fn should_handle_trailing_delimiter() {
    let entries = parse_memory_entries("first\n§\n");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "first");
}
    // ── serialize_entries ─────────────────────────────────
#[test]
fn should_serialize_empty_list() {
    let s = serialize_entries(&[]);
    assert_eq!(s, "");
}
#[test]
fn should_serialize_single_entry() {
    let s = serialize_entries(&[MemoryEntry {
        index: 0,
        content: "only one".to_string(),
    }]);
    assert_eq!(s, "only one");
}
#[test]
fn should_serialize_multiple_entries() {
    let entries = vec![
        MemoryEntry { index: 0, content: "a".to_string() },
        MemoryEntry { index: 1, content: "b".to_string() },
        MemoryEntry { index: 2, content: "c".to_string() },
    ];
    let s = serialize_entries(&entries);
    assert_eq!(s, "a\n§\nb\n§\nc");
}
#[test]
fn should_parse_serialize_roundtrip() {
    let original = "alpha\n§\nbeta\n§\ngamma";
    let entries = parse_memory_entries(original);
    let serialized = serialize_entries(&entries);
    assert_eq!(serialized, original);
}
#[test]
fn should_preserve_unicode_in_roundtrip() {
    let original = "你好\n§\n中文\n§\n🌍 world";
    let entries = parse_memory_entries(original);
    let serialized = serialize_entries(&entries);
    assert_eq!(serialized, original);
}
#[test]
fn should_handle_special_chars_in_content() {
    let entries = parse_memory_entries("line1\n§\nline2 with § inside\n§\nline3");
        // The § inside content won't confuse parser since we split on § with trailing newline
    assert_eq!(entries.len(), 3);
    assert!(entries[1].content.contains("§"));
}
    // ── check_provider_installed ──────────────────────────
#[test]
fn should_return_false_when_no_env_var_set() {
    let _lock = lock_test();
    unsafe { remove_env_var("MEMORY_TEST_API_KEY"); }
    let result = check_provider_installed(&["MEMORY_TEST_API_KEY"]);
    assert!(!result);
}
#[test]
fn should_return_true_when_env_var_set() {
    let _lock = lock_test();
    unsafe { set_env_var("MEMORY_TEST_API_KEY", "test-value"); }
    let result = check_provider_installed(&["MEMORY_TEST_API_KEY"]);
    assert!(result);
    unsafe { remove_env_var("MEMORY_TEST_API_KEY"); }
}
#[test]
fn should_return_true_when_any_env_var_set() {
    let _lock = lock_test();
    unsafe { remove_env_var("FIRST_KEY"); }
    unsafe { set_env_var("SECOND_KEY", "value"); }
    let result = check_provider_installed(&["FIRST_KEY", "SECOND_KEY"]);
    assert!(result);
    unsafe { remove_env_var("SECOND_KEY"); }
}
#[test]
fn should_return_false_when_no_env_keys_provided() {
    let result = check_provider_installed(&[]);
    assert!(!result);
}
    // ── get_session_stats ─────────────────────────────────
#[test]
fn should_return_zero_stats_when_no_state_db() {
    with_temp_home(|tmp| {
        let stats = get_session_stats();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_messages, 0);
    });
}
    // ── read_memory command ────────────────────────────────
#[test]
fn should_read_memory_when_files_dont_exist() {
    with_temp_home(|tmp| {
        let info = read_memory().unwrap();
        assert!(!info.memory.exists);
        assert_eq!(info.memory.content, "");
        assert!(info.memory.entries.is_empty());
        assert_eq!(info.memory.char_count, 0);
        assert_eq!(info.memory.char_limit, MEMORY_CHAR_LIMIT);
        assert!(!info.user.exists);
        assert_eq!(info.user.content, "");
        assert_eq!(info.user.char_count, 0);
        assert_eq!(info.user.char_limit, USER_CHAR_LIMIT);
        assert_eq!(info.stats.total_sessions, 0);
        assert_eq!(info.stats.total_messages, 0);
    });
}
#[test]
fn should_read_memory_with_content() {
    with_temp_home(|tmp| {
        let mem_file = tmp.join("memories").join("MEMORY.md");
        fs::create_dir_all(mem_file.parent().unwrap()).unwrap();
        fs::write(&mem_file, "entry1\n§\nentry2").unwrap();
        let user_file = tmp.join("memories").join("USER.md");
        fs::write(&user_file, "user profile").unwrap();
        let info = read_memory().unwrap();
        assert!(info.memory.exists);
        assert_eq!(info.memory.entries.len(), 2);
        assert_eq!(info.memory.entries[0].content, "entry1");
        assert_eq!(info.memory.entries[1].content, "entry2");
        assert!(info.user.exists);
        assert_eq!(info.user.content, "user profile");
    });
}
    // ── add_memory_entry ──────────────────────────────────
#[test]
fn should_add_first_entry() {
    with_temp_home(|tmp| {
        let result = add_memory_entry("first entry".to_string());
        assert!(result.success);
        assert!(result.error.is_none());
        let mem_file = tmp.join("memories").join("MEMORY.md");
        assert!(mem_file.exists());
        let content = fs::read_to_string(&mem_file).unwrap();
        assert_eq!(content, "first entry");
    });
}
#[test]
fn should_append_entry_to_existing() {
    with_temp_home(|tmp| {
        add_memory_entry("first".to_string());
        let result = add_memory_entry("second".to_string());
        assert!(result.success);
        let mem_file = tmp.join("memories").join("MEMORY.md");
        let content = fs::read_to_string(&mem_file).unwrap();
        let entries = parse_memory_entries(&content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].content, "second");
    });
}
#[test]
fn should_reject_entry_exceeding_char_limit() {
    with_temp_home(|_tmp| {
        let oversized = "X".repeat(MEMORY_CHAR_LIMIT + 1);
        let result = add_memory_entry(oversized);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("limit"));
    });
}
#[test]
fn should_trim_entry_content_on_persist() {
    with_temp_home(|_tmp| {
        let result = add_memory_entry("  padded entry  ".to_string());
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.memory.entries[0].content, "padded entry");
    });
}
#[test]
fn should_accept_empty_string_entry() {
    with_temp_home(|_tmp| {
        let result = add_memory_entry("".to_string());
        assert!(result.success);
        let info = read_memory().unwrap();
        assert!(info.memory.entries.is_empty()); // empty trimmed string = no entry
    });
}
    // ── update_memory_entry ───────────────────────────────
#[test]
fn should_update_existing_entry() {
    with_temp_home(|_tmp| {
        add_memory_entry("original".to_string());
        let result = update_memory_entry(0, "updated".to_string());
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.memory.entries[0].content, "updated");
    });
}
#[test]
fn should_fail_to_update_nonexistent_entry() {
    with_temp_home(|_tmp| {
        let result = update_memory_entry(0, "anything".to_string());
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Entry not found");
    });
}
#[test]
fn should_fail_to_update_with_negative_index() {
    with_temp_home(|_tmp| {
        add_memory_entry("only one".to_string());
        let result = update_memory_entry(-1, "test".to_string());
        assert!(!result.success);
    });
}
#[test]
fn should_reject_update_exceeding_char_limit() {
    with_temp_home(|_tmp| {
        add_memory_entry("small".to_string());
        let oversized = "X".repeat(MEMORY_CHAR_LIMIT + 1);
        let result = update_memory_entry(0, oversized);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("limit"));
    });
}
    // ── remove_memory_entry ───────────────────────────────
#[test]
fn should_remove_entry() {
    with_temp_home(|_tmp| {
        add_memory_entry("first".to_string());
        add_memory_entry("second".to_string());
        add_memory_entry("third".to_string());
        let result = remove_memory_entry(1);
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.memory.entries.len(), 2);
        assert_eq!(info.memory.entries[0].content, "first");
        assert_eq!(info.memory.entries[1].content, "third");
    });
}
#[test]
fn should_fail_to_remove_nonexistent_entry() {
    with_temp_home(|_tmp| {
        let result = remove_memory_entry(0);
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Entry not found");
    });
}
#[test]
fn should_fail_to_remove_with_negative_index() {
    with_temp_home(|_tmp| {
        add_memory_entry("only one".to_string());
        let result = remove_memory_entry(-1);
        assert!(!result.success);
    });
}
#[test]
fn should_remove_last_entry_gracefully() {
    with_temp_home(|_tmp| {
        add_memory_entry("sole entry".to_string());
        let result = remove_memory_entry(0);
        assert!(result.success);
        let info = read_memory().unwrap();
        assert!(info.memory.entries.is_empty());
        assert_eq!(info.memory.content, "");
    });
}
    // ── write_user_profile ────────────────────────────────
#[test]
fn should_write_user_profile() {
    with_temp_home(|tmp| {
        let result = write_user_profile("my profile".to_string());
        assert!(result.success);
        let user_file = tmp.join("memories").join("USER.md");
        assert!(user_file.exists());
        assert_eq!(fs::read_to_string(&user_file).unwrap(), "my profile");
    });
}
#[test]
fn should_reject_user_profile_exceeding_limit() {
    with_temp_home(|_tmp| {
        let oversized = "X".repeat(USER_CHAR_LIMIT + 1);
        let result = write_user_profile(oversized);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("limit"));
    });
}
#[test]
fn should_write_empty_user_profile() {
    with_temp_home(|tmp| {
        let result = write_user_profile("".to_string());
        assert!(result.success);
        let user_file = tmp.join("memories").join("USER.md");
        assert!(user_file.exists());
        assert_eq!(fs::read_to_string(&user_file).unwrap(), "");
    });
}
#[test]
fn should_overwrite_existing_user_profile() {
    with_temp_home(|_tmp| {
        write_user_profile("old".to_string());
        let result = write_user_profile("new profile".to_string());
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.user.content, "new profile");
    });
}
    // ── list_memory_providers ─────────────────────────────
#[test]
fn should_list_all_known_providers() {
    with_temp_home(|_tmp| {
        let result = list_memory_providers();
        assert_eq!(result.providers.len(), 6);
        assert_eq!(result.providers[0].name, "honcho");
        assert_eq!(result.providers[5].name, "byterover");
        assert!(result.active_provider.is_empty());
    });
}
#[test]
fn should_read_active_provider_from_config() {
    with_temp_home(|tmp| {
        let cfg_dir = tmp.join("config.yaml");
        fs::write(&cfg_dir, "memory:\n  provider: mem0\n").unwrap();
        let result = list_memory_providers();
        assert_eq!(result.active_provider, "mem0");
        let mem0 = result.providers.iter().find(|p| p.name == "mem0").unwrap();
        assert!(mem0.active);
    });
}
#[test]
fn should_show_all_providers_as_inactive_when_no_config() {
    with_temp_home(|_tmp| {
        let result = list_memory_providers();
        assert!(result.active_provider.is_empty());
        for p in &result.providers {
            assert!(!p.active);
        }
    });
}
    // ── set_memory_provider ───────────────────────────────
#[test]
fn should_fail_to_set_provider_without_config() {
    with_temp_home(|_tmp| {
        let result = set_memory_provider("mem0".to_string());
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "Config file not found");
    });
}
#[test]
fn should_set_provider_in_existing_config() {
    with_temp_home(|tmp| {
        let cfg_dir = tmp.join("config.yaml");
        fs::write(&cfg_dir, "memory:\n  provider: honcho\n").unwrap();
        let result = set_memory_provider("retaindb".to_string());
        assert!(result.success);
            // Verify via list
        let providers = list_memory_providers();
        assert_eq!(providers.active_provider, "retaindb");
    });
}
#[test]
fn should_add_memory_section_when_missing() {
    with_temp_home(|tmp| {
        let cfg_dir = tmp.join("config.yaml");
        fs::write(&cfg_dir, "other_section: true\n").unwrap();
        let result = set_memory_provider("hindsight".to_string());
        assert!(result.success);
        let providers = list_memory_providers();
        assert_eq!(providers.active_provider, "hindsight");
    });
}
#[test]
fn should_clear_provider_with_empty_string() {
    with_temp_home(|tmp| {
        let cfg_dir = tmp.join("config.yaml");
        fs::write(&cfg_dir, "memory:\n  provider: honcho\n").unwrap();
        let result = set_memory_provider("".to_string());
        assert!(result.success);
        let providers = list_memory_providers();
        assert_eq!(providers.active_provider, "");
    });
}
#[test]
fn should_fail_to_set_provider_when_config_is_invalid_yaml() {
    with_temp_home(|tmp| {
        let cfg_dir = tmp.join("config.yaml");
        fs::write(&cfg_dir, "{ invalid: yaml: broken: ").unwrap();
        let result = set_memory_provider("mem0".to_string());
        assert!(!result.success);
        assert!(result.error.unwrap().contains("parse"));
    });
}
    // ── Boundary: large entries ───────────────────────────
#[test]
fn should_add_entry_at_exact_char_limit() {
    with_temp_home(|_tmp| {
        let exact = "A".repeat(MEMORY_CHAR_LIMIT);
        let result = add_memory_entry(exact.clone());
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.memory.char_count, MEMORY_CHAR_LIMIT);
        assert_eq!(info.memory.content, exact);
    });
}
#[test]
fn should_reject_user_profile_at_exact_limit_plus_one() {
    with_temp_home(|_tmp| {
        let oversized = "B".repeat(USER_CHAR_LIMIT + 1);
        let result = write_user_profile(oversized);
        assert!(!result.success);
    });
}
#[test]
fn should_accept_user_profile_at_exact_limit() {
    with_temp_home(|_tmp| {
        let exact = "C".repeat(USER_CHAR_LIMIT);
        let result = write_user_profile(exact);
        assert!(result.success);
        let info = read_memory().unwrap();
        assert_eq!(info.user.char_count, USER_CHAR_LIMIT);
    });
}
    // ── Provider installed check via env ──────────────────
#[test]
fn should_detect_installed_providers_via_env() {
    let _lock = lock_test();
    unsafe { set_env_var("HONCHO_API_KEY", "test-key"); }
    unsafe { remove_env_var("HIND_API_KEY"); }
    unsafe { remove_env_var("MEM0_API_KEY"); }
        // Override HERMES_HOME to some non-existent dir for list_memory_providers
    let dir = std::env::temp_dir().join(format!("hermes_prov_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    unsafe { set_env_var("HERMES_HOME", &dir.to_string_lossy()); }
    let result = list_memory_providers();
    assert!(result.providers[0].installed); // honcho
    assert!(!result.providers[1].installed); // hindsight
    assert!(!result.providers[2].installed); // mem0
    unsafe { remove_env_var("HONCHO_API_KEY"); }
    let _ = fs::remove_dir_all(&dir);
}
    // ── IPC-style tests via get_ipc_response ────────────────
fn build_test_app() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            crate::commands::hermes_memory::read_memory,
            crate::commands::hermes_memory::add_memory_entry,
            crate::commands::hermes_memory::update_memory_entry,
            crate::commands::hermes_memory::remove_memory_entry,
            crate::commands::hermes_memory::write_user_profile,
            crate::commands::hermes_memory::list_memory_providers,
            crate::commands::hermes_memory::set_memory_provider,
            crate::commands::hermes_memory::read_env_vars,
            crate::commands::hermes_memory::save_env_var,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app should build");
    let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview window should build");
    (app, ww)
}
fn invoke_ipc<R: serde::de::DeserializeOwned>(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<R, String> {
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
    match res {
        Ok(response) => response
            .deserialize::<R>()
            .map_err(|e| format!("deserialize error: {e:?}")),
        Err(e) => Err(format!("IPC error: {e:?}")),
    }
}
#[test]
fn test_ipc_mock_builder_creates_app() {
    with_temp_home(|_tmp| {
        let (_app, _ww) = build_test_app();
            // No panic = success
    });
}
#[test]
fn test_ipc_read_memory_returns_defaults() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value =
            invoke_ipc(&ww, "read_memory", json!({}))
                .expect("read_memory should succeed");
        assert_eq!(result["memory"]["exists"], false);
        assert_eq!(result["memory"]["content"], "");
        assert!(result["memory"]["entries"].as_array().unwrap().is_empty());
        assert_eq!(
            result["memory"]["charLimit"],
            serde_json::json!(MEMORY_CHAR_LIMIT)
        );
        assert_eq!(result["user"]["exists"], false);
        assert_eq!(
            result["user"]["charLimit"],
            serde_json::json!(USER_CHAR_LIMIT)
        );
        assert_eq!(result["stats"]["totalSessions"], 0);
        assert_eq!(result["stats"]["totalMessages"], 0);
    });
}
#[test]
fn test_ipc_add_memory_entry_and_read_back() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
            // Add an entry via IPC
        let add_result: serde_json::Value = invoke_ipc(
            &ww,
            "add_memory_entry",
            json!({"content": "hello via IPC"}),
        )
        .expect("add_memory_entry should succeed");
        assert_eq!(add_result["success"], true);
            // Read back via IPC to verify persistence
        let info: serde_json::Value =
            invoke_ipc(&ww, "read_memory", json!({}))
                .expect("read_memory should succeed");
        let entries = info["memory"]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["content"], "hello via IPC");
    });
}
#[test]
fn test_ipc_add_memory_entry_rejects_oversized() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let oversized = "X".repeat(MEMORY_CHAR_LIMIT + 1);
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "add_memory_entry",
            json!({"content": oversized}),
        )
        .expect("add_memory_entry should return a value");
        assert_eq!(result["success"], false);
        let err = result["error"].as_str().unwrap_or("");
        assert!(err.contains("limit"), "error should mention limit, got: {err}");
    });
}
#[test]
fn test_ipc_update_memory_entry_via_ipc() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
            // Add first entry
        invoke_ipc::<serde_json::Value>(
            &ww,
            "add_memory_entry",
            json!({"content": "original"}),
        )
        .expect("add should succeed");
            // Update via IPC
        let upd: serde_json::Value = invoke_ipc(
            &ww,
            "update_memory_entry",
            json!({"index": 0, "content": "updated"}),
        )
        .expect("update should succeed");
        assert_eq!(upd["success"], true);
            // Verify
        let info: serde_json::Value =
            invoke_ipc(&ww, "read_memory", json!({}))
                .expect("read_memory should succeed");
        assert_eq!(info["memory"]["entries"][0]["content"], "updated");
    });
}
#[test]
fn test_ipc_update_nonexistent_entry_fails() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "update_memory_entry",
            json!({"index": 0, "content": "nope"}),
        )
        .expect("update should return a value");
        assert_eq!(result["success"], false);
        assert_eq!(result["error"], "Entry not found");
    });
}
#[test]
fn test_ipc_remove_memory_entry_via_ipc() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
            // Add two entries
        invoke_ipc::<serde_json::Value>(
            &ww,
            "add_memory_entry",
            json!({"content": "first"}),
        )
        .expect("add first");
        invoke_ipc::<serde_json::Value>(
            &ww,
            "add_memory_entry",
            json!({"content": "second"}),
        )
        .expect("add second");
            // Remove the second
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "remove_memory_entry",
            json!({"index": 1}),
        )
        .expect("remove should succeed");
        assert_eq!(result["success"], true);
            // Verify only first remains
        let info: serde_json::Value =
            invoke_ipc(&ww, "read_memory", json!({}))
                .expect("read_memory should succeed");
        let entries = info["memory"]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["content"], "first");
    });
}
#[test]
fn test_ipc_write_user_profile_via_ipc() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "write_user_profile",
            json!({"content": "IPC user profile"}),
        )
        .expect("write_user_profile should succeed");
        assert_eq!(result["success"], true);
            // Verify via read_memory IPC
        let info: serde_json::Value =
            invoke_ipc(&ww, "read_memory", json!({}))
                .expect("read_memory should succeed");
        assert_eq!(info["user"]["content"], "IPC user profile");
    });
}
#[test]
fn test_ipc_list_memory_providers_returns_six() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "list_memory_providers",
            json!({}),
        )
        .expect("list_memory_providers should succeed");
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 6);
        assert_eq!(providers[0]["name"], "honcho");
        assert_eq!(providers[5]["name"], "byterover");
        assert_eq!(result["activeProvider"], "");
        assert_eq!(result["memoryEnabled"], true);
    });
}
#[test]
fn test_ipc_set_memory_provider_via_ipc() {
    with_temp_home(|tmp| {
            // Need config.yaml first
        let cfg_file = tmp.join("config.yaml");
        fs::write(&cfg_file, "memory:\n  provider: honcho\n").unwrap();
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "set_memory_provider",
            json!({"provider": "mem0"}),
        )
        .expect("set_memory_provider should succeed");
        assert_eq!(result["success"], true);
            // Verify via list_memory_providers IPC
        let providers: serde_json::Value = invoke_ipc(
            &ww,
            "list_memory_providers",
            json!({}),
        )
        .expect("list should succeed");
        assert_eq!(providers["activeProvider"], "mem0");
    });
}
#[test]
fn test_ipc_set_memory_provider_fails_without_config() {
    with_temp_home(|_tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "set_memory_provider",
            json!({"provider": "mem0"}),
        )
        .expect("set_memory_provider should return a value");
        assert_eq!(result["success"], false);
        let err = result["error"].as_str().unwrap_or("");
        assert!(
            err.contains("Config file not found"),
            "expected Config file not found, got: {err}"
        );
    });
}
#[test]
fn test_ipc_read_env_vars_from_process() {
    let _lock = lock_test();
    unsafe { set_env_var("IPC_TEST_KEY", "ipc-test-value"); }
        // No with_temp_home needed — read_env_vars reads from process env
    let (_app, ww) = build_test_app();
    let result: serde_json::Value = invoke_ipc(
        &ww,
        "read_env_vars",
        json!({"keys": ["IPC_TEST_KEY", "NONEXISTENT_KEY"]}),
    )
    .expect("read_env_vars should succeed");
    assert_eq!(result["IPC_TEST_KEY"], "ipc-test-value");
    assert_eq!(result["NONEXISTENT_KEY"], "");
    unsafe { remove_env_var("IPC_TEST_KEY"); }
}
#[test]
fn test_ipc_save_env_var_writes_file() {
    with_temp_home(|tmp| {
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "save_env_var",
            json!({"key": "MY_VAR", "value": "my_value"}),
        )
        .expect("save_env_var should succeed");
        assert_eq!(result["success"], true);
        let env_file = tmp.join(".env");
        assert!(env_file.exists());
        let content = fs::read_to_string(&env_file).unwrap();
        assert!(
            content.contains("MY_VAR=my_value"),
            "expected .env to contain MY_VAR=my_value, got: {content}"
        );
    });
}
#[test]
fn test_ipc_save_env_var_updates_existing() {
    with_temp_home(|tmp| {
        let env_file = tmp.join(".env");
        fs::write(&env_file, "export OLD_VAR=old_value\n").unwrap();
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "save_env_var",
            json!({"key": "OLD_VAR", "value": "new_value"}),
        )
        .expect("save_env_var should succeed");
        assert_eq!(result["success"], true);
        let content = fs::read_to_string(&env_file).unwrap();
        assert!(content.contains("OLD_VAR=new_value"));
        assert!(!content.contains("OLD_VAR=old_value"));
    });
}
#[test]
fn test_ipc_save_env_var_removes_by_empty_value() {
    with_temp_home(|tmp| {
        let env_file = tmp.join(".env");
        fs::write(&env_file, "export TO_REMOVE=some_value\n").unwrap();
        let (_app, ww) = build_test_app();
        let result: serde_json::Value = invoke_ipc(
            &ww,
            "save_env_var",
            json!({"key": "TO_REMOVE", "value": ""}),
        )
        .expect("save_env_var should succeed");
        assert_eq!(result["success"], true);
        let content = fs::read_to_string(&env_file).unwrap();
        assert!(
            content.contains("(removed)"),
            "expected commented-out line, got: {content}"
        );
    });
}
