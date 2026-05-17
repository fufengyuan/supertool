use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;

pub async fn cmd_project(runtime: &mut CliRuntime, action: &ProjectCommands) -> Result<()> {
    match action {
        ProjectCommands::List { json } => {
            let projects: serde_json::Value = runtime
                .core
                .get_all_projects(false)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let projects = projects.as_array().cloned().unwrap_or_default();
            if *json {
                print_json(&projects);
            } else {
                println!("\n  项目 ({}):", projects.len());
                // 按 category 分组
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for p in &projects {
                    let cat = p
                        .get("category")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .unwrap_or("未分类");
                    if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == cat) {
                        items.push(p);
                    } else {
                        groups.push((cat.to_string(), vec![p]));
                    }
                }
                for (cat, items) in &groups {
                    println!("▸ {}", cat);
                    for p in items {
                        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}  {}", id, name);
                    }
                    println!();
                }
            }
        }
        ProjectCommands::Add { name, description } => {
            let _ = runtime
                .core
                .add_project(serde_json::json!({
                    "name": name,
                    "description": description.as_deref().unwrap_or(""),
                    "active": true
                }))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("项目已添加: {}", name));
        }
        ProjectCommands::Show { id, json: _ } => {
            let projects: serde_json::Value = runtime
                .core
                .get_all_projects(false)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let projects = projects.as_array().cloned().unwrap_or_default();
            if let Some(p) = projects
                .iter()
                .find(|p| p.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                print_json(p);
            } else {
                anyhow::bail!("未找到项目: {}", id);
            }
        }
        ProjectCommands::Update {
            id,
            name,
            description,
        } => {
            // Get current project to fill in missing fields
            let projects: serde_json::Value = runtime
                .core
                .get_all_projects(false)
                .await
                .unwrap_or(serde_json::json!([]));
            let current = projects
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .find(|p| p.get("id").and_then(|v| v.as_str()) == Some(id.as_str()));

            let mut update = serde_json::Map::new();
            update.insert("id".into(), serde_json::Value::String(id.clone()));

            if let Some(n) = name {
                update.insert("name".into(), serde_json::Value::String(n.clone()));
            } else if let Some(cur) = &current {
                update.insert(
                    "name".into(),
                    cur.get("name")
                        .cloned()
                        .unwrap_or(serde_json::Value::String("".to_string())),
                );
            } else {
                update.insert("name".into(), serde_json::Value::String("".to_string()));
            }

            if let Some(d) = description {
                update.insert("description".into(), serde_json::Value::String(d.clone()));
            } else if let Some(cur) = &current {
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

            // Fill in required fields for Project deserialization
            if let Some(cur) = &current {
                update.insert(
                    "active".into(),
                    cur.get("active")
                        .cloned()
                        .unwrap_or(serde_json::Value::Bool(true)),
                );
                if let Some(cat) = cur.get("category") {
                    update.insert("category".into(), cat.clone());
                }
            } else {
                update.insert("active".into(), serde_json::Value::Bool(true));
                update.insert("category".into(), serde_json::Value::String("".to_string()));
            }

            let _ = runtime
                .core
                .update_project(serde_json::Value::Object(update))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("项目 {} 已更新", id));
        }
        ProjectCommands::Delete { id } => {
            let _ = runtime
                .core
                .delete_project(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("项目 {} 已删除", id));
        }
        ProjectCommands::Stats { id, json } => {
            let stats: serde_json::Value = runtime
                .core
                .get_project_stats(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json {
                print_json(&stats);
            } else {
                println!("\n  项目统计: {}", id);
                println!("  {}", "─".repeat(40));
                if let Some(total) = stats.get("total").and_then(|v| v.as_u64()) {
                    println!("    总任务: {}", total);
                }
                if let Some(completed) = stats.get("completed").and_then(|v| v.as_u64()) {
                    println!("    已完成: {}", completed);
                }
                if let Some(pending) = stats.get("pending").and_then(|v| v.as_u64()) {
                    println!("    待完成: {}", pending);
                }
                if let Some(p) = stats.get("progress").and_then(|v| v.as_f64()) {
                    println!("    进度: {:.1}%", p);
                }
            }
        }
        ProjectCommands::Todos { id, json } => {
            let todos: serde_json::Value = runtime
                .core
                .get_project_todos(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let todos = todos.as_array().cloned().unwrap_or_default();
            if *json {
                print_json(&todos);
            } else {
                println!("\n  项目任务 ({}):", todos.len());
                for t in &todos {
                    let id = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let text = t.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let done = if t
                        .get("completed")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        "●"
                    } else {
                        "○"
                    };
                    println!("    {} {} {}", id, done, text);
                }
            }
        }
    }
    Ok(())
}
