use crate::output;
use crate::output::*;
use crate::runtime::CliRuntime;
use crate::types::*;
use crate::utils::*;
use anyhow::Result;

/// 生产环境护栏：服务器开启审批（requiresApproval=true）时，CLI 完全禁止操作（读写都不行）。
/// 命中返回 exit code 3（未授权），供 AI/脚本识别后转 GUI。
async fn check_server_approval(rt: &mut CliRuntime, id: &str) -> Result<()> {
    let servers: serde_json::Value = rt
        .core
        .get_all_servers()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    for s in servers.as_array().cloned().unwrap_or_default() {
        if s.get("id").and_then(|v| v.as_str()) == Some(id) {
            if s.get("requiresApproval").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = s.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                return Err(output::fail(
                    output::EXIT_UNAUTHORIZED,
                    format!(
                        "服务器「{}」已开启审批（生产环境），CLI 禁止操作。请在 GUI 中操作。",
                        name
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub async fn cmd_server(runtime: &mut CliRuntime, action: &ServerCommands) -> Result<()> {
    match action {
        ServerCommands::List { json } => {
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            if *json || runtime.json_mode {
                print_json(&servers);
            } else {
                println!("\n  服务器 ({}):", servers.len());
                // 获取分组
                let groups: serde_json::Value = runtime
                    .core
                    .get_all_server_groups()
                    .await
                    .unwrap_or(serde_json::json!([]));
                let group_map: std::collections::HashMap<String, String> = groups
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|g| {
                        let id = g.get("id").and_then(|v| v.as_str())?;
                        let name = g.get("name").and_then(|v| v.as_str())?;
                        Some((id.to_string(), name.to_string()))
                    })
                    .collect();
                // 按分组聚合
                let mut groups_order: Vec<(String, Vec<&serde_json::Value>)> = Vec::new();
                for s in &servers {
                    let gid = s.get("groupId").and_then(|v| v.as_str()).unwrap_or("");
                    let group_name = if gid.is_empty() {
                        "未分组".to_string()
                    } else {
                        group_map
                            .get(gid)
                            .cloned()
                            .unwrap_or_else(|| gid.to_string())
                    };
                    if let Some((_, items)) =
                        groups_order.iter_mut().find(|(g, _)| g == &group_name)
                    {
                        items.push(s);
                    } else {
                        groups_order.push((group_name, vec![s]));
                    }
                }
                for (group_name, items) in &groups_order {
                    println!("▸ {}", group_name);
                    for s in items {
                        let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}  {}", id, name);
                    }
                    println!();
                }
            }
        }
        ServerCommands::Add {
            name,
            host,
            port,
            user,
        } => {
            let _ = runtime
                .core
                .add_server(serde_json::json!({
                    "name": name,
                    "host": host,
                    "port": port.unwrap_or(22),
                    "username": user.as_deref().unwrap_or("root"),
                    "type": "ssh",
                    "password": ""
                }))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("服务器已添加: {} ({})", name, host));
        }
        ServerCommands::Delete { id } => {
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            let name = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
                .and_then(|s| s.get("name").and_then(|v| v.as_str()))
                .unwrap_or(id.as_str());
            let _ = runtime
                .core
                .delete_server(id)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            print_success(&format!("服务器已删除: {}", name));
        }
        ServerCommands::Test { id } => {
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            if let Some(s) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                let config = serde_json::json!({
                    "id": id,
                    "host": s.get("host").and_then(|v| v.as_str()).unwrap_or(""),
                    "port": s.get("port").and_then(|v| v.as_u64()).unwrap_or(22),
                    "username": s.get("username").and_then(|v| v.as_str()).unwrap_or("root"),
                    "name": s.get("name").and_then(|v| v.as_str()).unwrap_or("")
                });
                let resp: serde_json::Value = runtime
                    .core
                    .test_server_connection(config)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if resp
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    print_success("连接测试成功");
                } else {
                    print_error(&format!(
                        "连接失败: {}",
                        resp.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知错误")
                    ));
                }
            } else {
                anyhow::bail!("未找到服务器: {}（可用 stool server list 核对 id）", id);
            }
        }
        ServerCommands::Exec {
            id,
            command,
            timeout,
            json,
        } => {
            runtime.set_json(*json);
            // 拦截高危命令（CLI 层防护，防止 AI 误操作）
            if is_dangerous_command(command) {
                return Err(output::fail(
                    output::EXIT_DANGEROUS,
                    "检测到高危命令，CLI 已拦截。如需执行请在 GUI 中手动操作。",
                ));
            }
            // 检查服务器是否开启执行审核
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            if let Some(server) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                if server
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let name = server.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                    return Err(output::fail(
                        output::EXIT_UNAUTHORIZED,
                        format!(
                            "服务器「{}」已开启审批（生产环境），CLI 禁止操作。请在 GUI 中操作。",
                            name
                        ),
                    ));
                }
            }
            let resp: serde_json::Value = runtime
                .core
                .exec_ssh_command(id, command)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                if runtime.json_mode {
                    print_json(&resp);
                } else if let Some(output) = resp.get("output").and_then(|v| v.as_str()) {
                    println!("{}", output);
                }
            } else {
                anyhow::bail!(
                    "执行失败: {}",
                    resp.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                );
            }
            let _ = timeout; // timeout is handled by CoreService internally
        }
        ServerCommands::Health { id, json } => {
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            let server = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()));
            let server = match server {
                Some(s) => s,
                None => anyhow::bail!("未找到服务器: {}", id),
            };
            let name = server
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id.as_str());
            let checks = [
                (
                    "磁盘",
                    "df -h / | tail -1 | awk '{print \"已用: \" $3 \" / \" $2 \" (\" $5 \")\"}'",
                ),
                (
                    "内存",
                    "free -m | awk 'NR==2{printf \"已用: %sMB / %sMB (%.1f%%)\\n\", $3, $2, $3*100/$2}'",
                ),
                (
                    "CPU 负载",
                    "uptime | awk -F'load average:' '{print $2}' | xargs",
                ),
                (
                    "Docker",
                    "docker ps --format 'table {{.Names}}\t{{.Status}}' 2>/dev/null || echo 'Docker 未运行'",
                ),
                (
                    "关键进程",
                    "ps aux | grep -E 'java|nginx|node|python' | grep -v grep | head -5 || echo '无关键进程'",
                ),
            ];
            if *json || runtime.json_mode {
                let mut results = serde_json::Map::new();
                results.insert("server".into(), serde_json::json!({"name": name, "id": id}));
                let mut items = Vec::new();
                for (label, cmd) in &checks {
                    let resp: serde_json::Value = runtime
                        .core
                        .exec_ssh_command(id, cmd)
                        .await
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    items.push(serde_json::json!({
                        "check": label,
                        "ok": resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                        "output": resp.get("output").and_then(|v| v.as_str()).unwrap_or("").trim()
                    }));
                }
                results.insert("checks".into(), serde_json::Value::Array(items));
                print_json(&results);
            } else {
                println!("\n  🏥 服务器健康检查: {}", name);
                println!("  {}", "─".repeat(40));
                for (label, cmd) in &checks {
                    let resp: serde_json::Value = runtime
                        .core
                        .exec_ssh_command(id, cmd)
                        .await
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    let output = resp
                        .get("output")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    println!(
                        "\n  {} {}:",
                        if resp
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                        {
                            "✅"
                        } else {
                            "❌"
                        },
                        label
                    );
                    if !output.is_empty() {
                        for line in output.lines() {
                            println!("    {}", line);
                        }
                    } else {
                        println!("    (无输出)");
                    }
                }
            }
        }
        ServerCommands::Diagnose { id, json } => {
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            let server = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()));
            let server = match server {
                Some(s) => s,
                None => anyhow::bail!("未找到服务器: {}", id),
            };
            let name = server
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(id.as_str());
            let script = r#"echo "=== SYSTEM ===" && uname -a && echo "=== DISK ===" && df -h / && echo "=== MEMORY ===" && free -m && echo "=== LOAD ===" && uptime && echo "=== DOCKER ===" && docker ps --format 'table {{.Names}}\t{{.Status}}' 2>/dev/null || echo "No Docker" && echo "=== ERRORS (syslog) ===" && tail -n 20 /var/log/syslog 2>/dev/null | grep -i error && echo "=== ERRORS (kern) ===" && dmesg 2>/dev/null | tail -n 10 && echo "=== TOP PROCESSES ===" && ps aux --sort=-%cpu | head -6"#;
            let resp: serde_json::Value = runtime
                .core
                .exec_ssh_command(id, script)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || runtime.json_mode {
                print_json(&serde_json::json!({
                    "server": name,
                    "success": resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                    "output": resp.get("output").and_then(|v| v.as_str()).unwrap_or("")
                }));
            } else {
                println!("\n  🩺 智能诊断: {}", name);
                println!("  {}", "─".repeat(40));
                if resp
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    if let Some(output) = resp.get("output").and_then(|v| v.as_str()) {
                        if !output.trim().is_empty() {
                            println!("{}", output);
                        } else {
                            println!("  (无输出)");
                        }
                    }
                } else {
                    println!(
                        "  ❌ 失败: {}",
                        resp.get("error").and_then(|v| v.as_str()).unwrap_or("")
                    );
                }
            }
        }
        ServerCommands::Read { id, path, json } => {
            runtime.set_json(*json);
            // Try SFTP download first (returns base64 content)
            let result = runtime.core.sftp_download_file(id, path).await;
            if let Ok(resp) = result {
                if let Some(content) = resp.get("content").and_then(|v| v.as_str()) {
                    // Try to decode base64
                    if let Ok(decoded) = base64_decode(content) {
                        if let Ok(text) = String::from_utf8(decoded) {
                            if runtime.json_mode {
                                print_json(&serde_json::json!({"path": path, "content": text}));
                            } else {
                                println!("  📄 {}\n  {}", path, "─".repeat(40));
                                println!("{}", text);
                            }
                            return Ok(());
                        }
                    }
                    // Not valid UTF-8, print as text anyway
                    if runtime.json_mode {
                        print_json(&serde_json::json!({"path": path, "content": content}));
                    } else {
                        println!("  📄 {}\n  {}", path, "─".repeat(40));
                        println!("{}", content);
                    }
                    return Ok(());
                }
            }
            // Fallback to exec
            let resp: serde_json::Value = runtime
                .core
                .exec_ssh_command(id, &format!("cat {}", shell_quote(path)))
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                if runtime.json_mode {
                    print_json(&resp);
                } else if let Some(output) = resp.get("output").and_then(|v| v.as_str()) {
                    println!("  📄 {}\n  {}", path, "─".repeat(40));
                    println!("{}", output);
                }
            } else {
                anyhow::bail!(
                    "读取文件失败: {}",
                    resp.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                );
            }
        }
        ServerCommands::Ls { id, path, json } => {
            let path_ref = path.as_deref().unwrap_or("/");
            let resp: serde_json::Value = runtime
                .core
                .sftp_list_dir(id, path_ref)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if let Some(error) = resp.get("error").and_then(|v| v.as_str()) {
                anyhow::bail!("{}", error);
            }
            let files: Vec<serde_json::Value> = resp.as_array().cloned().unwrap_or_default();
            if *json || runtime.json_mode {
                print_json(&files);
            } else {
                println!(
                    "\n  📁 {} ({} 项):",
                    path.as_deref().unwrap_or("/"),
                    files.len()
                );
                println!("  {}", "─".repeat(60));
                for f in &files {
                    let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let ftype = f.get("type").and_then(|v| v.as_str()).unwrap_or("file");
                    let size = f.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let icon = if ftype == "directory" { "📁" } else { "📄" };
                    let size_str = if ftype == "directory" {
                        "-"
                    } else {
                        &format_size(size)
                    };
                    println!("    {} {:>8}  {}", icon, size_str, name);
                }
            }
        }
        ServerCommands::Download {
            id,
            remote,
            output,
            json,
        } => {
            runtime.set_json(*json);
            let local_path = output
                .as_deref()
                .unwrap_or(&remote.split('/').last().unwrap_or("downloaded"));
            let resp: serde_json::Value = runtime
                .core
                .sftp_download_to_local(id, remote, local_path)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let size = std::fs::metadata(local_path).map(|m| m.len()).unwrap_or(0);
                if runtime.json_mode {
                    print_json(&serde_json::json!({
                        "remote": remote,
                        "local": local_path,
                        "bytes": size,
                    }));
                } else {
                    print_success(&format!(
                        "已下载: {} → {} ({} bytes)",
                        remote, local_path, size
                    ));
                }
            } else {
                anyhow::bail!(
                    "{}",
                    resp.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("下载失败")
                );
            }
        }
        ServerCommands::Mkdir { id, path, json } => {
            check_server_approval(runtime, id).await?;
            runtime.set_json(*json);
            let resp: serde_json::Value = runtime
                .core
                .sftp_create_dir(id, path)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                if runtime.json_mode {
                    print_json(&serde_json::json!({"ok": true, "path": path}));
                } else {
                    print_success(&format!("目录已创建: {}", path));
                }
            } else {
                anyhow::bail!(
                    "创建失败: {}",
                    resp.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误")
                );
            }
        }
        ServerCommands::JavaPs { id, json } => {
            let cmd = r#"ps aux | grep 'java.*\.jar' | grep -v grep | while read line; do
  pid=$(echo "$line" | awk '{print $2}')
  port=$(ss -tlnp 2>/dev/null | grep "pid=$pid" | grep -oP '(?<=:)\d+' | sort -u | tr '\n' ',')
  uptime=$(ps -p $pid -o etime= | xargs)
  heap=$(jstat -gc $pid 2>/dev/null | tail -1 | awk '{printf "%.0fM/%.0fM", ($3+$4+$6+$8)/1024, ($1+$2+$3+$4+$5+$6+$7+$8)/1024}')
  mem=$(echo "$line" | awk '{printf "%.1f%%", $4}')
  echo "PID:$pid | Port:${port:-N/A} | Uptime:$uptime | Heap:${heap:-N/A} | Mem:$mem"
done || echo "未找到 Java 进程""#;
            let resp: serde_json::Value = runtime
                .core
                .exec_ssh_command(id, cmd)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if *json || runtime.json_mode {
                let output = resp.get("output").and_then(|v| v.as_str()).unwrap_or("");
                print_json(&serde_json::json!({
                    "processes": output.lines().map(|l| l.to_string()).collect::<Vec<_>>()
                }));
            } else {
                if resp
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let output = resp
                        .get("output")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    if !output.is_empty() {
                        println!("\n  ☕ Java 进程 ({}):", output.lines().count());
                        println!("  {}", "─".repeat(60));
                        for line in output.lines() {
                            println!("  {}", line);
                        }
                    } else {
                        println!("  未找到 Java 进程");
                    }
                } else {
                    anyhow::bail!(
                        "查询失败: {}",
                        resp.get("error").and_then(|v| v.as_str()).unwrap_or("")
                    );
                }
            }
        }
        ServerCommands::ExecBatch {
            id,
            script,
            timeout,
            json,
        } => {
            runtime.set_json(*json);
            // 拦截高危命令
            if is_dangerous_command(script) {
                return Err(output::fail(
                    output::EXIT_DANGEROUS,
                    "检测到高危命令，CLI 已拦截。如需执行请在 GUI 中手动操作。",
                ));
            }
            // 检查服务器是否开启执行审核
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            if let Some(server) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                if server
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let name = server.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                    return Err(output::fail(
                        output::EXIT_UNAUTHORIZED,
                        format!(
                            "服务器「{}」已开启审批（生产环境），CLI 禁止操作。请在 GUI 中操作。",
                            name
                        ),
                    ));
                }
            }
            // Split script into lines (commands), filter empty lines
            let commands: Vec<String> = script
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            if commands.is_empty() {
                anyhow::bail!("脚本为空或全是注释行");
            }
            // JSON 模式：收集每步结果（不打印进度行，避免 stdout 混流）；人读模式：逐行打印
            let mut results: Vec<serde_json::Value> = Vec::new();
            let human = !runtime.json_mode;
            if human {
                println!("  📦 批量执行 ({} 条命令)...", commands.len());
            }
            for (i, cmd) in commands.iter().enumerate() {
                if human {
                    println!("  [{}] $ {}", i + 1, cmd);
                }
                let resp: serde_json::Value = runtime
                    .core
                    .exec_ssh_command(id, cmd)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                let ok = resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                let output = resp.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("").to_string();
                results.push(serde_json::json!({
                    "index": i + 1,
                    "command": cmd,
                    "success": ok,
                    "output": output,
                    "error": err,
                }));
                if ok {
                    if human && !output.trim().is_empty() {
                        for line in output.lines() {
                            println!("        {}", line);
                        }
                    }
                } else {
                    // 失败：由 main 统一输出错误 envelope（exit 1），JSON 模式下不在此打印
                    if human {
                        eprintln!("        ❌ 失败: {}", err);
                    }
                    anyhow::bail!("命令 #{} 执行失败: {}", i + 1, err);
                }
            }
            if runtime.json_mode {
                print_json(&serde_json::json!({"ok": true, "results": results}));
            } else {
                print_success(&format!("批量执行完成 ({} 条命令)", commands.len()));
            }
            let _ = timeout;
        }
        ServerCommands::Rm { id, path, json } => {
            check_server_approval(runtime, id).await?;
            runtime.set_json(*json);
            // 拦截高危路径
            let dangerous_paths = ["/", "/etc", "/usr", "/bin", "/boot", "/sys", "/proc"];
            let normalized = path.trim_end_matches('/');
            if dangerous_paths.contains(&normalized) {
                return Err(output::fail(
                    output::EXIT_DANGEROUS,
                    format!("拒绝删除高危路径: {}。如需删除请在 GUI 中手动操作。", path),
                ));
            }
            let resp: serde_json::Value = runtime
                .core
                .sftp_delete_file(id, path)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                if runtime.json_mode {
                    print_json(&serde_json::json!({"ok": true, "path": path}));
                } else {
                    print_success(&format!("已删除: {}", path));
                }
            } else {
                anyhow::bail!(
                    "删除失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        ServerCommands::JavaRestart {
            id,
            name,
            timeout,
            json,
        } => {
            runtime.set_json(*json);
            // Check server approval
            let servers: serde_json::Value = runtime
                .core
                .get_all_servers()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let servers = servers.as_array().cloned().unwrap_or_default();
            if let Some(server) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                if server
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let sname = server.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                    return Err(output::fail(
                        output::EXIT_UNAUTHORIZED,
                        format!(
                            "服务器「{}」已开启审批（生产环境），CLI 禁止操作。请在 GUI 中操作。",
                            sname
                        ),
                    ));
                }
            }

            // Find the Java process by jar name
            let find_cmd = format!(
                r#"ps aux | grep 'java.*\.jar.*{}' | grep -v grep | awk '{{print $2}}'"#,
                name
            );
            let find_resp: serde_json::Value = runtime
                .core
                .exec_ssh_command(id, &find_cmd)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let pids: Vec<String> = find_resp
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect();

            if pids.is_empty() {
                anyhow::bail!("未找到匹配 '{}' 的 Java 进程", name);
            }

            println!("  🔄 找到 {} 个 Java 进程:", pids.len());
            for pid in &pids {
                println!("    PID: {}", pid);
            }

            // Kill the processes
            for pid in &pids {
                let kill_cmd = format!("kill {}", pid);
                let kill_resp: serde_json::Value = runtime
                    .core
                    .exec_ssh_command(id, &kill_cmd)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if kill_resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                    println!("    ✅ 已发送 kill 信号给 PID {}", pid);
                } else {
                    eprintln!(
                        "    ⚠️ kill PID {} 失败: {}",
                        pid,
                        kill_resp
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("未知")
                    );
                }
            }

            // Wait for the process to stop
            println!("  ⏳ 等待进程退出...");
            let mut stopped = false;
            for _ in 0..(timeout / 5) {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let check_cmd = format!(
                    r#"ps aux | grep 'java.*\.jar.*{}' | grep -v grep | awk '{{print $2}}'"#,
                    name
                );
                let check_resp: serde_json::Value = runtime
                    .core
                    .exec_ssh_command(id, &check_cmd)
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                let remaining: Vec<&str> = check_resp
                    .get("output")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                if remaining.is_empty() {
                    stopped = true;
                    break;
                }
            }

            if !stopped {
                // Force kill
                eprintln!("  ⚠️ 进程未在 {}s 内退出，发送 SIGKILL...", timeout);
                let kill9_cmd = format!("kill -9 {}", pids.join(" "));
                let _ = runtime.core.exec_ssh_command(id, &kill9_cmd).await;
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            if runtime.json_mode {
                print_json(&serde_json::json!({
                    "ok": true,
                    "name": name,
                    "pids": pids,
                    "stoppedGracefully": stopped,
                }));
            } else {
                print_success(&format!(
                    "Java 进程 '{}' 已停止 ({} 个 PID)。请通过部署或启动脚本重新启动服务。",
                    name,
                    pids.len()
                ));
            }
        }
    }
    Ok(())
}
