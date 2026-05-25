/// CI/CD 部署执行引擎
///
/// 完整部署流水线：Git同步 → 构建 → 收集产物 → SFTP上传 → 远程重启
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// 获取用户登录 shell 的完整环境变量，确保版本管理器工具（NVM/Homebrew/nvm-windows）可用
/// 全局缓存：整个应用生命周期只 fork 一次 zsh，后续调用直接返回缓存
static SHELL_ENV_CACHE: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    // 第一步：尝试从登录 shell 获取环境变量
    #[cfg(target_os = "windows")]
    let shell_output = std::process::Command::new("cmd")
        .args(["/c", "set"])
        .output()
        .ok();
    #[cfg(not(target_os = "windows"))]
    let shell_output = std::process::Command::new("zsh")
        .args(["-l", "-c", "env"])
        .output()
        .ok();

    let mut env = HashMap::new();
    if let Some(out) = shell_output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            #[cfg(target_os = "windows")]
            let separator = '=';
            #[cfg(not(target_os = "windows"))]
            let separator = '=';
            if let Some(pos) = line.find(separator) {
                let key = &line[..pos];
                let value = &line[pos + 1..];
                env.insert(key.to_string(), value.to_string());
            }
        }
    }

    // 第二步：确保 PATH 包含常见版本管理器路径（跨平台 fallback）
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let current_path = env
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();
        let mut extra_paths: Vec<String> = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // NVM (Node Version Manager)
            let nvm_dir = format!("{}/.nvm/versions/node", home);
            if std::path::Path::new(&nvm_dir).is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    let mut versions: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect();
                    versions.sort();
                    if let Some(latest) = versions.last() {
                        extra_paths.push(format!("{}/{}/bin", nvm_dir, latest));
                    }
                }
            }
            // Homebrew Apple Silicon
            extra_paths.push("/opt/homebrew/bin".to_string());
            extra_paths.push("/opt/homebrew/sbin".to_string());
        }

        #[cfg(target_os = "linux")]
        {
            // NVM Linux
            let nvm_dir = format!("{}/.nvm/versions/node", home);
            if std::path::Path::new(&nvm_dir).is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    let mut versions: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect();
                    versions.sort();
                    if let Some(latest) = versions.last() {
                        extra_paths.push(format!("{}/{}/bin", nvm_dir, latest));
                    }
                }
            }
            // Linux 常见路径
            extra_paths.push("/usr/local/bin".to_string());
            extra_paths.push("/snap/bin".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            // NVM Windows
            let nvm_dir = format!("{}\\nvm4w\\nodejs", home);
            if std::path::Path::new(&nvm_dir).is_dir() {
                extra_paths.push(nvm_dir);
            }
            // 也扫描 NVM 安装目录下的所有 node 版本
            let nvm_root = format!("{}\\AppData\\Roaming\\nvm", home);
            if std::path::Path::new(&nvm_root).is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_root) {
                    let mut versions: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .filter(|n| n.starts_with("v"))
                        .collect();
                    versions.sort();
                    if let Some(latest) = versions.last() {
                        extra_paths.push(format!("{}\\{}", nvm_root, latest));
                    }
                }
            }
        }

        // 合并原有 PATH（去重）
        let sep = if cfg!(windows) { ";" } else { ":" };
        let mut all_paths = extra_paths;
        for p in current_path.split(sep) {
            if !p.is_empty() && !all_paths.iter().any(|x| x == p) {
                all_paths.push(p.to_string());
            }
        }
        env.insert("PATH".to_string(), all_paths.join(sep));
    }
    env
});

/// 获取用户登录 shell 环境变量（从缓存读取，不重复 fork）
fn get_user_shell_env() -> &'static HashMap<String, String> {
    &SHELL_ENV_CACHE
}

/// 获取 shell 环境变量的副本（供外部模块使用，如 tauri commands 的 run_command）
pub fn get_shell_env_for_command() -> HashMap<String, String> {
    SHELL_ENV_CACHE.clone()
}

/// 创建继承用户 shell 环境变量的本地 Command（替代 Command::new）
/// 自动加载 NVM、Homebrew、nvm、rvm 等所有 shell 初始化的环境变量
pub fn user_shell_cmd(program: &str) -> Command {
    let mut cmd = Command::new(program);
    let shell_env = get_user_shell_env();
    // 注入用户 shell 的完整环境变量
    for (key, value) in shell_env {
        cmd.env(key, value);
    }
    cmd
}

/// 同步版本（用于 collect_artifacts 等非异步场景）
#[allow(dead_code)]
fn user_shell_cmd_sync(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    let shell_env = get_user_shell_env();
    for (key, value) in shell_env {
        cmd.env(key, value);
    }
    cmd
}

// =================== Types ===================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployServerConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    #[serde(rename = "privateKey")]
    pub private_key: Option<String>,
    #[serde(rename = "deployDir")]
    pub deploy_dir: String,
    #[serde(rename = "libDir")]
    pub lib_dir: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployModuleConfig {
    pub name: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "buildPath")]
    pub build_path: Option<String>,
    #[serde(rename = "buildCommand")]
    pub build_command: Option<String>,
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "outputPath")]
    pub output_path: Option<String>,
    #[serde(rename = "artifactName")]
    pub artifact_name: Option<String>,
    #[serde(rename = "artifactType")]
    pub artifact_type: Option<String>,
    #[serde(rename = "libFilterRules")]
    pub lib_filter_rules: Option<String>,
    #[serde(rename = "deployOrder")]
    pub deploy_order: i64,
    #[serde(rename = "deployPath")]
    pub deploy_path: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployConfig {
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    pub branch: String,
    #[serde(rename = "localPath")]
    pub local_path: Option<String>,
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "buildCommand")]
    pub build_command: Option<String>,
    #[serde(rename = "buildPath")]
    pub build_path: Option<String>,
    #[serde(rename = "npmScript")]
    pub npm_script: Option<String>,
    #[serde(rename = "npmCustomScript")]
    pub npm_custom_script: Option<String>,
    #[serde(rename = "mavenHome")]
    pub maven_home: Option<String>,
    #[serde(rename = "javaHome")]
    pub java_home: Option<String>,
    #[serde(rename = "npmHome")]
    pub npm_home: Option<String>,
    #[serde(rename = "nodeHome")]
    pub node_home: Option<String>,
    #[serde(rename = "mavenProfile")]
    pub maven_profile: Option<String>,
    #[serde(rename = "mavenSettings")]
    pub maven_settings: Option<String>,
    pub modules: Vec<DeployModuleConfig>,
    #[serde(rename = "skipTests")]
    pub skip_tests: bool,
    #[serde(rename = "parentBuildMode")]
    pub parent_build_mode: bool,
    #[serde(rename = "parentBuildPath")]
    pub parent_build_path: Option<String>,
    pub servers: Vec<DeployServerConfig>,
    #[serde(rename = "deployDir")]
    pub deploy_dir: String,
    #[serde(rename = "libDir")]
    pub lib_dir: Option<String>,
    #[serde(rename = "restartScript")]
    pub restart_script: Option<String>,
    #[serde(rename = "libSeparate")]
    pub lib_separate: bool,
    #[serde(rename = "buildMode")]
    pub build_mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub name: String,
    #[serde(rename = "localPath")]
    pub local_path: String,
    pub module: Option<String>,
    #[serde(rename = "isLib")]
    pub is_lib: bool,
    #[serde(rename = "isCompressed")]
    pub is_compressed: bool,
    #[serde(rename = "deployPath")]
    pub deploy_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub stage: String,
    pub status: String,
    pub message: String,
    pub progress: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployResult {
    #[serde(rename = "deployId")]
    pub deploy_id: String,
    pub success: bool,
    #[serde(rename = "logFilePath")]
    pub log_file_path: String,
    #[serde(rename = "artifactPaths")]
    pub artifact_paths: Vec<String>,
    pub error: Option<String>,
    pub cancelled: Option<bool>,
}

// =================== Deploy Engine ===================

pub async fn execute_deploy(
    config: &DeployConfig,
    data_dir: &str,
    deploy_id: &str,
    on_progress: impl Fn(ProgressEvent) + Send + Sync,
    is_cancelled: impl Fn() -> bool,
) -> Result<DeployResult, String> {
    let log_dir = PathBuf::from(data_dir).join("deploy-logs");
    fs::create_dir_all(&log_dir).map_err(|e| format!("创建日志目录失败: {}", e))?;

    let log_file = log_dir.join(format!("{}.log", deploy_id));
    let artifact_dir = PathBuf::from(data_dir)
        .join("deploy-artifacts")
        .join(deploy_id);

    let emit = |stage: &str, status: &str, msg: &str| {
        let event = ProgressEvent {
            stage: stage.to_string(),
            status: status.to_string(),
            message: msg.to_string(),
            progress: None,
        };
        // Log to file
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let _ = writeln!(
                f,
                "[{}] [{}] [{}] {}",
                chrono::Utc::now().to_rfc3339(),
                stage,
                status,
                msg
            );
        }
        on_progress(event);
    };

    emit(
        "deploy",
        "starting",
        &format!("开始部署 {}", config.repo_url),
    );

    // Step 1: Git sync or use local path
    let project_path = match do_git_sync(config, &emit).await {
        Ok(p) => p,
        Err(e) => {
            emit("deploy", "failed", &e);
            return Ok(DeployResult {
                deploy_id: deploy_id.to_string(),
                success: false,
                log_file_path: log_file.to_string_lossy().to_string(),
                artifact_paths: vec![],
                error: Some(e),
                cancelled: None,
            });
        }
    };

    emit("git", "success", "代码同步完成");

    // 检查是否已取消
    if is_cancelled() {
        emit("deploy", "cancelled", "部署已被用户取消");
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths: vec![],
            error: Some("用户取消部署".to_string()),
            cancelled: Some(true),
        });
    }

    // Step 1.5: Install dependencies (git_clone mode only)
    if config.build_mode == "git_clone" {
        if let Err(e) = install_dependencies(config, &project_path, &emit).await {
            emit("deps", "failed", &e);
            return Ok(DeployResult {
                deploy_id: deploy_id.to_string(),
                success: false,
                log_file_path: log_file.to_string_lossy().to_string(),
                artifact_paths: vec![],
                error: Some(e),
                cancelled: None,
            });
        }
    }

    // Step 2: Build
    if let Err(e) = do_build(config, &project_path, &emit).await {
        emit("build", "failed", &e);
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths: vec![],
            error: Some(e),
            cancelled: None,
        });
    }

    emit("build", "success", "构建完成");

    // 检查是否已取消
    if is_cancelled() {
        emit("deploy", "cancelled", "部署已被用户取消");
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths: vec![],
            error: Some("用户取消部署".to_string()),
            cancelled: Some(true),
        });
    }

    // Step 3: Collect artifacts
    let artifacts = match collect_artifacts(&project_path, config) {
        Ok(a) => a,
        Err(e) => {
            emit("collect", "failed", &e);
            return Ok(DeployResult {
                deploy_id: deploy_id.to_string(),
                success: false,
                log_file_path: log_file.to_string_lossy().to_string(),
                artifact_paths: vec![],
                error: Some(e),
                cancelled: None,
            });
        }
    };

    if artifacts.is_empty() {
        let err = "未收集到任何构建产物，请检查模块配置和产物路径".to_string();
        emit("collect", "failed", &err);
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths: vec![],
            error: Some(err),
            cancelled: None,
        });
    }

    emit(
        "collect",
        "success",
        &format!("产物收集完成 ({} 个)", artifacts.len()),
    );

    // Copy artifacts to deploy-artifacts directory
    fs::create_dir_all(&artifact_dir).map_err(|e| format!("创建产物目录失败: {}", e))?;
    let mut artifact_paths = vec![];
    for artifact in &artifacts {
        let dest = artifact_dir.join(&artifact.name);
        if let Err(e) = fs::copy(&artifact.local_path, &dest) {
            emit(
                "collect",
                "warning",
                &format!("复制 {} 失败: {}", artifact.name, e),
            );
            continue;
        }
        artifact_paths.push(dest.to_string_lossy().to_string());
    }

    // Step 4: Deploy to servers (parallel)
    if config.servers.is_empty() {
        let err = "未配置部署服务器".to_string();
        emit("ssh", "failed", &err);
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths,
            error: Some(err),
            cancelled: None,
        });
    }

    // 检查是否已取消（构建完成后、上传前）
    if is_cancelled() {
        emit("deploy", "cancelled", "部署已被用户取消");
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths,
            error: Some("用户取消部署".to_string()),
            cancelled: Some(true),
        });
    }

    let mut deploy_results = Vec::new();
    for srv in &config.servers {
        match deploy_to_server(srv, &artifacts, config, &emit).await {
            Ok(_) => {
                let label = srv.label.as_deref().unwrap_or("服务器");
                emit(
                    "ssh",
                    "success",
                    &format!("{} ({}) 部署完成", label, srv.host),
                );
                deploy_results.push(true);
            }
            Err(e) => {
                let label = srv.label.as_deref().unwrap_or("服务器");
                emit(
                    "ssh",
                    "failed",
                    &format!("{} ({}) 部署失败: {}", label, srv.host, e),
                );
                deploy_results.push(false);
            }
        }
    }

    let failed_count = deploy_results.iter().filter(|&&r| !r).count();
    if failed_count > 0 {
        let err = format!("{} 台服务器部署失败", failed_count);
        emit("deploy", "failed", &err);
        return Ok(DeployResult {
            deploy_id: deploy_id.to_string(),
            success: false,
            log_file_path: log_file.to_string_lossy().to_string(),
            artifact_paths,
            error: Some(err),
            cancelled: None,
        });
    }

    // Step 5: Restart (if configured)
    // 前端项目（npm/pnpm/yarn）直接替换静态文件，不需要重启脚本
    let build_tool = config.build_tool.as_deref().unwrap_or("maven");
    let is_frontend = ["npm", "pnpm", "yarn"].contains(&build_tool);

    if let Some(ref script) = config.restart_script.as_ref().filter(|s| !s.is_empty()) {
        if !is_frontend {
            for srv in &config.servers {
                if let Err(e) = execute_restart(srv, script, &config.deploy_dir, &emit).await {
                    emit("restart", "failed", &e);
                    // Non-fatal: restart might fail but deploy succeeded
                }
            }
        } else {
            emit(
                "restart",
                "skipped",
                "前端项目无需重启脚本，静态文件已直接替换",
            );
        }
    }

    emit("deploy", "complete", "部署成功完成");

    Ok(DeployResult {
        deploy_id: deploy_id.to_string(),
        success: true,
        log_file_path: log_file.to_string_lossy().to_string(),
        artifact_paths,
        error: None,
        cancelled: None,
    })
}

// =================== Git Sync ===================

async fn do_git_sync(
    config: &DeployConfig,
    emit: &impl Fn(&str, &str, &str),
) -> Result<PathBuf, String> {
    // Determine working directory based on build mode
    if config.build_mode == "local" {
        // Local mode: use local project directory
        if let Some(ref local_path) = config.local_path {
            let path = PathBuf::from(local_path);
            if !path.exists() {
                return Err(format!("本地路径不存在: {}", local_path));
            }

            // Check if it's a git repo
            let git_dir = path.join(".git");
            if !git_dir.exists() {
                emit(
                    "git",
                    "warning",
                    &format!("使用本地目录: {} (非 Git 仓库，跳过分支切换)", local_path),
                );
                return Ok(path);
            }

            // Fetch and pull
            emit("git", "pulling", "拉取最新代码...");

            let output = user_shell_cmd(&crate::logic::git::find_git())
                .args(["fetch", "origin"])
                .current_dir(&path)
                .output()
                .await
                .map_err(|e| format!("git fetch 失败: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("git fetch 失败: {}", err.trim()));
            }

            // Checkout branch（对齐 Electron 原版逻辑）
            let raw_branch = if config.branch.is_empty() {
                "main"
            } else {
                &config.branch
            };
            // 剥离 origin/ 前缀，避免 git pull origin origin/xxx 双重前缀
            let branch = raw_branch.strip_prefix("origin/").unwrap_or(raw_branch);
            let output = user_shell_cmd(&crate::logic::git::find_git())
                .args(["checkout", branch])
                .current_dir(&path)
                .output()
                .await
                .map_err(|e| format!("git checkout 失败: {}", e))?;

            if !output.status.success() {
                // 分支不存在，从 origin 创建
                let output2 = user_shell_cmd(&crate::logic::git::find_git())
                    .args(["checkout", "-b", branch, &format!("origin/{}", branch)])
                    .current_dir(&path)
                    .output()
                    .await
                    .map_err(|e| format!("git checkout -b 失败: {}", e))?;

                if !output2.status.success() {
                    let err = String::from_utf8_lossy(&output2.stderr);
                    return Err(format!("git checkout 失败: {}", err.trim()));
                }
            }

            // Pull latest - 本地模式智能合并（不强制要求提交）
            // 先检查是否有未提交改动
            let status_output = user_shell_cmd(&crate::logic::git::find_git())
                .args(["status", "--porcelain"])
                .current_dir(&path)
                .output()
                .await
                .map_err(|e| format!("git status 失败: {}", e))?;

            let has_changes = !status_output.stdout.is_empty();

            if has_changes {
                // 有未提交改动，先 stash
                emit("git", "info", "检测到未提交改动，暂存后拉取...");

                let stash_output = user_shell_cmd(&crate::logic::git::find_git())
                    .args(["stash", "push", "-m", "supertool-auto-stash"])
                    .current_dir(&path)
                    .output()
                    .await
                    .map_err(|e| format!("git stash 失败: {}", e))?;

                if !stash_output.status.success() {
                    let err = String::from_utf8_lossy(&stash_output.stderr);
                    // stash 失败可能是因为没有实际改动（如空文件），继续尝试 pull
                    emit("git", "warning", &format!("stash 跳过: {}", err.trim()));
                }
            }

            // 执行 pull（使用 --rebase 确保线性合并）
            let pull_output = user_shell_cmd(&crate::logic::git::find_git())
                .args(["pull", "--rebase", "origin", branch])
                .current_dir(&path)
                .output()
                .await
                .map_err(|e| format!("git pull 失败: {}", e))?;

            if !pull_output.status.success() {
                let err = String::from_utf8_lossy(&pull_output.stderr);
                // pull 失败，可能是冲突，尝试恢复 stash 后继续
                emit("git", "warning", &format!("pull 有警告: {}", err.trim()));
            }

            // 如果之前有改动，恢复 stash
            if has_changes {
                let pop_output = user_shell_cmd(&crate::logic::git::find_git())
                    .args(["stash", "pop"])
                    .current_dir(&path)
                    .output()
                    .await;

                match pop_output {
                    Ok(o) if o.status.success() => {
                        emit("git", "info", "已恢复本地改动");
                    }
                    Ok(o) => {
                        let err = String::from_utf8_lossy(&o.stderr);
                        // stash pop 有冲突，给警告但不阻塞构建
                        emit(
                            "git",
                            "warning",
                            &format!("恢复改动有冲突，请手动处理: {}", err.trim()),
                        );
                    }
                    Err(e) => {
                        emit("git", "warning", &format!("stash pop 失败: {}", e));
                    }
                }
            }

            emit(
                "git",
                "success",
                &format!("使用本地目录: {} (已同步 {})", local_path, branch),
            );
            return Ok(path);
        }
    }

    // Clone from remote (git_clone mode or fallback)
    let repo_url = &config.repo_url;
    let repo_name = get_repo_name(repo_url);
    let workspace = crate::logic::data_dir::cicd_workspace_dir();
    fs::create_dir_all(&workspace).map_err(|e| format!("创建工作目录失败: {}", e))?;

    let target = workspace.join(&repo_name);

    if target.exists() {
        emit("git", "pulling", "拉取最新代码...");

        // 剥离 origin/ 前缀，避免 git pull origin origin/xxx 双重前缀
        let raw_branch = if config.branch.is_empty() {
            "main"
        } else {
            &config.branch
        };
        let branch = raw_branch.strip_prefix("origin/").unwrap_or(raw_branch);

        // Fetch latest
        let fetch_output = user_shell_cmd(&crate::logic::git::find_git())
            .args(["fetch", "origin"])
            .current_dir(&target)
            .output()
            .await
            .map_err(|e| format!("git fetch 失败: {}", e))?;

        if !fetch_output.status.success() {
            let err = String::from_utf8_lossy(&fetch_output.stderr);
            return Err(format!("git fetch 失败: {}", err.trim()));
        }

        // Checkout branch
        let checkout_output = user_shell_cmd(&crate::logic::git::find_git())
            .args(["checkout", branch])
            .current_dir(&target)
            .output()
            .await
            .map_err(|e| format!("git checkout 失败: {}", e))?;

        if !checkout_output.status.success() {
            // 分支不存在本地，尝试从 origin 创建
            let checkout_output2 = user_shell_cmd(&crate::logic::git::find_git())
                .args(["checkout", "-b", branch, &format!("origin/{}", branch)])
                .current_dir(&target)
                .output()
                .await
                .map_err(|e| format!("git checkout -b 失败: {}", e))?;

            if !checkout_output2.status.success() {
                let err = String::from_utf8_lossy(&checkout_output2.stderr);
                return Err(format!("git checkout 失败: {}", err.trim()));
            }
        }

        // Pull with rebase to ensure linear history and proper merge
        // 如果 rebase 失败，尝试 reset --hard 强制同步到远程
        let pull_output = user_shell_cmd(&crate::logic::git::find_git())
            .args(["pull", "--rebase", "origin", branch])
            .current_dir(&target)
            .output()
            .await
            .map_err(|e| format!("git pull 失败: {}", e))?;

        if !pull_output.status.success() {
            let err = String::from_utf8_lossy(&pull_output.stderr);
            emit(
                "git",
                "warning",
                &format!("rebase 失败，尝试硬重置: {}", err.trim()),
            );

            // 尝试硬重置到远程分支（丢弃本地改动）
            let reset_output = user_shell_cmd(&crate::logic::git::find_git())
                .args(["reset", "--hard", &format!("origin/{}", branch)])
                .current_dir(&target)
                .output()
                .await
                .map_err(|e| format!("git reset 失败: {}", e))?;

            if !reset_output.status.success() {
                let err2 = String::from_utf8_lossy(&reset_output.stderr);
                return Err(format!(
                    "git reset --hard 失败: {}（请检查分支是否存在）",
                    err2.trim()
                ));
            }

            emit(
                "git",
                "success",
                &format!("代码已强制同步 (分支: {})", branch),
            );
        } else {
            emit("git", "success", &format!("代码已更新 (分支: {})", branch));
        }
    } else {
        emit("git", "cloning", &format!("克隆仓库 {}", repo_url));

        let output = Command::new(crate::logic::git::find_git())
            .args(["clone", "-b", &config.branch, repo_url])
            .current_dir(&workspace)
            .output()
            .await
            .map_err(|e| format!("git clone 失败: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git clone 失败: {}", err.trim()));
        }

        emit("git", "success", "仓库已克隆");
    }

    Ok(target)
}

fn get_repo_name(url: &str) -> String {
    let name = url
        .split('/')
        .last()
        .unwrap_or("repo")
        .trim_end_matches(".git");
    name.to_string()
}

// =================== Dependencies ===================

/// Install dependencies for git_clone mode (npm/pnpm/yarn install)
/// Only runs when node_modules is missing or empty
async fn install_dependencies(
    config: &DeployConfig,
    project_path: &PathBuf,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    // For multi-module projects, check root first, then module paths
    // Determine where to install dependencies:
    // 1. Root project path (if has package.json)
    // 2. Module build paths (if root doesn't have package.json but modules do)

    // First check root project path
    let root_package_json = project_path.join("package.json");
    let install_paths: Vec<PathBuf> = if root_package_json.exists() {
        // Monorepo style: package.json at root
        vec![project_path.clone()]
    } else if !config.modules.is_empty() {
        // Multi-module non-monorepo: each module may have its own package.json
        config
            .modules
            .iter()
            .filter_map(|m| {
                let module_path = if let Some(ref bp) = m.build_path {
                    project_path.join(bp)
                } else if let Some(ref p) = m.path {
                    project_path.join(p)
                } else {
                    return None;
                };
                if module_path.join("package.json").exists() {
                    Some(module_path)
                } else {
                    None
                }
            })
            .collect()
    } else {
        // Single project with build_path
        let build_path = if let Some(ref bp) = config.build_path {
            project_path.join(bp)
        } else {
            project_path.clone()
        };
        if build_path.join("package.json").exists() {
            vec![build_path]
        } else {
            vec![]
        }
    };

    if install_paths.is_empty() {
        emit("deps", "skipped", "非前端项目，跳过依赖安装");
        return Ok(());
    }

    // Install dependencies for each path
    for install_path in &install_paths {
        // Check if node_modules exists and is non-empty
        let node_modules = install_path.join("node_modules");
        let needs_install = if node_modules.exists() {
            match fs::read_dir(&node_modules) {
                Ok(entries) => entries.count() == 0,
                Err(_) => true,
            }
        } else {
            true
        };

        if !needs_install {
            let path_name = if install_path == project_path {
                "根目录"
            } else {
                install_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("module")
            };
            emit(
                "deps",
                "skipped",
                &format!("{} node_modules 已存在，跳过安装", path_name),
            );
            continue;
        }

        // Determine install tool from build_tool config (use first module's build_tool if available)
        let tool = config
            .modules
            .first()
            .and_then(|m| m.build_tool.as_deref())
            .or(config.build_tool.as_deref())
            .unwrap_or("npm");

        let install_cmd = match tool {
            "pnpm" => "pnpm install",
            "yarn" => "yarn",
            _ => "npm install",
        };

        let path_name = if install_path == project_path {
            "根目录"
        } else {
            install_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("module")
        };
        emit(
            "deps",
            "installing",
            &format!("{} 安装依赖: {}", path_name, install_cmd),
        );

        let mut cmd = user_shell_cmd("sh");
        cmd.arg("-c")
            .arg(install_cmd)
            .current_dir(install_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Inject NODE_HOME and npm paths
        if let Some(ref node_home) = config.node_home {
            cmd.env("NODE_HOME", node_home);
        }
        extend_path_npm(&mut cmd, &config.node_home, &config.npm_home);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("依赖安装启动失败: {}", e))?;

        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

        // Stream output
        let stdout_fut = async {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader
                .read_line(&mut line)
                .await
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    emit("deps", "installing", trimmed);
                }
                line.clear();
            }
        };
        let stderr_fut = async {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader
                .read_line(&mut line)
                .await
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    emit("deps", "installing", trimmed);
                }
                line.clear();
            }
        };
        let status_fut = child.wait();
        let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
        let status = status.map_err(|e| format!("等待安装进程失败: {}", e))?;

        if !status.success() {
            return Err(format!(
                "{} 依赖安装失败 (exit {})",
                path_name,
                status.code().unwrap_or(-1)
            ));
        }

        emit("deps", "success", &format!("{} 依赖安装完成", path_name));
    }

    Ok(())
}

// =================== Build ===================

async fn do_build(
    config: &DeployConfig,
    project_path: &PathBuf,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    let has_modules = !config.modules.is_empty();

    if has_modules && config.parent_build_mode && config.build_tool.as_deref() == Some("maven") {
        // Parent unified build — delegate to run_maven_build for streaming output
        emit(
            "maven",
            "starting",
            "父模块统一构建 (Maven multi-module)...",
        );

        let parent_cwd = if let Some(ref pbp) = config.parent_build_path {
            project_path.join(pbp)
        } else {
            project_path.clone()
        };

        run_maven_build(&parent_cwd, config, emit).await?;

        emit(
            "maven",
            "success",
            &format!("父模块构建成功 ({} 个子模块)", config.modules.len()),
        );
    } else if has_modules {
        // Per-module build
        let mut sorted_modules = config.modules.clone();
        sorted_modules.sort_by_key(|m| m.deploy_order);

        for module in &sorted_modules {
            if let Err(e) = build_single_module(project_path, module, config, emit).await {
                return Err(format!(
                    "模块 {} 构建失败: {}",
                    module.name.as_deref().unwrap_or("unknown"),
                    e
                ));
            }
        }
    } else {
        // Single project build
        let build_path = if let Some(ref bp) = config.build_path {
            project_path.join(bp)
        } else {
            project_path.clone()
        };

        let build_tool = config.build_tool.as_deref().unwrap_or_else(|| {
            if config.maven_home.is_some() {
                "maven"
            } else {
                "npm"
            }
        });

        match build_tool {
            "maven" => run_maven_build(&build_path, config, emit).await?,
            "npm" | "pnpm" | "yarn" => run_npm_build(&build_path, config, build_tool, emit).await?,
            "gradle" => run_gradle_build(&build_path, emit).await?,
            "cargo" => run_cargo_build(&build_path, config, emit).await?,
            _ => return Err(format!("不支持的构建工具: {}", build_tool)),
        }
    }

    Ok(())
}

async fn build_single_module(
    project_path: &PathBuf,
    module: &DeployModuleConfig,
    config: &DeployConfig,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    let build_path = if let Some(ref bp) = module.build_path {
        project_path.join(bp)
    } else if let Some(ref mp) = module.path {
        project_path.join(mp)
    } else {
        project_path.clone()
    };

    // Custom build command (stream output for real-time logs)
    if let Some(ref cmd) = module.build_command.as_ref().filter(|s| !s.is_empty()) {
        let final_cmd = if cmd.contains("mvn") && config.skip_tests && !cmd.contains("skipTests") {
            let appended = format!("{} -DskipTests", cmd);
            emit(
                "build",
                "info",
                &format!("已追加 -DskipTests（原始命令中未含）"),
            );
            appended
        } else {
            cmd.to_string()
        };
        emit("build", "starting", &format!("执行构建命令: {}", final_cmd));

        let mut child_cmd = user_shell_cmd("sh");
        child_cmd
            .arg("-c")
            .arg(&final_cmd)
            .current_dir(&build_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 注入配置的工具路径，确保自定义命令能找到 mvn/npm/node
        // 只在命令包含 mvn 时设置 JAVA_HOME，避免前端项目污染
        if cmd.contains("mvn") {
            if let Some(ref java_home) = config.java_home {
                emit("build", "info", &format!("JAVA_HOME: {}", java_home));
                child_cmd.env("JAVA_HOME", java_home);
            }
        }
        extend_path(&mut child_cmd, &config.java_home, &config.maven_home);
        extend_path_npm(&mut child_cmd, &config.node_home, &config.npm_home);

        let mut child = child_cmd
            .spawn()
            .map_err(|e| format!("构建命令启动失败: {}", e))?;

        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

        let stdout_fut = async {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader
                .read_line(&mut line)
                .await
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    emit("build", "building", trimmed);
                }
                line.clear();
            }
        };
        let stderr_fut = async {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader
                .read_line(&mut line)
                .await
                .map(|n| n > 0)
                .unwrap_or(false)
            {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    emit("build", "building", trimmed);
                }
                line.clear();
            }
        };
        let status_fut = child.wait();
        let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
        let status = status.map_err(|e| format!("等待进程失败: {}", e))?;

        if !status.success() {
            return Err(format!("构建失败 (exit {})", status.code().unwrap_or(-1)));
        }
        emit("build", "success", "构建完成");
        return Ok(());
    }

    let tool = module
        .build_tool
        .as_deref()
        .or(config.build_tool.as_deref())
        .unwrap_or("npm");

    match tool {
        "maven" => run_maven_build(&build_path, config, emit).await,
        "npm" | "pnpm" | "yarn" => run_npm_build(&build_path, config, tool, emit).await,
        "cargo" => run_cargo_build(&build_path, config, emit).await,
        _ => Err(format!("不支持的构建工具: {}", tool)),
    }
}

async fn run_maven_build(
    build_path: &Path,
    config: &DeployConfig,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    emit("maven", "starting", "开始 Maven 构建");

    let mvn = resolve_mvn_bin(&config.maven_home);

    let mut args = vec!["clean", "package"];
    if config.skip_tests {
        args.push("-DskipTests");
    }
    if let Some(ref profile) = config.maven_profile {
        args.push("-P");
        args.push(profile);
    }
    if let Some(ref settings) = config.maven_settings {
        if !settings.is_empty() {
            args.push("-s");
            args.push(settings);
        }
    }

    let mut cmd = user_shell_cmd(&mvn.to_string_lossy());
    cmd.args(&args).current_dir(build_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(ref java_home) = config.java_home {
        cmd.env("JAVA_HOME", java_home);
    }
    extend_path(&mut cmd, &config.java_home, &config.maven_home);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Maven 构建启动失败: {}", e))?;

    let stdout = child.stdout.take().ok_or("无法获取 Maven stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 Maven stderr")?;

    // Stream stdout + stderr concurrently
    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("maven", "building", trimmed);
            }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("maven", "building", trimmed);
            }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 Maven 进程失败: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Maven 构建失败 (exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    emit("maven", "success", "构建成功");
    Ok(())
}

async fn run_npm_build(
    build_path: &Path,
    config: &DeployConfig,
    tool: &str,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    let script = config
        .npm_custom_script
        .as_deref()
        .or(config.npm_script.as_deref())
        .unwrap_or("build");

    emit("npm", "starting", &format!("开始 {} {} 构建", tool, script));

    let npm_cmd = resolve_npm_cmd(config, tool);

    let mut cmd = user_shell_cmd(&npm_cmd);
    cmd.args(["run", script]).current_dir(build_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(ref node_home) = config.node_home {
        cmd.env("NODE_HOME", node_home);
    }
    extend_path_npm(&mut cmd, &config.node_home, &config.npm_home);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{} 构建启动失败: {}", tool, e))?;

    let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

    // Stream stdout + stderr concurrently
    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("npm", "building", trimmed);
            }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("npm", "building", trimmed);
            }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 {} 进程失败: {}", tool, e))?;

    if !status.success() {
        return Err(format!(
            "{} 构建失败 (exit {})",
            tool,
            status.code().unwrap_or(-1)
        ));
    }

    emit("npm", "success", &format!("{} 构建成功", tool));
    Ok(())
}

async fn run_gradle_build(
    build_path: &Path,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    emit("gradle", "starting", "开始 Gradle 构建");

    let gradle = if build_path.join("gradlew").exists() {
        "./gradlew".to_string()
    } else {
        "gradle".to_string()
    };

    let mut cmd = user_shell_cmd("sh");
    cmd.arg("-c").arg(format!("{} clean build", gradle));
    cmd.current_dir(build_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Gradle 构建启动失败: {}", e))?;

    let stdout = child.stdout.take().ok_or("无法获取 Gradle stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 Gradle stderr")?;

    // Stream stdout + stderr concurrently（对齐 Maven/npm 的实时日志模式）
    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("gradle", "building", trimmed);
            }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("gradle", "building", trimmed);
            }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 Gradle 进程失败: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Gradle 构建失败 (exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    emit("gradle", "success", "Gradle 构建成功");
    Ok(())
}

fn resolve_npm_cmd(config: &DeployConfig, tool: &str) -> String {
    // 1. 优先用 npm_home（如果填的是 npm 目录或 npm 可执行文件路径）
    if let Some(ref npm_home) = config.npm_home {
        if !npm_home.is_empty() {
            let npm_path = PathBuf::from(npm_home);
            if npm_path.is_dir() {
                let bin_tool = npm_path.join("bin").join(tool);
                if bin_tool.exists() {
                    return bin_tool.to_string_lossy().to_string();
                }
            }
            // 如果填的是完整路径（如 .../bin/npm），直接用
            if npm_path.is_file() || npm_path.exists() {
                return npm_home.clone();
            }
        }
    }
    // 2. Fallback 到 node_home（如果填的是 Node 版本目录，npm 就在 bin/ 下）
    if let Some(ref node_home) = config.node_home {
        if !node_home.is_empty() {
            let node_path = PathBuf::from(node_home);
            if node_path.is_dir() {
                let bin_tool = node_path.join("bin").join(tool);
                if bin_tool.exists() {
                    return bin_tool.to_string_lossy().to_string();
                }
            }
        }
    }
    // 3. 实在没有配置，回退到系统 PATH 里的命令
    tool.to_string()
}

/// Resolve the Maven binary: if maven_home is a file (e.g., /opt/homebrew/bin/mvn),
/// use it directly; if it's a directory, append /bin/mvn.
fn resolve_mvn_bin(maven_home: &Option<String>) -> PathBuf {
    if let Some(home) = maven_home {
        let path = PathBuf::from(home);
        if path.is_file() {
            return path;
        }
        let bin = path.join("bin").join("mvn");
        if bin.exists() {
            return bin;
        }
    }
    PathBuf::from("mvn")
}

fn extend_path(cmd: &mut Command, java_home: &Option<String>, maven_home: &Option<String>) {
    let mut extra_paths = Vec::new();
    if let Some(jh) = java_home {
        extra_paths.push(format!("{}/bin", jh));
    }
    if let Some(mh) = maven_home {
        let p = PathBuf::from(mh);
        if p.is_file() {
            // maven_home 指向二进制文件如 /opt/homebrew/bin/mvn，加其父目录
            if let Some(parent) = p.parent() {
                extra_paths.push(parent.to_string_lossy().to_string());
            }
        } else {
            // maven_home 指向安装目录如 /opt/apache-maven-3.9，加 bin 子目录
            extra_paths.push(format!("{}/bin", mh));
        }
    }
    extra_paths.push("/usr/local/bin".to_string());
    extra_paths.push("/usr/bin".to_string());
    extra_paths.push("/bin".to_string());

    // 优先从用户登录 shell 获取 PATH（含 sdkman/nvm 等），而非 Tauri 进程的 PATH
    let shell_env = get_user_shell_env();
    let current_path = shell_env
        .get("PATH")
        .cloned()
        .or_else(|| std::env::var("PATH").ok())
        .unwrap_or_default();
    extra_paths.push(current_path);

    cmd.env("PATH", extra_paths.join(":"));
}

fn extend_path_npm(cmd: &mut Command, node_home: &Option<String>, npm_home: &Option<String>) {
    let mut extra_paths = Vec::new();
    if let Some(nh) = node_home {
        let p = PathBuf::from(nh);
        if p.is_file() {
            if let Some(parent) = p.parent() {
                extra_paths.push(parent.to_string_lossy().to_string());
            }
        } else {
            extra_paths.push(format!("{}/bin", nh));
        }
    }
    if let Some(nh) = npm_home {
        let npm_path = PathBuf::from(nh);
        if npm_path.is_dir() {
            extra_paths.push(format!("{}/bin", nh));
        }
    }
    extra_paths.push("/usr/local/bin".to_string());
    extra_paths.push("/usr/bin".to_string());
    extra_paths.push("/bin".to_string());

    // 优先从用户登录 shell 获取 PATH（含 sdkman/nvm 等），而非 Tauri 进程的 PATH
    let shell_env = get_user_shell_env();
    let current_path = shell_env
        .get("PATH")
        .cloned()
        .or_else(|| std::env::var("PATH").ok())
        .unwrap_or_default();
    extra_paths.push(current_path);

    cmd.env("PATH", extra_paths.join(":"));
}

// =================== Cargo Build ===================

async fn run_cargo_build(
    build_path: &Path,
    config: &DeployConfig,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    // Use custom build command if provided, otherwise default
    // If build_command contains "cargo", treat it as a full command
    // If build_command is "debug", use "cargo build"
    // Otherwise default to "cargo build --release"
    let cmd_str = config.build_command.as_deref().unwrap_or("");
    let cargo_cmd = if cmd_str.contains("cargo") {
        cmd_str.to_string()
    } else if cmd_str == "debug" {
        "cargo build".to_string()
    } else {
        "cargo build --release".to_string()
    };

    emit(
        "cargo",
        "starting",
        &format!("开始 Cargo 构建 ({})", cargo_cmd),
    );

    let mut cmd = user_shell_cmd("sh");
    cmd.arg("-c").arg(&cargo_cmd);
    cmd.current_dir(build_path);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Cargo 构建启动失败: {}", e))?;

    let stdout = child.stdout.take().ok_or("无法获取 Cargo stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 Cargo stderr")?;

    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("cargo", "building", trimmed);
            }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader
            .read_line(&mut line)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
        {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                emit("cargo", "building", trimmed);
            }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 Cargo 进程失败: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Cargo 构建失败 (exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    emit("cargo", "success", "Cargo 构建成功");
    Ok(())
}

#[allow(dead_code)]
fn get_last_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = if lines.len() > n { lines.len() - n } else { 0 };
    lines[start..].join("\\n")
}

// =================== Artifact Collection ===================

fn collect_artifacts(
    project_path: &PathBuf,
    config: &DeployConfig,
) -> Result<Vec<Artifact>, String> {
    let mut artifacts = Vec::new();

    if config.modules.is_empty() {
        // Single project
        let is_cargo = config.build_tool.as_deref() == Some("cargo");

        let output_dir = if let Some(ref bp) = config.build_path {
            if is_cargo {
                project_path.join(bp).join("target/release")
            } else {
                project_path.join(bp).join("target")
            }
        } else {
            if is_cargo {
                project_path.join("target/release")
            } else {
                project_path.join("target")
            }
        };

        if output_dir.exists() {
            if is_cargo {
                collect_cargo_binaries(&output_dir, &config.deploy_dir, &mut artifacts)?;
            } else {
                collect_from_dir(
                    &output_dir,
                    None,
                    &config.deploy_dir,
                    config.lib_separate,
                    None,
                    &mut artifacts,
                )?;
            }
        }
    } else {
        // Multi-module
        for module in &config.modules {
            if !module.enabled {
                continue;
            }

            let output_dir = if let Some(ref mp) = module.path {
                let artifact_root = project_path.join(mp);
                if let Some(ref op) = module.output_path {
                    artifact_root.join(op)
                } else {
                    artifact_root.join("target")
                }
            } else if let Some(ref bp) = module.build_path {
                let bp_path = project_path.join(bp);
                if let Some(ref op) = module.output_path {
                    bp_path.join(op)
                } else {
                    bp_path.join("target")
                }
            } else {
                project_path.join(module.output_path.as_deref().unwrap_or("target"))
            };

            if !output_dir.exists() {
                continue;
            }

            // Specific artifact name (non-empty)
            if let Some(ref an) = module.artifact_name {
                if !an.is_empty() {
                    let artifact_path = output_dir.join(an);
                    if artifact_path.exists() {
                        artifacts.push(Artifact {
                            name: an.clone(),
                            local_path: artifact_path.to_string_lossy().to_string(),
                            module: module.name.clone(),
                            is_lib: false,
                            is_compressed: false,
                            deploy_path: module
                                .deploy_path
                                .clone()
                                .filter(|s| !s.is_empty())
                                .or(Some(config.deploy_dir.clone())),
                        });
                    }
                    // Collect lib/ directory if lib_separate is enabled
                    if config.lib_separate {
                        let lib_dir = output_dir.join("lib");
                        if lib_dir.exists() && lib_dir.is_dir() {
                            let lib_name =
                                format!("{}-lib.zip", module.name.as_deref().unwrap_or("main"));
                            let lib_zip = output_dir.join(&lib_name);
                            if !lib_zip.exists() {
                                create_zip(
                                    &lib_dir,
                                    &lib_zip,
                                    module.lib_filter_rules.as_deref(),
                                    true,
                                )
                                .map_err(|e| format!("压缩 lib 目录失败: {}", e))?;
                            }
                            artifacts.push(Artifact {
                                name: lib_name,
                                local_path: lib_zip.to_string_lossy().to_string(),
                                module: module.name.clone(),
                                is_lib: true,
                                is_compressed: true,
                                deploy_path: module
                                    .deploy_path
                                    .clone()
                                    .filter(|s| !s.is_empty())
                                    .or(Some(config.deploy_dir.clone())),
                            });
                        }
                    }
                    continue;
                }
            }

            // Handle by artifact type
            let at = module.artifact_type.as_deref().unwrap_or("");
            if at == "dist" {
                let zip_name = format!("{}.zip", module.name.as_deref().unwrap_or("dist"));
                let zip_path = output_dir.join(&zip_name);
                if !zip_path.exists() {
                    create_zip(&output_dir, &zip_path, None, false)
                        .map_err(|e| format!("压缩产物目录失败: {}", e))?;
                }
                artifacts.push(Artifact {
                    name: zip_name,
                    local_path: zip_path.to_string_lossy().to_string(),
                    module: module.name.clone(),
                    is_lib: false,
                    is_compressed: true,
                    deploy_path: module
                        .deploy_path
                        .clone()
                        .filter(|s| !s.is_empty())
                        .or(Some(config.deploy_dir.clone())),
                });
                continue;
            }

            collect_from_dir(
                &output_dir,
                module.name.as_deref(),
                module.deploy_path.as_deref().unwrap_or(&config.deploy_dir),
                config.lib_separate,
                module.lib_filter_rules.as_deref(),
                &mut artifacts,
            )?;
        }
    }

    Ok(artifacts)
}

fn collect_from_dir(
    output_dir: &Path,
    module: Option<&str>,
    default_deploy_path: &str,
    lib_separate: bool,
    lib_filter: Option<&str>,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), String> {
    // Find .jar files
    let entries = fs::read_dir(output_dir).map_err(|e| format!("读取产物目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip sources/javadoc jars
        if name.contains("-sources") || name.contains("-javadoc") {
            continue;
        }

        if name.ends_with(".jar") {
            artifacts.push(Artifact {
                name: name.clone(),
                local_path: path.to_string_lossy().to_string(),
                module: module.map(|s| s.to_string()),
                is_lib: false,
                is_compressed: false,
                deploy_path: Some(default_deploy_path.to_string()),
            });
        }
    }

    // Collect lib/ if libSeparate is enabled
    if lib_separate {
        let lib_dir = output_dir.join("lib");
        if lib_dir.exists() && lib_dir.is_dir() {
            let zip_name = format!("{}-lib.zip", module.unwrap_or("main"));
            let zip_path = output_dir.join(&zip_name);

            // Create zip of lib directory (skip if already exists)
            if !zip_path.exists() {
                create_zip(&lib_dir, &zip_path, lib_filter, true)
                    .map_err(|e| format!("压缩 lib 目录失败: {}", e))?;
            }

            artifacts.push(Artifact {
                name: zip_name,
                local_path: zip_path.to_string_lossy().to_string(),
                module: module.map(|s| s.to_string()),
                is_lib: true,
                is_compressed: true,
                deploy_path: Some(default_deploy_path.to_string()),
            });
        }
    }

    Ok(())
}

/// Collect Cargo binary artifacts from `target/release/`.
/// Finds executable files (no extension) at the top level,
/// excluding well-known Cargo build artifacts.
fn collect_cargo_binaries(
    output_dir: &Path,
    default_deploy_path: &str,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), String> {
    let entries = fs::read_dir(output_dir).map_err(|e| format!("读取产物目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip known Cargo build artifacts
        if name.ends_with(".d")
            || name.ends_with(".rlib")
            || name.ends_with(".rmeta")
            || name.ends_with(".so")
            || name.ends_with(".dylib")
            || name.ends_with(".dll")
            || name.ends_with(".pdb")
            || name == "build"
            || name.starts_with(".")
        {
            continue;
        }

        // Treat files without extension as Cargo binaries
        if !name.contains('.') && !name.starts_with('_') {
            #[cfg(target_os = "windows")]
            let binary_name = format!("{}.exe", name);
            #[cfg(not(target_os = "windows"))]
            let binary_name = name.clone();

            // Check that it's actually an executable (not a directory artifact)
            if path.is_file() {
                artifacts.push(Artifact {
                    name: binary_name,
                    local_path: path.to_string_lossy().to_string(),
                    module: None,
                    is_lib: false,
                    is_compressed: false,
                    deploy_path: Some(default_deploy_path.to_string()),
                });
            }
        }
    }

    Ok(())
}

fn create_zip(
    src_dir: &Path,
    dest_zip: &Path,
    filter: Option<&str>,
    junk_paths: bool,
) -> Result<(), String> {
    let output = if let Some(filter_str) = filter {
        let filter_str = filter_str.trim();
        if filter_str.is_empty() {
            return Ok(());
        }

        // 支持多行过滤模式（用户可能在 textarea 里每行一个 pattern）
        let patterns: Vec<&str> = filter_str
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if patterns.is_empty() {
            return Ok(());
        }

        // 用安全的方式构建 find 命令（避免 shell 注入）
        let mut find_cmd = std::process::Command::new("find");
        find_cmd.arg(".");
        find_cmd.arg("-type").arg("f");
        find_cmd.arg("-maxdepth").arg("1");
        find_cmd.current_dir(src_dir);
        for (i, p) in patterns.iter().enumerate() {
            if i == 0 {
                find_cmd.args(["-name", p]);
            } else {
                find_cmd.args(["-o", "-name", p]);
            }
        }

        let find_output = find_cmd
            .output()
            .map_err(|e| format!("find 命令失败: {}", e))?;

        if !find_output.status.success() {
            let err = String::from_utf8_lossy(&find_output.stderr);
            return Err(format!("查找匹配文件失败: {}", err.trim()));
        }

        let files: Vec<String> = String::from_utf8_lossy(&find_output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.strip_prefix("./").unwrap_or(l).to_string())
            .collect();

        // 没有匹配的文件 → 跳过，不是错误
        if files.is_empty() {
            return Ok(());
        }

        // 通过 stdin pipe 传递文件列表给 zip
        let mut child = std::process::Command::new("zip")
            .arg("-q")
            .arg(if junk_paths { "-j" } else { "" })
            .arg(dest_zip.as_os_str())
            .arg("-@")
            .stdin(std::process::Stdio::piped())
            .current_dir(src_dir)
            .spawn()
            .map_err(|e| format!("zip 启动失败: {}", e))?;

        if let Some(ref mut stdin) = child.stdin {
            for f in files {
                writeln!(stdin, "{}", f).map_err(|e| format!("写入文件列表失败: {}", e))?;
            }
        }

        child
            .wait_with_output()
            .map_err(|e| format!("zip 完成失败: {}", e))?
    } else {
        // 无过滤：压缩整个目录
        let mut cmd = std::process::Command::new("zip");
        cmd.arg("-q");
        cmd.arg("-r");
        cmd.arg(dest_zip);
        cmd.arg(".");
        if junk_paths {
            cmd.arg("-j");
        }
        cmd.current_dir(src_dir);
        cmd.output().map_err(|e| format!("zip 命令失败: {}", e))?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        return Err(format!(
            "zip 失败 (exit={}): {} {}",
            exit_code, stdout, stderr
        ));
    }

    Ok(())
}

// =================== SSH Deploy ===================

async fn deploy_to_server(
    srv: &DeployServerConfig,
    artifacts: &[Artifact],
    config: &DeployConfig,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    use ssh2::Session;
    use std::net::TcpStream;

    emit("ssh", "connecting", &format!("连接服务器 {}", srv.host));

    let tcp = TcpStream::connect(format!("{}:{}", srv.host, srv.port))
        .map_err(|e| format!("连接 {}:{} 失败: {}", srv.host, srv.port, e))?;

    let mut sess = Session::new().map_err(|e| format!("创建 SSH session 失败: {}", e))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH 握手失败: {}", e))?;

    // Authenticate
    if let Some(ref key) = srv.private_key.as_ref().filter(|k| !k.is_empty()) {
        sess.userauth_pubkey_file(&srv.username, None, Path::new(key), srv.password.as_deref())
            .map_err(|e| format!("SSH 密钥认证失败: {}", e))?;
    } else if let Some(ref pw) = srv.password.as_ref().filter(|p| !p.is_empty()) {
        sess.userauth_password(&srv.username, pw)
            .map_err(|e| format!("SSH 密码认证失败: {}", e))?;
    } else {
        return Err("缺少认证信息".to_string());
    }

    let deploy_dir = if srv.deploy_dir.is_empty() {
        if config.deploy_dir.is_empty() {
            return Err(
                "部署路径未配置：请在配置中设置「部署路径」或在服务器节点设置「部署路径」"
                    .to_string(),
            );
        }
        &config.deploy_dir
    } else {
        &srv.deploy_dir
    };

    // Resolve remote home directory for ~ expansion (SFTP doesn't expand ~)
    let remote_home = ssh_exec(&sess, "echo $HOME")
        .unwrap_or_default()
        .trim()
        .to_string();
    let expand_path = |p: &str| -> String {
        if p.starts_with("~/") && !remote_home.is_empty() {
            format!("{}/{}", remote_home.trim_end_matches('/'), &p[2..])
        } else if p == "~" && !remote_home.is_empty() {
            remote_home.clone()
        } else {
            p.to_string()
        }
    };
    // Resolve deploy_dir for SFTP usage (keep original for shell commands)
    let deploy_dir_resolved = expand_path(deploy_dir);

    // Create deploy directory (shell command, ~ works fine)
    let mkdir_cmd = format!("mkdir -p {}", shell_escape(deploy_dir));
    ssh_exec(&sess, &mkdir_cmd)?;

    if config.lib_separate {
        if let Some(ref lib_dir) = config.lib_dir.as_ref().filter(|d| !d.is_empty()) {
            let cmd = format!("mkdir -p {}", shell_escape(lib_dir));
            ssh_exec(&sess, &cmd)?;
        }
    }

    // Create module-specific deploy paths (shell expand ~ fine)
    for artifact in artifacts {
        if let Some(ref dp) = artifact.deploy_path {
            if dp != deploy_dir {
                let cmd = format!("mkdir -p {}", shell_escape(dp));
                ssh_exec(&sess, &cmd)?;
            }
        }
    }

    // Upload via SFTP
    emit(
        "ssh",
        "uploading",
        &format!("上传 {} 个产物", artifacts.len()),
    );

    let mut sftp = sess.sftp().map_err(|e| format!("SFTP 初始化失败: {}", e))?;

    for artifact in artifacts {
        let target_path = if let Some(ref dp) = artifact.deploy_path {
            let resolved = expand_path(dp);
            if artifact.is_lib {
                format!("{}/lib", resolved)
            } else {
                resolved
            }
        } else if artifact.is_lib && config.lib_separate {
            config
                .lib_dir
                .as_ref()
                .map(|ld| expand_path(ld))
                .unwrap_or_else(|| deploy_dir_resolved.clone())
        } else {
            deploy_dir_resolved.clone()
        };

        let remote_file = format!("{}/{}", target_path.trim_end_matches('/'), artifact.name);

        emit(
            "ssh",
            "uploading",
            &format!(
                "↑ {} ({})",
                artifact.name,
                file_size_display(&artifact.local_path)
            ),
        );

        upload_file(&mut sftp, &artifact.local_path, &remote_file)
            .map_err(|e| format!("上传 {} 失败: {}", artifact.name, e))?;

        emit(
            "ssh",
            "success",
            &format!("✅ {} 上传完成 → {}", artifact.name, target_path),
        );

        // Extract compressed artifacts
        if artifact.is_compressed {
            emit(
                "ssh",
                "info",
                &format!("解压 {} → {}", artifact.name, target_path),
            );
            let extract_cmd = format!(
                "cd {} && unzip -o {} && rm -f {}",
                shell_escape(&target_path),
                shell_escape(&artifact.name),
                shell_escape(&artifact.name)
            );
            ssh_exec(&sess, &extract_cmd)?;
            emit(
                "ssh",
                "success",
                &format!("✅ {} 解压完成 → {}", artifact.name, target_path),
            );
        }
    }

    drop(sftp);
    sess.disconnect(None, "", None).ok();

    Ok(())
}

fn upload_file(sftp: &mut ssh2::Sftp, local_path: &str, remote_path: &str) -> Result<(), String> {
    let mut local_file =
        fs::File::open(local_path).map_err(|e| format!("打开本地文件失败: {}", e))?;

    let mut remote_file = sftp
        .create(Path::new(remote_path))
        .map_err(|e| format!("创建远程文件失败: {}", e))?;

    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
    loop {
        let n = local_file
            .read(&mut buf)
            .map_err(|e| format!("读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        remote_file
            .write_all(&buf[..n])
            .map_err(|e| format!("写入失败: {}", e))?;
    }

    Ok(())
}

fn ssh_exec(sess: &ssh2::Session, cmd: &str) -> Result<String, String> {
    let mut channel = sess
        .channel_session()
        .map_err(|e| format!("SSH channel 创建失败: {}", e))?;
    // 直接执行命令，避免 bash -l -c 包装破坏管道/重定向等 shell 特性
    channel
        .exec(cmd)
        .map_err(|e| format!("SSH exec 失败: {}", e))?;

    let mut output = String::new();
    channel.read_to_string(&mut output).ok();
    channel.wait_close().ok();

    let exit_status = channel.exit_status().unwrap_or(-1);
    if exit_status != 0 {
        return Err(format!(
            "SSH 命令失败 (exit {}): cmd={}\noutput={}",
            exit_status,
            cmd,
            output.trim().chars().take(500).collect::<String>()
        ));
    }

    Ok(output)
}

// =================== Restart ===================

async fn execute_restart(
    srv: &DeployServerConfig,
    script: &str,
    deploy_dir_fallback: &str,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    use ssh2::Session;
    use std::net::TcpStream;

    let label = srv.label.as_deref().unwrap_or("服务器");
    emit(
        "restart",
        "starting",
        &format!("在 {} ({}) 执行重启脚本", label, srv.host),
    );

    let tcp = TcpStream::connect(format!("{}:{}", srv.host, srv.port))
        .map_err(|e| format!("连接失败: {}", e))?;

    let mut sess = Session::new().map_err(|e| format!("SSH session 失败: {}", e))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH 握手失败: {}", e))?;

    if let Some(ref key) = srv.private_key.as_ref().filter(|k| !k.is_empty()) {
        sess.userauth_pubkey_file(&srv.username, None, Path::new(key), srv.password.as_deref())
            .map_err(|e| format!("认证失败: {}", e))?;
    } else if let Some(ref pw) = srv.password.as_ref().filter(|p| !p.is_empty()) {
        sess.userauth_password(&srv.username, pw)
            .map_err(|e| format!("认证失败: {}", e))?;
    } else {
        return Err("缺少认证信息".to_string());
    }

    if !sess.authenticated() {
        return Err("SSH 认证失败（密钥或密码不正确）".to_string());
    }

    // 与 Electron 版本对齐：使用 bash -l -c 加载用户环境变量，等待脚本完成
    // 解析脚本路径和参数（第一个是脚本文件，剩余是参数）
    let parts: Vec<&str> = script.trim().split_whitespace().collect();
    let script_file = parts.first().map_or(script, |v| *v);
    let script_args = if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        "".to_string()
    };

    // 将 ~/ 开头的路径转换为 $HOME/，确保在 bash -l -c 引号内能正确展开
    let script_file = if script_file.starts_with("~/") {
        format!("$HOME/{}", &script_file[2..])
    } else if script_file == "~" {
        "$HOME".to_string()
    } else {
        script_file.to_string()
    };

    // 根据脚本路径决定执行方式（~/ 已解析为 $HOME/，以 / 开头或 $HOME 开头都视为绝对路径）
    let is_absolute = script_file.starts_with('/') || script_file.starts_with("$HOME");
    let exec_cmd = if is_absolute {
        // 绝对路径：直接执行，chmod +x 确保可执行
        if script_args.is_empty() {
            format!(
                "chmod +x {} && bash -l -c '{}' 2>&1",
                script_file, script_file
            )
        } else {
            format!(
                "chmod +x {} && bash -l -c '{} {}' 2>&1",
                script_file, script_file, script_args
            )
        }
    } else {
        // 相对路径：需要先 cd 到 deployDir 再执行
        let restart_deploy_dir = if srv.deploy_dir.is_empty() {
            deploy_dir_fallback
        } else {
            &srv.deploy_dir
        };
        if script_args.is_empty() {
            format!(
                "cd {} && chmod +x {} && bash -l -c '{}' 2>&1",
                shell_escape(restart_deploy_dir),
                script_file,
                script_file
            )
        } else {
            format!(
                "cd {} && chmod +x {} && bash -l -c '{} {}' 2>&1",
                shell_escape(restart_deploy_dir),
                script_file,
                script_file,
                script_args
            )
        }
    };

    emit(
        "restart",
        "info",
        &format!(
            "执行命令: {}",
            exec_cmd.chars().take(120).collect::<String>()
        ),
    );

    // 执行并等待完成（与 Electron sshExec 一致）
    let mut channel = sess
        .channel_session()
        .map_err(|e| format!("创建 SSH channel 失败: {}", e))?;
    channel
        .exec(&exec_cmd)
        .map_err(|e| format!("执行重启命令失败: {}", e))?;

    // 收集输出
    let mut output = String::new();
    use std::io::Read;
    channel.read_to_string(&mut output).ok();
    channel.wait_close().ok();

    let exit_status = channel.exit_status().unwrap_or(-1);
    if exit_status != 0 {
        emit(
            "restart",
            "failed",
            &format!(
                "脚本退出码 {}，输出: {}",
                exit_status,
                output.trim().chars().take(200).collect::<String>()
            ),
        );
        // Non-fatal: 继续执行，不阻断部署流程
    } else {
        emit(
            "restart",
            "success",
            &format!(
                "应用已重启 (输出: {})",
                output.trim().chars().take(200).collect::<String>()
            ),
        );
    }
    sess.disconnect(None, "", None).ok();
    Ok(())
}

fn shell_escape(s: &str) -> String {
    // ~ 需要在引号外展开为 $HOME，否则单引号内的 ~ 不会被 shell 解释
    if s.starts_with('~') {
        let rest = &s[1..];
        if rest.is_empty() {
            "$HOME".to_string()
        } else {
            // rest 部分（如 /app）需要引号保护
            format!("$HOME'{}'", rest.replace('\'', "'\\''"))
        }
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

fn file_size_display(path: &str) -> String {
    let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

#[allow(dead_code)]
fn file_size_mb(path: &str) -> f64 {
    fs::metadata(path)
        .map(|m| m.len() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0)
}
