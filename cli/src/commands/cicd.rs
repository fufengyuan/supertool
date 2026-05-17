use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;
use supertool_core::db::cicd::{CicdConfig, DeployHistory};

pub async fn cmd_cicd(rt: &mut CliRuntime, action: &CicdCommands) -> Result<()> {
    check_connection(rt)?;
    match action {
        CicdCommands::List { json } => {
            let configs = rt
                .core
                .get_cicd_configs()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json {
                print_json(&configs);
            } else {
                println!("\n  CI/CD 配置 ({}):", configs.len());
                // 按分组聚合
                let mut groups: Vec<(String, Vec<&CicdConfig>)> = Vec::new();
                for c in &configs {
                    let group_name = if !c.group_name.is_empty() {
                        &c.group_name
                    } else {
                        "未分组"
                    };
                    if let Some((_, items)) = groups.iter_mut().find(|(g, _)| g == group_name) {
                        items.push(c);
                    } else {
                        groups.push((group_name.to_string(), vec![c]));
                    }
                }
                for (group_name, items) in &groups {
                    println!("▸ {}", group_name);
                    for c in items {
                        println!("  {}  {}", c.id, c.name);
                    }
                    println!();
                }
            }
        }
        CicdCommands::Status { project_id, json } => {
            let configs = rt
                .core
                .get_cicd_configs()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let config = configs.iter().find(|c| c.id == *project_id);
            if let Some(c) = config {
                if *json {
                    print_json(c);
                } else {
                    let name = if !c.name.is_empty() {
                        &c.name
                    } else if !c.group_name.is_empty() {
                        &c.group_name
                    } else {
                        "-"
                    };
                    let build_tool = c.build_tool.as_deref().unwrap_or("-");
                    let deploy_branch = &c.deploy_branch;
                    let deploy_path = &c.deploy_path;
                    let restart_script = &c.restart_script;
                    let last_deployed = c.last_deployed_at.as_deref().unwrap_or("-");
                    let server_count = c
                        .servers
                        .as_ref()
                        .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(s).ok())
                        .map(|v| v.len())
                        .unwrap_or(0);
                    println!("\n  CI/CD 配置详情:");
                    println!("  {}", "─".repeat(40));
                    println!(
                        "    名称: {}\n    构建工具: {}\n    部署分支: {}\n    部署路径: {}\n    重启脚本: {}\n    服务器数: {}\n    上次部署: {}",
                        name,
                        build_tool,
                        deploy_branch,
                        deploy_path,
                        restart_script,
                        server_count,
                        last_deployed
                    );
                    if !c.maven_profile.is_empty() {
                        println!("    Maven Profile: {}", c.maven_profile);
                    }
                    if c.lib_separate {
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
                // Stream mode: deploy blocks until complete, output progress inline
                let resp = rt
                    .core
                    .cicd_deploy(config_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
                    if resp.get("requiresApproval").and_then(|v| v.as_bool()) == Some(true) {
                        let name = resolve_cicd_name(rt, config_id);
                        anyhow::bail!(
                            "⚠️ 配置「{}」已开启部署审核，请在 GUI 中手动确认部署。",
                            name
                        );
                    }
                    anyhow::bail!(
                        "部署失败: {}",
                        resp.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误")
                    );
                }
                let name = resolve_cicd_name(rt, config_id);
                let deploy_id = resp.get("deployId").and_then(|v| v.as_str()).unwrap_or("");
                let log_path = resp
                    .get("logFilePath")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                print_success(&format!(
                    "部署完成: {} (ID: {})\n日志: {}",
                    name, deploy_id, log_path
                ));
            } else {
                let resp = rt
                    .core
                    .cicd_deploy(config_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
                    if resp.get("requiresApproval").and_then(|v| v.as_bool()) == Some(true) {
                        let name = resolve_cicd_name(rt, config_id);
                        anyhow::bail!(
                            "⚠️ 配置「{}」已开启部署审核，请在 GUI 中手动确认部署。",
                            name
                        );
                    }
                    anyhow::bail!(
                        "部署失败: {}",
                        resp.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误")
                    );
                }
                let name = resolve_cicd_name(rt, config_id);
                let deploy_id = resp.get("deployId").and_then(|v| v.as_str()).unwrap_or("");
                let log_path = resp
                    .get("logFilePath")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if *watch {
                    // Poll deploy status until complete
                    println!("  🚀 部署已启动: {} ({}), 等待完成...", name, config_id);
                    let mut attempts = 0;
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        let history = rt
                            .core
                            .get_deploy_history_by_config(config_id, 1)
                            .map_err(|e| anyhow::anyhow!("{}", e))?;
                        if let Some(latest) = history.first() {
                            let status = &latest.status;
                            let deployed = &latest.deployed_at;
                            println!(
                                "    [{}] 部署时间: {} | 状态: {}",
                                attempts * 5,
                                deployed.get(..16).unwrap_or(""),
                                status
                            );
                            if status == "success" || status.contains("success") {
                                print_success(&format!("部署完成: {}", name));
                                break;
                            }
                            if status == "failed" {
                                print_error(&format!("部署失败: {}", name));
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
                        }
                    }
                } else {
                    print_success(&format!(
                        "部署完成: {} (ID: {})\n日志: {}",
                        name, deploy_id, log_path
                    ));
                }
            }
        }
        CicdCommands::Logs { config_id, limit } => {
            let logs = rt
                .core
                .get_deploy_logs_by_config(config_id, *limit as i64)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("  部署日志 ({} 条):", logs.len());
            for l in &logs {
                println!("    {} {}", l.start_time, l.status);
            }
        }
        CicdCommands::StepLogs {
            deploy_log_id,
            json,
        } => {
            let step_logs = rt
                .core
                .get_deploy_step_logs(deploy_log_id)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json {
                print_json(&step_logs);
            } else {
                if !step_logs.is_empty() {
                    println!("\n  部署步骤日志: {}", deploy_log_id);
                    println!("  {}", "─".repeat(60));
                    for step in &step_logs {
                        let icon = match step.status.as_str() {
                            "success" => "✅",
                            "failed" => "❌",
                            _ => "⏳",
                        };
                        println!("  {} {}", icon, step.stage);
                        if let Some(ref msg) = step.message {
                            if !msg.trim().is_empty() {
                                for line in msg.lines().take(10) {
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
            let resp = rt
                .core
                .cicd_rollback(config_id, deploy_log_id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
                let name = resolve_cicd_name(rt, config_id);
                print_success(&format!("回滚成功: {} → 版本 {}", name, deploy_log_id));
            } else {
                let errors = resp
                    .get("errors")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let error_msgs: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.as_str().map(|s| s.to_string()))
                    .collect();
                if error_msgs.is_empty() {
                    anyhow::bail!("回滚失败: 未知错误");
                } else {
                    anyhow::bail!("回滚部分失败: {}", error_msgs.join("; "));
                }
            }
        }
        CicdCommands::Cancel { config_id } => {
            // Get latest running deploy for this config
            let history = rt
                .core
                .get_deploy_history_by_config(config_id, 1)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let deploy_log_id = history
                .first()
                .filter(|h| h.status == "running" || h.status == "pending")
                .map(|h| h.id.clone())
                .ok_or_else(|| anyhow::anyhow!("未找到正在进行的部署"))?;
            let resp = rt
                .core
                .cicd_cancel_deploy(&deploy_log_id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
                let name = resolve_cicd_name(rt, config_id);
                print_success(&format!("部署已取消: {} ({})", name, config_id));
            } else {
                anyhow::bail!(
                    "取消失败: {}",
                    resp.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                );
            }
        }
        CicdCommands::Modules { config_id, json } => {
            let modules = rt
                .core
                .get_deploy_modules(config_id)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json {
                print_json(&modules);
            } else {
                println!("  部署模块 ({}):", modules.len());
                for m in &modules {
                    println!(
                        "    {} → {}",
                        m.module_name,
                        m.deploy_path.as_deref().unwrap_or("")
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
            let history = rt
                .core
                .get_deploy_history_by_config(config_id, *limit as i64)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Filter by status if specified
            let filtered: Vec<&DeployHistory> = if let Some(s) = status {
                history.iter().filter(|h| h.status == *s).collect()
            } else {
                history.iter().collect()
            };

            if *json {
                print_json(&filtered);
            } else if filtered.is_empty() {
                println!("  无部署记录");
            } else {
                println!("  部署历史 ({} 条):", filtered.len());
                for h in filtered {
                    let deploy_id = &h.id;
                    let status_val = &h.status;
                    let icon = match status_val.as_str() {
                        "success" => "✅",
                        "failed" => "❌",
                        "rolled_back" => "↩️",
                        "cancelled" => "⛔",
                        _ => "⏳",
                    };
                    let deployed = h.deployed_at.get(..16).unwrap_or("");
                    let triggered = "manual"; // TODO: get from deploy log if available
                    println!(
                        "    {} {} {} by {} ({})",
                        deploy_id, icon, status_val, triggered, deployed
                    );
                }
            }
        }
    }
    Ok(())
}

/// Check connection by attempting a simple DB read
fn check_connection(rt: &CliRuntime) -> Result<()> {
    let result = rt.core.db_read(|conn| {
        conn.prepare("SELECT 1")
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    if result.is_err() {
        anyhow::bail!("无法连接到 SuperTool 数据库\n请确保数据目录存在且数据库文件未损坏。");
    }
    Ok(())
}

fn resolve_cicd_name(rt: &CliRuntime, config_id: &str) -> String {
    let configs = match rt.core.get_cicd_configs() {
        Ok(c) => c,
        Err(_) => return config_id.to_string(),
    };
    configs
        .iter()
        .find(|c| c.id == config_id)
        .map(|c| c.name.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| config_id.to_string())
}
