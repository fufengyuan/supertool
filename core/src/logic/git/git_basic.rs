/// Git 基本操作 — status, log, branches, add, commit, checkout, merge

use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use super::super::git::find_git;

/// Internal helper for run_git
async fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    // 验证路径存在
    if repo_path.is_empty() {
        return Err("仓库路径为空".to_string());
    }
    let path = Path::new(repo_path);
    if !path.exists() {
        return Err(format!("仓库路径不存在: {}", repo_path));
    }
    if !path.join(".git").exists() {
        return Err(format!("不是 Git 仓库: {}", repo_path));
    }

    let git_bin = find_git();
    let output = Command::new(&git_bin)
        .args(args)
        .current_dir(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(stderr.trim().to_string());
    }

    Ok(stdout)
}

// ============ Git 基础读取操作 ============

pub async fn git_status(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["status", "--porcelain"]).await?;
    let files: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let status = line.get(0..2).unwrap_or("").trim().to_string();
            let path = line.get(3..).unwrap_or("").trim().to_string();
            let is_staged = status.starts_with("M") || status.starts_with("A") || status.starts_with("D");
            json!({
                "path": path,
                "status": status,
                "isStaged": is_staged,
                "type": if status.contains("D") { "deleted" } else if status.contains("A") { "added" } else if status.contains("M") { "modified" } else { "untracked" }
            })
        })
        .collect();
    Ok(json!({"files": files}))
}

pub async fn git_log(repo_path: &str, limit: Option<usize>) -> Result<Value, String> {
    let n = limit.unwrap_or(50);
    // %H=hash, %an=author name, %ae=author email, %aI=date ISO 8601 strict (JS parseable), %s=subject, %P=parent hashes, %d=decorate (refs)
    let fmt = "%H|%an|%ae|%aI|%s|%P|%d";
    let output = run_git(repo_path, &["log", &format!("--format={}", fmt), &format!("-n{}", n), "--shortstat"]).await?;
    
    let mut commits: Vec<Value> = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        if line.is_empty() {
            i += 1;
            continue;
        }
        
        let parts: Vec<&str> = line.splitn(7, '|').collect();
        if parts.len() >= 5 {
            let hash = parts[0];
            let author_name = parts[1];
            let author_email = parts[2];
            let date = parts[3];
            let message = parts[4];
            let parent_hashes = if parts.len() > 5 { parts[5] } else { "" };
            let refs_raw = if parts.len() > 6 { parts[6].trim() } else { "" };
            
            // Clean refs: remove "HEAD ->" prefix, parse branch/tag names
            let refs = parse_refs(refs_raw);
            
            // Parse shortstat from next line (e.g., " 2 files changed, 10 insertions(+), 5 deletions(-)")
            let mut file_count: Option<usize> = None;
            if i + 1 < lines.len() {
                let stat_line = lines[i + 1].trim();
                if stat_line.contains("files changed") || stat_line.contains("file changed") {
                    // Extract file count
                    if let Some(fc_str) = stat_line.split_whitespace().next() {
                        file_count = fc_str.parse().ok();
                    }
                    i += 1; // Skip stat line
                }
            }
            
            commits.push(json!({
                "hash": hash,
                "authorName": author_name,
                "authorEmail": author_email,
                "date": date,
                "message": message,
                "parentHashes": parent_hashes,
                "refs": refs,
                "fileCount": file_count
            }));
        }
        i += 1;
    }
    
    Ok(json!({"commits": commits}))
}

/// Parse refs from git log --decorate output
fn parse_refs(refs_raw: &str) -> Vec<String> {
    if refs_raw.is_empty() {
        return Vec::new();
    }
    // refs_raw format: " (HEAD -> main, origin/main, tag: v1.0)"
    let refs_str = refs_raw.trim();
    if !refs_str.starts_with('(') || !refs_str.ends_with(')') {
        return Vec::new();
    }
    let inner = &refs_str[1..refs_str.len()-1];
    inner.split(',')
        .map(|r| r.trim())
        .filter(|r| !r.is_empty())
        .map(|r| {
            // Convert "tag: v1.0" to "v1.0 (tag)"
            if r.starts_with("tag: ") {
                format!("{} (tag)", r[5..].trim())
            } else {
                r.to_string()
            }
        })
        .collect()
}

pub async fn git_branches(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["branch", "-a", "--format=%(refname:short)|%(upstream:short)|%(HEAD)"]).await?;
    let mut branches: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let upstream = if parts.len() > 1 && !parts[1].is_empty() { 
                Some(parts[1].to_string()) 
            } else { 
                None 
            };
            let is_current = parts.len() > 2 && parts[2] == "*";
            
            json!({
                "name": name,
                "upstream": upstream,
                "isCurrent": is_current,
                "ahead": 0,
                "behind": 0
            })
        })
        .collect();
    
    // Calculate ahead/behind for branches with upstream
    for branch in branches.iter_mut() {
        if let Some(upstream) = branch.get("upstream").and_then(|u| u.as_str()) {
            let branch_name = branch.get("name").and_then(|n| n.as_str()).unwrap_or("");
            // Skip remote branches (they start with "remotes/")
            if branch_name.starts_with("remotes/") {
                continue;
            }
            let count_output = run_git(repo_path, &["rev-list", "--left-right", "--count", &format!("{}...{}", branch_name, upstream)]).await;
            if let Ok(count_str) = count_output {
                let counts: Vec<&str> = count_str.trim().split_whitespace().collect();
                if counts.len() >= 2 {
                    let ahead = counts.first().and_then(|c| c.parse::<u32>().ok()).unwrap_or(0);
                    let behind = counts.get(1).and_then(|c| c.parse::<u32>().ok()).unwrap_or(0);
                    branch["ahead"] = Value::Number(ahead.into());
                    branch["behind"] = Value::Number(behind.into());
                }
            }
        }
    }
    
    Ok(json!({"branches": branches}))
}

pub async fn git_current_branch(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["branch", "--show-current"]).await?;
    Ok(json!({"branch": output.trim()}))
}

pub async fn git_diff(repo_path: &str, file: Option<&str>) -> Result<Value, String> {
    let mut args = vec!["diff", "--no-color"];
    if let Some(f) = file {
        args.push("--");
        args.push(f);
    }
    let output = run_git(repo_path, &args).await?;
    Ok(json!({"diff": output}))
}

pub async fn git_commit_diff(repo_path: &str, commit_hash: &str) -> Result<Value, String> {
    // Get commit info with --stat to show file changes summary
    let stat_output = run_git(repo_path, &["show", "--stat", "--no-color", "--format=%H|%an|%ae|%ai|%s", commit_hash]).await?;
    
    // Parse commit info from first line
    let lines: Vec<&str> = stat_output.lines().collect();
    let info_line = lines.first().map_or("", |v| *v);
    let info_parts: Vec<&str> = info_line.split('|').collect();
    
    let hash = info_parts.first().unwrap_or(&"").to_string();
    let author = info_parts.get(1).unwrap_or(&"").to_string();
    let author_email = info_parts.get(2).unwrap_or(&"").to_string();
    let date = info_parts.get(3).unwrap_or(&"").to_string();
    let message = info_parts.get(4).unwrap_or(&"").to_string();
    
    // Parse file changes from stat output
    let files: Vec<Value> = lines.iter()
        .skip(1)  // Skip commit info line
        .take_while(|line| !line.starts_with(" ") && !line.is_empty() && !line.contains("files changed"))
        .filter_map(|line| {
            // Format: " file_path | X insertions(+), Y deletions(-)"
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let file_path = parts.first().unwrap_or(&"").trim();
                let changes = parts.get(1).unwrap_or(&"").trim();
                Some(json!({
                    "path": file_path,
                    "changes": changes,
                    "status": if changes.contains("insertion") || changes.contains("deletion") { "modified" } 
                              else if line.contains("new file") { "added" }
                              else if line.contains("deleted") { "deleted" }
                              else { "modified" }
                }))
            } else {
                None
            }
        })
        .collect();
    
    // Get full diff for each file
    let full_diff = run_git(repo_path, &["show", "--no-color", commit_hash]).await?;
    
    Ok(json!({
        "hash": hash,
        "author": author,
        "authorEmail": author_email,
        "date": date,
        "message": message,
        "files": files,
        "diff": full_diff
    }))
}

// ============ Git 写操作 ============

pub async fn git_commit(repo_path: &str, message: &str, files: Option<&[&str]>) -> Result<Value, String> {
    if let Some(files) = files {
        if !files.is_empty() {
            let mut args = vec!["add"];
            args.extend_from_slice(files);
            run_git(repo_path, &args).await?;
        }
    }
    run_git(repo_path, &["commit", "-m", message]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_add(repo_path: &str, files: &[&str]) -> Result<Value, String> {
    if files.is_empty() {
        run_git(repo_path, &["add", "."]).await?;
    } else {
        let mut args = vec!["add"];
        args.extend_from_slice(files);
        run_git(repo_path, &args).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_reset(repo_path: &str, file: Option<&str>) -> Result<Value, String> {
    if let Some(f) = file {
        run_git(repo_path, &["reset", "HEAD", "--", f]).await?;
    } else {
        run_git(repo_path, &["reset"]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_checkout(repo_path: &str, branch: &str) -> Result<Value, String> {
    run_git(repo_path, &["checkout", branch]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_create_branch(repo_path: &str, branch_name: &str, from: Option<&str>) -> Result<Value, String> {
    if let Some(f) = from {
        run_git(repo_path, &["checkout", "-b", branch_name, f]).await?;
    } else {
        run_git(repo_path, &["checkout", "-b", branch_name]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_delete_branch(repo_path: &str, branch_name: &str, force: bool) -> Result<Value, String> {
    let flag = if force { "-D" } else { "-d" };
    run_git(repo_path, &["branch", flag, branch_name]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_merge(repo_path: &str, branch: &str) -> Result<Value, String> {
    run_git(repo_path, &["merge", branch]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_discard_changes(repo_path: &str, file: &str) -> Result<Value, String> {
    run_git(repo_path, &["checkout", "--", file]).await?;
    Ok(json!({"success": true}))
}