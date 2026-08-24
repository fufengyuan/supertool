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
                        result.module_names = Some(modules);
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

        let deploy_result = cicd_deploy::execute_deploy(
            &deploy_config,
            &app_dir,
            &deploy_id_arc,
            move |event| {
                let payload = serde_json::json!({
                    "deployLogId": *did_for_closure,
                    "configId": *cid_for_closure,
                    "stage": event.stage,
                    "status": event.status,
                    "message": event.message,
                    "progress": event.progress,
                });
                let _ = app_for_closure.emit("deploy-progress", &payload);
            },
            move || is_deploy_cancelled(&did_for_cancel),
        )
        .await;

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

                // Execute restart script via SSH
                match execute_remote_restart(
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

/// Execute a restart command on a remote server via SSH
/// 使用 spawn_blocking 避免阻塞 tokio async 运行时
async fn execute_remote_restart(
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
    restart_script: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        use ssh2::Session;
        use std::net::TcpStream;

        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr).map_err(|e| format!("连接 {} 失败: {}", addr, e))?;

        let mut sess = Session::new().map_err(|e| format!("创建 SSH session 失败: {}", e))?;
        sess.set_tcp_stream(tcp);
        sess.set_timeout(30_000);
        sess.handshake()
            .map_err(|e| format!("SSH 握手失败: {}", e))?;

        if let Some(key_path) = private_key {
            sess.userauth_pubkey_file(
                &username,
                None,
                std::path::Path::new(&key_path),
                password.as_deref(),
            )
            .map_err(|e| format!("SSH 密钥认证失败: {}", e))?;
        } else if let Some(ref pw) = password {
            sess.userauth_password(&username, pw)
                .map_err(|e| format!("SSH 密码认证失败: {}", e))?;
        } else {
            return Err("缺少认证信息".to_string());
        }

        if !sess.authenticated() {
            return Err("SSH 认证失败".to_string());
        }

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
            log::error!(
                "[rollback] restart failed (exit {}): {}",
                exit_status,
                output.trim()
            );
            return Err(format!(
                "重启脚本退出码 {}: {}",
                exit_status,
                output.trim().chars().take(200).collect::<String>()
            ));
        } else {
            log::info!("[rollback] restart success: {}", output.trim());
        }
        sess.disconnect(None, "", None).ok();

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
        // 代码实际目录优先：localPath 可能指向仓库子目录（如 SRC/front/corp-mobile），
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
