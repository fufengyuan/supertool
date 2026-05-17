use super::super::git::find_git;
/// Git 远程仓库操作 — remotes, push, pull, fetch
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

// ============ Remote 管理 ============

pub async fn git_remotes(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["remote", "-v"]).await?;
    let mut remotes: Vec<Value> = Vec::new();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0];
            let url = parts[1];
            let direction = parts.get(2).unwrap_or(&"");
            let existing = remotes.iter_mut().find(|r| r["name"] == name);
            match existing {
                Some(r) => {
                    if *direction == "(push)" {
                        r["pushUrl"] = json!(url);
                    } else {
                        r["fetchUrl"] = json!(url);
                    }
                }
                None => {
                    remotes.push(json!({
                        "name": name,
                        "fetchUrl": if *direction != "(push)" { url } else { "" },
                        "pushUrl": if *direction == "(push)" { url } else { "" }
                    }));
                }
            }
        }
    }
    Ok(json!({"remotes": remotes}))
}

pub async fn git_add_remote(repo_path: &str, name: &str, url: &str) -> Result<Value, String> {
    run_git(repo_path, &["remote", "add", name, url]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_remove_remote(repo_path: &str, name: &str) -> Result<Value, String> {
    run_git(repo_path, &["remote", "remove", name]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_set_remote_url(repo_path: &str, name: &str, url: &str) -> Result<Value, String> {
    run_git(repo_path, &["remote", "set-url", name, url]).await?;
    Ok(json!({"success": true}))
}

// ============ Push/Pull/Fetch ============

pub async fn git_pull(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["pull"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_push(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["push"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_fetch(repo_path: &str, remote: Option<&str>) -> Result<Value, String> {
    if let Some(r) = remote {
        run_git(repo_path, &["fetch", r]).await?;
    } else {
        run_git(repo_path, &["fetch", "--all"]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_force_push(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["push", "--force"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_push_tags(repo_path: &str, remote: &str) -> Result<Value, String> {
    run_git(repo_path, &["push", remote, "--tags"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_delete_remote_branch(
    repo_path: &str,
    remote: &str,
    branch: &str,
) -> Result<Value, String> {
    run_git(repo_path, &["push", remote, "--delete", branch]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_checkout_remote_branch(
    repo_path: &str,
    remote: &str,
    branch: &str,
) -> Result<Value, String> {
    run_git(
        repo_path,
        &["checkout", "-b", branch, &format!("{}/{}", remote, branch)],
    )
    .await?;
    Ok(json!({"success": true}))
}

// ============ 远程分支状态 ============

pub async fn git_unpushed_commits(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["log", "--format=%H|%ai|%s", "@{push}.."])
        .await
        .unwrap_or_default();
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]}))
            } else {
                None
            }
        })
        .collect();
    Ok(json!({"commits": commits, "count": commits.len()}))
}

pub async fn git_incoming_commits(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["log", "--format=%H|%ai|%s", "..@{upstream}"])
        .await
        .unwrap_or_default();
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]}))
            } else {
                None
            }
        })
        .collect();
    Ok(json!({"commits": commits, "count": commits.len()}))
}
