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
    let output = Command::new(crate::core::git::find_git())
        .args(["-C", &repo_path, "branch", "-a"])
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
    limit: Option<usize>,
) -> Result<Vec<GitCommit>, String> {
    log::info!("[Tauri CMD] get_git_commits() called");
    let n = limit.unwrap_or(50).to_string();

    // Use a custom format that we can parse reliably
    let format = "%H|||%s|||%an|||%ai";
    let output = Command::new(crate::core::git::find_git())
        .args([
            "-C",
            &repo_path,
            "log",
            "--format",
            &format,
            "-n",
            &n,
        ])
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
pub fn scan_local_repos(directories: Vec<String>) -> Result<Vec<RepoScanResult>, String> {
    log::info!("[Tauri CMD] scan_local_repos() called with {} directories", directories.len());

    let mut repos = Vec::new();

    for scan_path_str in &directories {
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
    let branch_output = Command::new(crate::core::git::find_git())
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
    let remote_output = Command::new(crate::core::git::find_git())
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
