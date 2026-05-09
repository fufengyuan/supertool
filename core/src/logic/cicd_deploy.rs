/// CI/CD 部署执行引擎
///
/// 完整部署流水线：Git同步 → 构建 → 收集产物 → SFTP上传 → 远程重启
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// 获取用户登录 shell 的完整环境变量，确保版本管理器工具（NVM/Homebrew/nvm-windows）可用
fn get_user_shell_env() -> HashMap<String, String> {
    // 第一步：尝试从登录 shell 获取环境变量
    #[cfg(target_os = "windows")]
    let shell_output = std::process::Command::new("cmd")
        .args(["/c", "set"])
        .output().ok();
    #[cfg(not(target_os = "windows"))]
    let shell_output = std::process::Command::new("zsh")
        .args(["-l", "-c", "env"])
        .output().ok();
    
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
        let current_path = env.get("PATH").cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();
        let mut extra_paths: Vec<String> = Vec::new();
        
        #[cfg(target_os = "macos")]
        {
            // NVM (Node Version Manager)
            let nvm_dir = format!("{}/.nvm/versions/node", home);
            if std::path::Path::new(&nvm_dir).is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    let mut versions: Vec<_> = entries.filter_map(|e| e.ok())
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
                    let mut versions: Vec<_> = entries.filter_map(|e| e.ok())
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
                    let mut versions: Vec<_> = entries.filter_map(|e| e.ok())
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
    pub enabled: bool,
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
) -> Result<DeployResult, String> {
    // Load user shell environment (zsh login shell gets NVM, Homebrew, etc.)
    let shell_env = get_user_shell_env();

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
            let _ = writeln!(f, "[{}] [{}] [{}] {}", chrono::Utc::now().to_rfc3339(), stage, status, msg);
        }
        on_progress(event);
    };

    emit("deploy", "starting", &format!("开始部署 {}", config.repo_url));

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

    emit("collect", "success", &format!("产物收集完成 ({} 个)", artifacts.len()));

    // Copy artifacts to deploy-artifacts directory
    fs::create_dir_all(&artifact_dir).map_err(|e| format!("创建产物目录失败: {}", e))?;
    let mut artifact_paths = vec![];
    for artifact in &artifacts {
        let dest = artifact_dir.join(&artifact.name);
        if let Err(e) = fs::copy(&artifact.local_path, &dest) {
            emit("collect", "warning", &format!("复制 {} 失败: {}", artifact.name, e));
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

    let mut deploy_results = Vec::new();
    for srv in &config.servers {
        match deploy_to_server(srv, &artifacts, config, &emit).await {
            Ok(_) => {
                let label = srv.label.as_deref().unwrap_or("服务器");
                emit("ssh", "success", &format!("{} ({}) 部署完成", label, srv.host));
                deploy_results.push(true);
            }
            Err(e) => {
                let label = srv.label.as_deref().unwrap_or("服务器");
                emit("ssh", "failed", &format!("{} ({}) 部署失败: {}", label, srv.host, e));
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
    if let Some(ref script) = config.restart_script {
        for srv in &config.servers {
            if let Err(e) = execute_restart(srv, script, &emit).await {
                emit("restart", "failed", &e);
                // Non-fatal: restart might fail but deploy succeeded
            }
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
    // Use local path
    if let Some(ref local_path) = config.local_path {
        let path = PathBuf::from(local_path);
        if !path.exists() {
            return Err(format!("本地路径不存在: {}", local_path));
        }

        // Check if it's a git repo
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            emit("git", "warning", &format!("使用本地目录: {} (非 Git 仓库，跳过分支切换)", local_path));
            return Ok(path);
        }

        // Fetch and pull
        emit("git", "pulling", "拉取最新代码...");

        let output = Command::new(crate::logic::git::find_git())
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
        let raw_branch = if config.branch.is_empty() { "main" } else { &config.branch };
        // 剥离 origin/ 前缀，避免 git pull origin origin/xxx 双重前缀
        let branch = raw_branch.strip_prefix("origin/").unwrap_or(raw_branch);
        let output = Command::new(crate::logic::git::find_git())
            .args(["checkout", branch])
            .current_dir(&path)
            .output()
            .await
            .map_err(|e| format!("git checkout 失败: {}", e))?;

        if !output.status.success() {
            // 分支不存在，从 origin 创建
            let output2 = Command::new(crate::logic::git::find_git())
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

        // Pull latest
        let output = Command::new(crate::logic::git::find_git())
            .args(["pull", "origin", branch, "--no-edit", "--rebase"])
            .current_dir(&path)
            .output()
            .await
            .map_err(|e| format!("git pull 失败: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git pull 失败: {}", err.trim()));
        }

        emit("git", "success", &format!("使用本地目录: {} (已切换到 {})", local_path, branch));
        return Ok(path);
    }

    // Clone from remote
    let repo_url = &config.repo_url;
    let repo_name = get_repo_name(repo_url);
    let workspace = crate::logic::data_dir::cicd_workspace_dir();
    fs::create_dir_all(&workspace).map_err(|e| format!("创建工作目录失败: {}", e))?;

    let target = workspace.join(&repo_name);

    if target.exists() {
        emit("git", "pulling", "拉取最新代码...");

        // 剥离 origin/ 前缀，避免 git pull origin origin/xxx 双重前缀
        let branch = config.branch.strip_prefix("origin/").unwrap_or(&config.branch);

        let _ = Command::new(crate::logic::git::find_git())
            .args(["fetch", "origin"])
            .current_dir(&target)
            .output()
            .await;

        let _ = Command::new(crate::logic::git::find_git())
            .args(["checkout", branch])
            .current_dir(&target)
            .output()
            .await;

        let _ = Command::new(crate::logic::git::find_git())
            .args(["pull", "origin", branch])
            .current_dir(&target)
            .output()
            .await;

        emit("git", "success", "代码已更新");
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

// =================== Build ===================

async fn do_build(
    config: &DeployConfig,
    project_path: &PathBuf,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    let has_modules = !config.modules.is_empty();

    if has_modules && config.parent_build_mode && config.build_tool.as_deref() == Some("maven") {
        // Parent unified build — delegate to run_maven_build for streaming output
        emit("maven", "starting", "父模块统一构建 (Maven multi-module)...");

        let parent_cwd = if let Some(ref pbp) = config.parent_build_path {
            project_path.join(pbp)
        } else {
            project_path.clone()
        };

        run_maven_build(&parent_cwd, config, emit).await?;

        emit("maven", "success", &format!("父模块构建成功 ({} 个子模块)", config.modules.len()));
    } else if has_modules {
        // Per-module build
        let mut sorted_modules = config.modules.clone();
        sorted_modules.sort_by_key(|m| m.deploy_order);

        for module in &sorted_modules {
            if let Err(e) = build_single_module(project_path, module, config, emit).await {
                return Err(format!("模块 {} 构建失败: {}", module.name.as_deref().unwrap_or("unknown"), e));
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
            if config.maven_home.is_some() { "maven" } else { "npm" }
        });

        match build_tool {
            "maven" => run_maven_build(&build_path, config, emit).await?,
            "npm" | "pnpm" | "yarn" => run_npm_build(&build_path, config, build_tool, emit).await?,
            "gradle" => run_gradle_build(&build_path, emit).await?,
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
        emit("build", "starting", &format!("执行构建命令: {}", cmd));

        let mut child_cmd = user_shell_cmd("sh");
        child_cmd.arg("-c").arg(cmd).current_dir(&build_path)
            .stdout(Stdio::piped()).stderr(Stdio::piped());

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

        let mut child = child_cmd.spawn()
            .map_err(|e| format!("构建命令启动失败: {}", e))?;

        let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
        let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

        let stdout_fut = async {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() { emit("build", "building", trimmed); }
                line.clear();
            }
        };
        let stderr_fut = async {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
                let trimmed = line.trim_end();
                if !trimmed.is_empty() { emit("build", "building", trimmed); }
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

    let tool = module.build_tool.as_deref()
        .or(config.build_tool.as_deref())
        .unwrap_or("npm");

    match tool {
        "maven" => run_maven_build(&build_path, config, emit).await,
        "npm" | "pnpm" | "yarn" => run_npm_build(&build_path, config, tool, emit).await,
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
        args.push("-s");
        args.push(settings);
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

    let stdout = child.stdout.take()
        .ok_or("无法获取 Maven stdout")?;
    let stderr = child.stderr.take()
        .ok_or("无法获取 Maven stderr")?;

    // Stream stdout + stderr concurrently
    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() { emit("maven", "building", trimmed); }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() { emit("maven", "building", trimmed); }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 Maven 进程失败: {}", e))?;

    if !status.success() {
        return Err(format!("Maven 构建失败 (exit {})", status.code().unwrap_or(-1)));
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
    let script = config.npm_custom_script.as_deref()
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

    let stdout = child.stdout.take()
        .ok_or("无法获取 stdout")?;
    let stderr = child.stderr.take()
        .ok_or("无法获取 stderr")?;

    // Stream stdout + stderr concurrently
    let stdout_fut = async {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() { emit("npm", "building", trimmed); }
            line.clear();
        }
    };
    let stderr_fut = async {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await.map(|n| n > 0).unwrap_or(false) {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() { emit("npm", "building", trimmed); }
            line.clear();
        }
    };
    let status_fut = child.wait();
    let (_, _, status) = tokio::join!(stdout_fut, stderr_fut, status_fut);
    let status = status.map_err(|e| format!("等待 {} 进程失败: {}", tool, e))?;

    if !status.success() {
        return Err(format!("{} 构建失败 (exit {})", tool, status.code().unwrap_or(-1)));
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

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Gradle 构建启动失败: {}", e))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let last_lines = get_last_lines(&stdout, 10);
        return Err(format!("Gradle 构建失败 (exit {})\n最近 10 行输出:\n{}", output.status.code().unwrap_or(-1), last_lines));
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

    if let Ok(current_path) = std::env::var("PATH") {
        extra_paths.push(current_path);
    }

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

    if let Ok(current_path) = std::env::var("PATH") {
        extra_paths.push(current_path);
    }

    cmd.env("PATH", extra_paths.join(":"));
}

fn get_last_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = if lines.len() > n { lines.len() - n } else { 0 };
    lines[start..].join("\n")
}

// =================== Artifact Collection ===================

fn collect_artifacts(
    project_path: &PathBuf,
    config: &DeployConfig,
) -> Result<Vec<Artifact>, String> {
    let mut artifacts = Vec::new();

    if config.modules.is_empty() {
        // Single project
        let output_dir = if let Some(ref bp) = config.build_path {
            project_path.join(bp).join("target")
        } else {
            project_path.join("target")
        };

        if output_dir.exists() {
            collect_from_dir(&output_dir, None, &config.deploy_dir, config.lib_separate, None, &mut artifacts)?;
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
                            deploy_path: module.deploy_path.clone()
                                .filter(|s| !s.is_empty())
                                .or(Some(config.deploy_dir.clone())),
                        });
                    }
                    // Collect lib/ directory if lib_separate is enabled
                    if config.lib_separate {
                        let lib_dir = output_dir.join("lib");
                        if lib_dir.exists() && lib_dir.is_dir() {
                            let lib_name = format!("{}-lib.zip", module.name.as_deref().unwrap_or("main"));
                            let lib_zip = output_dir.join(&lib_name);
                            if !lib_zip.exists() {
                                create_zip(&lib_dir, &lib_zip, module.lib_filter_rules.as_deref(), true).map_err(|e| {
                                    format!("压缩 lib 目录失败: {}", e)
                                })?;
                            }
                            artifacts.push(Artifact {
                                name: lib_name,
                                local_path: lib_zip.to_string_lossy().to_string(),
                                module: module.name.clone(),
                                is_lib: true,
                                is_compressed: true,
                                deploy_path: module.deploy_path.clone()
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
                    create_zip(&output_dir, &zip_path, None, false).map_err(|e| {
                        format!("压缩产物目录失败: {}", e)
                    })?;
                }
                artifacts.push(Artifact {
                    name: zip_name,
                    local_path: zip_path.to_string_lossy().to_string(),
                    module: module.name.clone(),
                    is_lib: false,
                    is_compressed: true,
                    deploy_path: module.deploy_path.clone()
                        .filter(|s| !s.is_empty())
                        .or(Some(config.deploy_dir.clone())),
                });
                continue;
            }

            collect_from_dir(&output_dir, module.name.as_deref(), module.deploy_path.as_deref().unwrap_or(&config.deploy_dir), config.lib_separate, module.lib_filter_rules.as_deref(), &mut artifacts)?;
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
    let entries = fs::read_dir(output_dir)
        .map_err(|e| format!("读取产物目录失败: {}", e))?;

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

            // Create zip of lib directory
            create_zip(&lib_dir, &zip_path, lib_filter, true).map_err(|e| {
                format!("压缩 lib 目录失败: {}", e)
            })?;

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

fn create_zip(src_dir: &Path, dest_zip: &Path, filter: Option<&str>, junk_paths: bool) -> Result<(), String> {
    // Use shell: cd src_dir && find . -name "filter" | zip dest_zip -@
    let zip_flag = if junk_paths { "-j" } else { "" };
    let output = if let Some(pattern) = filter {
        std::process::Command::new("sh")
            .args(["-c", &format!(
                "cd '{}' && find . -name '{}' -type f -maxdepth 1 | zip {} '{}' -@",
                src_dir.display(),
                pattern,
                zip_flag,
                dest_zip.display()
            )])
            .output()
            .map_err(|e| format!("zip 命令失败: {}", e))?
    } else {
        let mut cmd = std::process::Command::new("zip");
        cmd.arg("-r");
        if junk_paths { cmd.arg("-j"); }
        cmd.arg(dest_zip)
            .arg(".")
            .current_dir(src_dir);
        cmd.output()
            .map_err(|e| format!("zip 命令失败: {}", e))?
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("zip 失败: {}", err.trim()));
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
    sess.handshake().map_err(|e| format!("SSH 握手失败: {}", e))?;

    // Authenticate
    if let Some(ref key) = srv.private_key {
        sess.userauth_pubkey_file(
            &srv.username,
            None,
            Path::new(key),
            srv.password.as_deref(),
        ).map_err(|e| format!("SSH 密钥认证失败: {}", e))?;
    } else if let Some(ref pw) = srv.password {
        sess.userauth_password(&srv.username, pw)
            .map_err(|e| format!("SSH 密码认证失败: {}", e))?;
    } else {
        return Err("缺少认证信息".to_string());
    }

    // Create deploy directory
    let mkdir_cmd = format!("mkdir -p {}", shell_escape(&srv.deploy_dir));
    ssh_exec(&sess, &mkdir_cmd)?;

    if config.lib_separate {
        if let Some(ref lib_dir) = config.lib_dir {
            let cmd = format!("mkdir -p {}", shell_escape(lib_dir));
            ssh_exec(&sess, &cmd)?;
        }
    }

    // Create module-specific deploy paths
    for artifact in artifacts {
        if let Some(ref dp) = artifact.deploy_path {
            if dp != &config.deploy_dir {
                let cmd = format!("mkdir -p {}", shell_escape(dp));
                ssh_exec(&sess, &cmd)?;
            }
        }
    }

    // Upload via SFTP
    emit("ssh", "uploading", &format!("上传 {} 个产物", artifacts.len()));

    let mut sftp = sess.sftp().map_err(|e| format!("SFTP 初始化失败: {}", e))?;

    for artifact in artifacts {
        let target_path = if let Some(ref dp) = artifact.deploy_path {
            if artifact.is_lib {
                format!("{}/lib", dp)
            } else {
                dp.clone()
            }
        } else if artifact.is_lib && config.lib_separate {
            config.lib_dir.clone().unwrap_or_else(|| config.deploy_dir.clone())
        } else {
            config.deploy_dir.clone()
        };

        let remote_file = format!("{}/{}", target_path.trim_end_matches('/'), artifact.name);

        emit("ssh", "uploading", &format!("↑ {} ({})", artifact.name, file_size_display(&artifact.local_path)));

        upload_file(&mut sftp, &artifact.local_path, &remote_file)
            .map_err(|e| format!("上传 {} 失败: {}", artifact.name, e))?;

        emit("ssh", "success", &format!("✅ {} 上传完成 → {}", artifact.name, target_path));

        // Extract compressed artifacts
        if artifact.is_compressed {
            emit("ssh", "info", &format!("解压 {} → {}", artifact.name, target_path));
            let extract_cmd = format!(
                "cd {} && unzip -o {} && rm -f {}",
                shell_escape(&target_path),
                shell_escape(&artifact.name),
                shell_escape(&artifact.name)
            );
            ssh_exec(&sess, &extract_cmd)?;
            emit("ssh", "success", &format!("✅ {} 解压完成 → {}", artifact.name, target_path));
        }
    }

    drop(sftp);
    sess.disconnect(None, "", None).ok();

    Ok(())
}

fn upload_file(
    sftp: &mut ssh2::Sftp,
    local_path: &str,
    remote_path: &str,
) -> Result<(), String> {
    let mut local_file = fs::File::open(local_path)
        .map_err(|e| format!("打开本地文件失败: {}", e))?;

    let mut remote_file = sftp.create(Path::new(remote_path))
        .map_err(|e| format!("创建远程文件失败: {}", e))?;

    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
    loop {
        let n = local_file.read(&mut buf).map_err(|e| format!("读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        remote_file.write_all(&buf[..n]).map_err(|e| format!("写入失败: {}", e))?;
    }

    Ok(())
}

fn ssh_exec(sess: &ssh2::Session, cmd: &str) -> Result<String, String> {
    let mut channel = sess.channel_session().map_err(|e| format!("SSH channel 创建失败: {}", e))?;
    // 使用 bash -l -c 加载用户环境变量（对齐 Electron 版）
    let login_cmd = format!("bash -l -c {}", shell_escape(cmd));
    channel.exec(&login_cmd).map_err(|e| format!("SSH exec 失败: {}", e))?;

    let mut output = String::new();
    channel.read_to_string(&mut output).ok();
    channel.wait_close().ok();

    let exit_status = channel.exit_status().unwrap_or(-1);
    if exit_status != 0 {
        return Err(format!("SSH 命令失败 (exit {}): cmd={}\noutput={}", exit_status, cmd, output.trim().chars().take(500).collect::<String>()));
    }

    Ok(output)
}

// =================== Restart ===================

async fn execute_restart(
    srv: &DeployServerConfig,
    script: &str,
    emit: &impl Fn(&str, &str, &str),
) -> Result<(), String> {
    use ssh2::Session;
    use std::net::TcpStream;

    let label = srv.label.as_deref().unwrap_or("服务器");
    emit("restart", "starting", &format!("在 {} ({}) 执行重启脚本", label, srv.host));

    let tcp = TcpStream::connect(format!("{}:{}", srv.host, srv.port))
        .map_err(|e| format!("连接失败: {}", e))?;

    let mut sess = Session::new().map_err(|e| format!("SSH session 失败: {}", e))?;
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| format!("SSH 握手失败: {}", e))?;

    if let Some(ref key) = srv.private_key {
        sess.userauth_pubkey_file(&srv.username, None, Path::new(key), srv.password.as_deref())
            .map_err(|e| format!("认证失败: {}", e))?;
    } else if let Some(ref pw) = srv.password {
        sess.userauth_password(&srv.username, pw)
            .map_err(|e| format!("认证失败: {}", e))?;
    }

    ssh_exec(&sess, script)?;
    emit("restart", "success", &format!("{} 重启完成", label));

    sess.disconnect(None, "", None).ok();
    Ok(())
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
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
