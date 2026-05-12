/// Git Operations — 模块拆分架构
///
/// 将 git.rs 拆分成多个子模块，避免单文件过大：
/// - git_basic: 基本操作（status, log, branches, add, commit, checkout）
/// - git_remote: 远程仓库操作（remotes, push, pull, fetch）
/// - git_stash: Stash 操作
/// - git_tag: Tag 操作
/// - git_rebase: Rebase 操作
/// - git_advanced: 高级操作（file_history, cherry_pick, submodule, patch 等）

mod git_basic;
mod git_remote;
mod git_stash;
mod git_tag;
mod git_rebase;
mod git_advanced;

// 重新导出所有公共函数
pub use git_basic::*;
pub use git_remote::*;
pub use git_stash::*;
pub use git_tag::*;
pub use git_rebase::*;
pub use git_advanced::*;

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