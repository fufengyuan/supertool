use super::super::git::find_git;
/// Git Tag 操作 — list, create, delete
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::process::Command;

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

pub async fn git_list_tags(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["tag", "-l", "--sort=-v:refname"]).await?;
    let tags: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|t| json!({"name": t}))
        .collect();
    Ok(json!({"tags": tags}))
}

pub async fn git_create_tag(
    repo_path: &str,
    tag_name: &str,
    message: Option<&str>,
    force: bool,
) -> Result<Value, String> {
    let mut args = vec!["tag"];
    if force {
        args.push("-f");
    }
    if let Some(m) = message {
        args.push("-a");
        args.push(tag_name);
        args.push("-m");
        args.push(m);
    } else {
        args.push(tag_name);
    }
    run_git(repo_path, &args).await?;
    Ok(json!({"success": true}))
}

pub async fn git_delete_tag(repo_path: &str, tag_name: &str) -> Result<Value, String> {
    run_git(repo_path, &["tag", "-d", tag_name]).await?;
    Ok(json!({"success": true}))
}
