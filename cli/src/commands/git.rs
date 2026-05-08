use crate::types::*;
use crate::transport::ApiClient;
use crate::output::*;
use anyhow::Result;

pub fn cmd_git(client: &ApiClient, action: &GitCommands) -> Result<()> {
    crate::commands::todo::check_connection(client)?;
    match action {
        GitCommands::List { json } => {
            let repos: Vec<serde_json::Value> = client.request("git:repos:get-all", None)?;
            if *json {
                print_json(&repos);
            } else {
                println!("\n  Git 仓库 ({}):", repos.len());
                for r in &repos {
                    let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    println!("  {}  {}", id, name);
                }
            }
        }
        GitCommands::Status { path, json } => {
            let resp: serde_json::Value =
                client.request("git:status", Some(&serde_json::json!({"path": path})))?;
            if *json {
                print_json(&resp);
            } else {
                let branch = resp
                    .get("currentBranch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let clean = resp.get("isClean").and_then(|v| v.as_bool()).unwrap_or(false);
                let ahead = resp.get("ahead").and_then(|v| v.as_u64()).unwrap_or(0);
                let behind = resp.get("behind").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("\n  Git 状态: {} ({}):", path, branch);
                println!("  {}", "─".repeat(40));
                println!(
                    "    干净: {}",
                    if clean {
                        "✅"
                    } else {
                        "❌ 有未提交更改"
                    }
                );
                if ahead > 0 || behind > 0 {
                    println!("    领先: {} | 落后: {}", ahead, behind);
                }
                if let Some(files) = resp.get("files").and_then(|v| v.as_array()) {
                    if !files.is_empty() {
                        println!("    变更文件:");
                        for f in files {
                            println!(
                                "      {} {}",
                                f.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                                f.get("status").and_then(|v| v.as_str()).unwrap_or("")
                            );
                        }
                    }
                }
            }
        }
        GitCommands::Log { path, limit, json } => {
            let resp: serde_json::Value = client.request(
                "git:log",
                Some(&serde_json::json!({"path": path, "limit": limit})),
            )?;
            if *json {
                print_json(&resp);
            } else {
                let logs: Vec<serde_json::Value> = resp.as_array().cloned().unwrap_or_default();
                println!("\n  Git 提交历史 ({}):", path);
                println!("  {}", "─".repeat(60));
                for l in &logs {
                    let hash = l
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .get(..7)
                        .unwrap_or("");
                    let msg = l.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    let author = l.get("author").and_then(|v| v.as_str()).unwrap_or("");
                    let date = l
                        .get("date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .get(..16)
                        .unwrap_or("");
                    println!("    {} {} | {} | {}", hash, msg, author, date);
                }
            }
        }
        GitCommands::Branches { path, json } => {
            let resp: serde_json::Value =
                client.request("git:branches", Some(&serde_json::json!({"path": path})))?;
            if *json {
                print_json(&resp);
            } else {
                let branches: Vec<serde_json::Value> = resp.as_array().cloned().unwrap_or_default();
                println!("\n  Git 分支 ({}):", path);
                for b in &branches {
                    let name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let current = b.get("current").and_then(|v| v.as_bool()).unwrap_or(false);
                    let marker = if current { "* " } else { "  " };
                    println!("    {}{}", marker, name);
                }
            }
        }
        GitCommands::Pull { path } => {
            let resp: serde_json::Value =
                client.request("git:pull", Some(&serde_json::json!({"path": path})))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Pull 成功: {}", path));
            } else {
                anyhow::bail!(
                    "Pull 失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        GitCommands::Push { path } => {
            let resp: serde_json::Value =
                client.request("git:push", Some(&serde_json::json!({"path": path})))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Push 成功: {}", path));
            } else {
                anyhow::bail!(
                    "Push 失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        GitCommands::Commit {
            path,
            message,
            files,
        } => {
            let body = if files.is_empty() {
                serde_json::json!({"path": path, "message": message})
            } else {
                serde_json::json!({"path": path, "message": message, "files": files})
            };
            let resp: serde_json::Value = client.request("git:commit", Some(&body))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let hash = resp.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                print_success(&format!("提交成功: {} ({})", message, hash));
            } else {
                anyhow::bail!(
                    "提交失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
        GitCommands::Checkout { path, branch } => {
            let resp: serde_json::Value = client.request(
                "git:checkout",
                Some(&serde_json::json!({"path": path, "branch": branch})),
            )?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("已切换到分支: {}", branch));
            } else {
                anyhow::bail!(
                    "切换失败: {}",
                    resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
                );
            }
        }
    }
    Ok(())
}
