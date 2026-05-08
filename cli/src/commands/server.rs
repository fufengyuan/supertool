use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use crate::utils::*;
use anyhow::Result;

pub fn cmd_server(client: &ApiClient, action: &ServerCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        ServerCommands::List { json } => {
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
            if *json {
                print_json(&servers);
            } else {
                println!("\n  服务器 ({}):", servers.len());
                // 获取分组
                let groups: Vec<serde_json::Value> = client.request("servers:groups:get-all", None).unwrap_or_default();
                let group_map: std::collections::HashMap<String, String> = groups
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
                        group_map.get(gid).cloned().unwrap_or_else(|| gid.to_string())
                    };
                    if let Some((_, items)) = groups_order.iter_mut().find(|(g, _)| g == &group_name) {
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
            let _ = client.request::<serde_json::Value>(
                "servers:add",
                Some(&serde_json::json!({"name": name, "host": host, "port": port.unwrap_or(22), "username": user.as_deref().unwrap_or("root"), "type": "ssh"})),
            )?;
            print_success(&format!("服务器已添加: {} ({})", name, host));
        }
        ServerCommands::Delete { id } => {
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
            let name = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
                .and_then(|s| s.get("name").and_then(|v| v.as_str()))
                .unwrap_or(id.as_str());
            let _ = client.request::<serde_json::Value>(
                "servers:delete",
                Some(&serde_json::json!({"id": id})),
            )?;
            print_success(&format!("服务器已删除: {}", name));
        }
        ServerCommands::Test { id } => {
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
            if let Some(s) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                let config = serde_json::json!({
                    "serverId": id,
                    "host": s.get("host").and_then(|v| v.as_str()).unwrap_or(""),
                    "port": s.get("port").and_then(|v| v.as_u64()).unwrap_or(22),
                    "username": s.get("username").and_then(|v| v.as_str()).unwrap_or("root")
                });
                let resp: serde_json::Value =
                    client.request("servers:test-connection", Some(&config))?;
                if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                    print_success("连接测试成功");
                } else {
                    print_error(&format!(
                        "连接失败: {}",
                        resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                    ));
                }
            } else {
                anyhow::bail!("未找到服务器: {}", id);
            }
        }
        ServerCommands::Exec {
            id,
            command,
            timeout,
        } => {
            // 拦截高危命令（CLI 层防护，防止 AI 误操作）
            if is_dangerous_command(command) {
                anyhow::bail!("⚠️ 检测到高危命令，CLI 已拦截。如需执行请在 GUI 中手动操作。");
            }
            // 检查服务器是否开启执行审核
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
            if let Some(server) = servers
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            {
                if server
                    .get("requiresApproval")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let name = server
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(id);
                    anyhow::bail!("⚠️ 服务器「{}」已开启执行审核，CLI 不支持远程命令执行。请在 GUI 中手动操作。", name);
                }
            }
            let body = serde_json::json!({"serverId": id, "command": command, "timeout": timeout.unwrap_or(60)});
            let resp: serde_json::Value =
                client.request("servers:exec", Some(&body))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(output) = resp.get("output").and_then(|v| v.as_str()) {
                    println!("{}", output);
                }
            } else {
                anyhow::bail!(
                    "执行失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        ServerCommands::Health { id, json } => {
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
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
            // 检查服务器是否开启执行审核
            if server
                .get("requiresApproval")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                anyhow::bail!("⚠️ 服务器「{}」已开启执行审核，CLI 不支持健康检查（需执行命令）。请在 GUI 中操作。", name);
            }
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
            if *json {
                let mut results = serde_json::Map::new();
                results.insert(
                    "server".into(),
                    serde_json::json!({"name": name, "id": id}),
                );
                let mut items = Vec::new();
                for (label, cmd) in &checks {
                    let resp: serde_json::Value = client.request(
                        "servers:exec",
                        Some(&serde_json::json!({"serverId": id, "command": cmd})),
                    )?;
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
                    let resp: serde_json::Value = client.request(
                        "servers:exec",
                        Some(&serde_json::json!({"serverId": id, "command": cmd})),
                    )?;
                    let output = resp
                        .get("output")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    println!(
                        "\n  {} {}:",
                        if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
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
            let servers: Vec<serde_json::Value> = client.request("servers:get-all", None)?;
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
            // 检查服务器是否开启执行审核
            if server
                .get("requiresApproval")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                anyhow::bail!("⚠️ 服务器「{}」已开启执行审核，CLI 不支持智能诊断（需执行命令）。请在 GUI 中操作。", name);
            }
            let script = r#"echo "=== SYSTEM ===" && uname -a && echo "=== DISK ===" && df -h / && echo "=== MEMORY ===" && free -m && echo "=== LOAD ===" && uptime && echo "=== DOCKER ===" && docker ps --format 'table {{.Names}}\t{{.Status}}' 2>/dev/null || echo "No Docker" && echo "=== ERRORS (syslog) ===" && tail -n 20 /var/log/syslog 2>/dev/null | grep -i error && echo "=== ERRORS (kern) ===" && dmesg 2>/dev/null | tail -n 10 && echo "=== TOP PROCESSES ===" && ps aux --sort=-%cpu | head -6"#;
            let resp: serde_json::Value = client.request(
                "servers:exec",
                Some(&serde_json::json!({"serverId": id, "command": script})),
            )?;
            if *json {
                print_json(&serde_json::json!({
                    "server": name,
                    "success": resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                    "output": resp.get("output").and_then(|v| v.as_str()).unwrap_or("")
                }));
            } else {
                println!("\n  🩺 智能诊断: {}", name);
                println!("  {}", "─".repeat(40));
                if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
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
        ServerCommands::Read { id, path } => {
            let resp: serde_json::Value = client.request(
                "servers:sftp:read-file",
                Some(&serde_json::json!({"serverId": id, "path": path})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(content) = resp.get("content").and_then(|v| v.as_str()) {
                    println!("  📄 {}\n  {}", path, "─".repeat(40));
                    println!("{}", content);
                }
            } else {
                let resp: serde_json::Value = client.request(
                    "servers:exec",
                    Some(&serde_json::json!({"serverId": id, "command": format!("cat {}", shell_quote(path))})),
                )?;
                if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some(output) = resp.get("output").and_then(|v| v.as_str()) {
                        println!("  📄 {}\n  {}", path, "─".repeat(40));
                        println!("{}", output);
                    }
                } else {
                    anyhow::bail!(
                        "读取文件失败: {}",
                        resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                    );
                }
            }
        }
        ServerCommands::Ls { id, path, json } => {
            let resp: serde_json::Value = client.request(
                "servers:sftp:list",
                Some(&serde_json::json!({"serverId": id, "path": path})),
            )?;
            if let Some(error) = resp.get("error").and_then(|v| v.as_str()) {
                anyhow::bail!("{}", error);
            }
            let files: Vec<serde_json::Value> = resp.as_array().cloned().unwrap_or_default();
            if *json {
                print_json(&files);
            } else {
                println!("\n  📁 {} ({} 项):", path, files.len());
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
        ServerCommands::Download { id, remote, output } => {
            let resp: serde_json::Value = client.request(
                "servers:sftp:read-file",
                Some(&serde_json::json!({"serverId": id, "path": remote})),
            )?;
            if let Some(error) = resp.get("error").and_then(|v| v.as_str()) {
                anyhow::bail!("{}", error);
            }
            if let Some(success) = resp.get("success").and_then(|v| v.as_bool()) {
                if !success {
                    anyhow::bail!(
                        "{}",
                        resp.get("error").and_then(|v| v.as_str()).unwrap_or("下载失败")
                    );
                }
            }
            if let Some(content) = resp.get("content").and_then(|v| v.as_str()) {
                // Try to decode base64
                if let Ok(decoded) = base64_decode(content) {
                    let out_path = output
                        .as_deref()
                        .unwrap_or(&remote.split('/').last().unwrap_or("downloaded"));
                    std::fs::write(out_path, &decoded)?;
                    print_success(&format!(
                        "已下载: {} → {} ({} bytes)",
                        remote,
                        out_path,
                        decoded.len()
                    ));
                } else {
                    // Not base64, might be text output from exec fallback
                    let out_path = output
                        .as_deref()
                        .unwrap_or(&remote.split('/').last().unwrap_or("downloaded"));
                    std::fs::write(out_path, content)?;
                    print_success(&format!(
                        "已下载: {} → {} ({} bytes)",
                        remote,
                        out_path,
                        content.len()
                    ));
                }
            } else {
                anyhow::bail!("下载内容为空");
            }
        }
        ServerCommands::Mkdir { id, path } => {
            let resp: serde_json::Value = client.request(
                "servers:sftp:mkdir",
                Some(&serde_json::json!({"serverId": id, "path": path})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("目录已创建: {}", path));
            } else {
                anyhow::bail!(
                    "创建失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
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
            let resp: serde_json::Value = client.request(
                "servers:exec",
                Some(&serde_json::json!({"serverId": id, "command": cmd})),
            )?;
            if *json {
                let output = resp.get("output").and_then(|v| v.as_str()).unwrap_or("");
                print_json(&serde_json::json!({
                    "processes": output.lines().map(|l| l.to_string()).collect::<Vec<_>>()
                }));
            } else {
                if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
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
    }
    Ok(())
}
