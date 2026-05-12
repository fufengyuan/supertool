/// Git 高级操作 — file_history, cherry_pick, submodule, patch, compare_commits 等

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

// ============ 文件历史与对比 ============

pub async fn git_file_history(repo_path: &str, file_path: &str, limit: Option<usize>) -> Result<Value, String> {
    let n = limit.unwrap_or(50);
    let output = run_git(repo_path, &["log", "--follow", &format!("-n{}", n), "--format=%H|%ai|%s", "--", file_path]).await?;
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]}))
            } else { None }
        })
        .collect();
    Ok(json!({"commits": commits}))
}

pub async fn git_compare_branches(repo_path: &str, target: &str, source: Option<&str>) -> Result<Value, String> {
    let src = source.unwrap_or("HEAD");
    let output = run_git(repo_path, &["log", "--format=%H|%ai|%s", &format!("{}..{}", src, target)]).await?;
    let ahead: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]}))
            } else { None }
        })
        .collect();
    Ok(json!({"ahead": ahead, "aheadCount": ahead.len()}))
}

/// 对比两个提交的差异
pub async fn git_compare_commits(repo_path: &str, commit1: &str, commit2: &str) -> Result<Value, String> {
    let diff_output = run_git(repo_path, &["diff", "--no-color", commit1, commit2]).await?;
    let stats_output = run_git(repo_path, &["diff", "--stat", commit1, commit2]).await?;
    Ok(json!({
        "diff": diff_output,
        "stats": stats_output,
        "commit1": commit1,
        "commit2": commit2
    }))
}

pub async fn git_file_at_revision(repo_path: &str, file_path: &str, revision: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["show", &format!("{}:{}", revision, file_path)]).await?;
    Ok(json!({"content": output}))
}

pub async fn git_file_blame(repo_path: &str, file_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["blame", "--line-porcelain", file_path]).await?;
    Ok(json!({"blame": output}))
}

pub async fn git_changed_files(repo_path: &str, commit1: &str, commit2: Option<&str>) -> Result<Value, String> {
    let range = if let Some(c2) = commit2 {
        format!("{}..{}", commit1, c2)
    } else {
        commit1.to_string()
    };
    let output = run_git(repo_path, &["diff", "--name-only", &range]).await?;
    let files: Vec<Value> = output.lines().filter(|l| !l.is_empty()).map(|f| json!({"path": f})).collect();
    Ok(json!({"files": files, "count": files.len()}))
}

// ============ 冲突处理 ============

pub async fn git_conflict_files(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["diff", "--name-only", "--diff-filter=U"]).await?;
    let files: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|f| json!({"path": f}))
        .collect();
    Ok(json!({"files": files}))
}

pub async fn git_accept_conflict(repo_path: &str, file: &str, strategy: &str) -> Result<Value, String> {
    let arg = if strategy == "ours" { "--ours" } else { "--theirs" };
    run_git(repo_path, &["checkout", arg, "--", file]).await?;
    run_git(repo_path, &["add", file]).await?;
    Ok(json!({"success": true}))
}

// ============ 重置与撤销 ============

pub async fn git_clean(repo_path: &str, dry_run: bool, force: bool) -> Result<Value, String> {
    let mut args = vec!["clean"];
    if dry_run { args.push("-n"); }
    if force { args.push("-f"); }
    let output = run_git(repo_path, &args).await?;
    Ok(json!({"success": true, "output": output}))
}

pub async fn git_rename_branch(repo_path: &str, old_name: &str, new_name: &str) -> Result<Value, String> {
    run_git(repo_path, &["branch", "-m", old_name, new_name]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_undo_last_commit(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["reset", "--soft", "HEAD~1"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_reset_to_commit(repo_path: &str, commit_hash: &str, mode: &str) -> Result<Value, String> {
    let flag = match mode {
        "soft" => "--soft",
        "mixed" => "--mixed",
        "hard" => "--hard",
        _ => "--mixed",
    };
    run_git(repo_path, &["reset", flag, commit_hash]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_amend_commit(repo_path: &str, message: &str) -> Result<Value, String> {
    if !message.is_empty() {
        run_git(repo_path, &["commit", "--amend", "-m", message]).await?;
    } else {
        run_git(repo_path, &["commit", "--amend", "--no-edit"]).await?;
    }
    Ok(json!({"success": true}))
}

// ============ Cherry-pick & Revert ============

pub async fn git_cherry_pick(repo_path: &str, commit_hash: &str, no_commit: bool) -> Result<Value, String> {
    if no_commit {
        run_git(repo_path, &["cherry-pick", "--no-commit", commit_hash]).await?;
    } else {
        run_git(repo_path, &["cherry-pick", commit_hash]).await?;
    }
    Ok(json!({"success": true}))
}

pub async fn git_revert(repo_path: &str, commit_hash: &str, no_commit: bool) -> Result<Value, String> {
    if no_commit {
        run_git(repo_path, &["revert", "--no-commit", commit_hash]).await?;
    } else {
        run_git(repo_path, &["revert", commit_hash]).await?;
    }
    Ok(json!({"success": true}))
}

// ============ 子模块 ============

pub async fn git_submodule_list(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["submodule", "status"]).await.unwrap_or_default();
    let submodules: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(json!({"hash": parts[0].trim_start_matches('-').trim_start_matches('+'), "path": parts[1]}))
            } else { None }
        })
        .collect();
    Ok(json!({"submodules": submodules}))
}

pub async fn git_submodule_init(repo_path: &str, recursive: bool) -> Result<Value, String> {
    if recursive {
        run_git(repo_path, &["submodule", "update", "--init", "--recursive"]).await?;
    } else {
        run_git(repo_path, &["submodule", "update", "--init"]).await?;
    }
    Ok(json!({"success": true}))
}

/// 更新单个子模块
pub async fn git_submodule_update(repo_path: &str, submodule_path: &str, recursive: bool) -> Result<Value, String> {
    let mut args = vec!["submodule", "update", submodule_path];
    if recursive {
        args.push("--recursive");
    }
    run_git(repo_path, &args).await?;
    Ok(json!({"success": true}))
}

/// 批量更新所有子模块
pub async fn git_submodule_update_all(repo_path: &str, recursive: bool) -> Result<Value, String> {
    let mut args = vec!["submodule", "update", "--init"];
    if recursive {
        args.push("--recursive");
    }
    run_git(repo_path, &args).await?;
    Ok(json!({"success": true}))
}

pub async fn git_submodules(repo_path: &str) -> Result<Value, String> {
    git_submodule_list(repo_path).await
}

// ============ 补丁操作 ============

/// 创建补丁文件（两个提交之间的差异）
pub async fn git_create_patch(repo_path: &str, commit1: &str, commit2: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["format-patch", "--stdout", &format!("{}..{}", commit1, commit2)]).await?;
    Ok(json!({"patch": output, "success": true}))
}

/// 应用补丁文件
pub async fn git_apply_patch(repo_path: &str, patch_content: &str) -> Result<Value, String> {
    // 需要通过 stdin 传入补丁内容
    let git_bin = find_git();
    let mut child = Command::new(&git_bin)
        .args(["apply"])
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行 git apply 失败: {}", e))?;

    // 写入补丁内容到 stdin
    use tokio::io::AsyncWriteExt;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(patch_content.as_bytes()).await
            .map_err(|e| format!("写入补丁内容失败: {}", e))?;
    }

    let output = child.wait_with_output().await
        .map_err(|e| format!("等待 git apply 完成: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(stderr.trim().to_string());
    }

    Ok(json!({"success": true, "output": stdout}))
}

// ============ 通用执行 ============

pub async fn git_exec(repo_path: &str, args: &[&str]) -> Result<Value, String> {
    let output = run_git(repo_path, args).await?;
    Ok(json!({"success": true, "output": output}))
}

/// 执行任意 git 命令并返回原始输出
pub async fn git_raw_command(repo_path: &str, args: &[String]) -> Result<String, String> {
    let git_bin = find_git();
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = Command::new(&git_bin)
        .args(&args_str)
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