//! ACP tool metadata helpers.
//!
//! These helpers keep ACP tool events compact but informative for clients.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

const TOOL_KIND_MAP: &[(&str, &str)] = &[
    ("read_file", "read"),
    ("search_files", "search"),
    ("terminal", "execute"),
    ("bash", "execute"),
    ("process", "execute"),
    ("execute_code", "execute"),
    ("patch", "edit"),
    ("write_file", "edit"),
    ("web_search", "fetch"),
    ("web_extract", "fetch"),
    ("browser_navigate", "fetch"),
    ("browser_click", "fetch"),
    ("skill_view", "read"),
    ("skill_manage", "edit"),
    ("todo", "other"),
    ("memory", "other"),
    ("session_search", "read"),
    ("delegate_task", "other"),
];

static TOOL_CALL_IDS: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolStartMetadata {
    pub kind: &'static str,
    pub title: String,
}

pub fn make_tool_call_id() -> String {
    let id = TOOL_CALL_IDS.fetch_add(1, Ordering::Relaxed);
    format!("tc-{id}")
}

pub fn tool_kind(tool_name: &str) -> &'static str {
    TOOL_KIND_MAP
        .iter()
        .find_map(|(name, kind)| (*name == tool_name).then_some(*kind))
        .unwrap_or("other")
}

pub fn tool_start_metadata(tool_name: &str, arguments: Option<&Value>) -> ToolStartMetadata {
    ToolStartMetadata {
        kind: tool_kind(tool_name),
        title: tool_title(tool_name, arguments),
    }
}

pub fn tool_title(tool_name: &str, arguments: Option<&Value>) -> String {
    let value = arguments.unwrap_or(&Value::Null);
    match tool_name {
        "terminal" | "bash" => value_string(value, "command")
            .map(|cmd| truncate_chars(&cmd, 110))
            .unwrap_or_else(|| tool_name.to_string()),
        "read_file" => value_string(value, "path")
            .map(|path| format!("read: {path}"))
            .unwrap_or_else(|| "read_file".to_string()),
        "search_files" => value_string(value, "pattern")
            .map(|pattern| format!("search: {pattern}"))
            .unwrap_or_else(|| "search_files".to_string()),
        "patch" | "write_file" => value_string(value, "path")
            .map(|path| format!("{tool_name}: {path}"))
            .unwrap_or_else(|| tool_name.to_string()),
        "web_search" => value_string(value, "query")
            .map(|query| format!("search: {query}"))
            .unwrap_or_else(|| "web_search".to_string()),
        "web_extract" => value_urls(value, "urls")
            .first()
            .map(|url| format!("extract: {url}"))
            .unwrap_or_else(|| "web_extract".to_string()),
        "browser_navigate" => value_string(value, "url")
            .map(|url| format!("navigate: {url}"))
            .unwrap_or_else(|| "browser_navigate".to_string()),
        "skill_view" => {
            let name = value_string(value, "name").unwrap_or_else(|| "unknown".to_string());
            match value_string(value, "file_path") {
                Some(file_path) if !file_path.trim().is_empty() => {
                    format!("skill view ({name}/{file_path})")
                }
                _ => format!("skill view ({name})"),
            }
        }
        "skill_manage" => {
            let action = value_string(value, "action").unwrap_or_else(|| "manage".to_string());
            let name = value_string(value, "name").unwrap_or_else(|| "unknown".to_string());
            match value_string(value, "file_path") {
                Some(file_path) if !file_path.trim().is_empty() => {
                    format!("skill {action}: {name}/{file_path}")
                }
                _ => format!("skill {action}: {name}"),
            }
        }
        "execute_code" => {
            let language = value_string(value, "language").unwrap_or_else(|| "code".to_string());
            value_string(value, "code")
                .and_then(|code| first_non_empty_line(&code))
                .map(|line| format!("{language}: {}", truncate_chars(&line, 90)))
                .unwrap_or_else(|| "execute_code".to_string())
        }
        "todo" => todo_title(value),
        other => other.to_string(),
    }
}

pub fn tool_completion_status(tool_name: &str, result: Option<&str>) -> &'static str {
    if result
        .map(|output| tool_output_failed(tool_name, output))
        .unwrap_or(false)
    {
        "failed"
    } else {
        "completed"
    }
}

pub fn tool_output_failed(tool_name: &str, output: &str) -> bool {
    let trimmed = output.trim();
    if trimmed.starts_with("Error executing tool '") {
        return true;
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };
    let Some(obj) = value.as_object() else {
        return false;
    };

    if obj.get("success").and_then(Value::as_bool) == Some(false)
        || obj.get("ok").and_then(Value::as_bool) == Some(false)
    {
        return true;
    }
    if obj
        .get("exit_code")
        .or_else(|| obj.get("returncode"))
        .and_then(Value::as_i64)
        .is_some_and(|code| code != 0)
    {
        return true;
    }
    obj.contains_key("error")
        && matches!(
            tool_name,
            "read_file" | "write_file" | "patch" | "skill_manage" | "execute_code" | "terminal"
        )
}

fn value_string(value: &Value, key: &str) -> Option<String> {
    let raw = value.get(key)?;
    if let Some(text) = raw.as_str() {
        Some(text.to_string())
    } else {
        Some(raw.to_string())
    }
    .map(|text| text.trim().to_string())
    .filter(|text| !text.is_empty())
}

fn value_urls(value: &Value, key: &str) -> Vec<String> {
    let Some(raw) = value.get(key) else {
        return Vec::new();
    };
    if let Some(url) = raw.as_str() {
        return vec![url.to_string()];
    }
    raw.as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToString::to_string)
}

fn todo_title(value: &Value) -> String {
    let count = value
        .get("todos")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    match count {
        1 => "todo (1 item)".to_string(),
        n => format!("todo ({n} items)"),
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(3);
    let head: String = text.chars().take(keep).collect();
    format!("{head}...")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_kind_covers_common_hermes_tools() {
        for (tool, expected) in [
            ("read_file", "read"),
            ("search_files", "search"),
            ("terminal", "execute"),
            ("patch", "edit"),
            ("write_file", "edit"),
            ("process", "execute"),
            ("web_search", "fetch"),
            ("execute_code", "execute"),
            ("todo", "other"),
            ("skill_view", "read"),
            ("browser_navigate", "fetch"),
            ("unknown_tool", "other"),
        ] {
            assert_eq!(tool_kind(tool), expected);
        }
    }

    #[test]
    fn make_tool_call_id_uses_stable_prefix_and_unique_values() {
        let first = make_tool_call_id();
        let second = make_tool_call_id();
        assert!(first.starts_with("tc-"));
        assert!(second.starts_with("tc-"));
        assert_ne!(first, second);
    }

    #[test]
    fn tool_title_uses_human_readable_arguments() {
        assert_eq!(
            tool_title("terminal", Some(&json!({"command": "ls -la /tmp"}))),
            "ls -la /tmp"
        );
        assert_eq!(
            tool_title("read_file", Some(&json!({"path": "/etc/hosts"}))),
            "read: /etc/hosts"
        );
        assert_eq!(
            tool_title("search_files", Some(&json!({"pattern": "TODO"}))),
            "search: TODO"
        );
        assert_eq!(
            tool_title("web_search", Some(&json!({"query": "rust acp"}))),
            "search: rust acp"
        );
        assert_eq!(
            tool_title("browser_navigate", Some(&json!({"url": "https://x.com"}))),
            "navigate: https://x.com"
        );
        assert_eq!(
            tool_title(
                "skill_view",
                Some(&json!({"name": "github-pitfalls", "file_path": "references/api.md"}))
            ),
            "skill view (github-pitfalls/references/api.md)"
        );
        assert_eq!(
            tool_title(
                "execute_code",
                Some(&json!({"language": "rust", "code": "\nprintln!(\"hello\");"}))
            ),
            "rust: println!(\"hello\");"
        );
        assert_eq!(
            tool_title(
                "skill_manage",
                Some(&json!({"action": "patch", "name": "ops", "file_path": "ref.md"}))
            ),
            "skill patch: ops/ref.md"
        );
        assert_eq!(
            tool_title(
                "todo",
                Some(&json!({"todos": [{"id": "one", "content": "Fix ACP"}]}))
            ),
            "todo (1 item)"
        );
    }

    #[test]
    fn terminal_titles_are_truncated() {
        let title = tool_title("terminal", Some(&json!({"command": "x".repeat(200)})));
        assert!(title.len() < 120);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn completion_status_detects_structured_failures() {
        assert_eq!(
            tool_completion_status("terminal", Some(r#"{"exit_code": 2}"#)),
            "failed"
        );
        assert_eq!(
            tool_completion_status("execute_code", Some(r#"{"returncode": 1}"#)),
            "failed"
        );
        assert_eq!(
            tool_completion_status("skill_manage", Some(r#"{"success": false}"#)),
            "failed"
        );
        assert_eq!(
            tool_completion_status("some_tool", Some(r#"{"ok": false}"#)),
            "failed"
        );
        assert_eq!(
            tool_completion_status("read_file", Some(r#"{"error": "File not found"}"#)),
            "failed"
        );
        assert_eq!(
            tool_completion_status("some_tool", Some(r#"{"error": "optional timeout"}"#)),
            "completed"
        );
        assert_eq!(
            tool_completion_status("terminal", Some("Error: pytest collected 0 items")),
            "completed"
        );
        assert_eq!(
            tool_completion_status("patch", Some("Error executing tool 'patch': boom")),
            "failed"
        );
    }
}
