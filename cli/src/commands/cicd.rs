use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_cicd(client: &ApiClient, action: &CicdCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        CicdCommands::List { json } => {
            let configs: Vec<serde_json::Value> = client.request("cicd:get-all-configs", None)?;
            if *json {
                print_json(&configs);
            } else {
                println!("\n  CI/CD 配置 ({}):", configs.len());
                // 按分组聚合
                let mut groups: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for c in &configs {
                    let group_name = c.get("groupName").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("未分组");
                    if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == group_name) {
                        items.push(c);
                    } else {
                        groups.push((group_name.to_string(), vec![c]));
                    }
                }
                for (group_name, items) in &groups {
                    println!("▸ {}", group_name);
                    for c in items {
                        let config_id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let config_name = c.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("-");
                        println!("  {}  {}", config_id, config_name);
                    }
                    println!();
                }
            }
        }
        CicdCommands::Status { project_id, json } => {
            let configs: Vec<serde_json::Value> = client.request("cicd:get-all-configs", None)?;
            let config = configs.iter().find(|c| {
                c.get("projectId").and_then(|v| v.as_str()) == Some(project_id.as_str())
                    || c.get("id").and_then(|v| v.as_str()) == Some(project_id.as_str())
            });
            if let Some(c) = config {
                if *json {
                    print_json(c);
                } else {
                    let name = c
                        .get("name")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .or_else(|| c.get("groupName").and_then(|v| v.as_str()))
                        .unwrap_or("-");
                    let build_tool = c.get("buildTool").and_then(|v| v.as_str()).unwrap_or("-");
                    let deploy_path = c.get("deployPath").and_then(|v| v.as_str()).unwrap_or("-");
                    let deploy_branch = c.get("deployBranch").and_then(|v| v.as_str()).unwrap_or("-");
                    let restart_script = c.get("restartScript").and_then(|v| v.as_str()).unwrap_or("-");
                    let last_deployed = c.get("lastDeployedAt").and_then(|v| v.as_str()).unwrap_or("-");
                    let servers_raw = c.get("servers").and_then(|v| v.as_str()).unwrap_or("[]");
                    let server_count =
                        serde_json::from_str::<Vec<serde_json::Value>>(servers_raw)
                            .map(|v| v.len())
                            .unwrap_or(0);
                    println!("\n  CI/CD 配置详情:");
                    println!("  {}", "─".repeat(40));
                    println!(
                        "    名称: {}\n    构建工具: {}\n    部署分支: {}\n    部署路径: {}\n    重启脚本: {}\n    服务器数: {}\n    上次部署: {}",
                        name, build_tool, deploy_branch, deploy_path, restart_script, server_count, last_deployed
                    );
                    if let Some(maven) = c.get("mavenProfile").and_then(|v| v.as_str()) {
                        println!("    Maven Profile: {}", maven);
                    }
                    if c.get("libSeparate").and_then(|v| v.as_bool()).unwrap_or(false) {
                        println!("    Lib 分离: 是");
                    }
                }
            } else {
                anyhow::bail!("未找到配置: {}", project_id);
            }
        }
        CicdCommands::Deploy {
            config_id,
            stream,
            watch,
        } => {
            if *stream && *watch {
                eprintln!("  ⚠️ --stream 和 --watch 互斥，同时指定时 --stream 优先");
            }
            if *stream {
                // Stream mode
                let mut reader = client.stream_request(
                    "cicd:deploy-stream",
                    Some(&serde_json::json!({"configId": config_id})),
                )?;
                use std::io::Write;
                use std::io::BufRead;
                let mut line_buf = String::new();
                loop {
                    line_buf.clear();
                    let n = reader.read_line(&mut line_buf)?;
                    if n == 0 {
                        break;
                    }
                    let line = line_buf.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                        if event.get("stream").and_then(|v| v.as_bool()) == Some(true) {
                            let event_type = event.get("event").and_then(|v| v.as_str()).unwrap_or("");
                            match event_type {
                                "start" => println!("\n  🚀 部署开始: {}", config_id),
                                "progress" | "step" | "data" => {
                                    let step = event.get("step").and_then(|v| v.as_str()).unwrap_or("");
                                    let detail = event
                                        .get("detail")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    println!("  ⏳ {} {}", step, detail);
                                }
                                "error" => {
                                    let err = event
                                        .get("error")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    eprintln!("  ❌ {}", err);
                                }
                                "complete" => {
                                    if event.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                        print_success("部署完成");
                                    } else {
                                        print_error("部署失败");
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    std::io::stdout().flush()?;
                }
            } else {
                let resp: serde_json::Value = client.request(
                    "cicd:deploy",
                    Some(&serde_json::json!({"configId": config_id})),
                )?;
                // Check if approval is required
                if resp
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let cfg_name =
                        resp.get("configName").and_then(|v| v.as_str()).unwrap_or("");
                    anyhow::bail!("⚠️ 配置「{}」已开启部署审核，CLI 不支持跳过审核。请在 GUI 中手动确认部署。", cfg_name);
                }
                let name = resolve_cicd_name(client, config_id);
                if *watch {
                    // Poll deploy status until complete
                    println!(
                        "  🚀 部署已启动: {} ({}), 等待完成...",
                        name, config_id
                    );
                    let mut attempts = 0;
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        let history: Vec<serde_json::Value> = client.request(
                            "cicd:get-deploy-history",
                            Some(&serde_json::json!({"configId": config_id, "limit": 1})),
                        )?;
                        if let Some(latest) = history.first() {
                            let status = latest
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("running");
                            let version = latest
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("-");
                            println!(
                                "    [{}] 版本: {} | 状态: {}",
                                attempts * 5,
                                version,
                                status
                            );
                            if status == "success" {
                                print_success(&format!("部署完成: {} 版本 {}", name, version));
                                break;
                            }
                            if status == "failed" {
                                let err = latest
                                    .get("errorMessage")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("未知错误");
                                print_error(&format!("部署失败: {}", err));
                                break;
                            }
                            if status == "cancelled" {
                                print_error("部署已取消");
                                break;
                            }
                        }
                        attempts += 1;
                        if attempts >= 120 {
                            print_error("部署超时 (10 分钟)");
                            break;
                        } // 5s * 120 = 600s
                    }
                } else {
                    print_success(&format!(
                        "部署已启动: {} ({})\n使用 --stream 或 --watch 查看进度",
                        name, config_id
                    ));
                }
            }
        }
        CicdCommands::Logs { project_id, .. } => {
            let logs: Vec<serde_json::Value> =
                client.request("cicd:get-logs", Some(&serde_json::json!({"configId": project_id})))?;
            println!("  部署日志 ({} 条):", logs.len());
            for l in &logs {
                println!(
                    "    {} {}",
                    l.get("timestamp").and_then(|v| v.as_str()).unwrap_or(""),
                    l.get("status").and_then(|v| v.as_str()).unwrap_or("")
                );
            }
        }
        CicdCommands::StepLogs {
            deploy_log_id,
            json,
        } => {
            let step_logs: serde_json::Value = client.request(
                "cicd:get-step-logs",
                Some(&serde_json::json!({"deployLogId": deploy_log_id})),
            )?;
            if *json {
                print_json(&step_logs);
            } else {
                if let Some(steps) = step_logs.get("steps").and_then(|v| v.as_array()) {
                    println!("\n  部署步骤日志: {}", deploy_log_id);
                    println!("  {}", "─".repeat(60));
                    for step in steps {
                        let name = step.get("step").and_then(|v| v.as_str()).unwrap_or("");
                        let status = step.get("status").and_then(|v| v.as_str()).unwrap_or("");
                        let icon = match status {
                            "success" => "✅",
                            "failed" => "❌",
                            "running" => "⏳",
                            _ => "⏳",
                        };
                        println!("  {} {}", icon, name);
                        if let Some(log) = step.get("log").and_then(|v| v.as_str()) {
                            if !log.trim().is_empty() {
                                for line in log.lines().take(10) {
                                    println!("    {}", line);
                                }
                            }
                        }
                    }
                } else {
                    println!("  无步骤日志");
                }
            }
        }
        CicdCommands::Rollback {
            config_id,
            deploy_log_id,
        } => {
            let resp: serde_json::Value = client.request(
                "cicd:rollback",
                Some(&serde_json::json!({"configId": config_id, "deployHistoryId": deploy_log_id})),
            )?;
            // Check if approval is required
            if resp
                .get("requiresApproval")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let cfg_name = resolve_cicd_name(client, &config_id);
                anyhow::bail!("⚠️ 配置「{}」已开启部署审核，CLI 不支持回滚。请在 GUI 中操作。", cfg_name);
            }
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = resolve_cicd_name(client, config_id);
                print_success(&format!(
                    "回滚已启动: {} → 版本 {}",
                    name, deploy_log_id
                ));
            } else {
                anyhow::bail!(
                    "回滚失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        CicdCommands::Cancel { config_id } => {
            let history: Vec<serde_json::Value> = client.request(
                "cicd:get-deploy-history",
                Some(&serde_json::json!({"configId": config_id, "limit": 1})),
            )?;
            let deploy_log_id = history
                .first()
                .and_then(|h| h.get("id").and_then(|v| v.as_str()))
                .unwrap_or("");
            if deploy_log_id.is_empty() {
                anyhow::bail!("未找到正在进行的部署");
            }
            let resp: serde_json::Value = client.request(
                "cicd:cancel",
                Some(&serde_json::json!({"deployLogId": deploy_log_id})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = resolve_cicd_name(client, config_id);
                print_success(&format!("部署已取消: {} ({})", name, config_id));
            } else {
                anyhow::bail!(
                    "取消失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        CicdCommands::Modules { config_id, json } => {
            let modules: Vec<serde_json::Value> =
                client.request("cicd:get-modules", Some(&serde_json::json!({"configId": config_id})))?;
            if *json {
                print_json(&modules);
            } else {
                println!("  部署模块 ({}):", modules.len());
                for m in &modules {
                    println!(
                        "    {} → {}",
                        m.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        m.get("deployPath").and_then(|v| v.as_str()).unwrap_or("")
                    );
                }
            }
        }
        CicdCommands::History {
            config_id,
            limit,
            status,
            json,
        } => {
            let mut params = serde_json::json!({"configId": config_id, "limit": limit});
            if let Some(s) = status {
                params["status"] = serde_json::Value::String(s.clone());
            }
            let history: Vec<serde_json::Value> = client.request(
                "cicd:get-deploy-history",
                Some(&params),
            )?;
            if *json {
                print_json(&history);
            } else if history.is_empty() {
                println!("  无部署记录");
            } else {
                println!("  部署历史 ({} 条):", history.len());
                for h in &history {
                    let deploy_id = h.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let status_val = h.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    let icon = match status_val {
                        "success" => "✅",
                        "failed" => "❌",
                        "rolled_back" => "↩️",
                        "cancelled" => "⛔",
                        _ => "⏳",
                    };
                    let version = h.get("version").and_then(|v| v.as_str()).unwrap_or("-");
                    let deployed = h
                        .get("deployedAt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .get(..16)
                        .unwrap_or("");
                    let trigger = h
                        .get("triggeredBy")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    println!(
                        "    {} {} {} {} by {} ({})",
                        deploy_id, icon, status_val, version, trigger, deployed
                    );
                }
            }
        }
    }
    Ok(())
}

fn resolve_cicd_name(client: &ApiClient, config_id: &str) -> String {
    let configs: Vec<serde_json::Value> = match client.request("cicd:get-all-configs", None) {
        Ok(c) => c,
        Err(_) => return config_id.to_string(),
    };
    configs
        .iter()
        .find(|c| {
            c.get("id").and_then(|v| v.as_str()) == Some(config_id)
                || c.get("projectId").and_then(|v| v.as_str()) == Some(config_id)
        })
        .and_then(|c| c.get("name").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .unwrap_or_else(|| config_id.to_string())
}
