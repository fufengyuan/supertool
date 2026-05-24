//! Hermes Kanban Board Integration
//!
//! Provides IPC commands for interacting with Hermes Kanban system.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Kanban board info
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub default_workdir: Option<String>,
    pub archived: bool,
    pub db_path: String,
    pub is_current: bool,
    pub counts: serde_json::Value,
    pub total: u32,
}

/// Kanban task info
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanTask {
    pub task_id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: String,
    pub assignee: Option<String>,
    pub priority: Option<u32>,
    pub parents: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub claimed_at: Option<String>,
    pub claimed_by: Option<String>,
    pub workspace: Option<String>,
    pub tenant: Option<String>,
}

/// Kanban task detail (with comments and events)
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanTaskDetail {
    pub task: KanbanTask,
    pub comments: Vec<KanbanComment>,
    pub events: Vec<KanbanEvent>,
    pub runs: Vec<KanbanRun>,
}

/// Kanban comment
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanComment {
    pub id: u64,
    pub task_id: String,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

/// Kanban event
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanEvent {
    pub id: u64,
    pub task_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

/// Kanban run (worker attempt)
#[derive(Debug, Serialize, Deserialize)]
pub struct KanbanRun {
    pub run_id: u64,
    pub task_id: String,
    pub profile: String,
    pub outcome: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub elapsed_seconds: Option<u64>,
}

/// Run hermes kanban CLI and parse JSON output
fn run_kanban_cmd(args: &[String]) -> Result<serde_json::Value, String> {
    // Build the full command string
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let full_cmd = format!("hermes kanban {} --json", args_str.join(" "));

    // Use login shell (-l) to load user's full environment including PATH
    let output = Command::new("/bin/bash")
        .args(["-l", "-c", &full_cmd])
        .output()
        .map_err(|e| format!("Failed to run hermes kanban: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Kanban command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }

    serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse kanban output: {} - {}", e, stdout))
}

/// List all kanban boards
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_list_boards() -> Result<Vec<KanbanBoard>, String> {
    let args: Vec<String> = vec!["boards".into(), "list".into()];
    let value = run_kanban_cmd(&args)?;
    let boards: Vec<KanbanBoard> = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse boards: {}", e))?;
    Ok(boards)
}

/// Get current board info
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_get_current_board() -> Result<KanbanBoard, String> {
    let boards = kanban_list_boards()?;
    boards
        .into_iter()
        .find(|b| b.is_current)
        .ok_or_else(|| "No current board found".to_string())
}

/// List tasks on a board (optionally filtered by status/assignee)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_list_tasks(
    board: Option<String>,
    status: Option<String>,
    assignee: Option<String>,
) -> Result<Vec<KanbanTask>, String> {
    let mut args: Vec<String> = vec!["list".into()];
    if let Some(b) = board {
        args.push("--board".into());
        args.push(b);
    }
    if let Some(s) = status {
        args.push("--status".into());
        args.push(s);
    }
    if let Some(a) = assignee {
        args.push("--assignee".into());
        args.push(a);
    }

    let value = run_kanban_cmd(&args)?;
    let tasks: Vec<KanbanTask> = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse tasks: {}", e))?;
    Ok(tasks)
}

/// Get task detail (with comments, events, runs)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_show_task(task_id: String) -> Result<KanbanTaskDetail, String> {
    // First get task info
    let task_value = run_kanban_cmd(&["show".into(), task_id.clone()])?;
    let task: KanbanTask = serde_json::from_value(task_value.clone())
        .map_err(|e| format!("Failed to parse task: {}", e))?;

    // Get comments
    let comments_value = run_kanban_cmd(&["comment".into(), "--list".into(), task_id.clone()])?;
    let comments: Vec<KanbanComment> = if comments_value.is_array() {
        serde_json::from_value(comments_value)
            .map_err(|e| format!("Failed to parse comments: {}", e))?
    } else {
        vec![]
    };

    // Get events
    let events_value = run_kanban_cmd(&["log".into(), task_id.clone()])?;
    let events: Vec<KanbanEvent> = if events_value.is_array() {
        serde_json::from_value(events_value)
            .map_err(|e| format!("Failed to parse events: {}", e))?
    } else {
        vec![]
    };

    // Get runs
    let runs_value = run_kanban_cmd(&["runs".into(), task_id])?;
    let runs: Vec<KanbanRun> = if runs_value.is_array() {
        serde_json::from_value(runs_value)
            .map_err(|e| format!("Failed to parse runs: {}", e))?
    } else {
        vec![]
    };

    Ok(KanbanTaskDetail {
        task,
        comments,
        events,
        runs,
    })
}

/// Create a new task
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_create_task(
    title: String,
    body: Option<String>,
    assignee: Option<String>,
    parents: Option<Vec<String>>,
    priority: Option<u32>,
    board: Option<String>,
) -> Result<KanbanTask, String> {
    let mut args: Vec<String> = vec!["create".into(), title.clone()];
    if let Some(b) = body {
        args.push("--body".into());
        args.push(b);
    }
    if let Some(a) = assignee {
        args.push("--assignee".into());
        args.push(a);
    }
    for p in parents.unwrap_or_default() {
        args.push("--parent".into());
        args.push(p);
    }
    if let Some(pr) = priority {
        args.push("--priority".into());
        args.push(pr.to_string());
    }
    if let Some(b) = board {
        args.push("--board".into());
        args.push(b);
    }

    let value = run_kanban_cmd(&args)?;
    let task: KanbanTask = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse created task: {}", e))?;
    Ok(task)
}

/// Assign a task to a profile
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_assign_task(task_id: String, assignee: String) -> Result<(), String> {
    run_kanban_cmd(&["assign".into(), task_id, assignee])?;
    Ok(())
}

/// Reclaim a running task (release worker claim)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_reclaim_task(task_id: String) -> Result<(), String> {
    run_kanban_cmd(&["reclaim".into(), task_id])?;
    Ok(())
}

/// Complete a task
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_complete_task(task_id: String, summary: Option<String>) -> Result<(), String> {
    let mut args: Vec<String> = vec!["complete".into(), task_id];
    if let Some(s) = summary {
        args.push("--summary".into());
        args.push(s);
    }
    run_kanban_cmd(&args)?;
    Ok(())
}

/// Block a task
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_block_task(task_id: String, reason: String) -> Result<(), String> {
    run_kanban_cmd(&["block".into(), task_id, reason])?;
    Ok(())
}

/// Unblock a task (return to ready)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_unblock_task(task_id: String) -> Result<(), String> {
    run_kanban_cmd(&["unblock".into(), task_id])?;
    Ok(())
}

/// Archive a task
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_archive_task(task_id: String) -> Result<(), String> {
    run_kanban_cmd(&["archive".into(), task_id])?;
    Ok(())
}

/// Add comment to a task
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_add_comment(task_id: String, body: String) -> Result<(), String> {
    run_kanban_cmd(&["comment".into(), task_id, body])?;
    Ok(())
}

/// List assignees (profiles that can receive tasks)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_list_assignees() -> Result<Vec<serde_json::Value>, String> {
    let value = run_kanban_cmd(&["assignees".into()])?;
    let assignees: Vec<serde_json::Value> = if value.is_array() {
        serde_json::from_value(value)
            .map_err(|e| format!("Failed to parse assignees: {}", e))?
    } else {
        vec![]
    };
    Ok(assignees)
}

/// Get board stats (per-status + per-assignee counts)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_get_stats(board: Option<String>) -> Result<serde_json::Value, String> {
    let mut args: Vec<String> = vec!["stats".into()];
    if let Some(b) = board {
        args.push("--board".into());
        args.push(b);
    }
    run_kanban_cmd(&args)
}

/// Create a new board
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_create_board(
    slug: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Result<KanbanBoard, String> {
    let mut args: Vec<String> = vec!["boards".into(), "create".into(), slug.clone(), "--name".into(), name.clone()];
    if let Some(d) = description {
        args.push("--description".into());
        args.push(d);
    }
    if let Some(i) = icon {
        args.push("--icon".into());
        args.push(i);
    }
    if let Some(c) = color {
        args.push("--color".into());
        args.push(c);
    }

    let value = run_kanban_cmd(&args)?;
    let board: KanbanBoard = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse created board: {}", e))?;
    Ok(board)
}

/// Switch to a board
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_switch_board(slug: String) -> Result<(), String> {
    run_kanban_cmd(&["boards".into(), "switch".into(), slug])?;
    Ok(())
}