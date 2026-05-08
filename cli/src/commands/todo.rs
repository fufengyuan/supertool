use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_todo(client: &ApiClient, action: &TodoCommands) -> Result<()> {
    match action {
        TodoCommands::Add {
            text,
            priority,
            due,
            tag,
            description,
        } => cmd_add(client, text, priority, due, tag, &description.as_deref().unwrap_or(""), false),
        TodoCommands::List {
            completed,
            tag,
            limit,
            json,
        } => cmd_list(client, *completed, tag, *limit, *json),
        TodoCommands::Complete { id } => cmd_complete(client, id),
        TodoCommands::Delete { id } => cmd_delete(client, id),
        TodoCommands::Show { id, json } => cmd_show(client, id, *json),
        TodoCommands::Stats { json } => cmd_stats(client, *json),
        TodoCommands::Clear => cmd_clear(client),
        TodoCommands::Search { keyword, json } => cmd_search(client, keyword, *json),
        TodoCommands::Edit {
            id,
            text,
            priority,
            due,
            tag,
            description,
        } => cmd_edit(client, id, text, priority, due, tag, description),
        TodoCommands::Uncomplete { id } => cmd_uncomplete(client, id),
    }
}

pub fn cmd_subtask(client: &ApiClient, action: &SubtaskCommands) -> Result<()> {
    check_connection(client)?;
    match action {
        SubtaskCommands::List { todo_id, json } => {
            let subtasks: Vec<serde_json::Value> =
                client.request("subtasks:get-for-todo", Some(&serde_json::json!({"todoId": todo_id})))?;
            if *json {
                print_json(&subtasks);
            } else {
                println!("  子任务 ({}):", subtasks.len());
                for s in &subtasks {
                    let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let done = if s.get("completed").and_then(|v| v.as_bool()).unwrap_or(false) {
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
            let _ = client.request::<serde_json::Value>(
                "subtasks:add",
                Some(&serde_json::json!({"todoId": todo_id, "text": text, "description": description, "completed": false})),
            )?;
            print_success(&format!("子任务已添加: {}", text));
        }
        SubtaskCommands::Complete { id } => {
            let text = find_subtask_text(client, id);
            let _ = client.request::<serde_json::Value>(
                "subtasks:update",
                Some(&serde_json::json!({"id": id, "completed": true})),
            )?;
            print_success(&format!("子任务「{}」已完成", text));
        }
        SubtaskCommands::Delete { id } => {
            let text = find_subtask_text(client, id);
            let _ = client.request::<serde_json::Value>(
                "subtasks:delete",
                Some(&serde_json::json!({"id": id})),
            )?;
            print_success(&format!("子任务「{}」已删除", text));
        }
    }
    Ok(())
}

pub fn check_connection(client: &ApiClient) -> Result<()> {
    if !client.health_check() {
        anyhow::bail!("无法连接到 SuperTool (UDS socket)\n请确保 GUI 已启动，~/.supertool/supertool.sock 存在。\n设置 SUPERTOOL_SOCKET 环境变量可指定 socket 路径。");
    }
    Ok(())
}

fn print_todo(t: &Todo) {
    let status = if t.completed {
        "\x1b[32m●\x1b[0m"
    } else {
        "\x1b[33m○\x1b[0m"
    };
    println!("  {} {} {}", t.id, status, t.text);
}

fn resolve_todo_text(client: &ApiClient, id: &str) -> String {
    let todos: Vec<Todo> = match client.request("todos:get-all", None) {
        Ok(t) => t,
        Err(_) => return id.to_string(),
    };
    todos
        .iter()
        .find(|t| t.id == id)
        .map(|t| t.text.clone())
        .unwrap_or_else(|| id.to_string())
}

fn find_subtask_text(client: &ApiClient, id: &str) -> String {
    let todos: Vec<Todo> = match client.request("todos:get-all", None) {
        Ok(t) => t,
        Err(_) => return id.to_string(),
    };
    for todo in &todos {
        if let Ok(subtasks) =
            client.request::<Vec<serde_json::Value>>("subtasks:get-for-todo", Some(&serde_json::json!({"todoId": todo.id})))
        {
            if let Some(st) = subtasks
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id))
            {
                if let Some(text) = st.get("text").and_then(|v| v.as_str()) {
                    return text.to_string();
                }
            }
        }
    }
    id.to_string()
}

pub fn cmd_add(
    client: &ApiClient,
    text: &str,
    priority: &str,
    due: &Option<String>,
    tag: &str,
    description: &str,
    json: bool,
) -> Result<()> {
    check_connection(client)?;
    let todo = serde_json::json!({
        "text": text,
        "priority": priority,
        "dueDate": due.as_deref().unwrap_or(""),
        "tag": tag,
        "description": description,
        "completed": false
    });
    let resp: serde_json::Value = client.request("todos:add", Some(&todo))?;
    if json {
        print_json(&resp);
    } else {
        print_success(&format!("任务已添加: {}", text));
    }
    Ok(())
}

pub fn cmd_list(
    client: &ApiClient,
    completed: Option<bool>,
    tag: &Option<String>,
    limit: usize,
    json: bool,
) -> Result<()> {
    check_connection(client)?;
    let todos: Vec<Todo> = client.request("todos:get-all", None)?;
    let mut filtered: Vec<&Todo> = todos.iter().collect();
    if let Some(c) = completed {
        filtered.retain(|t| t.completed == c);
    }
    if let Some(t) = tag {
        filtered.retain(|todo| &todo.tag == t);
    }
    filtered.sort_by(|a, b| a.created_at.cmp(&b.created_at).reverse());
    filtered.truncate(limit);
    if json {
        print_json(&filtered);
    } else {
        if filtered.is_empty() {
            println!("  暂无任务");
        } else {
            // 获取项目名称
            let projects: Vec<serde_json::Value> = client.request("projects:get-all", None).unwrap_or_default();
            let project_map: std::collections::HashMap<String, String> = projects
                .iter()
                .filter_map(|p| {
                    let id = p.get("id").and_then(|v| v.as_str())?;
                    let name = p.get("name").and_then(|v| v.as_str())?;
                    Some((id.to_string(), name.to_string()))
                })
                .collect();
            // 按项目分组
            let mut groups: Vec<(String, Vec<&Todo>)> = Vec::new();
            for t in &filtered {
                let project_name = t.project_id.as_ref()
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

pub fn cmd_complete(client: &ApiClient, id: &str) -> Result<()> {
    check_connection(client)?;
    let text = resolve_todo_text(client, id);
    let _ = client.request::<serde_json::Value>(
        "todos:update",
        Some(&serde_json::json!({"id": id, "completed": true})),
    )?;
    print_success(&format!("任务「{}」已标记为完成", text));
    Ok(())
}

pub fn cmd_delete(client: &ApiClient, id: &str) -> Result<()> {
    check_connection(client)?;
    let text = resolve_todo_text(client, id);
    let _ = client.request::<serde_json::Value>(
        "todos:delete",
        Some(&serde_json::json!({"id": id})),
    )?;
    print_success(&format!("任务「{}」已删除", text));
    Ok(())
}

pub fn cmd_show(client: &ApiClient, id: &str, json: bool) -> Result<()> {
    check_connection(client)?;
    let todos: Vec<Todo> = client.request("todos:get-all", None)?;
    if let Some(todo) = todos.iter().find(|t| t.id == id) {
        if json {
            print_json(todo);
        } else {
            println!("\n  任务详情:");
            print_todo(todo);
            if !todo.description.is_empty() {
                println!("\n  描述: {}", todo.description);
            }
            if let Some(due) = &todo.due_date {
                println!("  截止日期: {}", due);
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

pub fn cmd_stats(client: &ApiClient, json: bool) -> Result<()> {
    check_connection(client)?;
    let todos: Vec<Todo> = client.request("todos:get-all", None)?;
    let total = todos.len();
    let completed = todos.iter().filter(|t| t.completed).count();
    let pending = total - completed;
    let high = todos
        .iter()
        .filter(|t| t.priority == "high" && !t.completed)
        .count();
    if json {
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

pub fn cmd_clear(client: &ApiClient) -> Result<()> {
    check_connection(client)?;
    let todos: Vec<Todo> = client.request("todos:get-all", None)?;
    let completed: Vec<_> = todos.iter().filter(|t| t.completed).collect();
    let count = completed.len();
    for t in &completed {
        let _ = client.request::<serde_json::Value>(
            "todos:delete",
            Some(&serde_json::json!({"id": t.id})),
        );
    }
    print_success(&format!("已清空 {} 个已完成任务", count));
    Ok(())
}

pub fn cmd_search(client: &ApiClient, keyword: &str, json: bool) -> Result<()> {
    check_connection(client)?;
    let todos: Vec<Todo> = client.request("todos:get-all", None)?;
    let kw = keyword.to_lowercase();
    let matched: Vec<_> = todos
        .iter()
        .filter(|t| {
            t.text.to_lowercase().contains(&kw)
                || t.description.to_lowercase().contains(&kw)
                || t.tag.to_lowercase().contains(&kw)
        })
        .collect();
    if json {
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

pub fn cmd_edit(
    client: &ApiClient,
    id: &str,
    text: &Option<String>,
    priority: &Option<String>,
    due: &Option<String>,
    tag: &Option<String>,
    description: &Option<String>,
) -> Result<()> {
    check_connection(client)?;
    let mut update = serde_json::Map::new();
    update.insert("id".into(), serde_json::Value::String(id.to_string()));
    if let Some(t) = text {
        update.insert("text".into(), serde_json::Value::String(t.clone()));
    }
    if let Some(p) = priority {
        update.insert("priority".into(), serde_json::Value::String(p.clone()));
    }
    if let Some(d) = due {
        update.insert("dueDate".into(), serde_json::Value::String(d.clone()));
    }
    if let Some(t) = tag {
        update.insert("tag".into(), serde_json::Value::String(t.clone()));
    }
    if let Some(d) = description {
        update.insert(
            "description".into(),
            serde_json::Value::String(d.clone()),
        );
    }
    let _ = client.request::<serde_json::Value>("todos:update", Some(&serde_json::Value::Object(update)))?;
    let text = resolve_todo_text(client, id);
    print_success(&format!("任务「{}」已更新", text));
    Ok(())
}

pub fn cmd_uncomplete(client: &ApiClient, id: &str) -> Result<()> {
    check_connection(client)?;
    let text = resolve_todo_text(client, id);
    let _ = client.request::<serde_json::Value>(
        "todos:update",
        Some(&serde_json::json!({"id": id, "completed": false})),
    )?;
    print_success(&format!("任务「{}」已恢复为未完成", text));
    Ok(())
}
