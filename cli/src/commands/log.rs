use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_log(client: &ApiClient, action: &LogCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        LogCommands::List { json } => {
            let presets: Vec<serde_json::Value> = client.request("log-presets:get-all", None)?;
            if *json {
                print_json(&presets);
            } else {
                println!("\n  日志预设 ({}):", presets.len());
                // 按分组聚合
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for p in &presets {
                    let group = p.get("presetGroup").and_then(|v| v.as_str()).unwrap_or("未分组");
                    if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == group) {
                        items.push(p);
                    } else {
                        groups.push((group.to_string(), vec![p]));
                    }
                }
                for (group, items) in &groups {
                    println!("▸ {}", group);
                    for p in items {
                        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}  {}", id, name);
                    }
                    println!();
                }
            }
        }
        LogCommands::Add {
            name,
            server_ids,
            log_path,
            log_type,
        } => {
            let ids: Vec<String> = server_ids
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let _ = client.request::<serde_json::Value>(
                "log-presets:add",
                Some(&serde_json::json!({"name": name, "serverIds": ids, "logPath": log_path, "logType": log_type, "keywords": [], "maxLines": 1000})),
            )?;
            print_success(&format!(
                "日志预设已添加: {} ({} 服务器, {})",
                name, server_ids, log_path
            ));
        }
        LogCommands::Delete { id } => {
            let _ = client.request::<serde_json::Value>(
                "log-presets:delete",
                Some(&serde_json::json!({"id": id})),
            )?;
            print_success(&format!("日志预设已删除: {}", id));
        }
        LogCommands::Search {
            preset_id,
            keyword,
            lines,
        } => {
            let actual_id = resolve_preset_id(client, preset_id)?;
            let resp: serde_json::Value = client.request(
                "log:search",
                Some(&serde_json::json!({"presetId": actual_id, "keyword": keyword, "lines": lines})),
            )?;
            if !resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                anyhow::bail!(
                    "{}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
            let preset_name = resp
                .get("presetName")
                .and_then(|v| v.as_str())
                .unwrap_or(preset_id);
            println!(
                "\n  🔍 搜索日志: {} → \"{}\"",
                preset_name, keyword
            );
            println!("  {}", "─".repeat(60));
            if let Some(results) = resp.get("results").and_then(|v| v.as_array()) {
                for r in results {
                    let sid = r.get("serverId").and_then(|v| v.as_str()).unwrap_or("");
                    let sname = r.get("serverName").and_then(|v| v.as_str()).unwrap_or("");
                    let ok = r.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    if ok {
                        if let Some(output) = r.get("output").and_then(|v| v.as_str()) {
                            if !output.trim().is_empty() {
                                for line in output.trim().lines() {
                                    println!("  {}", line);
                                }
                            } else {
                                println!("  {} ({}) (无匹配)", sid, sname);
                            }
                        }
                    } else {
                        println!(
                            "  {} ({}) (失败: {})",
                            sid,
                            sname,
                            r.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                        );
                    }
                }
            }
        }
        LogCommands::Tail { preset_id, lines } => {
            let actual_id = resolve_preset_id(client, preset_id)?;
            client.sse_tail(&actual_id, *lines, true)?;
        }
    }
    Ok(())
}

fn resolve_preset_id(client: &ApiClient, preset_id: &str) -> Result<String> {
    if let Ok(idx) = preset_id.parse::<usize>() {
        let presets: Vec<serde_json::Value> = client.request("log-presets:get-all", None)?;
        if idx == 0 || idx > presets.len() {
            anyhow::bail!("预设序号 {} 超出范围 (1-{})", idx, presets.len());
        }
        let actual_id = presets
            .get(idx - 1)
            .and_then(|p| p.get("id").and_then(|v| v.as_str()))
            .unwrap_or("");
        if actual_id.is_empty() {
            anyhow::bail!("预设 #{} 没有有效 ID", idx);
        }
        return Ok(actual_id.to_string());
    }
    Ok(preset_id.to_string())
}
