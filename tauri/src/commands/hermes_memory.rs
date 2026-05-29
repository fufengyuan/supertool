//! Hermes Memory Management — pure Rust, no Python bridge
//!
//! Manages MEMORY.md and USER.md in ~/.hermes/memories/,
//! reads session/message stats from state.db,
//! and lists memory providers from config.yaml.
//!
//! Reference: hermes-desktop/src/main/memory.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const ENTRY_DELIMITER: &str = "\n§\n";
const MEMORY_CHAR_LIMIT: usize = 2200;
const USER_CHAR_LIMIT: usize = 1375;

// ── Data types ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub index: i32,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileInfo {
    pub content: String,
    pub exists: bool,
    pub last_modified: Option<i64>,
    pub entries: Vec<MemoryEntry>,
    pub char_count: usize,
    pub char_limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub total_sessions: i64,
    pub total_messages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub memory: MemoryFileInfo,
    pub user: MemoryFileInfo,
    pub stats: SessionStats,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProviderInfo {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub active: bool,
    pub env_vars: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProviderResult {
    pub providers: Vec<MemoryProviderInfo>,
    pub active_provider: String,
    pub memory_enabled: bool,
    pub user_profile_enabled: bool,
    pub memory_char_limit: usize,
    pub user_char_limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryWriteResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Paths ────────────────────────────────────────────────

fn hermes_home() -> PathBuf {
    if let Ok(home) = std::env::var("HERMES_HOME") {
        PathBuf::from(home)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".hermes")
    }
}

fn memories_dir() -> PathBuf {
    hermes_home().join("memories")
}

fn memory_path() -> PathBuf {
    memories_dir().join("MEMORY.md")
}

fn user_path() -> PathBuf {
    memories_dir().join("USER.md")
}

fn state_db_path() -> PathBuf {
    hermes_home().join("state.db")
}

fn config_path() -> PathBuf {
    hermes_home().join("config.yaml")
}

// ── File helpers ─────────────────────────────────────────

fn read_file_safe(file_path: &PathBuf) -> (String, bool, Option<i64>) {
    if !file_path.exists() {
        return (String::new(), false, None);
    }
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            let last_modified = file_path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            (content, true, last_modified)
        }
        Err(_) => (String::new(), false, None),
    }
}

fn write_file_atomic(file_path: &PathBuf, content: &str) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let tmp_path = file_path.with_extension("md.tmp");
    std::fs::write(&tmp_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    std::fs::rename(&tmp_path, file_path)
        .map_err(|e| format!("Failed to rename file: {}", e))?;
    Ok(())
}

fn parse_memory_entries(content: &str) -> Vec<MemoryEntry> {
    if content.trim().is_empty() {
        return Vec::new();
    }
    content
        .split(ENTRY_DELIMITER)
        .enumerate()
        .map(|(i, entry)| MemoryEntry {
            index: i as i32,
            content: entry.trim().to_string(),
        })
        .filter(|e| !e.content.is_empty())
        .collect()
}

fn serialize_entries(entries: &[MemoryEntry]) -> String {
    entries
        .iter()
        .map(|e| e.content.as_str())
        .collect::<Vec<&str>>()
        .join(ENTRY_DELIMITER)
}

fn get_session_stats() -> SessionStats {
    let db_path = state_db_path();
    if !db_path.exists() {
        return SessionStats {
            total_sessions: 0,
            total_messages: 0,
        };
    }

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let total_sessions: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE parent_session_id IS NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let total_messages: i64 = conn
                .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
                .unwrap_or(0);

            SessionStats {
                total_sessions,
                total_messages,
            }
        }
        Err(_) => SessionStats {
            total_sessions: 0,
            total_messages: 0,
        },
    }
}

// ── Known memory providers ───────────────────────────────

const KNOWN_PROVIDERS: &[(&str, &str, &[&str])] = &[
    ("honcho", "Managed memory via Honcho API — best for long-term memory persistence", &["HONCHO_API_KEY"]),
    ("hindsight", "Vectorize.io Hindsight — automatic memory recall and context injection", &["HIND_API_KEY"]),
    ("mem0", "Mem0 — self-improving memory layer for AI agents", &["MEM0_API_KEY"]),
    ("retaindb", "RetainDB — serverless memory with semantic search", &["RETAINDB_API_KEY", "RETAINDB_URL"]),
    ("supermemory", "SuperMemory — AI-powered memory manager with web integration", &["SUPERMEMORY_API_KEY"]),
    ("byterover", "ByteRover — cross-session memory router for long-running agents", &["BYTEROVER_API_KEY"]),
];

/// Check if a provider's env vars are set in the environment
fn check_provider_installed(env_keys: &[&str]) -> bool {
    env_keys.iter().any(|k| std::env::var(k).is_ok())
}

// ── Tauri Commands ───────────────────────────────────────

/// Read memory info: MEMORY.md content + entries + USER.md + stats
#[tauri::command(rename_all = "camelCase")]
pub fn read_memory() -> Result<MemoryInfo, String> {
    let mem_file = read_file_safe(&memory_path());
    let user_file = read_file_safe(&user_path());

    Ok(MemoryInfo {
        memory: MemoryFileInfo {
            content: mem_file.0.clone(),
            exists: mem_file.1,
            last_modified: mem_file.2,
            entries: parse_memory_entries(&mem_file.0),
            char_count: mem_file.0.len(),
            char_limit: MEMORY_CHAR_LIMIT,
        },
        user: MemoryFileInfo {
            content: user_file.0.clone(),
            exists: user_file.1,
            last_modified: user_file.2,
            entries: Vec::new(), // user file is freeform, not entry-based
            char_count: user_file.0.len(),
            char_limit: USER_CHAR_LIMIT,
        },
        stats: get_session_stats(),
    })
}

/// Add a new memory entry
#[tauri::command(rename_all = "camelCase")]
pub fn add_memory_entry(content: String) -> MemoryWriteResult {
    let file_path = memory_path();
    let (existing, _, _) = read_file_safe(&file_path);
    let mut entries = parse_memory_entries(&existing);
    entries.push(MemoryEntry {
        index: entries.len() as i32,
        content: content.trim().to_string(),
    });

    let new_content = serialize_entries(&entries);
    if new_content.len() > MEMORY_CHAR_LIMIT {
        return MemoryWriteResult {
            success: false,
            error: Some(format!(
                "Would exceed memory limit ({}/{})",
                new_content.len(),
                MEMORY_CHAR_LIMIT
            )),
        };
    }

    match write_file_atomic(&file_path, &new_content) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(e),
        },
    }
}

/// Update a memory entry by index
#[tauri::command(rename_all = "camelCase")]
pub fn update_memory_entry(index: i32, content: String) -> MemoryWriteResult {
    let file_path = memory_path();
    let (existing, _, _) = read_file_safe(&file_path);
    let mut entries = parse_memory_entries(&existing);

    if index < 0 || (index as usize) >= entries.len() {
        return MemoryWriteResult {
            success: false,
            error: Some("Entry not found".to_string()),
        };
    }

    entries[index as usize].content = content.trim().to_string();
    let new_content = serialize_entries(&entries);

    if new_content.len() > MEMORY_CHAR_LIMIT {
        return MemoryWriteResult {
            success: false,
            error: Some(format!(
                "Would exceed memory limit ({}/{})",
                new_content.len(),
                MEMORY_CHAR_LIMIT
            )),
        };
    }

    match write_file_atomic(&file_path, &new_content) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(e),
        },
    }
}

/// Remove a memory entry by index
#[tauri::command(rename_all = "camelCase")]
pub fn remove_memory_entry(index: i32) -> MemoryWriteResult {
    let file_path = memory_path();
    let (existing, _, _) = read_file_safe(&file_path);
    let mut entries = parse_memory_entries(&existing);

    if index < 0 || (index as usize) >= entries.len() {
        return MemoryWriteResult {
            success: false,
            error: Some("Entry not found".to_string()),
        };
    }

    entries.remove(index as usize);
    let new_content = serialize_entries(&entries);

    match write_file_atomic(&file_path, &new_content) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(e),
        },
    }
}

/// Write user profile (USER.md)
#[tauri::command(rename_all = "camelCase")]
pub fn write_user_profile(content: String) -> MemoryWriteResult {
    if content.len() > USER_CHAR_LIMIT {
        return MemoryWriteResult {
            success: false,
            error: Some(format!(
                "Exceeds limit ({}/{})",
                content.len(),
                USER_CHAR_LIMIT
            )),
        };
    }

    match write_file_atomic(&user_path(), &content) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(e),
        },
    }
}

/// List memory providers with their status
/// Reads active provider from config.yaml, checks env vars for each provider
#[tauri::command(rename_all = "camelCase")]
pub fn list_memory_providers() -> MemoryProviderResult {
    // Read config.yaml for memory section
    let config_path = config_path();
    let active_provider = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                // Parse the memory section using serde_yaml
                match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    Ok(value) => value
                        .get("memory")
                        .and_then(|m| m.get("provider"))
                        .and_then(|p| p.as_str())
                        .unwrap_or("")
                        .to_string(),
                    Err(_) => String::new(),
                }
            }
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    // Read memory_char_limit and user_char_limit from config
    let memory_char_limit = MEMORY_CHAR_LIMIT;
    let user_char_limit = USER_CHAR_LIMIT;
    let memory_enabled = true;
    let user_profile_enabled = true;

    let providers: Vec<MemoryProviderInfo> = KNOWN_PROVIDERS
        .iter()
        .map(|(name, desc, env_keys)| {
            let env_vars: Vec<String> = env_keys.iter().map(|k| k.to_string()).collect();
            MemoryProviderInfo {
                name: name.to_string(),
                description: desc.to_string(),
                installed: check_provider_installed(env_keys),
                active: active_provider == *name,
                env_vars,
            }
        })
        .collect();

    MemoryProviderResult {
        providers,
        active_provider,
        memory_enabled,
        user_profile_enabled,
        memory_char_limit,
        user_char_limit,
    }
}

/// Set the active memory provider in config.yaml
#[tauri::command(rename_all = "camelCase")]
pub fn set_memory_provider(provider: String) -> MemoryWriteResult {
    let config_path = config_path();
    if !config_path.exists() {
        return MemoryWriteResult {
            success: false,
            error: Some("Config file not found".to_string()),
        };
    }

    // Read existing config
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return MemoryWriteResult {
                success: false,
                error: Some(format!("Failed to read config: {}", e)),
            }
        }
    };

    let mut yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return MemoryWriteResult {
                success: false,
                error: Some(format!("Failed to parse config: {}", e)),
            }
        }
    };

    // Ensure memory section exists
    if yaml_value.get("memory").is_none() {
        if let Some(map) = yaml_value.as_mapping_mut() {
            map.insert(
                serde_yaml::Value::String("memory".to_string()),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
        }
    }

    // Set provider value
    if let Some(memory) = yaml_value.get_mut("memory") {
        if let Some(map) = memory.as_mapping_mut() {
            map.insert(
                serde_yaml::Value::String("provider".to_string()),
                serde_yaml::Value::String(provider),
            );
        }
    }

    // Write back atomically
    let new_content = match serde_yaml::to_string(&yaml_value) {
        Ok(s) => s,
        Err(e) => {
            return MemoryWriteResult {
                success: false,
                error: Some(format!("Failed to serialize config: {}", e)),
            }
        }
    };

    let tmp_path = config_path.with_extension("yaml.tmp");
    match std::fs::write(&tmp_path, &new_content) {
        Ok(()) => {}
        Err(e) => {
            return MemoryWriteResult {
                success: false,
                error: Some(format!("Failed to write config: {}", e)),
            }
        }
    }
    match std::fs::rename(&tmp_path, &config_path) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to rename config: {}", e)),
        },
    }
}

/// Read specified environment variables from the process
#[tauri::command]
pub fn read_env_vars(keys: Vec<String>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for key in keys {
        let val = std::env::var(&key).unwrap_or_default();
        result.insert(key, val);
    }
    result
}

/// Save an environment variable to ~/.hermes/.env
#[tauri::command(rename_all = "camelCase")]
pub fn save_env_var(key: String, value: String) -> MemoryWriteResult {
    let env_path = hermes_home().join(".env");
    let mut existing = String::new();
    if env_path.exists() {
        existing = match std::fs::read_to_string(&env_path) {
            Ok(c) => c,
            Err(e) => {
                return MemoryWriteResult {
                    success: false,
                    error: Some(format!("Failed to read .env: {}", e)),
                }
            }
        };
    }

    // Update or append the key=value line
    let key_pattern = format!("{}=", key);
    let mut found = false;
    let mut new_lines: Vec<String> = existing
        .lines()
        .map(|line| {
            if line.trim().starts_with(&key_pattern) || line.trim().starts_with(&format!("export {}", key_pattern))
            {
                found = true;
                if value.is_empty() {
                    // Comment out instead of removing
                    format!("# {} (removed)", line)
                } else {
                    format!("export {}={}", key, value)
                }
            } else {
                line.to_string()
            }
        })
        .collect();

    if !found && !value.is_empty() {
        new_lines.push(format!("export {}={}", key, value));
    }

    let new_content = new_lines.join("\n") + "\n";

    match write_file_atomic(&env_path, &new_content) {
        Ok(()) => MemoryWriteResult {
            success: true,
            error: None,
        },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to write .env: {}", e)),
        },
    }
}

// ── Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::fs;

    /// Serial helper so parallel test runs don't clobber each other's env vars.
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper: set env var inside unsafe block.
    unsafe fn set_env_var(key: &str, val: &str) {
        unsafe { std::env::set_var(key, val) }
    }
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
}
