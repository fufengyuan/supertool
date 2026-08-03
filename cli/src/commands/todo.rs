use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;

pub async fn cmd_todo(runtime: &mut CliRuntime, action: &TodoCommands) -> Result<()> {
    match action {
        TodoCommands::Add {
            text,
            priority,
            due,
            tag,
            description,
            project_id,
        } => {
            cmd_add(
                runtime,
                text,
                &priority.as_deref().unwrap_or("medium"),
                due,
                tag.as_deref().unwrap_or(""),
                &description.as_deref().unwrap_or(""),
                project_id,
            )
            .await
        }
        TodoCommands::List {
            completed,
            tag,
            limit,
            json,
        } => {
            cmd_list(
                runtime,
                completed.as_ref().and_then(|s| match s.as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                }),
                tag,
                *limit,
                *json,
            )
            .await
        }
        TodoCommands::Complete { id } => cmd_complete(runtime, id).await,
        TodoCommands::Delete { id } => cmd_delete(runtime, id).await,
        TodoCommands::Show { id, json } => cmd_show(runtime, id, *json).await,
        TodoCommands::Stats { json } => cmd_stats(runtime, *json).await,
        TodoCommands::Clear => cmd_clear(runtime).await,
        TodoCommands::Search { keyword, json } => cmd_search(runtime, keyword, *json).await,
        TodoCommands::Edit {
            id,
            text,
            priority,
            due,
            tag,
            description,
        } => cmd_edit(runtime, id, text, priority, due, tag, description).await,
        TodoCommands::Uncomplete { id } => cmd_uncomplete(runtime, id).await,
    }
}

pub async fn cmd_subtask(runtime: &mut CliRuntime, action: &SubtaskCommands) -> Result<()> {
    match action {
        SubtaskCommands::List { todo_id, json } => {
            let subtasks: serde_json::Value = runtime
                .core
                .get_subtasks_for_todo(todo_id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let subtasks = subtasks.as_array().cloned().unwrap_or_default();
            if *json || runtime.json_mode {
                print_json(&subtasks);
            } else {
                println!("  子任务 ({}):", subtasks.len());
                for s in &subtasks {
                    let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let done = if s
                        .get("completed")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        "●"
                    } else {
                        "○"
                    };
                    let text = s.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    println!("    {} {} {}", id, done, text);
                }
            }
        }
        SubtaskCommands::Add {
            todo_id,
            text,
            description,
        } => {
            let _ = runtime
                .core
                .add_subtask(serde_json::json!({
                    "todoId": todo_id,
                    "text": text,
                    "description": description.as_deref().unwrap_or("")
                }))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("子任务已添加: {}", text));
        }
        SubtaskCommands::Complete { id } => {
            let text = find_subtask_text(runtime, id).await;
            let _ = runtime
                .core
                .update_subtask(serde_json::json!({
                    "id": id,
                    "text": "",
                    "description": "",
                    "completed": true
                }))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("子任务「{}」已完成", text));
        }
        SubtaskCommands::Delete { id } => {
            let text = find_subtask_text(runtime, id).await;
            let _ = runtime
                .core
                .delete_subtask(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("子任务「{}」已删除", text));
        }
    }
    Ok(())
}

fn print_todo(t: &Todo) {
    let status = if t.completed {
        "\x1b[32m●\x1b[0m"
    } else {
        "\x1b[33m○\x1b[0m"
    };

    // Priority indicator
    let prio = match t.priority.as_str() {
        "high" => "\x1b[31m!\x1b[0m",
        "urgent" => "\x1b[31m!!!\x1b[0m",
        "low" => "\x1b[2m↓\x1b[0m",
        _ => " ",
    };

    // Tag (colored)
    let tag_str = if !t.tag.is_empty() {
        format!(" \x1b[36m[{}]\x1b[0m", t.tag)
    } else {
        String::new()
    };

    // Due date
    let due_str = if let Some(due) = &t.due_date {
        if !due.is_empty() {
            format!(" \x1b[2m📅{}\x1b[0m", due)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    println!("  {} {} {}{}{}{}", t.id, status, prio, t.text, tag_str, due_str);
}

async fn resolve_todo_text(runtime: &mut CliRuntime, id: &str) -> String {
    let todos: serde_json::Value = match runtime.core.get_all_todos().await {
        Ok(t) => t,
        Err(_) => return id.to_string(),
    };
    let todos = todos.as_array().cloned().unwrap_or_default();
    todos
        .iter()
        .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(id))
        .and_then(|t| t.get("text").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| id.to_string())
}

async fn find_subtask_text(runtime: &mut CliRuntime, id: &str) -> String {
    let todos: serde_json::Value = match runtime.core.get_all_todos().await {
        Ok(t) => t,
        Err(_) => return id.to_string(),
    };
    let todos = todos.as_array().cloned().unwrap_or_default();
    for todo in &todos {
        if let Some(todo_id) = todo.get("id").and_then(|v| v.as_str()) {
            if let Ok(subtasks) = runtime.core.get_subtasks_for_todo(todo_id).await {
                if let Some(arr) = subtasks.as_array() {
                    if let Some(st) = arr
                        .iter()
                        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id))
                    {
                        if let Some(text) = st.get("text").and_then(|v| v.as_str()) {
                            return text.to_string();
                        }
                    }
                }
            }
        }
    }
    id.to_string()
}

pub async fn cmd_add(
    runtime: &mut CliRuntime,
    text: &str,
    priority: &str,
    due: &Option<String>,
    tag: &str,
    description: &str,
    project_id: &Option<String>,
) -> Result<()> {
    let todo = serde_json::json!({
        "text": text,
        "priority": priority,
        "dueDate": due.as_deref().unwrap_or(""),
        "tag": tag,
        "description": description,
        "projectId": project_id.as_deref().unwrap_or(""),
        "completed": false
    });
    let resp: serde_json::Value = runtime
        .core
        .add_todo(todo)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    print_success(&format!(
        "任务已添加: {}",
        resp.get("text").and_then(|v| v.as_str()).unwrap_or(text)
    ));
    Ok(())
}

pub async fn cmd_list(
    runtime: &mut CliRuntime,
    completed: Option<bool>,
    tag: &Option<String>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos: Vec<serde_json::Value> = todos.as_array().cloned().unwrap_or_default();

    // Deserialize to Todo structs for filtering
    let todo_structs: Vec<Todo> = todos
        .iter()
        .filter_map(|v| serde_json::from_value::<Todo>(v.clone()).ok())
        .collect();

    let mut filtered: Vec<&Todo> = todo_structs.iter().collect();
    if let Some(c) = completed {
        filtered.retain(|t| t.completed == c);
    }
    if let Some(t) = tag {
        filtered.retain(|todo| &todo.tag == t);
    }
    filtered.sort_by(|a, b| a.created_at.cmp(&b.created_at).reverse());
    filtered.truncate(limit);

    if json || runtime.json_mode {
        print_json(&filtered);
    } else {
        if filtered.is_empty() {
            println!("  暂无任务");
        } else {
            // 获取项目名称
            let project_map = load_project_map(runtime).await;
            // 按项目分组
            let mut groups: Vec<(String, Vec<&Todo>)> = Vec::new();
            for t in &filtered {
                let project_name = t
                    .project_id
                    .as_ref()
                    .and_then(|pid| project_map.get(pid))
                    .map(|s| s.as_str())
                    .unwrap_or("未关联项目");
                if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == project_name) {
                    items.push(t);
                } else {
                    groups.push((project_name.to_string(), vec![t]));
                }
            }
            println!("\n  共 {} 个任务:", filtered.len());
            for (project_name, items) in &groups {
                println!("▸ {}", project_name);
                for t in items {
                    print_todo(t);
                }
                println!();
            }
        }
    }
    Ok(())
}

pub async fn cmd_complete(runtime: &mut CliRuntime, id: &str) -> Result<()> {
    let text = resolve_todo_text(runtime, id).await;
    let _ = runtime
        .core
        .update_todo(serde_json::json!({
            "id": id,
            "text": "",
            "completed": true,
            "priority": "medium",
            "dueDate": "",
            "description": "",
            "tag": ""
        }))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    print_success(&format!("任务「{}」已标记为完成", text));
    Ok(())
}

pub async fn cmd_delete(runtime: &mut CliRuntime, id: &str) -> Result<()> {
    let text = resolve_todo_text(runtime, id).await;
    let _ = runtime
        .core
        .delete_todo(id)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    print_success(&format!("任务「{}」已删除", text));
    Ok(())
}

pub async fn cmd_show(runtime: &mut CliRuntime, id: &str, json: bool) -> Result<()> {
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos = todos.as_array().cloned().unwrap_or_default();

    if let Some(todo_json) = todos
        .iter()
        .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(id))
    {
        let todo: Todo = serde_json::from_value(todo_json.clone())
            .map_err(|e| anyhow::anyhow!("解析任务数据失败: {}", e))?;
        if json || runtime.json_mode {
            print_json(&todo);
        } else {
            // Load projects for name resolution
            let project_map = load_project_map(runtime).await;

            println!("\n  ── 任务详情 ──");
            print_todo(&todo);
            println!("  优先级: {}", priority_display(&todo.priority));
            if !todo.tag.is_empty() {
                println!("  标签: {}", todo.tag);
            }
            if let Some(pid) = &todo.project_id {
                let pname = project_map.get(pid).map(|s| s.as_str()).unwrap_or(pid);
                println!("  项目: {}", pname);
            }
            if !todo.description.is_empty() {
                println!("\n  描述: {}", todo.description);
            }
            if let Some(due) = &todo.due_date {
                if !due.is_empty() {
                    println!("  截止日期: {}", due);
                }
            }
            println!(
                "  创建时间: {}\n  更新时间: {}",
                todo.created_at, todo.updated_at
            );
        }
    } else {
        anyhow::bail!("未找到任务: {}", id);
    }
    Ok(())
}

pub async fn cmd_stats(runtime: &mut CliRuntime, json: bool) -> Result<()> {
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos = todos.as_array().cloned().unwrap_or_default();
    let total = todos.len();
    let completed = todos
        .iter()
        .filter(|t| {
            t.get("completed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();
    let pending = total - completed;
    let high = todos
        .iter()
        .filter(|t| {
            t.get("priority").and_then(|v| v.as_str()) == Some("high")
                && !t
                    .get("completed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
        })
        .count();
    if json || runtime.json_mode {
        print_json(&serde_json::json!({
            "total": total,
            "completed": completed,
            "pending": pending,
            "high_priority": high
        }));
    } else {
        println!(
            "\n  任务统计:\n    总计: {}\n    已完成: {}\n    待完成: {}\n    高优先级: {}",
            total, completed, pending, high
        );
    }
    Ok(())
}

pub async fn cmd_clear(runtime: &mut CliRuntime) -> Result<()> {
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos = todos.as_array().cloned().unwrap_or_default();
    let completed: Vec<_> = todos
        .iter()
        .filter(|t| {
            t.get("completed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .collect();
    let count = completed.len();
    for t in &completed {
        if let Some(id) = t.get("id").and_then(|v| v.as_str()) {
            let _ = runtime.core.delete_todo(id).await;
        }
    }
    print_success(&format!("已清空 {} 个已完成任务", count));
    Ok(())
}

pub async fn cmd_search(runtime: &mut CliRuntime, keyword: &str, json: bool) -> Result<()> {
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos = todos.as_array().cloned().unwrap_or_default();
    let kw = keyword.to_lowercase();
    let matched: Vec<_> = todos
        .iter()
        .filter(|t| {
            t.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase()
                .contains(&kw)
                || t.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&kw)
                || t.get("tag")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&kw)
        })
        .filter_map(|v| serde_json::from_value::<Todo>(v.clone()).ok())
        .collect();
    if json || runtime.json_mode {
        print_json(&matched);
    } else {
        if matched.is_empty() {
            println!("  未找到匹配的任务: {}", keyword);
        } else {
            println!("  找到 {} 个匹配:", matched.len());
            for t in &matched {
                print_todo(t);
            }
        }
    }
    Ok(())
}

pub async fn cmd_edit(
    runtime: &mut CliRuntime,
    id: &str,
    text: &Option<String>,
    priority: &Option<String>,
    due: &Option<String>,
    tag: &Option<String>,
    description: &Option<String>,
) -> Result<()> {
    // First get current todo to fill in missing fields
    let todos: serde_json::Value = runtime
        .core
        .get_all_todos()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let todos = todos.as_array().cloned().unwrap_or_default();
    let current = todos
        .iter()
        .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(id));

    let mut update = serde_json::Map::new();
    update.insert("id".into(), serde_json::Value::String(id.to_string()));

    if let Some(t) = text {
        update.insert("text".into(), serde_json::Value::String(t.clone()));
    } else if let Some(cur) = current {
        update.insert(
            "text".into(),
            cur.get("text")
                .cloned()
                .unwrap_or(serde_json::Value::String("".to_string())),
        );
    } else {
        update.insert("text".into(), serde_json::Value::String("".to_string()));
    }

    if let Some(p) = priority {
        update.insert("priority".into(), serde_json::Value::String(p.clone()));
    } else if let Some(cur) = current {
        update.insert(
            "priority".into(),
            cur.get("priority")
                .cloned()
                .unwrap_or(serde_json::Value::String("medium".to_string())),
        );
    } else {
        update.insert(
            "priority".into(),
            serde_json::Value::String("medium".to_string()),
        );
    }

    if let Some(d) = due {
        update.insert("dueDate".into(), serde_json::Value::String(d.clone()));
    } else if let Some(cur) = current {
        update.insert(
            "dueDate".into(),
            cur.get("dueDate")
                .cloned()
                .unwrap_or(serde_json::Value::String("".to_string())),
        );
    } else {
        update.insert("dueDate".into(), serde_json::Value::String("".to_string()));
    }

    if let Some(d) = description {
        update.insert("description".into(), serde_json::Value::String(d.clone()));
    } else if let Some(cur) = current {
        update.insert(
            "description".into(),
            cur.get("description")
                .cloned()
                .unwrap_or(serde_json::Value::String("".to_string())),
        );
    } else {
        update.insert(
            "description".into(),
            serde_json::Value::String("".to_string()),
        );
    }

    if let Some(t) = tag {
        update.insert("tag".into(), serde_json::Value::String(t.clone()));
    } else if let Some(cur) = current {
        update.insert(
            "tag".into(),
            cur.get("tag")
                .cloned()
                .unwrap_or(serde_json::Value::String("".to_string())),
        );
    } else {
        update.insert("tag".into(), serde_json::Value::String("".to_string()));
    }

    let _ = runtime
        .core
        .update_todo(serde_json::Value::Object(update))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let text_display = resolve_todo_text(runtime, id).await;
    print_success(&format!("任务「{}」已更新", text_display));
    Ok(())
}

pub async fn cmd_uncomplete(runtime: &mut CliRuntime, id: &str) -> Result<()> {
    let text = resolve_todo_text(runtime, id).await;
    let _ = runtime
        .core
        .update_todo(serde_json::json!({
            "id": id,
            "text": "",
            "completed": false,
            "priority": "medium",
            "dueDate": "",
            "description": "",
            "tag": ""
        }))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    print_success(&format!("任务「{}」已恢复为未完成", text));
    Ok(())
}

/// Load a map of project_id → project_name
async fn load_project_map(runtime: &mut CliRuntime) -> std::collections::HashMap<String, String> {
    let projects: serde_json::Value = runtime
        .core
        .get_all_projects(false)
        .await
        .unwrap_or(serde_json::json!([]));
    projects
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|p| {
            let id = p.get("id").and_then(|v| v.as_str())?;
            let name = p.get("name").and_then(|v| v.as_str())?;
            Some((id.to_string(), name.to_string()))
        })
        .collect()
}

fn priority_display(p: &str) -> &'static str {
    match p {
        "urgent" => "\x1b[31m!!! 紧急\x1b[0m",
        "high" => "\x1b[33m! 高\x1b[0m",
        "low" => "\x1b[2m↓ 低\x1b[0m",
        _ => "中",
    }
}
