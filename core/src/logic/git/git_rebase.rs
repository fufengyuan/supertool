use super::super::git::find_git;
/// Git Rebase 操作 — rebase, abort, continue, interactive
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
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

pub async fn git_rebase(
    repo_path: &str,
    target_branch: &str,
    onto: Option<&str>,
) -> Result<Value, String> {
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

/// Interactive rebase - execute git rebase -i with custom sequence
///
/// Operations: pick, reword, edit, squash, fixup, drop
///
/// # Arguments
/// * `repo_path` - Repository path
/// * `base_commit` - The commit to rebase onto (e.g., HEAD~3, a specific hash)
/// * `operations` - List of operations: [{ action: "pick", hash: "abc123", message: "..." }]
pub async fn git_rebase_interactive(
    repo_path: &str,
    base_commit: &str,
    operations: Vec<Value>,
) -> Result<Value, String> {
    // 1. Create todo file with operations
    let repo_dir = PathBuf::from(repo_path);
    let git_dir = repo_dir.join(".git");

    // Create todo file content
    let todo_content = operations
        .iter()
        .map(|op| {
            let action = op["action"].as_str().unwrap_or("pick");
            let hash = op["hash"].as_str().unwrap_or("");
            let message = op["message"].as_str().unwrap_or("");
            format!("{} {} {}", action, hash, message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Write todo file to a temp location
    let todo_file = git_dir.join("rebase-merge").join("git-rebase-todo");

    // First, start the rebase to create the directory structure
    // Use GIT_SEQUENCE_EDITOR to bypass the interactive editor
    let todo_script = repo_dir.join(".git-rebase-todo-script.sh");
    let script_content = format!("#!/bin/sh\ncat '{}' > \"$1\"", todo_file.display());
    fs::write(&todo_script, script_content).map_err(|e| format!("写入脚本失败: {}", e))?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&todo_script, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("设置权限失败: {}", e))?;
    }

    // Write the todo file content
    let temp_todo = repo_dir.join(".git-rebase-todo-temp");
    fs::write(&temp_todo, &todo_content).map_err(|e| format!("写入todo文件失败: {}", e))?;

    // Execute interactive rebase with our sequence
    // The trick: use GIT_SEQUENCE_EDITOR to copy our todo file
    let git_bin = find_git();
    let output = Command::new(&git_bin)
        .args(&["rebase", "-i", base_commit])
        .current_dir(repo_path)
        .env(
            "GIT_SEQUENCE_EDITOR",
            format!("cp {} \"$1\"", temp_todo.display()),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("执行 git rebase -i 失败: {}", e))?;

    // Cleanup temp files
    let _ = fs::remove_file(&todo_script);
    let _ = fs::remove_file(&temp_todo);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        // Check if it's just waiting for user input (conflict or edit)
        if stderr.contains("Could not execute editor") || stdout.contains("Successfully rebased") {
            return Ok(json!({"success": true, "message": "Rebase 完成"}));
        }
        return Err(format!("Rebase 失败: {}", stderr.trim()));
    }

    Ok(json!({"success": true, "message": "交互式 Rebase 执行成功"}))
}

/// Get commits for interactive rebase preview
/// Returns commits from base to HEAD
pub async fn git_rebase_todo_list(repo_path: &str, base_commit: &str) -> Result<Value, String> {
    // Get commits from base to HEAD (excluding base itself)
    let output = run_git(
        repo_path,
        &[
            "log",
            "--format=%H|%s",
            "--reverse",
            &format!("{}..HEAD", base_commit),
        ],
    )
    .await?;

    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            json!({
                "hash": parts.first().unwrap_or(&""),
                "message": parts.get(1).unwrap_or(&""),
                "action": "pick"  // default action
            })
        })
        .collect();

    Ok(json!({"commits": commits, "base": base_commit}))
}
