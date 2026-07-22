use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;

pub async fn cmd_log(runtime: &mut CliRuntime, action: &LogCommands) -> Result<()> {
    match action {
        LogCommands::List { json } => {
            let presets: serde_json::Value = runtime
                .core
                .get_log_presets()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let presets = presets.as_array().cloned().unwrap_or_default();
            if *json {
                print_json(&presets);
            } else {
                println!("\n  日志预设 ({}):", presets.len());
                // 按分组聚合
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for p in &presets {
                    let group = p
                        .get("presetGroup")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未分组");
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
            let _ = runtime
                .core
                .add_log_preset(serde_json::json!({
                    "name": name,
                    "serverIds": ids,
                    "logPath": log_path,
                    "logType": log_type,
                    "keywords": [],
                    "maxLines": 1000
                }))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!(
                "日志预设已添加: {} ({} 服务器, {})",
                name, server_ids, log_path
            ));
        }
        LogCommands::Delete { id } => {
            let _ = runtime
                .core
                .delete_log_preset(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("日志预设已删除: {}", id));
        }
        LogCommands::Search {
            preset_id,
            keyword,
            lines,
        } => {
            let actual_id = resolve_preset_id(runtime, preset_id).await?;
            let resp: serde_json::Value = runtime
                .core
                .log_search(&actual_id, keyword, *lines)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            
            let preset_name = resp
                .get("presetId")
                .and_then(|v| v.as_str())
                .unwrap_or(preset_id);
            println!("\n  🔍 搜索日志: {} → \"{}\"", preset_name, keyword);
            println!("  {}", "─".repeat(60));
            if let Some(matches) = resp.get("matches").and_then(|v| v.as_array()) {
                for r in matches {
                    let sid = r.get("serverId").and_then(|v| v.as_str()).unwrap_or("");
                    let sname = r.get("serverName").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(error) = r.get("error").and_then(|v| v.as_str()) {
                        if !error.is_empty() {
                            println!("  {} ({}) ❌ {}", sid, sname, error);
                            continue;
                        }
                    }
                    if let Some(lines_arr) = r.get("lines").and_then(|v| v.as_array()) {
                        let match_count = r.get("matchCount").and_then(|v| v.as_u64()).unwrap_or(0);
                        if match_count == 0 {
                            println!("  {} ({}) (无匹配)", sid, sname);
                        } else {
                            for line in lines_arr {
                                // lines 可能是字符串（tail）或对象 {content, isMatch, lineNum}（search）
                                let text = line
                                    .as_str()
                                    .or_else(|| {
                                        line.get("content").and_then(|v| v.as_str())
                                    })
                                    .unwrap_or("");
                                if !text.is_empty() {
                                    println!("  {}", text);
                                }
                            }
                        }
                    }
                }
            }
        }
        LogCommands::Tail { preset_id, lines } => {
            let actual_id = resolve_preset_id(runtime, preset_id).await?;
            // CoreService log_tail returns a static result (not streaming)
            let resp: serde_json::Value = runtime
                .core
                .log_tail(&actual_id, *lines)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if let Some(results) = resp.get("results").and_then(|v| v.as_array()) {
                for r in results {
                    let sname = r.get("serverName").and_then(|v| v.as_str()).unwrap_or("");
                    let sid = r.get("serverId").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(error) = r.get("error").and_then(|v| v.as_str()) {
                        if !error.is_empty() {
                            eprintln!("  ❌ {} ({}) 错误: {}", sid, sname, error);
                        }
                    }
                    if let Some(lines_arr) = r.get("lines").and_then(|v| v.as_array()) {
                        for line in lines_arr {
                            if let Some(l) = line.as_str() {
                                println!("  {}", l);
                            }
                        }
                    }
                }
            }
        }
        LogCommands::Context {
            preset_id,
            server_id,
            line_num,
            context_lines,
        } => {
            let actual_id = resolve_preset_id(runtime, preset_id).await?;
            let resp: serde_json::Value = runtime
                .core
                .log_context(&actual_id, server_id, *line_num, *context_lines)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let sname = resp
                .get("serverName")
                .and_then(|v| v.as_str())
                .unwrap_or(server_id);
            let start = resp.get("start").and_then(|v| v.as_u64()).unwrap_or(0);
            let end = resp.get("end").and_then(|v| v.as_u64()).unwrap_or(0);
            println!(
                "\n  📋 日志上下文: {} (行 {} 周边, 范围 {}-{})",
                sname, line_num, start, end
            );
            println!("  {}", "─".repeat(60));
            if let Some(lines) = resp.get("lines").and_then(|v| v.as_array()) {
                for l in lines {
                    let ln = l
                        .get("lineNum")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let content = l
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let marker = if ln as usize == *line_num {
                        "▶"
                    } else {
                        " "
                    };
                    println!("  {} {:>6} │ {}", marker, ln, content);
                }
            } else {
                println!("  (无上下文内容)");
            }
        }
    }
    Ok(())
}

async fn resolve_preset_id(runtime: &mut CliRuntime, preset_id: &str) -> Result<String> {
    if let Ok(idx) = preset_id.parse::<usize>() {
        let presets: serde_json::Value = runtime
            .core
            .get_log_presets()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let presets = presets.as_array().cloned().unwrap_or_default();
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
