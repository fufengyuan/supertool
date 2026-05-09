use crate::runtime::CliRuntime;
use crate::output::{print_json, print_error, print_success};
use anyhow::{Result, anyhow};

pub async fn cmd_note(runtime: &mut CliRuntime, action: &crate::types::NoteCommands) -> Result<()> {
    use crate::types::NoteCommands;
    match action {
        NoteCommands::List { query, group_id, json } => {
            let result = runtime.core.get_all_notes(query.clone(), group_id.clone()).await.map_err(|e| anyhow!(e))?;
            if *json { print_json(&result); } else { print_notes_list(&result); }
        }
        NoteCommands::Add { title, content, group_id, tags } => {
            let data = serde_json::json!({
                "title": title, "content": content.as_deref().unwrap_or(""),
                "groupId": group_id, "tags": tags.as_deref().unwrap_or(""),
                "createdAt": chrono::Utc::now().to_rfc3339(),
                "updatedAt": chrono::Utc::now().to_rfc3339(),
            });
            let result = runtime.core.add_note(data).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success(&format!("笔记已添加: {}", title)); }
            else { print_error(&format!("添加失败: {}", result)); }
        }
        NoteCommands::Update { id, title, content, group_id, tags } => {
            let mut updates = serde_json::Map::new();
            if let Some(t) = title { updates.insert("title".to_string(), serde_json::Value::String(t.clone())); }
            if let Some(c) = content { updates.insert("content".to_string(), serde_json::Value::String(c.clone())); }
            if let Some(g) = group_id { updates.insert("groupId".to_string(), serde_json::Value::String(g.clone())); }
            if let Some(t) = tags { updates.insert("tags".to_string(), serde_json::Value::String(t.clone())); }
            updates.insert("updatedAt".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
            let result = runtime.core.update_note(id, serde_json::Value::Object(updates)).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success("笔记已更新"); }
            else { print_error(&format!("更新失败: {}", result)); }
        }
        NoteCommands::Delete { id } => {
            let result = runtime.core.delete_note(id).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success("笔记已删除"); }
            else { print_error(&format!("删除失败: {}", result)); }
        }
        NoteCommands::Groups { json } => {
            let result = runtime.core.get_all_note_groups().await.map_err(|e| anyhow!(e))?;
            if *json { print_json(&result); } else { print_note_groups(&result); }
        }
        NoteCommands::AddGroup { name, color } => {
            let data = serde_json::json!({ "name": name, "color": color.as_deref().unwrap_or("#6366f1") });
            let result = runtime.core.add_note_group(data).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success(&format!("分组已添加: {}", name)); }
            else { print_error(&format!("添加失败: {}", result)); }
        }
        NoteCommands::UpdateGroup { id, name, color } => {
            let mut updates = serde_json::Map::new();
            if let Some(n) = name { updates.insert("name".to_string(), serde_json::Value::String(n.clone())); }
            if let Some(c) = color { updates.insert("color".to_string(), serde_json::Value::String(c.clone())); }
            let result = runtime.core.update_note_group(id, serde_json::Value::Object(updates)).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success("分组已更新"); }
            else { print_error(&format!("更新失败: {}", result)); }
        }
        NoteCommands::DeleteGroup { id } => {
            let result = runtime.core.delete_note_group(id).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) { print_success("分组已删除"); }
            else { print_error(&format!("删除失败: {}", result)); }
        }
    }
    Ok(())
}

fn print_notes_list(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() { println!("暂无笔记"); return; }
        for (i, n) in arr.iter().enumerate() {
            let title = n.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let tags = n.get("tags").and_then(|v| v.as_str()).unwrap_or("");
            let group = n.get("groupName").and_then(|v| v.as_str()).unwrap_or("");
            println!("[1;36m[{}][0m {} {} {}", i + 1, title, if group.is_empty() { "".to_string() } else { format!("[33m({})[0m", group) }, if tags.is_empty() { "".to_string() } else { format!("[32m#{}[0m", tags) });
        }
    }
}

fn print_note_groups(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() { println!("暂无分组"); return; }
        for (i, g) in arr.iter().enumerate() {
            let name = g.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let color = g.get("color").and_then(|v| v.as_str()).unwrap_or("");
            println!("{} {} {}", i + 1, name, if color.is_empty() { "".to_string() } else { format!("[33m({})[0m", color) });
        }
    }
}
