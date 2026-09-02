use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use supertool_core::db::cicd::*;
use supertool_core::logic::CoreService;
use supertool_core::logic::cicd_deploy::{
    self, DeployConfig, DeployModuleConfig, DeployServerConfig,
};
// 从 core 复用类型和工具函数，避免重复定义
use supertool_core::logic::cicd_tools::{ToolDetectionResult, ToolPaths, run_command};
use tauri::{Emitter, State};

// 部署取消状态管理：cancel_deploy 将 deploy_id 加入此集合，deploy 任务检查后提前退出
static CANCELLED_DEPLOYS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn is_deploy_cancelled(deploy_id: &str) -> bool {
    CANCELLED_DEPLOYS
        .lock()
        .map(|set| set.contains(deploy_id))
        .unwrap_or(false)
}

// 部署队列：同一配置同时只允许一个部署执行，后续请求排队等待（防止并发部署互相覆盖）
static DEPLOY_QUEUES: LazyLock<Mutex<HashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_config_deploy_lock(config_id: &str) -> std::sync::Arc<tokio::sync::Mutex<()>> {
    let mut map = DEPLOY_QUEUES.lock().unwrap();
    map.entry(config_id.to_string()).or_default().clone()
}

// =================== 部署进度事件合并（防主线程卡死） ===================
// 构建期 maven/npm 的 stdout 是逐行输出，实测峰值 700 行/秒（uni-app 构建见过 10000 行/秒，
// 单次部署 2 万+ 行）。Tauri 每次 emit 都要在 macOS 主线程 runloop 上做一次 webview eval
// （sample 抓栈可见 WebKitWebPage evaluateJavaScript_ + WKCommandDecodeEncodeSizes），
// 事件风暴会占满主线程 → 整个窗口点击无响应，直到部署结束才恢复（其他 app 不受影响）。
// 前端 50ms 批量只压住 Vue 重渲染，压不住事件投递本身，因此必须在后端合并：
// 高频日志行（building/installing）进缓冲，按「最小间隔 + 单批行数上限」发一个 batch 事件；
// 状态类事件（连接/上传/成功/失败/队列）先冲缓冲再立即发送，保证日志顺序。
/// 两次 batch 之间的最小间隔：把主线程 eval 次数压到 ~5 次/秒
const PROGRESS_BATCH_INTERVAL_MS: u64 = 200;
/// 单批最多保留的行数，超出丢弃最旧行并在批次头部标注
/// （实时面板本来就是尾随滚动，全量内容始终写入部署日志文件）
const PROGRESS_BATCH_MAX_BUFFER: usize = 200;
/// 推送给前端的单行最大长度（避免超长路径行撑爆 IPC）
const PROGRESS_LINE_MAX_CHARS: usize = 400;

/// 报错行不参与攒批：构建失败时用户要靠实时日志看原因，既不能延迟也不能被裁剪掉
fn looks_like_error(message: &str) -> bool {
    let upper = message.to_ascii_uppercase();
    upper.contains("[ERROR]")
        || upper.contains("ERROR:")
        || upper.contains("ERR!")
        || upper.contains("ERROR ")
        || upper.contains("BUILD FAILURE")
        || upper.contains("FAILED")
        || upper.contains("FATAL")
        || message.contains("异常")
}

/// 高频逐行输出（构建 / 依赖安装）才需要攒批；状态类事件一律立即发送
/// （ssh 上传进度自带 5% 阈值节流，无需批量）
fn is_noisy_progress(status: &str, message: &str) -> bool {
    matches!(status, "building" | "installing") && !looks_like_error(message)
}

fn clip_progress_line(msg: &str) -> String {
    match msg.chars().count() > PROGRESS_LINE_MAX_CHARS {
        true => {
            let mut s: String = msg.chars().take(PROGRESS_LINE_MAX_CHARS).collect();
            s.push('…');
            s
        }
        false => msg.to_string(),
    }
}

#[derive(Default)]
struct DeployProgressBatcher {
    lines: Vec<serde_json::Value>,
    dropped: usize,
    last_flush: Option<std::time::Instant>,
    progress: Option<i64>,
}

impl DeployProgressBatcher {
    /// 追加一行高频日志；到达最小发送间隔时返回可发送的 batch payload
    fn push(&mut self, line: serde_json::Value, progress: Option<i64>) -> Option<serde_json::Value> {
        if progress.is_some() {
            self.progress = progress;
        }
        self.lines.push(line);
        // 超出单批上限丢弃最旧行（部署日志文件仍是全量）
        if self.lines.len() > PROGRESS_BATCH_MAX_BUFFER {
            let excess = self.lines.len() - PROGRESS_BATCH_MAX_BUFFER;
            self.lines.drain(..excess);
            self.dropped += excess;
        }
        let due = self
            .last_flush
            .map(|t| t.elapsed().as_millis() >= PROGRESS_BATCH_INTERVAL_MS as u128)
            .unwrap_or(true);
        if due {
            self.drain()
        } else {
            None
        }
    }

    /// 取走缓冲并组成一个 batch 事件 payload（空缓冲返回 None）
    fn drain(&mut self) -> Option<serde_json::Value> {
        self.last_flush = Some(std::time::Instant::now());
        if self.lines.is_empty() {
            return None;
        }
        let mut lines = std::mem::take(&mut self.lines);
        let dropped = std::mem::take(&mut self.dropped);
        // 进度只带本批次内出现过的值，避免旧进度被后续批次重复回放
        let progress = self.progress.take();
        if dropped > 0 {
            lines.insert(
                0,
                serde_json::json!({
                    "stage": "info",
                    "status": "info",
                    "message": format!("… 输出过快，已省略 {} 行（完整内容见部署日志）", dropped),
                }),
            );
        }
        let message = lines
            .last()
            .and_then(|l| l.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or_default()
            .to_string();
        Some(serde_json::json!({
            "stage": "batch",
            "status": "lines",
            "message": message,
            "progress": progress,
            "lines": lines,
        }))
    }
}

fn emit_deploy_progress(
    app: &tauri::AppHandle,
    deploy_log_id: &str,
    config_id: &str,
    fields: serde_json::Value,
) {
    let mut payload = match fields {
        serde_json::Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("message".into(), other);
            map
        }
    };
    payload.insert("deployLogId".into(), serde_json::json!(deploy_log_id));
    payload.insert("configId".into(), serde_json::json!(config_id));
    let _ = app.emit("deploy-progress", serde_json::Value::Object(payload));
}

/// 部署任务结束（含 panic 回卷）时置位停止标志，让进度兜底定时器自行退出
struct TickerStopGuard(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for TickerStopGuard {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

// =================== 多环境配置类型（cicd_configs.environments JSON） ===================

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvServerRef {
    pub server_id: String,
    #[serde(default)]
    pub deploy_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvEntry {
    pub name: String,
    /// 环境部署路径（空则沿用配置级 deployPath）
    #[serde(default)]
    pub deploy_path: String,
    /// 环境专属服务器列表（空则沿用配置级服务器）
    #[serde(default)]
    pub servers: Vec<EnvServerRef>,
    /// 构建时注入的环境变量（如 NODE_ENV=production）
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub health_check_url: Option<String>,
    #[serde(default)]
    pub health_check_timeout: Option<u64>,
    #[serde(default)]
    pub health_check_retries: Option<u32>,
}

/// 解析配置的多环境列表
pub fn parse_environments(environments: Option<&str>) -> Vec<EnvEntry> {
    environments
        .and_then(|s| serde_json::from_str::<Vec<EnvEntry>>(s).ok())
        .unwrap_or_default()
}

// =================== Types ===================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectScanResult {
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "currentBranch")]
    pub current_branch: Option<String>,
    #[serde(rename = "gitRemoteUrl")]
    pub git_remote_url: Option<String>,
    #[serde(rename = "npmScripts")]
    pub npm_scripts: Option<Vec<String>>,
    #[serde(rename = "recommendedScript")]
    pub recommended_script: Option<String>,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    #[serde(rename = "mavenProfiles")]
    pub maven_profiles: Option<Vec<String>>,
    #[serde(rename = "recommendedProfile")]
    pub recommended_profile: Option<String>,
    #[serde(rename = "isMultiModule")]
    pub is_multi_module: Option<bool>,
    #[serde(rename = "moduleNames")]
    pub module_names: Option<Vec<String>>,
    #[serde(rename = "hasParent")]
    pub has_parent: Option<bool>,
    #[serde(rename = "suggestedDeployPath")]
    pub suggested_deploy_path: Option<String>,
}

// =================== Tool Detection Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub fn detect_tool_paths() -> ToolPaths {
    log::info!("[Tauri CMD] detect_tool_paths() called");
    detect_tool_paths_impl()
}

#[tauri::command(rename_all = "camelCase")]
pub fn detect_build_tools() -> HashMap<String, ToolDetectionResult> {
    log::info!("[Tauri CMD] detect_build_tools() called");
    detect_tools_impl()
}

#[tauri::command(rename_all = "camelCase")]
pub fn scan_project(local_path: String) -> ProjectScanResult {
    log::info!("[Tauri CMD] scan_project() called");
    scan_project_impl(&local_path)
}

// =================== SDK Version Check Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub fn detect_sdk_versions() -> serde_json::Value {
    log::info!("[Tauri CMD] detect_sdk_versions() called");
    supertool_core::logic::cicd_tools::detect_sdk_versions_impl()
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_java(java_home: Option<String>) -> ToolDetectionResult {
    log::info!("[Tauri CMD] check_java() called, javaHome={:?}", java_home);
    supertool_core::logic::cicd_tools::check_java(java_home)
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_maven(maven_home: Option<String>) -> ToolDetectionResult {
    log::info!("[Tauri CMD] check_maven() called, mavenHome={:?}", maven_home);
    supertool_core::logic::cicd_tools::check_maven(maven_home)
}

#[tauri::command(rename_all = "camelCase")]
pub fn check_node(node_home: Option<String>) -> ToolDetectionResult {
    log::info!("[Tauri CMD] check_node() called, nodeHome={:?}", node_home);
    supertool_core::logic::cicd_tools::check_node(node_home)
}

// =================== Implementation Functions ===================

pub fn detect_tools_impl() -> HashMap<String, ToolDetectionResult> {
    supertool_core::logic::cicd_tools::detect_tools_impl()
}

pub fn detect_tool_paths_impl() -> ToolPaths {
    supertool_core::logic::cicd_tools::detect_tool_paths_impl()
}

pub fn scan_project_impl(local_path: &str) -> ProjectScanResult {
    let mut result = ProjectScanResult {
        build_tool: None,
        current_branch: None,
        git_remote_url: None,
        npm_scripts: None,
        recommended_script: None,
        project_name: None,
        package_manager: None,
        maven_profiles: None,
        recommended_profile: None,
        is_multi_module: None,
        module_names: None,
        has_parent: None,
        suggested_deploy_path: None,
    };
    if !Path::new(local_path).exists() {
        return result;
    }
    let has_pom = Path::new(local_path).join("pom.xml").exists();
    let has_package = Path::new(local_path).join("package.json").exists();
    if has_pom {
        result.build_tool = Some("maven".to_string());
    } else if has_package {
        result.build_tool = Some("npm".to_string());
    }
    // Git branch
    let branch_result = run_command("git rev-parse --abbrev-ref HEAD", Some(local_path));
    if branch_result.success {
        let branch = branch_result.stdout.trim();
        if !branch.is_empty() && branch != "HEAD" {
            result.current_branch = Some(branch.to_string());
        }
    }
    // Git remote URL
    let remote_result = run_command("git remote get-url origin", Some(local_path));
    if remote_result.success {
        let remote = remote_result.stdout.trim();
        if !remote.is_empty() {
            result.git_remote_url = Some(remote.to_string());
        }
    }
    // Parse package.json
    if has_package {
        if let Ok(content) = fs::read_to_string(Path::new(local_path).join("package.json")) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                    let script_names: Vec<String> = scripts.keys().cloned().collect();
                    result.npm_scripts = Some(script_names.clone());
                    if scripts.contains_key("build") {
                        result.recommended_script = Some("build".to_string());
                    } else if scripts.contains_key("prod") {
                        result.recommended_script = Some("prod".to_string());
                    } else if scripts.contains_key("start") {
                        result.recommended_script = Some("start".to_string());
                    }
                }
                if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                    result.project_name = Some(name.to_string());
                }
                if Path::new(local_path).join("pnpm-lock.yaml").exists() {
                    result.package_manager = Some("pnpm".to_string());
                } else if Path::new(local_path).join("yarn.lock").exists() {
                    result.package_manager = Some("yarn".to_string());
                } else if Path::new(local_path).join("package-lock.json").exists() {
                    result.package_manager = Some("npm".to_string());
                }
            }
        }
    }
    // Parse pom.xml
    if has_pom {
        if let Ok(pom) = fs::read_to_string(Path::new(local_path).join("pom.xml")) {
            if let Some(cap) = regex::Regex::new(r"<artifactId>([^<]+)</artifactId>")
                .ok()
                .and_then(|re| re.captures(&pom))
            {
                result.project_name = Some(cap[1].to_string());
            }
            if let Ok(re) = regex::Regex::new(r"<profile>\s*<id>([^<]+)</id>") {
                let profiles: Vec<String> = re
                    .captures_iter(&pom)
                    .map(|cap| cap[1].to_string())
                    .collect();
                if !profiles.is_empty() {
                    let recommended = if profiles.contains(&"prod".to_string()) {
                        "prod".to_string()
                    } else if profiles.contains(&"production".to_string()) {
                        "production".to_string()
                    } else {
                        profiles[0].clone()
                    };
                    result.maven_profiles = Some(profiles);
                    result.recommended_profile = Some(recommended);
                }
            }
            if let Ok(re) = regex::Regex::new(r"<modules>\s*([\s\S]*?)</modules>") {
                if let Some(cap) = re.captures(&pom) {
                    let module_re = regex::Regex::new(r"<module>\s*([^<]+?)\s*</module>").unwrap();
                    let modules: Vec<String> = module_re
                        .captures_iter(&cap[1])
                        .map(|c| c[1].trim().to_string())
                        .filter(|m| !m.is_empty())
                        .collect();
                    if modules.len() > 1 {
                        result.is_multi_module = Some(true);
                        // 只保留 Spring Boot 可部署模块（含 @SpringBootApplication 启动类）；
                        // 纯依赖模块（framework/common 等）不作为部署单元
                        let deployable: Vec<String> = modules
                            .iter()
                            .filter(|m| {
                                supertool_core::logic::cicd_tools::has_spring_boot_main(
                                    &Path::new(local_path).join(m),
                                )
                            })
                            .cloned()
                            .collect();
                        result.module_names = Some(deployable);
                    }
                }
            }
            if regex::Regex::new(r"<parent>[\s\S]*?</parent>")
                .ok()
                .map(|re| re.is_match(&pom))
                .unwrap_or(false)
            {
                result.has_parent = Some(true);
            }
        }
    }
    if result.build_tool.as_deref() == Some("maven") {
        result.suggested_deploy_path = Some("~/apphome".to_string());
    } else if result.build_tool.as_deref() == Some("npm") {
        result.suggested_deploy_path = Some("/home/nginxWebUI/ui".to_string());
    }
    result
}

// =================== CICD Config CRUD Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_configs(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_configs() called");
    let configs = core.db_read(|conn| cicd_get_all_configs(conn).map_err(|e| e.to_string()))??;
    serde_json::to_value(&configs).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_config_by_id(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_config_by_id() called");
    let config = core.db_read(|conn| cicd_get_config_by_id(conn, &id).map_err(|e| e.to_string()))??;
    match config {
        Some(c) => serde_json::to_value(&c).map_err(|e| e.to_string()),
        None => Ok(serde_json::Value::Null),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cicd_groups(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_cicd_groups() called");
    let groups = core.db_read(|conn| cicd_get_groups(conn).map_err(|e| e.to_string()))??;
    serde_json::to_value(&groups).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_cicd_config(
    core: State<'_, CoreService>,
    config: serde_json::Value,
    modules: Option<Vec<serde_json::Value>>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_cicd_config() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut cicd_config: CicdConfig =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析配置失败: {}", e))?;
    // 保存前兜底清理条件字段（按构建工具/部署模式），防止 UI 未展示的残留值污染后续逻辑
    supertool_core::db::cicd::sanitize_cicd_config_conditional(&mut cicd_config);
    let result = core.db_write(|conn| {
        let existing = match cicd_get_config_by_id(conn, &cicd_config.id) {
            Ok(v) => v,
            Err(e) => return Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())),
        };
        if existing.is_some() {
            cicd_config.updated_at = now.clone();
            if let Err(e) = cicd_update_config(conn, &cicd_config) {
                return Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()));
            }
        } else {
            cicd_config.created_at = now.clone();
            cicd_config.updated_at = now.clone();
            if let Err(e) = cicd_add_config(conn, &cicd_config) {
                return Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()));
            }
        }
        // Handle modules
        if let Some(mods) = modules {
            if let Err(e) = conn.execute(
                "DELETE FROM deploy_modules WHERE configId = ?",
                [&cicd_config.id],
            ) {
                return Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()));
            }
            for m in &mods {
                let mut module: DeployModule = match serde_json::from_value(m.clone()) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("解析模块失败: {}", e)),
                };
                // 模块行条件字段兜底清理（与配置级同规则：maven 模块隐藏构建目录/构建命令等）
                let snap =
                    supertool_core::db::cicd::sanitize_snapshot(&cicd_config);
                supertool_core::db::cicd::sanitize_deploy_module_from_snapshot(&mut module, &snap);
                module.config_id = cicd_config.id.clone();
                module.created_at = now.clone();
                module.updated_at = now.clone();
                if let Err(e) = cicd_add_module(conn, &module) {
                    return Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()));
                }
            }
        }
        match cicd_get_config_by_id(conn, &cicd_config.id) {
            Ok(v) => Ok(v),
            Err(e) => Err(supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string())),
        }
    });
    match result {
        Ok(Ok(Some(c))) => serde_json::to_value(&c).map_err(|e| e.to_string()),
        Ok(Ok(None)) => Err("保存配置失败".to_string()),
        Ok(Err(e)) | Err(e) => Err(e),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_cicd_config(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_cicd_config() called");
    match core.db_write(|conn| -> Result<serde_json::Value, String> {
        conn.execute("DELETE FROM deploy_modules WHERE configId = ?", [&id])
            .map_err(|e| supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()))?;
        cicd_delete_config(conn, &id)
            .map_err(|e| supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()))?;
        Ok(serde_json::json!({ "id": id }))
    }) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) | Err(e) => Err(e),
    }
}

// =================== Deploy Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn deploy(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    config_id: String,
    confirmed: Option<bool>,
    branch: Option<String>,
    environment: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] deploy() called");
    // Get config from DB
    let cicd_config = core
        .db_read(|conn| cicd_get_config_by_id(conn, &config_id).map_err(|e| e.to_string()))??
        .ok_or("CI/CD 配置不存在")?;

    // Check approval requirement
    if cicd_config.requires_approval && confirmed != Some(true) {
        return Ok(serde_json::json!({
            "success": false,
            "requiresApproval": true,
            "message": "此配置需要审核确认，请确认后再次部署",
            "configName": cicd_config.name
        }));
    }

    let modules =
        core.db_read(|conn| cicd_get_modules(conn, &config_id).map_err(|e| e.to_string()))??;

    // Build DeployConfig
    let mut deploy_config = build_deploy_config(&core, &cicd_config, &modules)?;

    // Override branch if provided (per-deploy branch selection)
    if let Some(ref b) = branch {
        if !b.is_empty() {
            log::info!("[deploy] overriding branch: {} -> {}", deploy_config.branch, b);
            deploy_config.branch = b.clone();
        }
    }

    // 多环境覆盖：按环境名应用专属的部署路径 / 服务器 / 环境变量 / 健康检查配置
    if let Some(ref env_name) = environment.clone().filter(|s| !s.is_empty()) {
        let envs = parse_environments(cicd_config.environments.as_deref());
        let env = envs
            .into_iter()
            .find(|e| &e.name == env_name)
            .ok_or_else(|| format!("环境「{}」不存在，请检查配置", env_name))?;
        let lib_separate =
            cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven");

        if !env.servers.is_empty() {
            let fallback_dir = if env.deploy_path.is_empty() {
                cicd_config.deploy_path.clone()
            } else {
                env.deploy_path.clone()
            };
            let refs: Vec<(String, String)> = env
                .servers
                .iter()
                .map(|r| (r.server_id.clone(), r.deploy_dir.clone()))
                .collect();
            deploy_config.servers = resolve_deploy_servers(&core, &refs, &fallback_dir, lib_separate)?;
        } else if !env.deploy_path.is_empty() {
            // 沿用配置级服务器：未单独指定部署目录的节点切换到环境路径
            let old_dir = cicd_config.deploy_path.clone();
            let new_dir = env.deploy_path.clone();
            for srv in deploy_config.servers.iter_mut() {
                if srv.deploy_dir == old_dir {
                    srv.deploy_dir = new_dir.clone();
                    if let Some(ref mut ld) = srv.lib_dir {
                        *ld = format!("{}/lib", new_dir);
                    }
                }
            }
        }
        if !env.deploy_path.is_empty() {
            deploy_config.deploy_dir = env.deploy_path.clone();
            if lib_separate {
                deploy_config.lib_dir = Some(format!("{}/lib", env.deploy_path));
            }
        }
        deploy_config.env_vars = env.env_vars;
        if let Some(ref u) = env.health_check_url.filter(|u| !u.is_empty()) {
            deploy_config.health_check_url = Some(u.clone());
        }
        if let Some(t) = env.health_check_timeout {
            deploy_config.health_check_timeout = t;
        }
        if let Some(r) = env.health_check_retries {
            deploy_config.health_check_retries = r;
        }
        deploy_config.environment_name = Some(env.name.clone());
        log::info!("[deploy] environment override applied: {}", env.name);
    }

    // Create deploy log
    let deploy_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let deploy_log = DeployLog {
        id: deploy_id.clone(),
        config_id: config_id.clone(),
        status: "running".to_string(),
        start_time: now.clone(),
        end_time: None,
        error_message: None,
        progress: 0,
        triggered_by: "user".to_string(),
        created_at: now.clone(),
        log_file_path: None,
        artifact_paths: None,
        environment: environment.clone().filter(|s| !s.is_empty()),
    };

    // Save deploy log
    core.db_write(|conn| -> Result<(), String> {
        cicd_add_deploy_log(conn, &deploy_log).map_err(|e| e.to_string())?;
        cicd_touch_deploy(conn, &config_id).map_err(|e| e.to_string())?;
        Ok(())
    })??;

    // Get app dir for deploy logs
    let app_dir = core.db_read(|conn| {
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'appDataDir'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .unwrap_or_else(|| "/tmp".to_string())
    })?;

    // Emit deploy-log-id-created event immediately
    let _ = app.emit(
        "deploy-log-id-created",
        serde_json::json!({ "deployLogId": deploy_id }),
    );

    let deploy_id_arc = std::sync::Arc::new(deploy_id.clone());
    let config_id_arc = std::sync::Arc::new(config_id.clone());
    let app_arc = std::sync::Arc::new(app.clone());
    let core_clone = core.inner().clone();

    let did_for_closure = deploy_id_arc.clone();
    let cid_for_closure = config_id_arc.clone();
    let app_for_closure = app_arc.clone();

    // Spawn background task for deploy
    tokio::spawn(async move {
        let did_for_cancel = deploy_id_arc.clone();

        // 部署队列：同配置并发部署排队执行，防止产物互相覆盖
        let queue_lock = get_config_deploy_lock(&config_id_arc);
        let _guard = match queue_lock.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                let _ = app_for_closure.emit(
                    "deploy-progress",
                    serde_json::json!({
                        "deployLogId": *did_for_closure,
                        "configId": *cid_for_closure,
                        "stage": "queue",
                        "status": "waiting",
                        "message": "当前配置有部署正在进行，本任务排队等待中...",
                        "progress": None::<i64>,
                    }),
                );
                let guard = queue_lock.lock().await;
                let _ = app_for_closure.emit(
                    "deploy-progress",
                    serde_json::json!({
                        "deployLogId": *did_for_closure,
                        "configId": *cid_for_closure,
                        "stage": "queue",
                        "status": "acquired",
                        "message": "排队结束，开始部署",
                        "progress": None::<i64>,
                    }),
                );
                guard
            }
        };

        let batcher: std::sync::Arc<Mutex<DeployProgressBatcher>> =
            std::sync::Arc::new(Mutex::new(DeployProgressBatcher::default()));
        let batcher_for_closure = batcher.clone();

        // 兜底定时器：构建输出暂停（长时间无新行）时也要把缓冲行发出去，避免日志延迟到下一阶段才显示。
        // 用停止标志协作退出而非 abort：abort 可能正好落在「已取走缓冲、尚未发出」之间丢尾部批次，
        // panic 回卷时也需要靠 guard 停掉，否则定时器永久泄漏。
        let ticker_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let _ticker_guard = TickerStopGuard(ticker_stop.clone());
        let _ticker = {
            let batcher_for_ticker = batcher.clone();
            let app_for_ticker = app_arc.clone();
            let did_for_ticker = deploy_id_arc.clone();
            let cid_for_ticker = config_id_arc.clone();
            let stop_for_ticker = ticker_stop.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        PROGRESS_BATCH_INTERVAL_MS + 50,
                    ))
                    .await;
                    if stop_for_ticker.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    // 持锁发送：与闭包/最终冲刷共用一把锁，保证事件严格 FIFO
                    let mut b = match batcher_for_ticker.lock() {
                        Ok(g) => g,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Some(fields) = b.drain() {
                        emit_deploy_progress(
                            &app_for_ticker,
                            &did_for_ticker,
                            &cid_for_ticker,
                            fields,
                        );
                    }
                }
            })
        };

        let deploy_result = cicd_deploy::execute_deploy(
            &deploy_config,
            &app_dir,
            &deploy_id_arc,
            move |event| {
                // 全程持锁发送：批次与状态事件、兜底定时器共用一把锁 → 严格 FIFO
                let mut b = match batcher_for_closure.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if is_noisy_progress(&event.status, &event.message) {
                    if let Some(fields) = b.push(
                        serde_json::json!({
                            "stage": event.stage,
                            "status": event.status,
                            "message": clip_progress_line(&event.message),
                        }),
                        event.progress,
                    ) {
                        emit_deploy_progress(
                            &app_for_closure,
                            &did_for_closure,
                            &cid_for_closure,
                            fields,
                        );
                    }
                } else {
                    // 状态事件：先冲缓冲保持顺序，再立即发送（进度条/阶段提示不受批量影响）
                    b.progress = event.progress.or(b.progress);
                    if let Some(fields) = b.drain() {
                        emit_deploy_progress(
                            &app_for_closure,
                            &did_for_closure,
                            &cid_for_closure,
                            fields,
                        );
                    }
                    emit_deploy_progress(
                        &app_for_closure,
                        &did_for_closure,
                        &cid_for_closure,
                        serde_json::json!({
                            "stage": event.stage,
                            "status": event.status,
                            "message": event.message,
                            "progress": event.progress,
                        }),
                    );
                }
            },
            move || is_deploy_cancelled(&did_for_cancel),
        )
        .await;

        // 停止兜底定时器并冲掉最后残留，确保尾部日志不丢
        // （无需 await/abort：所有 deploy-progress 都在持锁期间发送，最终冲刷天然排在定时器之后）
        ticker_stop.store(true, std::sync::atomic::Ordering::SeqCst);
        {
            let mut b = match batcher.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(fields) = b.drain() {
                emit_deploy_progress(&app_arc, &deploy_id_arc, &config_id_arc, fields);
            }
        }

        // Update deploy log with result
        let final_status: String;
        let final_error: Option<String>;
        let final_log_path: Option<String>;
        let final_artifact_paths: Option<String>;
        let final_progress: i64;

        match &deploy_result {
            Ok(result) => {
                final_status = if result.cancelled == Some(true) {
                    "cancelled".to_string()
                } else if result.success {
                    "success".to_string()
                } else {
                    "failed".to_string()
                };
                final_error = result.error.clone();
                final_progress = if result.success { 100 } else { 0 };
                final_log_path = Some(result.log_file_path.clone());
                final_artifact_paths =
                    Some(serde_json::to_string(&result.artifact_paths).unwrap_or_default());
            }
            Err(e) => {
                final_status = "failed".to_string();
                final_error = Some(e.clone());
                final_progress = 0;
                final_log_path = None;
                final_artifact_paths = None;
            }
        };

        let new_log = DeployLog {
            id: (*deploy_id_arc).clone(),
            config_id: (*config_id_arc).clone(),
            status: final_status,
            start_time: now.clone(),
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            error_message: final_error,
            progress: final_progress,
            triggered_by: "user".to_string(),
            created_at: now.clone(),
            log_file_path: final_log_path,
            artifact_paths: final_artifact_paths,
            environment: environment.clone().filter(|s| !s.is_empty()),
        };
        let _ = core_clone.db_write(|conn| cicd_update_deploy_log(conn, &new_log));

        // deploy_history 表已废弃（2026-08 清理）：部署终态只写 deploy_logs，
        // 前端部署历史/回滚记录统一从 deploy_logs 读取

        // 清理取消标记
        if let Ok(mut set) = CANCELLED_DEPLOYS.lock() {
            set.remove(&*deploy_id_arc);
        }

        // Emit final notification (native system notification + event to frontend)
        let is_cancelled = match &deploy_result {
            Ok(result) => result.cancelled == Some(true),
            Err(_) => false,
        };
        match &deploy_result {
            Ok(result) => {
                crate::tray_notification::show_deploy_notification(
                    result.success && !is_cancelled,
                    &cicd_config.name,
                    result.error.as_deref(),
                );
                let _ = app_arc.emit(
                    "deploy-notification",
                    serde_json::json!({
                        "success": result.success,
                        "cancelled": is_cancelled,
                        "configId": *config_id_arc,
                        "deployLogId": *deploy_id_arc,
                        "error": result.error,
                    }),
                );
            }
            Err(e) => {
                crate::tray_notification::show_deploy_notification(
                    false,
                    &cicd_config.name,
                    Some(e),
                );
                let _ = app_arc.emit(
                    "deploy-notification",
                    serde_json::json!({
                        "success": false,
                        "configId": *config_id_arc,
                        "deployLogId": *deploy_id_arc,
                        "error": e,
                    }),
                );
            }
        }
    });

    // Return deploy ID immediately so frontend can track progress
    Ok(serde_json::json!({
        "deployId": deploy_id,
        "success": true,
        "message": "部署已启动",
        "status": "running",
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn cancel_deploy(
    core: State<'_, CoreService>,
    deploy_log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] cancel_deploy() called, deploy_log_id={}",
        deploy_log_id
    );

    // 标记为已取消，deploy 任务会在下一个检查点退出
    if let Ok(mut set) = CANCELLED_DEPLOYS.lock() {
        set.insert(deploy_log_id.clone());
    }

    let result = core.db_write(|conn| {
        let log = cicd_get_deploy_log_by_id(conn, &deploy_log_id);
        match log {
            Some(mut log) if log.status == "running" || log.status == "pending" => {
                log.status = "cancelled".to_string();
                log.end_time = Some(chrono::Utc::now().to_rfc3339());
                log.error_message = Some("用户取消部署".to_string());
                cicd_update_deploy_log(conn, &log)
                    .map_err(|e| supertool_core::logic::log_sanitizer::sanitize_string(&e.to_string()))?;
                Ok(serde_json::json!({ "success": true, "status": "cancelled" }))
            }
            Some(log) => Ok(serde_json::json!({
                "success": false,
                "error": format!("部署状态为 {}，无法取消", log.status)
            })),
            None => Ok(serde_json::json!({
                "success": false,
                "error": "部署记录不存在"
            })),
        }
    });
    match result {
        Ok(v) => v,
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn rollback(
    core: State<'_, CoreService>,
    config_id: String,
    log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] rollback() called");
    let _deploy_log = core
        .db_read(|conn| cicd_get_deploy_log_by_id(conn, &log_id))?
        .ok_or("部署记录不存在")?;

    // Get CICD config to read server info
    let cicd_config = core
        .db_read(|conn| cicd_get_config_by_id(conn, &config_id).map_err(|e| e.to_string()))??
        .ok_or("CI/CD 配置不存在")?;

    let now = chrono::Utc::now().to_rfc3339();

    // Parse servers from config JSON
    let mut rollback_errors: Vec<String> = Vec::new();
    if let Some(ref servers_str) = cicd_config.servers {
        if let Ok(servers) = serde_json::from_str::<Vec<serde_json::Value>>(servers_str) {
            for server_val in &servers {
                let host_or_id = server_val
                    .get("host")
                    .or_else(|| server_val.get("serverId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if host_or_id.is_empty() {
                    continue;
                }

                // Query full server config from servers table
                let server = match core.db_read(|conn| {
                    conn.query_row(
                        "SELECT * FROM servers WHERE id = ? OR host = ?",
                        rusqlite::params![host_or_id, host_or_id],
                        supertool_core::db::servers::row_to_server,
                    )
                }) {
                    Ok(Ok(s)) => s,
                    _ => {
                        rollback_errors.push(format!("{}: 服务器不存在", host_or_id));
                        continue;
                    }
                };

                let host = server.host.clone();
                let port = server.port as u16;
                let username = server.username.clone();
                let password = server
                    .password
                    .clone()
                    .map(|pw| supertool_core::encryption::try_decrypt_password(&pw));
                let ssh_key = server.ssh_key_path.clone();

                // Execute restart script via SSH（走 core SshService，认证/连接统一在 core）
                match execute_remote_restart(
                    &core,
                    host.clone(),
                    port,
                    username,
                    password,
                    ssh_key,
                    cicd_config.restart_script.clone(),
                )
                .await
                {
                    Ok(_) => log::info!("[rollback] {}:{} restart successful", host, port),
                    Err(e) => {
                        log::error!("[rollback] {}:{} restart failed: {}", host, port, e);
                        rollback_errors.push(format!("{}:{} → {}", host, port, e));
                    }
                }
            }
        } else {
            rollback_errors.push("服务器配置解析失败".to_string());
        }
    } else {
        rollback_errors.push("未配置部署服务器".to_string());
    }

    // Record rollback in deploy_logs（deploy_history 表已废弃，2026-08 清理）：
    // 原终态 status/errorMessage 保留，回滚结果以 "rolled-back:<结果>" 追加到 errorMessage
    let log_write_err = core
        .db_write(|conn| -> Result<(), String> {
            let existing = supertool_core::db::cicd::get_deploy_log_by_id(conn, &log_id)
                .map_err(|e| e.to_string())?
                .ok_or("部署记录不存在")?;
            let mut updated = existing;
            let rollback_mark = if rollback_errors.is_empty() {
                format!("rolled-back:success at {}", now)
            } else {
                format!(
                    "rolled-back:partial ({}) at {}",
                    rollback_errors.join("; "),
                    now
                )
            };
            updated.error_message = match updated.error_message.take() {
                Some(prev) if !prev.is_empty() => Some(format!("{} | {}", prev, rollback_mark)),
                _ => Some(rollback_mark),
            };
            cicd_update_deploy_log(conn, &updated)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .err()
        .or(None);

    Ok(serde_json::json!({
        "success": rollback_errors.is_empty() && log_write_err.is_none(),
        "rollbackId": log_id,
        "message": if rollback_errors.is_empty() {
            match log_write_err {
                Some(e) => format!("回滚已执行但记录更新失败: {}", e),
                None => "回滚成功：已在所有服务器执行重启".to_string(),
            }
        } else {
            format!("部分成功: {}", rollback_errors.join("; "))
        },
        "errors": rollback_errors,
    }))
}

/// Execute a restart command on a remote server via SSH.
///
/// 统一走 core 的 `SshService::exec_commands_independent`（连接/认证/通道/输出收集全在
/// core）。曾在这里手工 new `ssh2::Session` + `TcpStream::connect` + `userauth_*` +
/// `channel.exec`，与 core 重复实现 SSH 全链路；且认证逻辑(密码/密钥优先级)与 core
/// 不一致，易出现「GUI 能连、此处报私钥打不开」的分叉。现已下沉。
async fn execute_remote_restart(
    core: &CoreService,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
    restart_script: String,
) -> Result<(), String> {
    let ssh = core.clone_ssh();
    tokio::task::spawn_blocking(move || {
        // 对齐 core/src/logic/cicd_deploy.rs execute_restart 的修复
        // 使用 bash -l -c 加载用户环境变量（JAVA_HOME 等）
        let exec_cmd = if restart_script.starts_with('/') {
            format!(
                "chmod +x {} && bash -l -c '{}' 2>&1",
                restart_script, restart_script
            )
        } else {
            // 相对路径需要 cd，但 rollback 没有 deployDir，默认用 ~/apphome
            format!(
                "cd ~/apphome && chmod +x {} && bash -l -c '{}' 2>&1",
                restart_script, restart_script
            )
        };

        let config = supertool_core::logic::ssh::SshServerConfig {
            id: format!("{host}:{port}"),
            name: String::new(),
            host: host.clone(),
            port: port as u32,
            username,
            password,
            ssh_key_path: private_key,
        };

        let results = ssh
            .exec_commands_independent(&config, std::slice::from_ref(&exec_cmd))
            .map_err(|e| format!("SSH 执行重启命令失败: {e}"))?;

        let result = results
            .get(&exec_cmd)
            .cloned()
            .unwrap_or(supertool_core::logic::ssh::ExecResult {
                success: false,
                output: String::new(),
                error_output: "未返回结果".to_string(),
                exit_code: None,
            });

        if !result.success {
            let output = if result.output.trim().is_empty() {
                result.error_output.trim().to_string()
            } else {
                result.output.trim().to_string()
            };
            log::error!(
                "[rollback] restart failed (exit {:?}): {}",
                result.exit_code,
                output.chars().take(200).collect::<String>()
            );
            return Err(format!(
                "重启脚本失败（exit {:?}）: {}",
                result.exit_code,
                output.chars().take(200).collect::<String>()
            ));
        }
        log::info!("[rollback] restart success: {}", result.output.trim());
        Ok(())
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {}", e))?
}

// =================== Log Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_logs(
    core: State<'_, CoreService>,
    config_id: String,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deploy_logs() called");
    let lim = limit.unwrap_or(50);
    core.db_read(|conn| {
        let mut stmt = conn
            .prepare("SELECT * FROM deploy_logs WHERE configId = ? ORDER BY createdAt DESC LIMIT ?")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![config_id, lim], row_to_deploy_log)
            .map_err(|e| e.to_string())?;
        let logs: Vec<DeployLog> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&logs).map_err(|e| e.to_string())
    })?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_deploy_logs(
    core: State<'_, CoreService>,
    limit: Option<i64>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_deploy_logs() called");
    let lim = limit.unwrap_or(50);
    core.db_read(|conn| {
        // JOIN cicd_configs 带出配置名（deploy_history.get_all_deploy_history 的替代，
        // deploy_history 表已废弃）
        let mut stmt = conn
            .prepare(
                "SELECT l.id, l.configId, c.name as configName, l.status, l.createdAt \
                 FROM deploy_logs l LEFT JOIN cicd_configs c ON l.configId = c.id \
                 ORDER BY l.createdAt DESC LIMIT ?",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![lim], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>("id")?,
                    "configId": row.get::<_, String>("configId")?,
                    "configName": row.get::<_, Option<String>>("configName")?,
                    "status": row.get::<_, String>("status")?,
                    "createdAt": row.get::<_, String>("createdAt")?,
                }))
            })
            .map_err(|e| e.to_string())?;
        let items: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
        serde_json::to_value(&items).map_err(|e| e.to_string())
    })?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn read_log_file(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] read_log_file() called");
    let content =
        std::fs::read_to_string(&file_path).map_err(|e| format!("读取日志文件失败: {}", e))?;
    Ok(serde_json::json!({
        "success": true,
        "content": content,
        "filePath": file_path,
    }))
}

/// HTML 转义：仅处理 &, <, > 三个最常见字符，避免日志内容触发 HTML 解析
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// 检测日志级别并返回对应的 Tailwind 类名（仅扫前 200 字符，性能优先）
fn detect_level_class(line: &str) -> &'static str {
    if line.is_empty() {
        return "";
    }
    // 仅扫前 200 个字符（不是字节），避免在 UTF-8 多字节字符中间切片 panic
    let head: String = line.chars().take(200).collect();
    // 大小写不敏感：把 ASCII 字符转大写后再子串匹配，比 regex 快
    let head_upper = head.to_ascii_uppercase();
    if head_upper.contains("ERROR")
        || head_upper.contains("FATAL")
        || head_upper.contains("CRITICAL")
        || head_upper.contains("EXCEPTION")
    {
        return "text-red-400";
    }
    if head_upper.contains("WARN") || head_upper.contains("WARNING") {
        return "text-yellow-300";
    }
    if head_upper.contains("DEBUG") {
        return "text-white/50";
    }
    ""
}

/// 分页读取日志文件：后端预计算每行的 HTML（转义 + 级别色包装 + 关键字高亮），
/// 前端直接 v-html 渲染，避免在前端处理字符串性能瓶颈。
///
/// 参数：
/// - filePath: 本地文件绝对路径
/// - start: 起始行号（0-based，inclusive）
/// - count: 读取行数（建议 50-200）
/// - keyword: 可选关键字。非空时对返回的行做高亮（<mark> 包裹），不做过滤。
///   匹配行号列表由 find_log_matches 单独获取。
#[tauri::command(rename_all = "camelCase")]
pub fn read_log_file_lines(
    file_path: String,
    start: usize,
    count: usize,
    keyword: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] read_log_file_lines() path={} start={} count={} keyword={:?}",
        file_path,
        start,
        count,
        keyword
    );

    if count == 0 {
        return Ok(serde_json::json!({
            "success": true,
            "data": {
                "totalLines": 0usize,
                "lines": Vec::<serde_json::Value>::new(),
                "start": 0usize,
                "end": 0usize,
            }
        }));
    }

    let file = std::fs::File::open(&file_path)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);

    // 准备关键字正则（仅用于高亮，不过滤）
    let kw = keyword
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let highlight_re = if let Some(kw_str) = kw {
        let escaped = regex::escape(kw_str);
        regex::Regex::new(&format!("({})", escaped)).ok()
    } else {
        None
    };

    // 计算单行 HTML 的闭包
    let build_html = |line: &str, re: Option<&regex::Regex>| -> String {
        let escaped = html_escape(line);
        let highlighted = if let Some(re) = re {
            re.replace_all(&escaped, "<mark>$1</mark>").to_string()
        } else {
            escaped
        };
        let cls = detect_level_class(line);
        if cls.is_empty() {
            highlighted
        } else {
            format!("<span class=\"{}\">{}</span>", cls, highlighted)
        }
    };

    // 统一模式：按 [start, start+count) 区间读取，keyword 仅做高亮
    let mut total_lines: usize = 0;
    let end_exclusive = start.saturating_add(count);
    let mut cached_lines: Vec<String> = Vec::with_capacity(count);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => {
                total_lines = total_lines.saturating_add(1);
                continue;
            }
        };
        let idx = total_lines;
        total_lines = total_lines.saturating_add(1);

        if idx >= start && idx < end_exclusive {
            cached_lines.push(line);
        }
    }

    let lines_json: Vec<serde_json::Value> = cached_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            serde_json::json!({
                "lineNo": start + i,
                "html": build_html(line, highlight_re.as_ref()),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "totalLines": total_lines,
            "lines": lines_json,
            "start": start,
            "end": end_exclusive.min(total_lines),
        }
    }))
}

/// 查找日志文件中所有匹配关键字的行号（0-based）。
/// 用于 vim 式搜索：显示完整日志 + 高亮 + 上下跳转。
#[tauri::command(rename_all = "camelCase")]
pub fn find_log_matches(file_path: String, keyword: String) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] find_log_matches() path={} keyword={:?}",
        file_path,
        keyword
    );

    let kw = keyword.trim();
    if kw.is_empty() {
        return Ok(serde_json::json!({
            "success": true,
            "data": { "matchLineNos": Vec::<usize>::new() }
        }));
    }

    let escaped = regex::escape(kw);
    let re = regex::Regex::new(&format!("({})", escaped))
        .map_err(|e| format!("正则编译失败: {}", e))?;

    let file = std::fs::File::open(&file_path)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);

    let mut match_line_nos: Vec<usize> = Vec::new();
    let mut line_no: usize = 0;
    for line_result in reader.lines() {
        match line_result {
            Ok(l) => {
                if re.is_match(&l) {
                    match_line_nos.push(line_no);
                }
            }
            Err(_) => {}
        }
        line_no = line_no.saturating_add(1);
    }

    Ok(serde_json::json!({
        "success": true,
        "data": { "matchLineNos": match_line_nos }
    }))
}

/// 解压本地 .gz 文件到同目录下的 .decompressed 文件
/// 用于离线日志查看：下载 .gz 后解压，再交给 read_log_file_lines 读取
/// 返回解压后的文件路径
#[tauri::command(rename_all = "camelCase")]
pub fn gunzip_local_file(gz_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] gunzip_local_file() path={}", gz_path);

    let gz_file = std::fs::File::open(&gz_path)
        .map_err(|e| format!("打开 .gz 文件失败: {}", e))?;
    let decoder = flate2::read::GzDecoder::new(gz_file);
    use std::io::Read;

    // 解压到同目录下，文件名去掉 .gz 后缀，加 .decompressed 防止冲突
    let src_path = std::path::Path::new(&gz_path);
    let file_stem = src_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "decompressed".to_string());
    let parent = src_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let decompressed_path = parent.join(format!("{}.decompressed", file_stem));

    let mut out_file = std::fs::File::create(&decompressed_path)
        .map_err(|e| format!("创建解压文件失败: {}", e))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut decoder = decoder;
    let mut total: u64 = 0;
    loop {
        let n = decoder
            .read(&mut buf)
            .map_err(|e| format!("解压读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut out_file, &buf[..n])
            .map_err(|e| format!("解压写入失败: {}", e))?;
        total += n as u64;
    }

    log::info!(
        "[Tauri CMD] gunzip_local_file() decompressed {} -> {} ({} bytes)",
        gz_path,
        decompressed_path.display(),
        total
    );

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "decompressedPath": decompressed_path.to_string_lossy().to_string(),
            "bytesWritten": total,
        }
    }))
}

// deploy_history 表已废弃（2026-08 清理）：get_rollback_history、get_deploy_history、
// get_all_deploy_history 命令一并移除（前端 UI 均无调用，部署历史统一走 get_deploy_logs）；
// 回滚状态改写入 deploy_logs（见 rollback 命令）。

// =================== DB function aliases (avoid name collision with commands) ===================

fn cicd_get_all_configs(conn: &rusqlite::Connection) -> Result<Vec<CicdConfig>, String> {
    supertool_core::db::cicd::get_all_cicd_configs(conn).map_err(|e| e.to_string())
}
fn cicd_get_config_by_id(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<Option<CicdConfig>, String> {
    supertool_core::db::cicd::get_cicd_config_by_config_id(conn, id).map_err(|e| e.to_string())
}
fn cicd_get_groups(conn: &rusqlite::Connection) -> Result<Vec<String>, String> {
    supertool_core::db::cicd::get_cicd_groups(conn).map_err(|e| e.to_string())
}
fn cicd_add_config(conn: &rusqlite::Connection, c: &CicdConfig) -> Result<CicdConfig, String> {
    supertool_core::db::cicd::add_cicd_config(conn, c).map_err(|e| e.to_string())
}
fn cicd_update_config(
    conn: &rusqlite::Connection,
    c: &CicdConfig,
) -> Result<Option<CicdConfig>, String> {
    supertool_core::db::cicd::update_cicd_config(conn, c).map_err(|e| e.to_string())
}
fn cicd_delete_config(conn: &rusqlite::Connection, id: &str) -> Result<(), String> {
    supertool_core::db::cicd::delete_cicd_config(conn, id).map_err(|e| e.to_string())
}
fn cicd_get_modules(
    conn: &rusqlite::Connection,
    config_id: &str,
) -> Result<Vec<DeployModule>, String> {
    supertool_core::db::cicd::get_deploy_modules(conn, config_id).map_err(|e| e.to_string())
}
fn cicd_add_module(conn: &rusqlite::Connection, m: &DeployModule) -> Result<DeployModule, String> {
    supertool_core::db::cicd::add_deploy_module(conn, m).map_err(|e| e.to_string())
}
fn cicd_add_deploy_log(conn: &rusqlite::Connection, log: &DeployLog) -> Result<DeployLog, String> {
    supertool_core::db::cicd::add_deploy_log(conn, log).map_err(|e| e.to_string())
}
fn cicd_update_deploy_log(
    conn: &rusqlite::Connection,
    log: &DeployLog,
) -> Result<Option<DeployLog>, String> {
    supertool_core::db::cicd::update_deploy_log(conn, log).map_err(|e| e.to_string())
}
fn cicd_get_deploy_log_by_id(conn: &rusqlite::Connection, id: &str) -> Option<DeployLog> {
    supertool_core::db::cicd::get_deploy_log_by_id(conn, id)
        .map_err(|e| e.to_string())
        .ok()
        .flatten()
}
fn cicd_touch_deploy(conn: &rusqlite::Connection, id: &str) -> Result<(), String> {
    supertool_core::db::cicd::touch_cicd_config_deploy(conn, id).map_err(|e| e.to_string())
}

// =================== Helper Functions ===================

/// 解析服务器引用列表为完整部署服务器配置（查 servers 表 + 解密密码）
fn resolve_deploy_servers(
    core: &supertool_core::logic::CoreService,
    refs: &[(String, String)], // (serverId, deployDir)
    fallback_dir: &str,
    lib_separate: bool,
) -> Result<Vec<DeployServerConfig>, String> {
    refs.iter()
        .map(|(server_id, deploy_dir)| {
            // 直接查 servers 表 + 解密密码
            let server = core.db_read(|conn| {
                conn.query_row(
                    "SELECT * FROM servers WHERE id = ?1",
                    rusqlite::params![server_id],
                    supertool_core::db::servers::row_to_server,
                )
                .map_err(|e| e.to_string())
            })??;
            // 密码已在 row_to_server 中解密 (servers.rs 的 get_server_by_id 调用 decrypt_password)
            // 但 row_to_server 不解密，需要手动解密
            let password = server
                .password
                .map(|pw| supertool_core::encryption::try_decrypt_password(&pw));
            let base_deploy_dir = if deploy_dir.is_empty() {
                fallback_dir.to_string()
            } else {
                deploy_dir.clone()
            };
            Ok(DeployServerConfig {
                host: server.host,
                port: server.port as u16,
                username: server.username,
                password,
                private_key: server.ssh_key_path,
                deploy_dir: base_deploy_dir.clone(),
                lib_dir: if lib_separate {
                    Some(format!("{}/lib", base_deploy_dir))
                } else {
                    None
                },
                label: Some(server.name),
            })
        })
        .collect()
}

fn build_deploy_config(
    core: &supertool_core::logic::CoreService,
    cicd_config: &CicdConfig,
    modules: &[DeployModule],
) -> Result<DeployConfig, String> {
    // 解析服务器引用（DB 存的是 [{serverId, deployDir}]，需查 servers 表补全）
    let lib_separate =
        cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven");
    let servers: Vec<DeployServerConfig> = if let Some(ref servers_str) = cicd_config.servers {
        #[derive(Deserialize)]
        struct ServerRef {
            #[serde(rename = "serverId")]
            server_id: String,
            #[serde(rename = "deployDir")]
            deploy_dir: String,
        }
        let refs: Vec<ServerRef> =
            serde_json::from_str(servers_str).map_err(|e| format!("解析服务器引用失败: {}", e))?;
        let pairs: Vec<(String, String)> = refs
            .into_iter()
            .map(|r| (r.server_id, r.deploy_dir))
            .collect();
        resolve_deploy_servers(core, &pairs, &cicd_config.deploy_path, lib_separate)?
    } else {
        vec![]
    };
    let module_configs: Vec<DeployModuleConfig> = modules
        .iter()
        .map(|m| DeployModuleConfig {
            name: Some(m.module_name.clone()),
            path: Some(m.module_path.clone()),
            build_path: m.build_path.clone(),
            build_command: m.build_command.clone(),
            build_tool: m.build_tool.clone(),
            output_path: m.output_path.clone(),
            artifact_name: Some(m.artifact_name.clone()),
            artifact_type: m.artifact_type.clone(),
            lib_filter_rules: m.lib_filter_rules.clone(),
            deploy_order: m.deploy_order,
            deploy_path: m.deploy_path.clone(),
            enabled: m.enabled,
        })
        .collect();
    Ok(DeployConfig {
        repo_url: cicd_config
            .git_repo_id
            .as_ref()
            .and_then(|id| {
                core.db_read(|conn| {
                    supertool_core::db::git_repo::get_by_id(conn, id)
                        .ok()
                        .flatten()
                })
                .ok()
                .flatten()
            })
            .and_then(|r| r.remote.or(Some(r.path)))
            .unwrap_or_default(),
        branch: cicd_config.deploy_branch.clone(),
        // 代码实际目录优先：localPath 可能指向仓库子目录（如 SRC/front/mall-h5），
        // gitRepo.path 只是 git 仓库根；与 core/mod.rs 的 execute_deploy 保持同一规则
        local_path: match cicd_config.local_path.as_deref() {
            Some(p) if !p.trim().is_empty() => Some(p.trim().to_string()),
            _ => cicd_config
                .git_repo_id
                .as_ref()
                .and_then(|id| {
                    core.db_read(|conn| {
                        supertool_core::db::git_repo::get_by_id(conn, id)
                            .ok()
                            .flatten()
                    })
                    .ok()
                    .flatten()
                })
                .map(|r| r.path),
        },
        build_tool: cicd_config.build_tool.clone(),
        build_command: cicd_config.build_command.clone(),
        build_path: cicd_config.build_path.clone(),
        npm_script: cicd_config.npm_script.clone(),
        npm_custom_script: cicd_config.npm_custom_script.clone(),
        maven_home: cicd_config.maven_home.clone(),
        java_home: cicd_config.java_home.clone(),
        npm_home: cicd_config.npm_home.clone(),
        node_home: cicd_config.node_home.clone(),
        maven_profile: Some(cicd_config.maven_profile.clone()),
        maven_settings: cicd_config.maven_settings.clone(),
        modules: module_configs,
        skip_tests: true,
        parent_build_mode: cicd_config.parent_build_mode,
        parent_build_path: if cicd_config.parent_build_path.is_empty() {
            None
        } else {
            Some(cicd_config.parent_build_path.clone())
        },
        servers,
        deploy_dir: cicd_config.deploy_path.clone(),
        lib_dir: if cicd_config.lib_separate && cicd_config.build_tool.as_deref() == Some("maven") {
            Some(format!("{}/lib", cicd_config.deploy_path))
        } else {
            None
        },
        restart_script: if cicd_config.restart_script.is_empty() {
            None
        } else {
            Some(cicd_config.restart_script.clone())
        },
        lib_separate: cicd_config.lib_separate
            && cicd_config.build_tool.as_deref() == Some("maven"),
        build_mode: cicd_config.build_mode.clone(),
        env_vars: HashMap::new(),
        health_check_url: cicd_config
            .health_check_url
            .clone()
            .filter(|u| !u.is_empty()),
        health_check_timeout: cicd_config.health_check_timeout.max(1) as u64,
        health_check_retries: cicd_config.health_check_retries.max(1) as u32,
        output_path: cicd_config
            .output_path
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        lib_filter_rules: cicd_config
            .lib_filter_rules
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        incremental_upload: cicd_config.incremental_upload,
        environment_name: None,
    })
}

// =================== Module CRUD Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn save_deploy_module(
    core: State<'_, CoreService>,
    module: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_deploy_module() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut dm: DeployModule =
        serde_json::from_value(module.clone()).map_err(|e| format!("解析模块失败: {}", e))?;
    dm.created_at = now.clone();
    dm.updated_at = now.clone();
    let result = core
        .db_write(|conn| cicd_add_module(conn, &dm).map_err(|e| e.to_string()))??;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_deploy_module(
    core: State<'_, CoreService>,
    module: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_deploy_module() called");
    let now = chrono::Utc::now().to_rfc3339();
    let mut dm: DeployModule =
        serde_json::from_value(module.clone()).map_err(|e| format!("解析模块失败: {}", e))?;
    dm.updated_at = now.clone();
    let result = core.db_write(|conn| {
        supertool_core::db::cicd::update_deploy_module(conn, &dm).map_err(|e| e.to_string())
    })?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_deploy_module(
    core: State<'_, CoreService>,
    module_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_deploy_module() called");
    let _ = core.db_write(|conn| {
        supertool_core::db::cicd::delete_deploy_module(conn, &module_id).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn scan_project_modules(project_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] scan_project_modules() called");
    Ok(supertool_core::logic::cicd_tools::scan_project_modules(&project_path))
}

// =================== Missing Tauri Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_modules(
    core: State<'_, CoreService>,
    config_id: String,
) -> Result<Vec<DeployModule>, String> {
    log::info!("[Tauri CMD] get_deploy_modules() called");
    core.db_read(|conn| cicd_get_modules(conn, &config_id).map_err(|e| e.to_string()))?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_deploy_step_logs(
    core: State<'_, CoreService>,
    deploy_log_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_deploy_step_logs() called");
    let logs = core.db_read(|conn| {
        supertool_core::db::cicd::get_deploy_step_logs(conn, &deploy_log_id)
            .map_err(|e| e.to_string())
    })?;
    serde_json::to_value(&logs).map_err(|e| e.to_string())
}

#[cfg(test)]
mod progress_batcher_tests {
    use super::*;

    fn line(msg: &str) -> serde_json::Value {
        serde_json::json!({ "stage": "maven", "status": "building", "message": msg })
    }

    fn aged_batcher(ms_ago: u64) -> DeployProgressBatcher {
        let mut b = DeployProgressBatcher::default();
        b.last_flush = Some(std::time::Instant::now() - std::time::Duration::from_millis(ms_ago));
        b
    }

    /// 时间窗未到就只攒不发；到点一次性发出整批 —— 这就是压住主线程 eval 次数的关键
    #[test]
    fn holds_lines_until_interval() {
        let mut b = aged_batcher(0);
        for i in 0..5 {
            assert!(b.push(line(&format!("line {i}")), None).is_none());
        }
        b.last_flush = Some(
            std::time::Instant::now() - std::time::Duration::from_millis(PROGRESS_BATCH_INTERVAL_MS + 20),
        );
        let batch = b.push(line("last"), None).expect("超过最小间隔应产出 batch");
        assert_eq!(batch["stage"], "batch");
        assert_eq!(batch["lines"].as_array().unwrap().len(), 6);
        // message 取批次末行，供前端 currentStep 显示
        assert_eq!(batch["message"], "last");
        // 缓冲已清空：再次 drain 无内容
        assert!(b.drain().is_none());
    }

    /// burst 超过单批上限时裁剪最旧行，并在批次头部标注省略数量（全量仍在部署日志文件）
    #[test]
    fn trims_oldest_and_annotates_dropped() {
        let mut b = aged_batcher(0);
        let total = PROGRESS_BATCH_MAX_BUFFER + 50;
        for i in 0..total {
            assert!(b.push(line(&format!("line {i}")), None).is_none());
        }
        b.last_flush = Some(
            std::time::Instant::now() - std::time::Duration::from_millis(PROGRESS_BATCH_INTERVAL_MS + 20),
        );
        let batch = b.push(line("tail"), None).expect("到点应产出 batch");
        let lines = batch["lines"].as_array().unwrap();
        // 单批上限行 + 头部一行省略提示
        assert_eq!(lines.len(), PROGRESS_BATCH_MAX_BUFFER + 1);
        assert!(lines[0]["message"].as_str().unwrap().contains("已省略"));
        assert_eq!(batch["message"], "tail");
        // 省略计数随批次一次性消费
        assert_eq!(b.dropped, 0);
    }

    /// 只有构建/依赖安装的普通输出可以攒批，状态事件与报错行必须即时
    #[test]
    fn only_quiet_line_stream_is_batched() {
        assert!(is_noisy_progress("building", "[INFO] Compiling 12 source files"));
        assert!(is_noisy_progress("installing", "npm WARN deprecated x@1: use y"));
        for status in ["success", "failed", "connecting", "uploading", "warning", "info"] {
            assert!(
                !is_noisy_progress(status, "[INFO] whatever"),
                "{status} 不应被批量延迟"
            );
        }
        for msg in [
            "[ERROR] /src/Foo.java:42 找不到符号",
            "BUILD FAILURE",
            "npm ERR! code ELIFECYCLE",
            "测试用例 FAILED",
            "编译异常: NPE",
        ] {
            assert!(
                !is_noisy_progress("building", msg),
                "报错行不能被攒批或裁剪: {msg}"
            );
        }
    }

    /// 超长行截断，防止单条事件撑爆 IPC
    #[test]
    fn clips_overlong_lines() {
        let short = clip_progress_line("ok");
        assert_eq!(short, "ok");
        let long = clip_progress_line(&"字".repeat(PROGRESS_LINE_MAX_CHARS + 50));
        assert_eq!(long.chars().count(), PROGRESS_LINE_MAX_CHARS + 1);
        assert!(long.ends_with('…'));
    }
}
