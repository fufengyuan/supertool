/// Git Operations — 通过 git CLI 执行所有 Git 操作
///
/// 与 Electron 的 git-service.ts 对应。
/// 使用 tokio::process::Command 调用 git CLI（不使用 git2 crate，避免 libgit2 依赖）。

use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;

/// Find git binary — public for use by other modules
pub fn find_git() -> String {
    let candidates = [
        "git",
        "/usr/bin/git",
        "/usr/local/bin/git",
        "/opt/homebrew/bin/git",
        "/snap/bin/git",
        "/usr/lib/git-core/git",
        "/usr/local/git/bin/git",
    ];
    for path in &candidates {
        let output = std::process::Command::new(path).arg("--version").output();
        if let Ok(o) = output {
            if o.status.success() {
                return path.to_string();
            }
        }
    }
    "git".to_string()
}

/// Internal use for run_git (returns &'static str to avoid allocations)
fn _find_git_static() -> &'static str {
    let g = find_git();
    Box::leak(g.into_boxed_str())
}

async fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let git_bin = _find_git_static();
    let output = Command::new(git_bin)
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

/// 执行 git 命令并返回 JSON 格式结果
#[allow(dead_code)]
async fn run_git_json(repo_path: &str, args: &[&str]) -> Result<Value, String> {
    let output = run_git(repo_path, args).await?;
    Ok(json!({"output": output}))
}

// ============ Git 基础操作 ============

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

pub async fn git_discard_changes(repo_path: &str, file: &str) -> Result<Value, String> {
    run_git(repo_path, &["checkout", "--", file]).await?;
    Ok(json!({"success": true}))
}

// ============ Git Stash ============

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
    if let Some(s) = stash_ref {
        run_git(repo_path, &["stash", "show", "-p", s]).await?;
    } else {
        run_git(repo_path, &["stash", "show", "-p"]).await?;
    }
    Ok(json!({"success": true, "diff": ""})) // run_git returns stdout
}

// ============ Git Tag ============

pub async fn git_list_tags(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["tag", "-l", "--sort=-v:refname"]).await?;
    let tags: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|t| json!({"name": t}))
        .collect();
    Ok(json!({"tags": tags}))
}

pub async fn git_create_tag(repo_path: &str, tag_name: &str, message: Option<&str>, force: bool) -> Result<Value, String> {
    let mut args = vec!["tag"];
    if force { args.push("-f"); }
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

// ============ Git Remote ============

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
                    if *direction == "(push)" { r["pushUrl"] = json!(url); }
                    else { r["fetchUrl"] = json!(url); }
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

// ============ Git Rebase ============

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

// ============ Git Advanced ============

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

pub async fn git_clean(repo_path: &str, dry_run: bool, force: bool) -> Result<Value, String> {
    let mut args = vec!["clean"];
    if dry_run { args.push("-n"); }
    if force { args.push("-f"); }
    run_git(repo_path, &args).await?;
    Ok(json!({"success": true}))
}

pub async fn git_file_at_revision(repo_path: &str, file_path: &str, revision: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["show", &format!("{}:{}", revision, file_path)]).await?;
    Ok(json!({"content": output}))
}

pub async fn git_unpushed_commits(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["log", "--format=%H|%ai|%s", "@{push}.."]).await.unwrap_or_default();
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 { Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]})) } else { None }
        })
        .collect();
    Ok(json!({"commits": commits, "count": commits.len()}))
}

pub async fn git_incoming_commits(repo_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["log", "--format=%H|%ai|%s", "..@{upstream}"]).await.unwrap_or_default();
    let commits: Vec<Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 { Some(json!({"hash": parts[0], "date": parts[1], "message": parts[2]})) } else { None }
        })
        .collect();
    Ok(json!({"commits": commits, "count": commits.len()}))
}

pub async fn git_force_push(repo_path: &str) -> Result<Value, String> {
    run_git(repo_path, &["push", "--force"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_push_tags(repo_path: &str, remote: &str) -> Result<Value, String> {
    run_git(repo_path, &["push", remote, "--tags"]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_delete_remote_branch(repo_path: &str, remote: &str, branch: &str) -> Result<Value, String> {
    run_git(repo_path, &["push", remote, "--delete", branch]).await?;
    Ok(json!({"success": true}))
}

pub async fn git_checkout_remote_branch(repo_path: &str, remote: &str, branch: &str) -> Result<Value, String> {
    run_git(repo_path, &["checkout", "-b", branch, &format!("{}/{}", remote, branch)]).await?;
    Ok(json!({"success": true}))
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

pub async fn git_file_blame(repo_path: &str, file_path: &str) -> Result<Value, String> {
    let output = run_git(repo_path, &["blame", "--line-porcelain", file_path]).await?;
    Ok(json!({"blame": output}))
}

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

pub async fn git_exec(repo_path: &str, args: &[&str]) -> Result<Value, String> {
    let output = run_git(repo_path, args).await?;
    Ok(json!({"output": output}))
}

pub async fn git_submodules(repo_path: &str) -> Result<Value, String> {
    git_submodule_list(repo_path).await
}
