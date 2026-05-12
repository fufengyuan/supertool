use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

// =================== Types ===================

#[derive(Debug, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    #[serde(rename = "message")]
    pub subject: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoScanResult {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoValidationResult {
    pub valid: bool,
    pub branch: String,
    pub remote: String,
    pub error: Option<String>,
}

// =================== Commands ===================

/// Get all branches (local + remote) for a git repo
#[tauri::command(rename_all = "camelCase")]
pub fn get_git_branches(repo_path: String) -> Result<Vec<GitBranch>, String> {
    log::info!("[Tauri CMD] get_git_branches() called");
    let output = Command::new(supertool_core::logic::git::find_git())
        .args(["branch", "-a"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to run git branch: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git branch failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<GitBranch> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let trimmed = line.trim();
            let is_current = trimmed.starts_with('*');
            let is_remote = trimmed.contains("remotes/");
            let name = if is_current {
                trimmed.trim_start_matches("* ").to_string()
            } else {
                trimmed.to_string()
            };
            GitBranch {
                name,
                is_current,
                is_remote,
            }
        })
        .collect();

    Ok(branches)
}

/// Get recent commits for a git repo
#[tauri::command(rename_all = "camelCase")]
pub fn get_git_commits(
    repo_path: String,
    since: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<GitCommit>, String> {
    log::info!("[Tauri CMD] get_git_commits() called");

    let n = limit.unwrap_or(50);
    let fmt = "%H|||%s|||%an|||%ai";

    let mut cmd = Command::new(supertool_core::logic::git::find_git());
    cmd.current_dir(&repo_path);
    cmd.arg("log");
    cmd.arg(format!("--format={}", fmt));
    cmd.arg(format!("-n{}", n));
    if let Some(since_date) = since {
        if !since_date.is_empty() {
            cmd.arg(format!("--after={}", since_date));
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git log failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<GitCommit> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split("|||").collect();
            if parts.len() >= 4 {
                Some(GitCommit {
                    hash: parts[0].to_string(),
                    subject: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

/// Scan specified directories for local git repositories
#[tauri::command(rename_all = "camelCase")]
pub fn scan_local_repos(directories: Option<Vec<String>>) -> Result<Vec<RepoScanResult>, String> {
    let dirs = directories.unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_default();
        let mut d = Vec::new();
        for sub in &["projects", "workspace", "code", "repos", "IdeaProjects", "WebstormProjects"] {
            let p = home.join(sub);
            if p.exists() && p.is_dir() {
                d.push(p.to_string_lossy().to_string());
            }
        }
        if d.is_empty() {
            d.push(home.to_string_lossy().to_string());
        }
        d
    });
    log::info!("[Tauri CMD] scan_local_repos() called with {} directories", dirs.len());

    let mut repos = Vec::new();

    for scan_path_str in &dirs {
        let scan_path = Path::new(scan_path_str);
        if !scan_path.exists() || !scan_path.is_dir() {
            log::warn!("[scan_local_repos] Path does not exist or not a directory: {}", scan_path_str);
            continue;
        }

        // Read entries in the scan directory (depth 1)
        if let Ok(entries) = std::fs::read_dir(scan_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if !entry_path.is_dir() {
                    continue;
                }

                // Check if this directory has a .git subdirectory
                if entry_path.join(".git").exists() {
                    let name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let path = entry_path.to_string_lossy().to_string();
                    repos.push(RepoScanResult { path, name });
                }
            }
        }
    }

    // Deduplicate by path
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    repos.dedup_by(|a, b| a.path == b.path);

    log::info!("[scan_local_repos] Found {} repositories", repos.len());
    Ok(repos)
}

/// Validate that a path is a valid git repository and return status info
#[tauri::command(rename_all = "camelCase")]
pub fn validate_repo_path(path: String) -> Result<RepoValidationResult, String> {
    log::info!("[Tauri CMD] validate_repo_path() called");
    let path_buf = Path::new(&path);

    // Check directory exists
    if !path_buf.exists() {
        return Ok(RepoValidationResult {
            valid: false,
            branch: String::new(),
            remote: String::new(),
            error: Some("Path does not exist".to_string()),
        });
    }

    if !path_buf.is_dir() {
        return Ok(RepoValidationResult {
            valid: false,
            branch: String::new(),
            remote: String::new(),
            error: Some("Path is not a directory".to_string()),
        });
    }

    // Check .git directory exists
    if !path_buf.join(".git").exists() {
        return Ok(RepoValidationResult {
            valid: false,
            branch: String::new(),
            remote: String::new(),
            error: Some("Not a git repository (no .git directory)".to_string()),
        });
    }

    // Get current branch
    let branch_output = Command::new(supertool_core::logic::git::find_git())
        .args(["-C", &path, "branch", "--show-current"])
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;

    let branch = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string()
    } else {
        "(detached HEAD)".to_string()
    };

    // Get default remote URL
    let remote_output = Command::new(supertool_core::logic::git::find_git())
        .args(["-C", &path, "remote", "get-url", "origin"])
        .output()
        .map_err(|e| format!("Failed to get remote: {}", e))?;

    let remote = if remote_output.status.success() {
        String::from_utf8_lossy(&remote_output.stdout)
            .trim()
            .to_string()
    } else {
        String::new()
    };

    Ok(RepoValidationResult {
        valid: true,
        branch,
        remote,
        error: None,
    })
}

/// Open a file path in the system file manager (Finder / Nautilus / Explorer)
#[tauri::command(rename_all = "camelCase")]
pub fn open_in_file_manager(path: String) -> Result<(), String> {
    log::info!("[Tauri CMD] open_in_file_manager() called, path={}", path);

    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        return Err("不支持的操作系统".to_string());
    };

    let status = std::process::Command::new(opener)
        .arg(&path)
        .status()
        .map_err(|e| format!("打开文件管理器失败: {}", e))?;

    if !status.success() {
        return Err(format!("{} 返回了非零退出码", opener));
    }

    Ok(())
}

/// Get full commit details (body + diff) for a given commit hash
#[tauri::command(rename_all = "camelCase")]
pub async fn get_git_commit_detail(
    repo_path: String,
    commit_hash: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_git_commit_detail() called, hash={}", commit_hash);
    supertool_core::logic::git::git_commit_diff(&repo_path, &commit_hash)
        .await
        .map_err(|e| format!("获取提交详情失败: {}", e))
}

// ==================== Git 状态操作 ====================

/// Get git status (modified, added, deleted, untracked files)
#[tauri::command(rename_all = "camelCase")]
pub async fn git_status(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_status() called");
    supertool_core::logic::git::git_status(&repo_path)
        .await
        .map_err(|e| format!("获取状态失败: {}", e))
}

/// Get current branch name
#[tauri::command(rename_all = "camelCase")]
pub async fn git_current_branch(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_current_branch() called");
    supertool_core::logic::git::git_current_branch(&repo_path)
        .await
        .map_err(|e| format!("获取当前分支失败: {}", e))
}

/// Get all branches with details
#[tauri::command(rename_all = "camelCase")]
pub async fn git_branches(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_branches() called");
    supertool_core::logic::git::git_branches(&repo_path)
        .await
        .map_err(|e| format!("获取分支列表失败: {}", e))
}

/// Get git log with detailed info
#[tauri::command(rename_all = "camelCase")]
pub async fn git_log(repo_path: String, limit: Option<usize>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_log() called");
    supertool_core::logic::git::git_log(&repo_path, limit)
        .await
        .map_err(|e| format!("获取日志失败: {}", e))
}

/// Get diff for a file or entire repo
#[tauri::command(rename_all = "camelCase")]
pub async fn git_diff(repo_path: String, file: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_diff() called");
    supertool_core::logic::git::git_diff(&repo_path, file.as_deref())
        .await
        .map_err(|e| format!("获取差异失败: {}", e))
}

// ==================== Git 写操作 ====================

/// Add files to staging
#[tauri::command(rename_all = "camelCase")]
pub async fn git_add(repo_path: String, files: Vec<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_add() called");
    let files_ref: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    supertool_core::logic::git::git_add(&repo_path, &files_ref)
        .await
        .map_err(|e| format!("添加文件失败: {}", e))
}

/// Reset files from staging
#[tauri::command(rename_all = "camelCase")]
pub async fn git_reset(repo_path: String, file: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_reset() called");
    supertool_core::logic::git::git_reset(&repo_path, file.as_deref())
        .await
        .map_err(|e| format!("重置文件失败: {}", e))
}

/// Commit changes
#[tauri::command(rename_all = "camelCase")]
pub async fn git_commit(repo_path: String, message: String, files: Option<Vec<String>>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_commit() called");
    // Handle files: convert to Vec<&str> if provided
    match files {
        Some(f) if !f.is_empty() => {
            let files_str: Vec<&str> = f.iter().map(|s| s.as_str()).collect();
            supertool_core::logic::git::git_commit(&repo_path, &message, Some(&files_str))
                .await
                .map_err(|e| format!("提交失败: {}", e))
        },
        _ => {
            supertool_core::logic::git::git_commit(&repo_path, &message, None)
                .await
                .map_err(|e| format!("提交失败: {}", e))
        }
    }
}

/// Checkout a branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_checkout(repo_path: String, branch: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_checkout() called, branch={}", branch);
    supertool_core::logic::git::git_checkout(&repo_path, &branch)
        .await
        .map_err(|e| format!("切换分支失败: {}", e))
}

/// Create a new branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_create_branch(repo_path: String, branch_name: String, from: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_create_branch() called, branch={}", branch_name);
    supertool_core::logic::git::git_create_branch(&repo_path, &branch_name, from.as_deref())
        .await
        .map_err(|e| format!("创建分支失败: {}", e))
}

/// Delete a branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_delete_branch(repo_path: String, branch_name: String, force: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_delete_branch() called, branch={}", branch_name);
    supertool_core::logic::git::git_delete_branch(&repo_path, &branch_name, force)
        .await
        .map_err(|e| format!("删除分支失败: {}", e))
}

/// Merge a branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_merge(repo_path: String, branch: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_merge() called, branch={}", branch);
    supertool_core::logic::git::git_merge(&repo_path, &branch)
        .await
        .map_err(|e| format!("合并失败: {}", e))
}

// ==================== Git 远程操作 ====================

/// Pull from remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_pull(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_pull() called");
    supertool_core::logic::git::git_pull(&repo_path)
        .await
        .map_err(|e| format!("拉取失败: {}", e))
}

/// Push to remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_push(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_push() called");
    supertool_core::logic::git::git_push(&repo_path)
        .await
        .map_err(|e| format!("推送失败: {}", e))
}

/// Force push to remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_force_push(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_force_push() called");
    supertool_core::logic::git::git_force_push(&repo_path)
        .await
        .map_err(|e| format!("强制推送失败: {}", e))
}

/// Fetch from remote(s)
#[tauri::command(rename_all = "camelCase")]
pub async fn git_fetch(repo_path: String, remote: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_fetch() called");
    supertool_core::logic::git::git_fetch(&repo_path, remote.as_deref())
        .await
        .map_err(|e| format!("获取远程信息失败: {}", e))
}

/// Get remotes list
#[tauri::command(rename_all = "camelCase")]
pub async fn git_remotes(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_remotes() called");
    supertool_core::logic::git::git_remotes(&repo_path)
        .await
        .map_err(|e| format!("获取远程列表失败: {}", e))
}

/// Discard changes for a file
#[tauri::command(rename_all = "camelCase")]
pub async fn git_discard_changes(repo_path: String, file: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_discard_changes() called, file={}", file);
    supertool_core::logic::git::git_discard_changes(&repo_path, &file)
        .await
        .map_err(|e| format!("丢弃更改失败: {}", e))
}

// ==================== Git Stash 操作 ====================

/// Save stash
#[tauri::command(rename_all = "camelCase")]
pub async fn git_stash_save(
    repo_path: String,
    message: Option<String>,
    include_untracked: bool,
    keep_index: bool,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_stash_save() called");
    supertool_core::logic::git::git_stash_save(&repo_path, message.as_deref(), include_untracked, keep_index)
        .await
        .map_err(|e| format!("保存stash失败: {}", e))
}

/// List stashes
#[tauri::command(rename_all = "camelCase")]
pub async fn git_stash_list(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_stash_list() called");
    supertool_core::logic::git::git_stash_list(&repo_path)
        .await
        .map_err(|e| format!("获取stash列表失败: {}", e))
}

/// Apply stash
#[tauri::command(rename_all = "camelCase")]
pub async fn git_stash_apply(repo_path: String, stash_ref: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_stash_apply() called");
    supertool_core::logic::git::git_stash_apply(&repo_path, stash_ref.as_deref())
        .await
        .map_err(|e| format!("应用stash失败: {}", e))
}

/// Pop stash
#[tauri::command(rename_all = "camelCase")]
pub async fn git_stash_pop(repo_path: String, stash_ref: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_stash_pop() called");
    supertool_core::logic::git::git_stash_pop(&repo_path, stash_ref.as_deref())
        .await
        .map_err(|e| format!("弹出stash失败: {}", e))
}

/// Drop stash
#[tauri::command(rename_all = "camelCase")]
pub async fn git_stash_drop(repo_path: String, stash_ref: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_stash_drop() called");
    supertool_core::logic::git::git_stash_drop(&repo_path, stash_ref.as_deref())
        .await
        .map_err(|e| format!("删除stash失败: {}", e))
}

// ==================== Git Tag 操作 ====================

/// List tags
#[tauri::command(rename_all = "camelCase")]
pub async fn git_list_tags(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_list_tags() called");
    supertool_core::logic::git::git_list_tags(&repo_path)
        .await
        .map_err(|e| format!("获取tag列表失败: {}", e))
}

/// Create tag
#[tauri::command(rename_all = "camelCase")]
pub async fn git_create_tag(repo_path: String, tag_name: String, message: Option<String>, force: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_create_tag() called, tag={}", tag_name);
    supertool_core::logic::git::git_create_tag(&repo_path, &tag_name, message.as_deref(), force)
        .await
        .map_err(|e| format!("创建tag失败: {}", e))
}

/// Delete tag
#[tauri::command(rename_all = "camelCase")]
pub async fn git_delete_tag(repo_path: String, tag_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_delete_tag() called, tag={}", tag_name);
    supertool_core::logic::git::git_delete_tag(&repo_path, &tag_name)
        .await
        .map_err(|e| format!("删除tag失败: {}", e))
}

// ==================== Git Rebase 操作 ====================

/// Rebase onto branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rebase(repo_path: String, target_branch: String, onto: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rebase() called");
    supertool_core::logic::git::git_rebase(&repo_path, &target_branch, onto.as_deref())
        .await
        .map_err(|e| format!("rebase失败: {}", e))
}

/// Abort rebase
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rebase_abort(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rebase_abort() called");
    supertool_core::logic::git::git_rebase_abort(&repo_path)
        .await
        .map_err(|e| format!("中止rebase失败: {}", e))
}

/// Continue rebase
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rebase_continue(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rebase_continue() called");
    supertool_core::logic::git::git_rebase_continue(&repo_path)
        .await
        .map_err(|e| format!("继续rebase失败: {}", e))
}

/// Interactive rebase - execute with custom operations
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rebase_interactive(repo_path: String, base_commit: String, operations: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rebase_interactive() called, base={}", base_commit);
    supertool_core::logic::git::git_rebase_interactive(&repo_path, &base_commit, operations)
        .await
        .map_err(|e| format!("交互式rebase失败: {}", e))
}

/// Get commits for interactive rebase preview
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rebase_todo_list(repo_path: String, base_commit: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rebase_todo_list() called, base={}", base_commit);
    supertool_core::logic::git::git_rebase_todo_list(&repo_path, &base_commit)
        .await
        .map_err(|e| format!("获取rebase todo列表失败: {}", e))
}

// ==================== Git 高级操作 ====================

/// Get file history
#[tauri::command(rename_all = "camelCase")]
pub async fn git_file_history(repo_path: String, file_path: String, limit: Option<usize>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_file_history() called, file={}", file_path);
    supertool_core::logic::git::git_file_history(&repo_path, &file_path, limit)
        .await
        .map_err(|e| format!("获取文件历史失败: {}", e))
}

/// Get unpushed commits
#[tauri::command(rename_all = "camelCase")]
pub async fn git_unpushed_commits(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_unpushed_commits() called");
    supertool_core::logic::git::git_unpushed_commits(&repo_path)
        .await
        .map_err(|e| format!("获取未推送提交失败: {}", e))
}

/// Cherry-pick a commit
#[tauri::command(rename_all = "camelCase")]
pub async fn git_cherry_pick(repo_path: String, commit_hash: String, no_commit: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_cherry_pick() called, hash={}", commit_hash);
    supertool_core::logic::git::git_cherry_pick(&repo_path, &commit_hash, no_commit)
        .await
        .map_err(|e| format!("cherry-pick失败: {}", e))
}

/// Revert a commit
#[tauri::command(rename_all = "camelCase")]
pub async fn git_revert(repo_path: String, commit_hash: String, no_commit: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_revert() called, hash={}", commit_hash);
    supertool_core::logic::git::git_revert(&repo_path, &commit_hash, no_commit)
        .await
        .map_err(|e| format!("revert失败: {}", e))
}

/// Amend last commit
#[tauri::command(rename_all = "camelCase")]
pub async fn git_amend_commit(repo_path: String, message: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_amend_commit() called");
    supertool_core::logic::git::git_amend_commit(&repo_path, &message)
        .await
        .map_err(|e| format!("修改提交失败: {}", e))
}

/// Reset to a commit
#[tauri::command(rename_all = "camelCase")]
pub async fn git_reset_to_commit(repo_path: String, commit_hash: String, mode: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_reset_to_commit() called, hash={}", commit_hash);
    supertool_core::logic::git::git_reset_to_commit(&repo_path, &commit_hash, &mode)
        .await
        .map_err(|e| format!("重置到提交失败: {}", e))
}

/// Get file blame
#[tauri::command(rename_all = "camelCase")]
pub async fn git_file_blame(repo_path: String, file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_file_blame() called, file={}", file_path);
    supertool_core::logic::git::git_file_blame(&repo_path, &file_path)
        .await
        .map_err(|e| format!("获取blame失败: {}", e))
}

/// Get submodule list
#[tauri::command(rename_all = "camelCase")]
pub async fn git_submodule_list(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_submodule_list() called");
    supertool_core::logic::git::git_submodule_list(&repo_path)
        .await
        .map_err(|e| format!("获取子模块列表失败: {}", e))
}

/// Init submodule
#[tauri::command(rename_all = "camelCase")]
pub async fn git_submodule_init(repo_path: String, recursive: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_submodule_init() called");
    supertool_core::logic::git::git_submodule_init(&repo_path, recursive)
        .await
        .map_err(|e| format!("初始化子模块失败: {}", e))
}

// ==================== Git 远程仓库管理 ====================

/// Add a remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_add_remote(repo_path: String, name: String, url: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_add_remote() called, name={}", name);
    supertool_core::logic::git::git_add_remote(&repo_path, &name, &url)
        .await
        .map_err(|e| format!("添加远程仓库失败: {}", e))
}

/// Remove a remote
#[tauri::command(rename_all = "camelCase")]
pub async fn git_delete_remote(repo_path: String, name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_delete_remote() called, name={}", name);
    supertool_core::logic::git::git_remove_remote(&repo_path, &name)
        .await
        .map_err(|e| format!("删除远程仓库失败: {}", e))
}

/// Rename a branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_rename_branch(repo_path: String, old_name: String, new_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_rename_branch() called, {} -> {}", old_name, new_name);
    supertool_core::logic::git::git_rename_branch(&repo_path, &old_name, &new_name)
        .await
        .map_err(|e| format!("重命名分支失败: {}", e))
}

/// Compare branches
#[tauri::command(rename_all = "camelCase")]
pub async fn git_diff_branches(repo_path: String, target: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_diff_branches() called, target={}", target);
    supertool_core::logic::git::git_compare_branches(&repo_path, &target, None)
        .await
        .map_err(|e| format!("对比分支失败: {}", e))
}

/// Push tags
#[tauri::command(rename_all = "camelCase")]
pub async fn git_push_tags(repo_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_push_tags() called");
    supertool_core::logic::git::git_push_tags(&repo_path, "origin")
        .await
        .map_err(|e| format!("推送标签失败: {}", e))
}

/// Clean untracked files
#[tauri::command(rename_all = "camelCase")]
pub async fn git_clean(repo_path: String, dry_run: bool, force: bool, include_ignored: bool, directories: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_clean() called, dry_run={}, include_ignored={}, directories={}", dry_run, include_ignored, directories);
    supertool_core::logic::git::git_clean(&repo_path, dry_run, force, include_ignored, directories)
        .await
        .map_err(|e| format!("清理失败: {}", e))
}

/// Delete remote branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_delete_remote_branch(repo_path: String, branch: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_delete_remote_branch() called, branch={}", branch);
    supertool_core::logic::git::git_delete_remote_branch(&repo_path, "origin", &branch)
        .await
        .map_err(|e| format!("删除远程分支失败: {}", e))
}

/// Checkout remote branch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_checkout_remote_branch(repo_path: String, branch: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_checkout_remote_branch() called, branch={}", branch);
    supertool_core::logic::git::git_checkout_remote_branch(&repo_path, "origin", &branch)
        .await
        .map_err(|e| format!("检出远程分支失败: {}", e))
}

/// Get file at revision
#[tauri::command(rename_all = "camelCase")]
pub async fn git_get_file_at_revision(repo_path: String, commit: String, path: String) -> Result<String, String> {
    log::info!("[Tauri CMD] git_get_file_at_revision() called, commit={}, path={}", commit, path);
    supertool_core::logic::git::git_file_at_revision(&repo_path, &path, &commit)
        .await
        .map(|v| v.as_str().unwrap_or("").to_string())
        .map_err(|e| format!("获取文件版本失败: {}", e))
}

/// Update a single submodule
#[tauri::command(rename_all = "camelCase")]
pub async fn git_submodule_update(repo_path: String, submodule_path: String, recursive: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_submodule_update() called, path={}", submodule_path);
    supertool_core::logic::git::git_submodule_update(&repo_path, &submodule_path, recursive)
        .await
        .map_err(|e| format!("更新子模块失败: {}", e))
}

/// Update all submodules
#[tauri::command(rename_all = "camelCase")]
pub async fn git_submodule_update_all(repo_path: String, recursive: bool) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_submodule_update_all() called");
    supertool_core::logic::git::git_submodule_update_all(&repo_path, recursive)
        .await
        .map_err(|e| format!("更新子模块失败: {}", e))
}

/// Compare two commits
#[tauri::command(rename_all = "camelCase")]
pub async fn git_compare_commits(repo_path: String, commit1: String, commit2: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_compare_commits() called, {} vs {}", commit1, commit2);
    supertool_core::logic::git::git_compare_commits(&repo_path, &commit1, &commit2)
        .await
        .map_err(|e| format!("对比提交失败: {}", e))
}

/// Create a patch file
#[tauri::command(rename_all = "camelCase")]
pub async fn git_create_patch(repo_path: String, commit1: String, commit2: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_create_patch() called, {}..{}", commit1, commit2);
    supertool_core::logic::git::git_create_patch(&repo_path, &commit1, &commit2)
        .await
        .map_err(|e| format!("创建补丁失败: {}", e))
}

/// Apply a patch
#[tauri::command(rename_all = "camelCase")]
pub async fn git_apply_patch(repo_path: String, patch_content: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] git_apply_patch() called");
    supertool_core::logic::git::git_apply_patch(&repo_path, &patch_content)
        .await
        .map_err(|e| format!("应用补丁失败: {}", e))
}

/// Execute a raw git command
#[tauri::command(rename_all = "camelCase")]
pub async fn git_raw_command(repo_path: String, args: Vec<String>) -> Result<String, String> {
    log::info!("[Tauri CMD] git_raw_command() called, args={:?}", args);
    supertool_core::logic::git::git_raw_command(&repo_path, &args)
        .await
        .map_err(|e| format!("执行命令失败: {}", e))
}

// =================== File Browser Commands ===================

/// File entry for file tree
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileTreeEntry>>,
}

/// Get file tree for a directory (recursive)
#[tauri::command(rename_all = "camelCase")]
pub fn get_file_tree(repo_path: String, subdir: Option<String>) -> Result<Vec<FileTreeEntry>, String> {
    log::info!("[Tauri CMD] get_file_tree() called, repo_path={}", repo_path);
    let base_path = if let Some(sub) = subdir {
        Path::new(&repo_path).join(sub)
    } else {
        Path::new(&repo_path).to_path_buf()
    };
    
    scan_directory_recursive(&base_path, &repo_path, 3)  // Max depth 3 for performance
}

fn scan_directory_recursive(dir: &Path, base_path: &str, depth: u32) -> Result<Vec<FileTreeEntry>, String> {
    if depth == 0 {
        return Ok(Vec::new());
    }
    
    if !dir.exists() || !dir.is_dir() {
        return Err(format!("目录不存在或不是目录: {}", dir.display()));
    }
    
    let mut entries: Vec<FileTreeEntry> = Vec::new();
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| format!("无法读取目录: {}", e))?;
    
    for entry in read_dir {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        
        // Skip hidden files and .git directory
        if name.starts_with('.') {
            continue;
        }
        
        let is_dir = path.is_dir();
        let relative_path = path.strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        
        let children = if is_dir && depth > 1 {
            Some(scan_directory_recursive(&path, base_path, depth - 1)?)
        } else {
            None
        };
        
        entries.push(FileTreeEntry {
            path: relative_path,
            name,
            is_dir,
            children,
        });
    }
    
    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });
    
    Ok(entries)
}

/// Read file content
#[tauri::command(rename_all = "camelCase")]
pub fn read_file_content(repo_path: String, file_path: String) -> Result<String, String> {
    log::info!("[Tauri CMD] read_file_content() called, file={}", file_path);
    let full_path = Path::new(&repo_path).join(&file_path);
    
    if !full_path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }
    
    if full_path.is_dir() {
        return Err(format!("是目录，不是文件: {}", file_path));
    }
    
    std::fs::read_to_string(&full_path)
        .map_err(|e| format!("读取文件失败: {}", e))
}

/// Save file content
#[tauri::command(rename_all = "camelCase")]
pub fn save_file_content(repo_path: String, file_path: String, content: String) -> Result<(), String> {
    log::info!("[Tauri CMD] save_file_content() called, file={}", file_path);
    let full_path = Path::new(&repo_path).join(&file_path);
    
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }
    
    std::fs::write(&full_path, &content)
        .map_err(|e| format!("保存文件失败: {}", e))
}
