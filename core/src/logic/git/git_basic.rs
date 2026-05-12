/// Git 基本操作 — status, log, branches, add, commit, checkout, merge

use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;
use super::super::git::find_git;

/// Internal helper for run_git
async fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
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
    let fmt = "%H|%an|%ae|%ai|%s|%P";
    let output = run_git(repo_path, &["log", &format!("--format={}", fmt), &format!("-n{}", n)]).await?;
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if parts.len() >= 5 {
                Some(json!({
                    "hash": parts[0],
                    "authorName": parts[1],
                    "authorEmail": parts[2],
                    "date": parts[3],
                    "message": parts[4],
                    "parentHashes": if parts.len() > 5 { parts[5] } else { "" }
                }))
            } else {
                None
            }
        })
        .collect();
    Ok(json!({"commits": commits}))
}

pub async fn git_branches(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["branch", "-a", "--format=%(refname:short)|%(upstream:short)|%(HEAD)"]).await?;
    let branches: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            json!({
                "name": parts.first().unwrap_or(&""),
                "upstream": if parts.len() > 1 && !parts[1].is_empty() { Value::String(parts[1].to_string()) } else { Value::Null },
                "isCurrent": parts.len() > 2 && parts[2] == "*"
            })
        })
        .collect();
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
    let output = run_git(repo_path, &["show", "--no-color", commit_hash]).await?;
    Ok(json!({"diff": output}))
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