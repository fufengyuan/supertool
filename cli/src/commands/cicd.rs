use crate::output;
use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use anyhow::Result;
use supertool_core::db::cicd::CicdConfig;

pub async fn cmd_cicd(rt: &mut CliRuntime, action: &CicdCommands) -> Result<()> {
    check_connection(rt)?;
    match action {
        CicdCommands::List { json } => {
            let configs = rt
                .core
                .get_cicd_configs()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || rt.json_mode {
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
                if *json || rt.json_mode {
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
            branch,
            json,
        } => {
            rt.set_json(*json);
            if *stream && *watch {
                eprintln!("  ⚠️ --stream 和 --watch 互斥，同时指定时 --stream 优先");
            }
            if let Some(b) = branch {
                eprintln!("  🌿 使用分支: {}", b);
            }
            if *stream {
                // Stream mode: deploy blocks until complete, output progress inline
                let resp = rt
                    .core
                    .cicd_deploy_with_branch(config_id, branch.clone())
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
                    if resp.get("requiresApproval").and_then(|v| v.as_bool()) == Some(true) {
                        // 需审批：结构化错误（exit 3），AI/CLI 可据此转 GUI 审批
                        return Err(output::fail(
                            output::EXIT_UNAUTHORIZED,
                            format!(
                                "配置「{}」已开启部署审核，请在 GUI 中手动确认部署",
                                resolve_cicd_name(rt, config_id)
                            ),
                        ));
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
                if rt.json_mode {
                    print_json(&resp);
                } else {
                    print_success(&format!(
                        "部署完成: {} (ID: {})\n日志: {}",
                        name, deploy_id, log_path
                    ));
                }
            } else {
                let resp = rt
                    .core
                    .cicd_deploy_with_branch(config_id, branch.clone())
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
                    if resp.get("requiresApproval").and_then(|v| v.as_bool()) == Some(true) {
                        return Err(output::fail(
                            output::EXIT_UNAUTHORIZED,
                            format!(
                                "配置「{}」已开启部署审核，请在 GUI 中手动确认部署",
                                resolve_cicd_name(rt, config_id)
                            ),
                        ));
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
                    // Poll deploy status until complete（JSON 模式下进度行走 stderr，避免 stdout 混流）
                    let human = !rt.json_mode;
                    if human {
                        println!("  🚀 部署已启动: {} ({}), 等待完成...", name, config_id);
                    }
                    let mut attempts = 0;
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        let history = rt
                            .core
                            .get_deploy_history_by_config(config_id, 1)
                            .map_err(|e| anyhow::anyhow!("{}", e))?;
                        if let Some(latest) = history.first() {
                            let status = &latest.status;
                            let deployed = &latest.start_time;
                            if human {
                                println!(
                                    "    [{}] 部署时间: {} | 状态: {}",
                                    attempts * 5,
                                    deployed.get(..16).unwrap_or(""),
                                    status
                                );
                            }
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
        CicdCommands::Logs {
            config_id,
            limit,
            json,
        } => {
            rt.set_json(*json);
            let logs = rt
                .core
                .get_deploy_logs_by_config(config_id, *limit as i64)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if rt.json_mode {
                print_json(&logs);
            } else {
                println!("  部署日志 ({} 条):", logs.len());
                for l in &logs {
                    println!("    {} {}", l.start_time, l.status);
                }
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
            if *json || rt.json_mode {
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
            json,
        } => {
            check_cicd_approval(rt, config_id)?;
            rt.set_json(*json);
            let resp = rt
                .core
                .cicd_rollback(config_id, deploy_log_id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
                let name = resolve_cicd_name(rt, config_id);
                if rt.json_mode {
                    print_json(&resp);
                } else {
                    print_success(&format!("回滚成功: {} → 版本 {}", name, deploy_log_id));
                }
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
        CicdCommands::Cancel { config_id, json } => {
            check_cicd_approval(rt, config_id)?;
            rt.set_json(*json);
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
                if rt.json_mode {
                    print_json(&resp);
                } else {
                    print_success(&format!("部署已取消: {} ({})", name, config_id));
                }
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
            if *json || rt.json_mode {
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
            let filtered: Vec<&supertool_core::db::cicd::DeployLog> = if let Some(s) = status {
                history.iter().filter(|h| h.status == *s).collect()
            } else {
                history.iter().collect()
            };

            if *json || rt.json_mode {
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
                        "cancelled" => "⛔",
                        _ => "⏳",
                    };
                    // 回滚标记在 errorMessage（"rolled-back:..."），有则加 ↩️
                    let rolled = h
                        .error_message
                        .as_deref()
                        .map(|m| m.contains("rolled-back:"))
                        .unwrap_or(false);
                    let deployed = h.start_time.get(..16).unwrap_or("");
                    let triggered = if h.triggered_by.is_empty() {
                        "manual"
                    } else {
                        &h.triggered_by
                    };
                    println!(
                        "    {} {}{} {} by {} ({})",
                        deploy_id,
                        icon,
                        if rolled { " ↩️" } else { "" },
                        status_val,
                        triggered,
                        deployed
                    );
                }
            }
        }
        CicdCommands::Tools { scan_path, json } => {
            // Detect installed build tools
            let tools = rt.core.detect_tools();
            let tool_paths = rt.core.detect_tool_paths();
            let sdk_versions = rt.core.detect_sdk_versions();

            let mut output = serde_json::json!({
                "tools": tools,
                "toolPaths": tool_paths,
                "sdkVersions": sdk_versions,
            });

            // Optionally scan a project path for modules
            if let Some(path) = scan_path {
                let scan_result = rt.core.scan_project_modules(path);
                output["projectScan"] = scan_result;
            }

            if *json || rt.json_mode {
                print_json(&output);
            } else {
                println!("\n  🔧 构建工具检测:");
                println!("  {}", "─".repeat(50));
                println!("  工具状态:");
                for (name, info) in &tools {
                    let icon = if info.available { "✅" } else { "❌" };
                    let version = info.version.as_deref().unwrap_or("-");
                    let path = info.path.as_deref().unwrap_or("");
                    println!("    {} {:<10} {:<16} {}", icon, name, version, path);
                }
                println!();
                println!("  工具路径:");
                println!("    JAVA_HOME  : {}", tool_paths.java_home);
                println!("    MAVEN_HOME : {}", tool_paths.maven_home);
                println!("    NODE_HOME  : {}", tool_paths.node_home);
                if !tool_paths.npm_home.is_empty() {
                    println!("    NPM_HOME   : {}", tool_paths.npm_home);
                }
                if !tool_paths.pnpm_home.is_empty() {
                    println!("    PNPM_HOME  : {}", tool_paths.pnpm_home);
                }
                if !tool_paths.yarn_home.is_empty() {
                    println!("    YARN_HOME  : {}", tool_paths.yarn_home);
                }

                // Print SDK versions
                if let Some(sdk_obj) = sdk_versions.as_object() {
                    println!();
                    println!("  SDK 版本:");
                    for (sdk_name, versions) in sdk_obj {
                        if let Some(ver_arr) = versions.as_array() {
                            let installed: Vec<String> = ver_arr
                                .iter()
                                .filter_map(|v| {
                                    if v.get("installed").and_then(|i| i.as_bool()) == Some(true) {
                                        v.get("version")
                                            .and_then(|ver| ver.as_str())
                                            .map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if installed.is_empty() {
                                println!("    {:<10} (无已安装版本)", sdk_name);
                            } else {
                                println!("    {:<10} {}", sdk_name, installed.join(", "));
                            }
                        }
                    }
                }

                // Print project scan results
                if let Some(scan) = output.get("projectScan") {
                    println!();
                    println!("  项目扫描:");
                    let has_pom = scan
                        .get("hasPomXml")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let has_gradle = scan
                        .get("hasBuildGradle")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let has_pkg = scan
                        .get("hasPackageJson")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    println!(
                        "    pom.xml: {} | build.gradle: {} | package.json: {}",
                        if has_pom { "✅" } else { "❌" },
                        if has_gradle { "✅" } else { "❌" },
                        if has_pkg { "✅" } else { "❌" }
                    );
                    if let Some(modules) = scan.get("modules").and_then(|v| v.as_array()) {
                        if !modules.is_empty() {
                            println!("    模块 ({}):", modules.len());
                            for m in modules {
                                let mname = m
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let mpath = m
                                    .get("path")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                println!("      {} ({})", mname, mpath);
                            }
                        }
                    }
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

/// 生产环境护栏：CICD 配置开启审批（requiresApproval=true）时，CLI 禁止执行部署/回滚/取消等变更操作。
/// 命中返回 exit code 3（未授权），供 AI/脚本识别后转 GUI。
fn check_cicd_approval(rt: &CliRuntime, config_id: &str) -> Result<(), anyhow::Error> {
    let configs = match rt.core.get_cicd_configs() {
        Ok(c) => c,
        Err(e) => return Err(anyhow::anyhow!("{}", e)),
    };
    if let Some(c) = configs.iter().find(|c| c.id == config_id) {
        if c.requires_approval {
            let name = if c.name.is_empty() { config_id } else { &c.name };
            return Err(output::fail(
                output::EXIT_UNAUTHORIZED,
                format!(
                    "配置「{}」已开启审批（生产环境），CLI 禁止操作。请在 GUI 中操作。",
                    name
                ),
            ));
        }
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
