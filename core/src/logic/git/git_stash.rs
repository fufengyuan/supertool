/// Git Stash 操作 — save, list, apply, pop, drop, show

use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;
use super::super::git::find_git;

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

pub async fn git_stash_save(repo_path: &str, message: Option<&str>, include_untracked: bool, keep_index: bool) -> Result<Value, String> {
    let mut args = vec!["stash", "push"];
    if let Some(m) = message {
        args.push("-m");
        args.push(m);
    }
    if include_untracked {
        args.push("-u");
    }
    if keep_index {
        args.push("--keep-index");
    }
    run_git(repo_path, &args).await?;
    Ok(json!({"success": true}))
}

pub async fn git_stash_list(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["stash", "list", "--format=%gd|%H|%ai|%s"]).await?;
    let stashes: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                Some(json!({
                    "ref": parts[0],
                    "hash": parts[1],
                    "date": parts[2],
                    "message": parts[3]
                }))
            } else {
                None
            }
        })
        .collect();
    Ok(json!({"stashes": stashes}))
}

pub async fn git_stash_apply(repo_path: &str, stash_ref: Option<&str>) -> Result<Value, String> {
    if let Some(s) = stash_ref {
        run_git(repo_path, &["stash", "apply", s]).await?;
    } else {
        run_git(repo_path, &["stash", "apply"]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_stash_pop(repo_path: &str, stash_ref: Option<&str>) -> Result<Value, String> {
    if let Some(s) = stash_ref {
        run_git(repo_path, &["stash", "pop", s]).await?;
    } else {
        run_git(repo_path, &["stash", "pop"]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_stash_drop(repo_path: &str, stash_ref: Option<&str>) -> Result<Value, String> {
    if let Some(s) = stash_ref {
        run_git(repo_path, &["stash", "drop", s]).await?;
    } else {
        run_git(repo_path, &["stash", "drop"]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_stash_show(repo_path: &str, stash_ref: Option<&str>) -> Result<Value, String> {
    let output = if let Some(s) = stash_ref {
        run_git(repo_path, &["stash", "show", "-p", s]).await?
    } else {
        run_git(repo_path, &["stash", "show", "-p"]).await?
    };
    Ok(json!({"success": true, "diff": output}))
}