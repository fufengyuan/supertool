//! Hermes Memory Management — using hermes-config paths.
//!
//! Manages MEMORY.md and USER.md in ~/.hermes/memories/,
//! reads session/message stats from state.db,
//! and lists memory providers from config.yaml.

use std::collections::HashMap;
use std::path::PathBuf;

use hermes_config::paths;
use serde::{Deserialize, Serialize};

const ENTRY_DELIMITER: &str = "\n§\n";
pub(crate) const MEMORY_CHAR_LIMIT: usize = 2200;
pub(crate) const USER_CHAR_LIMIT: usize = 1375;

// ── Data types ────────────────────────────────────────────────

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

// ── Paths (ultra crate powered) ───────────────────────────────

fn state_db_path() -> PathBuf {
    paths::hermes_home().join("state.db")
}

fn config_path_fn() -> PathBuf {
    paths::config_path()
}

// ── File helpers ──────────────────────────────────────────────

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
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }
    let tmp = file_path.with_extension("md.tmp");
    std::fs::write(&tmp, content)
        .map_err(|e| format!("Failed to write file: {e}"))?;
    std::fs::rename(&tmp, file_path)
        .map_err(|e| format!("Failed to rename file: {e}"))?;
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
        return SessionStats { total_sessions: 0, total_messages: 0 };
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
            SessionStats { total_sessions, total_messages }
        }
        Err(_) => SessionStats { total_sessions: 0, total_messages: 0 },
    }
}

// ── Known memory providers ────────────────────────────────────

const KNOWN_PROVIDERS: &[(&str, &str, &[&str])] = &[
    ("honcho", "Managed memory via Honcho API — best for long-term memory persistence", &["HONCHO_API_KEY"]),
    ("hindsight", "Vectorize.io Hindsight — automatic memory recall and context injection", &["HIND_API_KEY"]),
    ("mem0", "Mem0 — self-improving memory layer for AI agents", &["MEM0_API_KEY"]),
    ("retaindb", "RetainDB — serverless memory with semantic search", &["RETAINDB_API_KEY", "RETAINDB_URL"]),
    ("supermemory", "SuperMemory — AI-powered memory manager with web integration", &["SUPERMEMORY_API_KEY"]),
    ("byterover", "ByteRover — cross-session memory router for long-running agents", &["BYTEROVER_API_KEY"]),
];

fn check_provider_installed(env_keys: &[&str]) -> bool {
    env_keys.iter().any(|k| std::env::var(k).is_ok())
}

// ── Tauri Commands ────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn read_memory() -> Result<MemoryInfo, String> {
    let mem_path = paths::memory_path();
    let user_path = paths::user_path();

    let mem_file = read_file_safe(&mem_path);
    let user_file = read_file_safe(&user_path);

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
            entries: Vec::new(),
            char_count: user_file.0.len(),
            char_limit: USER_CHAR_LIMIT,
        },
        stats: get_session_stats(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn add_memory_entry(content: String) -> MemoryWriteResult {
    let file_path = paths::memory_path();
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
            error: Some(format!("Would exceed memory limit ({}/{})", new_content.len(), MEMORY_CHAR_LIMIT)),
        };
    }

    match write_file_atomic(&file_path, &new_content) {
        Ok(()) => MemoryWriteResult { success: true, error: None },
        Err(e) => MemoryWriteResult { success: false, error: Some(e) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_memory_entry(index: i32, content: String) -> MemoryWriteResult {
    let file_path = paths::memory_path();
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
            error: Some(format!("Would exceed memory limit ({}/{})", new_content.len(), MEMORY_CHAR_LIMIT)),
        };
    }

    match write_file_atomic(&file_path, &new_content) {
        Ok(()) => MemoryWriteResult { success: true, error: None },
        Err(e) => MemoryWriteResult { success: false, error: Some(e) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_memory_entry(index: i32) -> MemoryWriteResult {
    let file_path = paths::memory_path();
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
        Ok(()) => MemoryWriteResult { success: true, error: None },
        Err(e) => MemoryWriteResult { success: false, error: Some(e) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn write_user_profile(content: String) -> MemoryWriteResult {
    if content.len() > USER_CHAR_LIMIT {
        return MemoryWriteResult {
            success: false,
            error: Some(format!("Exceeds limit ({}/{})", content.len(), USER_CHAR_LIMIT)),
        };
    }

    match write_file_atomic(&paths::user_path(), &content) {
        Ok(()) => MemoryWriteResult { success: true, error: None },
        Err(e) => MemoryWriteResult { success: false, error: Some(e) },
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_memory_providers() -> MemoryProviderResult {
    let config_path = config_path_fn();
    let active_provider = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
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

    let providers: Vec<MemoryProviderInfo> = KNOWN_PROVIDERS
        .iter()
        .map(|(name, desc, env_keys)| MemoryProviderInfo {
            name: name.to_string(),
            description: desc.to_string(),
            installed: check_provider_installed(env_keys),
            active: active_provider == *name,
            env_vars: env_keys.iter().map(|k| k.to_string()).collect(),
        })
        .collect();

    MemoryProviderResult {
        providers,
        active_provider,
        memory_enabled: true,
        user_profile_enabled: true,
        memory_char_limit: MEMORY_CHAR_LIMIT,
        user_char_limit: USER_CHAR_LIMIT,
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn set_memory_provider(provider: String) -> MemoryWriteResult {
    let config_path = config_path_fn();
    if !config_path.exists() {
        return MemoryWriteResult {
            success: false,
            error: Some("Config file not found".to_string()),
        };
    }

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => return MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to read config: {e}")),
        },
    };

    let mut yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => return MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to parse config: {e}")),
        },
    };

    if yaml_value.get("memory").is_none() {
        if let Some(map) = yaml_value.as_mapping_mut() {
            map.insert(
                serde_yaml::Value::String("memory".to_string()),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
        }
    }

    if let Some(memory) = yaml_value.get_mut("memory") {
        if let Some(map) = memory.as_mapping_mut() {
            map.insert(
                serde_yaml::Value::String("provider".to_string()),
                serde_yaml::Value::String(provider),
            );
        }
    }

    let new_content = match serde_yaml::to_string(&yaml_value) {
        Ok(s) => s,
        Err(e) => return MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to serialize config: {e}")),
        },
    };

    let tmp = config_path.with_extension("yaml.tmp");
    match std::fs::write(&tmp, &new_content) {
        Ok(_) => {}
        Err(e) => return MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to write config: {e}")),
        },
    }
    match std::fs::rename(&tmp, &config_path) {
        Ok(_) => MemoryWriteResult { success: true, error: None },
        Err(e) => MemoryWriteResult {
            success: false,
            error: Some(format!("Failed to rename config: {e}")),
        },
    }
}

/// Read env vars for memory providers (return values for display).
#[tauri::command(rename_all = "camelCase")]
pub fn read_env_vars() -> Result<serde_json::Value, String> {
    let env_path = paths::env_path();
    if !env_path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&env_path)
        .map_err(|e| format!("Failed to read .env: {e}"))?;

    let mut vars = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = &trimmed[..eq];
            let value = &trimmed[eq + 1..];
            // Only expose memory-related env vars
            if key.starts_with("MEM") || key.starts_with("HIND") || key.starts_with("RETAIN")
                || key.starts_with("SUPERMEM") || key.starts_with("BYTEROVER")
                || key.starts_with("HONCHO")
            {
                let masked = if value.len() > 8 {
                    format!("{}...{}", &value[..4], &value[value.len()-4..])
                } else {
                    "****".to_string()
                };
                vars.insert(key.to_string(), masked);
            }
        }
    }

    Ok(serde_json::json!(vars))
}

/// Save a memory env var.
#[tauri::command(rename_all = "camelCase")]
pub fn save_env_var(key: String, value: String) -> Result<serde_json::Value, String> {
    let env_path = paths::env_path();
    // Ensure .hermes directory exists
    if let Some(parent) = env_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dir: {e}"))?;
    }

    let existing_content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut lines: Vec<String> = existing_content
        .lines()
        .filter(|l| !l.trim().starts_with(&format!("{}=", key)))
        .map(String::from)
        .collect();

    lines.push(format!("{}={}", key, value));

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    let tmp = env_path.with_extension("env.tmp");
    std::fs::write(&tmp, &new_content)
        .map_err(|e| format!("Failed to write .env: {e}"))?;
    std::fs::rename(&tmp, &env_path)
        .map_err(|e| format!("Failed to update .env: {e}"))?;

    Ok(serde_json::json!({ "success": true }))
}
