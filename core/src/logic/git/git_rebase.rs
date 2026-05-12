/// Git Rebase 操作 — rebase, abort, continue

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

pub async fn git_rebase(repo_path: &str, target_branch: &str, onto: Option<&str>) -> Result<Value, String> {
    if let Some(o) = onto {
        run_git(repo_path, &["rebase", "--onto", o, target_branch]).await?;
    } else {
        run_git(repo_path, &["rebase", target_branch]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_rebase_abort(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["rebase", "--abort"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_rebase_continue(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["rebase", "--continue"]).await?;
    Ok(json!({"success": true}))
}