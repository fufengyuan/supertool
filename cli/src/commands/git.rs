use crate::types::*;
use crate::runtime::CliRuntime;
use crate::output::*;
use anyhow::Result;

pub async fn cmd_git(runtime: &mut CliRuntime, action: &GitCommands) -> Result<()> {
    match action {
        GitCommands::List { json } => {
            let repos: Vec<serde_json::Value> = runtime.core.db_read(|conn| {
                let mut stmt = conn
                    .prepare("SELECT * FROM git_repos ORDER BY createdAt DESC")
                    .expect("prepare git_repos");
                let rows: Vec<serde_json::Value> = stmt
                    .query_map([], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>("id")?,
                            "path": row.get::<_, String>("path")?,
                            "remote": row.get::<_, Option<String>>("remote")?,
                            "branch": row.get::<_, Option<String>>("branch")?,
                            "lastCommit": row.get::<_, Option<String>>("lastCommit")?,
                            "createdAt": row.get::<_, String>("createdAt")?,
                            "updatedAt": row.get::<_, String>("updatedAt")?,
                        }))
                    })
                    .expect("query git_repos")
                    .filter_map(|r| r.ok())
                    .collect();
                rows
            }).map_err(|e| anyhow::anyhow!("查询仓库列表失败: {}", e))?;

            if *json {
                print_json(&repos);
            } else {
                println!("\n  Git 仓库 ({}):", repos.len());
                for r in &repos {
                    let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let name = r.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    println!("  {}  {}", id, name);
                }
            }
        }
        GitCommands::Status { path, json } => {
            let resp = runtime.core.git_status(path).await
                .map_err(|e| anyhow::anyhow!("状态查询失败: {}", e))?;
            if *json {
                print_json(&resp);
            } else {
                let branch = resp.get("currentBranch").and_then(|v| v.as_str()).unwrap_or("?");
                let clean = resp.get("isClean").and_then(|v| v.as_bool()).unwrap_or(false);
                let ahead = resp.get("ahead").and_then(|v| v.as_u64()).unwrap_or(0);
                let behind = resp.get("behind").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("\n  Git 状态: {} ({}):", path, branch);
                println!("  {}", "─".repeat(40));
                println!("    干净: {}", if clean { "✅" } else { "❌ 有未提交更改" });
                if ahead > 0 || behind > 0 {
                    println!("    领先: {} | 落后: {}", ahead, behind);
                }
                if let Some(files) = resp.get("files").and_then(|v| v.as_array()) {
                    if !files.is_empty() {
                        println!("    变更文件:");
                        for f in files {
                            println!("      {} {}",
                                f.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                                f.get("status").and_then(|v| v.as_str()).unwrap_or(""));
                        }
                    }
                }
            }
        }
        GitCommands::Log { path, limit, json } => {
            let resp = runtime.core.git_log(path, Some(*limit)).await
                .map_err(|e| anyhow::anyhow!("日志查询失败: {}", e))?;
            if *json {
                print_json(&resp);
            } else {
                let logs: Vec<serde_json::Value> = resp.as_array().cloned().unwrap_or_default();
                println!("\n  Git 提交历史 ({}):", path);
                println!("  {}", "─".repeat(60));
                for l in &logs {
                    let hash = l.get("hash").and_then(|v| v.as_str()).unwrap_or("").get(..7).unwrap_or("");
                    let msg = l.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    let author = l.get("author").and_then(|v| v.as_str()).unwrap_or("");
                    let date = l.get("date").and_then(|v| v.as_str()).unwrap_or("").get(..16).unwrap_or("");
                    println!("    {} {} | {} | {}", hash, msg, author, date);
                }
            }
        }
        GitCommands::Branches { path, json } => {
            let resp = runtime.core.git_branches(path).await
                .map_err(|e| anyhow::anyhow!("分支查询失败: {}", e))?;
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
            let resp = runtime.core.git_pull(path).await
                .map_err(|e| anyhow::anyhow!("Pull 失败: {}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Pull 成功: {}", path));
            } else {
                anyhow::bail!("Pull 失败: {}", resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误"));
            }
        }
        GitCommands::Push { path } => {
            let resp = runtime.core.git_push(path).await
                .map_err(|e| anyhow::anyhow!("Push 失败: {}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("Push 成功: {}", path));
            } else {
                anyhow::bail!("Push 失败: {}", resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误"));
            }
        }
        GitCommands::Commit { path, message, files } => {
            let file_refs: Vec<String> = files.as_ref().cloned().unwrap_or_default();
            let resp = if file_refs.is_empty() {
                runtime.core.git_commit(path, message, None).await
            } else {
                let refs: Vec<&str> = file_refs.iter().map(|s| s.as_str()).collect();
                runtime.core.git_commit(path, message, Some(&refs)).await
            }.map_err(|e| anyhow::anyhow!("{}", e))?;
            let resp_val: serde_json::Value = resp;
            if resp_val.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                let hash = resp_val.get("hash").and_then(|v| v.as_str()).unwrap_or("");
                print_success(&format!("提交成功: {} ({})", message, hash));
            } else {
                anyhow::bail!("提交失败: {}", resp_val.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误"));
            }
        }
        GitCommands::Checkout { path, branch } => {
            let resp = runtime.core.git_checkout(path, branch).await
                .map_err(|e| anyhow::anyhow!("切换失败: {}", e))?;
            if resp.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success(&format!("已切换到分支: {}", branch));
            } else {
                anyhow::bail!("切换失败: {}", resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误"));
            }
        }
    }
    Ok(())
}
