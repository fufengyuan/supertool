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
pub(crate) const MEMORY_CHAR_LIMIT: usize = 2200;
pub(crate) const USER_CHAR_LIMIT: usize = 1375;

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

pub(crate) fn hermes_home() -> PathBuf {
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

pub(crate) fn read_file_safe(file_path: &PathBuf) -> (String, bool, Option<i64>) {
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

pub(crate) fn write_file_atomic(file_path: &PathBuf, content: &str) -> Result<(), String> {
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

pub(crate) fn parse_memory_entries(content: &str) -> Vec<MemoryEntry> {
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

pub(crate) fn serialize_entries(entries: &[MemoryEntry]) -> String {
    entries
        .iter()
        .map(|e| e.content.as_str())
        .collect::<Vec<&str>>()
        .join(ENTRY_DELIMITER)
}

pub(crate) fn get_session_stats() -> SessionStats {
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
pub(crate) fn check_provider_installed(env_keys: &[&str]) -> bool {
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
#[tauri::command(rename_all = "camelCase")]
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
