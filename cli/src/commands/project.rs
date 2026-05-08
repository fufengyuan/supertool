use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_project(client: &ApiClient, action: &ProjectCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        ProjectCommands::List { json } => {
            let projects: Vec<serde_json::Value> = client.request("projects:get-all", None)?;
            if *json {
                print_json(&projects);
            } else {
                println!("\n  项目 ({}):", projects.len());
                // 按 category 分组
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for p in &projects {
                    let cat = p.get("category").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("未分类");
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
            let _ = client.request::<serde_json::Value>(
                "projects:add",
                Some(&serde_json::json!({"name": name, "description": description, "active": true})),
            )?;
            print_success(&format!("项目已添加: {}", name));
        }
        ProjectCommands::Show { id, json: _ } => {
            let projects: Vec<serde_json::Value> = client.request("projects:get-all", None)?;
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
            let mut update = serde_json::Map::new();
            update.insert("id".into(), serde_json::Value::String(id.clone()));
            if let Some(n) = name {
                update.insert("name".into(), serde_json::Value::String(n.clone()));
            }
            if let Some(d) = description {
                update.insert(
                    "description".into(),
                    serde_json::Value::String(d.clone()),
                );
            }
            let _ = client.request::<serde_json::Value>("projects:update", Some(&serde_json::Value::Object(update)))?;
            print_success(&format!("项目 {} 已更新", id));
        }
        ProjectCommands::Delete { id } => {
            let _ = client.request::<serde_json::Value>(
                "projects:delete",
                Some(&serde_json::json!({"id": id})),
            )?;
            print_success(&format!("项目 {} 已删除", id));
        }
        ProjectCommands::Stats { id, json } => {
            let stats: serde_json::Value = client.request(
                "projects:stats",
                Some(&serde_json::json!({"id": id})),
            )?;
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
            let todos: Vec<serde_json::Value> =
                client.request("projects:get-todos", Some(&serde_json::json!({"id": id})))?;
            if *json {
                print_json(&todos);
            } else {
                println!("\n  项目任务 ({}):", todos.len());
                for t in &todos {
                    let id = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let text = t.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let done = if t.get("completed").and_then(|v| v.as_bool()).unwrap_or(false) {
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
