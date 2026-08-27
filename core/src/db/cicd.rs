use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// =================== Types ===================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CicdConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "deployBranch")]
    pub deploy_branch: String,
    #[serde(rename = "mavenSettings")]
    pub maven_settings: Option<String>,
    #[serde(rename = "mavenProfile")]
    pub maven_profile: String,
    #[serde(rename = "deployPath")]
    pub deploy_path: String,
    #[serde(rename = "libSeparate")]
    pub lib_separate: bool,
    #[serde(rename = "restartScript")]
    pub restart_script: String,
    #[serde(rename = "healthCheckUrl")]
    pub health_check_url: Option<String>,
    #[serde(rename = "healthCheckTimeout")]
    pub health_check_timeout: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "buildCommand")]
    pub build_command: Option<String>,
    #[serde(rename = "buildPath")]
    pub build_path: Option<String>,
    #[serde(rename = "repoUrl")]
    pub repo_url: Option<String>,
    #[serde(rename = "localPath")]
    pub local_path: Option<String>,
    #[serde(rename = "npmScript")]
    pub npm_script: Option<String>,
    #[serde(rename = "npmCustomScript")]
    pub npm_custom_script: Option<String>,
    #[serde(rename = "mavenHome")]
    pub maven_home: Option<String>,
    #[serde(rename = "npmHome")]
    pub npm_home: Option<String>,
    #[serde(rename = "pnpmHome")]
    pub pnpm_home: Option<String>,
    #[serde(rename = "yarnHome")]
    pub yarn_home: Option<String>,
    #[serde(rename = "javaHome")]
    pub java_home: Option<String>,
    #[serde(rename = "nodeHome")]
    pub node_home: Option<String>,
    pub servers: Option<String>, // JSON array of {serverId, deployDir}
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "lastDeployedAt")]
    pub last_deployed_at: Option<String>,
    #[serde(rename = "parentBuildMode")]
    pub parent_build_mode: bool,
    #[serde(rename = "parentBuildPath")]
    pub parent_build_path: String,
    #[serde(rename = "requiresApproval")]
    pub requires_approval: bool,
    #[serde(rename = "gitRepoId")]
    pub git_repo_id: Option<String>,
    #[serde(rename = "buildMode")]
    pub build_mode: String,
    /// 多环境配置 JSON：[{name, deployPath, servers, envVars, healthCheckUrl, healthCheckTimeout, healthCheckRetries}]
    #[serde(rename = "environments")]
    pub environments: Option<String>,
    /// 增量上传开关（对比文件 hash 只传变更，默认开启）
    #[serde(rename = "incrementalUpload", default = "default_true")]
    pub incremental_upload: bool,
    /// 配置级健康检查重试次数（默认 3）
    #[serde(rename = "healthCheckRetries", default = "default_retries")]
    pub health_check_retries: i64,
    /// 单体前端的产物输出目录（相对代码目录，如 build/h5；空则自动扫描 dist 候选）
    #[serde(rename = "outputPath", default)]
    pub output_path: Option<String>,
    /// 单体（单产物）部署的 lib 分离过滤规则（每行一个通配模式，仅打包匹配依赖；空=全部）
    #[serde(rename = "libFilterRules", default)]
    pub lib_filter_rules: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_retries() -> i64 {
    3
}

/// 按构建工具 / 部署模式清理配置级条件字段（保存前兜底，防止 UI 不展示的残留值污染后续逻辑）。
/// 规则与向导面板条件显示一一对应：
/// - 非 maven：清空 maven 专属路径与 lib 分离（lib 分离后端仅 maven 生效）
/// - 非 npm/pnpm/yarn：清空 node 相关字段
/// - 非 cargo：配置级 buildCommand 无入口，清空
/// - 非 maven：restartScript 仅 maven 面板有入口，非 maven 清空
/// - 非单体（多模块）：parentBuildPath / 配置级 outputPath 为单体面板字段，多模块时无意义，清空
pub fn sanitize_cicd_config_conditional(c: &mut CicdConfig) {
    let tool = c.build_tool.as_deref().unwrap_or("");
    let is_maven = tool == "maven";
    let is_node = matches!(tool, "npm" | "pnpm" | "yarn");
    let is_cargo = tool == "cargo";
    // 单体部署（parentBuildMode=true）才显示构建目录/产物目录面板
    let is_monolith = c.parent_build_mode;

    if !is_maven {
        c.maven_home = None;
        c.java_home = None;
        c.maven_settings = None;
        c.maven_profile = String::new();
        c.lib_separate = false;
        c.lib_filter_rules = None;
        c.restart_script = String::new();
    }
    if !is_node {
        c.npm_home = None;
        c.pnpm_home = None;
        c.yarn_home = None;
        c.node_home = None;
        c.npm_script = None;
        c.npm_custom_script = None;
    }
    if !is_cargo {
        // 配置级 buildCommand 仅 cargo 面板有输入；npm/maven 分别走 npmScript 与固定 mvn 命令
        c.build_command = None;
    }
    if !is_monolith {
        // 多模块模式：聚合根 = localPath（或模块行各自目录），配置级 parentBuildPath/outputPath 不适用
        c.parent_build_path = String::new();
        c.output_path = None;
    }
}

/// 模块行条件清理所需的配置级快照（摆脱 CicdConfig 所有权，供闭包内使用）
pub struct SanitizeSnapshot {
    pub global_build_tool: Option<String>,
    pub lib_separate: bool,
}

/// 从配置构造模块清理快照
pub fn sanitize_snapshot(config: &CicdConfig) -> SanitizeSnapshot {
    SanitizeSnapshot {
        global_build_tool: config.build_tool.clone(),
        lib_separate: config.lib_separate,
    }
}

/// 按构建工具 / lib 开关清理模块行条件字段（保存前兜底，使用预取快照）：
/// - maven 模块：构建路径统一在聚合根，模块行 buildPath/buildCommand 无入口（已隐藏），清空
/// - 非 maven 或未开启 lib 分离：libFilterRules 不适用，清空
/// - 产物类型 jar-plus-lib 仅 maven && libSeparate 时有效，否则降级为 jar
pub fn sanitize_deploy_module_from_snapshot(m: &mut DeployModule, snap: &SanitizeSnapshot) {
    let module_tool = m
        .build_tool
        .as_deref()
        .unwrap_or(snap.global_build_tool.as_deref().unwrap_or(""));
    let is_maven = module_tool == "maven";
    let lib_ok = is_maven && snap.lib_separate;

    if is_maven {
        m.build_path = None;
        m.build_command = None;
    }
    if !lib_ok {
        m.lib_filter_rules = None;
    }
    if m.artifact_type.as_deref() == Some("jar-plus-lib") && !lib_ok {
        m.artifact_type = Some("jar".to_string());
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployModule {
    pub id: String,
    #[serde(rename = "configId")]
    pub config_id: String,
    #[serde(rename = "moduleName")]
    pub module_name: String,
    #[serde(rename = "modulePath")]
    pub module_path: String,
    #[serde(rename = "buildPath")]
    pub build_path: Option<String>,
    #[serde(rename = "buildCommand")]
    pub build_command: Option<String>,
    #[serde(rename = "buildTool")]
    pub build_tool: Option<String>,
    #[serde(rename = "outputPath")]
    pub output_path: Option<String>,
    #[serde(rename = "artifactName")]
    pub artifact_name: String,
    #[serde(rename = "artifactType")]
    pub artifact_type: Option<String>,
    #[serde(rename = "libFilterRules")]
    pub lib_filter_rules: Option<String>,
    #[serde(rename = "deployOrder")]
    pub deploy_order: i64,
    #[serde(rename = "deployPath")]
    pub deploy_path: Option<String>,
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployLog {
    pub id: String,
    #[serde(rename = "configId")]
    pub config_id: String,
    pub status: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    pub progress: i64,
    #[serde(rename = "triggeredBy")]
    pub triggered_by: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "logFilePath")]
    pub log_file_path: Option<String>,
    #[serde(rename = "artifactPaths")]
    pub artifact_paths: Option<String>,
    /// 本次部署的环境名（多环境配置时记录）
    #[serde(rename = "environment")]
    pub environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployStepLog {
    pub id: i64,
    #[serde(rename = "deployLogId")]
    pub deploy_log_id: String,
    pub stage: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployHistory {
    pub id: String,
    #[serde(rename = "configId")]
    pub config_id: String,
    pub status: String,
    #[serde(rename = "deployedAt")]
    pub deployed_at: String,
    #[serde(rename = "rolledBack")]
    pub rolled_back: bool,
    #[serde(rename = "rolledBackAt")]
    pub rolled_back_at: Option<String>,
}

// =================== Helpers ===================

pub fn row_to_cicd_config(row: &rusqlite::Row) -> rusqlite::Result<CicdConfig> {
    Ok(CicdConfig {
        id: row.get("id")?,
        name: row.get("name")?,
        deploy_branch: row.get("deployBranch")?,
        maven_settings: row.get("mavenSettings")?,
        maven_profile: row.get("mavenProfile")?,
        deploy_path: row.get("deployPath")?,
        lib_separate: row.get::<_, i64>("libSeparate")? != 0,
        restart_script: row.get("restartScript")?,
        health_check_url: row.get("healthCheckUrl")?,
        health_check_timeout: row.get("healthCheckTimeout")?,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
        group_name: row.get("groupName")?,
        parent_build_mode: row.get::<_, i64>("parentBuildMode")? != 0,
        parent_build_path: row.get("parentBuildPath")?,
        requires_approval: row.get::<_, i64>("requiresApproval")? != 0,
        build_tool: row.get("buildTool")?,
        build_command: row.get("buildCommand")?,
        build_path: row.get("buildPath")?,
        repo_url: row.get("repoUrl")?,
        local_path: row.get("localPath")?,
        npm_script: row.get("npmScript")?,
        npm_custom_script: row.get("npmCustomScript")?,
        maven_home: row.get("mavenHome")?,
        npm_home: row.get("npmHome")?,
        java_home: row.get("javaHome")?,
        node_home: row.get("nodeHome")?,
        servers: row.get::<_, Option<String>>("servers")?,
        last_deployed_at: row.get::<_, Option<String>>("lastDeployedAt")?,
        pnpm_home: row.get("pnpmHome").ok(),
        yarn_home: row.get("yarnHome").ok(),
        git_repo_id: row.get("gitRepoId").ok(),
        build_mode: row.get("buildMode").unwrap_or("local".to_string()),
        environments: row.get("environments").ok().flatten(),
        incremental_upload: row
            .get::<_, Option<i64>>("incrementalUpload")
            .ok()
            .flatten()
            .map(|v| v != 0)
            .unwrap_or(true),
        health_check_retries: row
            .get::<_, Option<i64>>("healthCheckRetries")
            .ok()
            .flatten()
            .unwrap_or(3),
        output_path: row.get("outputPath").ok().flatten(),
        lib_filter_rules: row.get("libFilterRules").ok().flatten(),
    })
}

fn row_to_deploy_module(row: &rusqlite::Row) -> rusqlite::Result<DeployModule> {
    Ok(DeployModule {
        id: row.get("id")?,
        config_id: row.get("configId")?,
        module_name: row.get("moduleName")?,
        module_path: row.get("modulePath")?,
        artifact_name: row.get("artifactName")?,
        deploy_order: row.get("deployOrder")?,
        deploy_path: row.get("deployPath")?,
        enabled: row.get::<_, i64>("enabled")? != 0,
        created_at: row.get("createdAt")?,
        updated_at: row.get("updatedAt")?,
        lib_filter_rules: row.get("libFilterRules")?,
        build_command: row.get("buildCommand")?,
        build_path: row.get("buildPath")?,
        output_path: row.get("outputPath")?,
        build_tool: row.get("buildTool")?,
        artifact_type: row.get("artifactType").ok(),
    })
}

pub fn row_to_deploy_log(row: &rusqlite::Row) -> rusqlite::Result<DeployLog> {
    Ok(DeployLog {
        id: row.get("id")?,
        config_id: row.get("configId")?,
        status: row.get("status")?,
        start_time: row.get("startTime")?,
        end_time: row.get("endTime")?,
        error_message: row.get("errorMessage")?,
        progress: row.get("progress")?,
        triggered_by: row.get("triggeredBy")?,
        created_at: row.get("createdAt")?,
        log_file_path: row.get("logFilePath")?,
        artifact_paths: row.get("artifactPaths")?,
        environment: row.get("environment").ok().flatten(),
    })
}

fn row_to_deploy_step_log(row: &rusqlite::Row) -> rusqlite::Result<DeployStepLog> {
    Ok(DeployStepLog {
        // 表里 id 是 TEXT PRIMARY KEY，而 add_deploy_step_log 从不写入 id（沿用 Electron 时代的
        // 「自增」假设），结果是存量几十万行 id 全为 NULL。按 i64 直接 get 会对**每一行**报
        // 类型错误，再被上层的 filter_map(|r| r.ok()) 静默丢掉 → 步骤日志永远返回空列表。
        // 这里容忍缺失/非数字 id（它没有任何业务含义），避免整行被丢弃。
        id: row.get::<_, Option<i64>>(0).ok().flatten().unwrap_or(0),
        deploy_log_id: row.get(1)?,
        stage: row.get(2)?,
        status: row.get(3)?,
        message: row.get(4)?,
        timestamp: row.get(5)?,
    })
}

pub fn row_to_deploy_history(row: &rusqlite::Row) -> rusqlite::Result<DeployHistory> {
    Ok(DeployHistory {
        id: row.get("id")?,
        config_id: row.get("configId")?,
        status: row.get("status")?,
        deployed_at: row.get("deployedAt")?,
        rolled_back: row.get::<_, i64>("rolledBack")? != 0,
        rolled_back_at: row.get("rolledBackAt")?,
    })
}

// =================== CRUD Operations ===================

pub fn get_all_cicd_configs(conn: &Connection) -> Result<Vec<CicdConfig>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT * FROM cicd_configs ORDER BY lastDeployedAt DESC, updatedAt DESC")?;
    stmt.query_map([], row_to_cicd_config)
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

pub fn get_cicd_groups(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT DISTINCT groupName FROM cicd_configs ORDER BY groupName")?;
    stmt.query_map([], |row| row.get(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

pub fn get_cicd_config_by_config_id(
    conn: &Connection,
    config_id: &str,
) -> Result<Option<CicdConfig>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM cicd_configs WHERE id = ?")?;
    stmt.query_row([config_id], row_to_cicd_config)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
}

pub fn add_cicd_config(conn: &Connection, c: &CicdConfig) -> Result<CicdConfig, rusqlite::Error> {
    conn.execute(
        "INSERT INTO cicd_configs (id, name, deployBranch, mavenSettings, mavenProfile, \
         deployPath, libSeparate, restartScript, healthCheckUrl, healthCheckTimeout, createdAt, \
         updatedAt, buildTool, buildCommand, buildPath, repoUrl, localPath, npmScript, \
         npmCustomScript, mavenHome, npmHome, pnpmHome, yarnHome, javaHome, nodeHome, servers, groupName, \
         parentBuildMode, parentBuildPath, requiresApproval, gitRepoId, buildMode, environments, incrementalUpload, healthCheckRetries, outputPath, libFilterRules) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            c.id, &c.name, c.deploy_branch, c.maven_settings, c.maven_profile,
            c.deploy_path, if c.lib_separate { 1 } else { 0 }, c.restart_script, c.health_check_url,
            c.health_check_timeout, c.created_at, c.updated_at, c.build_tool, c.build_command,
            c.build_path, c.repo_url, c.local_path, c.npm_script, c.npm_custom_script,
            c.maven_home, c.npm_home, c.pnpm_home, c.yarn_home, c.java_home, c.node_home, c.servers, c.group_name,
            if c.parent_build_mode { 1 } else { 0 }, c.parent_build_path,
            if c.requires_approval { 1 } else { 0 },
            c.git_repo_id, c.build_mode, c.environments,
            if c.incremental_upload { 1 } else { 0 }, c.health_check_retries,
            c.output_path, c.lib_filter_rules
        ],
    )?;
    get_cicd_config_by_config_id(conn, &c.id).map(|opt| opt.unwrap())
}

pub fn update_cicd_config(
    conn: &Connection,
    c: &CicdConfig,
) -> Result<Option<CicdConfig>, rusqlite::Error> {
    conn.execute(
        "UPDATE cicd_configs SET name=?, deployBranch=?, mavenSettings=?, \
         mavenProfile=?, deployPath=?, libSeparate=?, restartScript=?, healthCheckUrl=?, \
         healthCheckTimeout=?, updatedAt=?, buildTool=?, buildCommand=?, buildPath=?, \
         repoUrl=?, localPath=?, npmScript=?, npmCustomScript=?, mavenHome=?, npmHome=?, pnpmHome=?, yarnHome=?, \
         javaHome=?, nodeHome=?, servers=?, groupName=?, parentBuildMode=?, \
         parentBuildPath=?, requiresApproval=?, gitRepoId=?, buildMode=?, environments=?, incrementalUpload=?, healthCheckRetries=?, outputPath=?, libFilterRules=? WHERE id=?",
        params![
            &c.name, c.deploy_branch, c.maven_settings, c.maven_profile,
            c.deploy_path, if c.lib_separate { 1 } else { 0 }, c.restart_script,
            c.health_check_url, c.health_check_timeout, c.updated_at, c.build_tool,
            c.build_command, c.build_path, c.repo_url, c.local_path, c.npm_script,
            c.npm_custom_script, c.maven_home, c.npm_home, c.pnpm_home, c.yarn_home,
            c.java_home, c.node_home, c.servers, c.group_name, if c.parent_build_mode { 1 } else { 0 },
            c.parent_build_path, if c.requires_approval { 1 } else { 0 },
            c.git_repo_id, c.build_mode, c.environments,
            if c.incremental_upload { 1 } else { 0 }, c.health_check_retries,
            c.output_path, c.lib_filter_rules, c.id
        ],
    )?;
    get_cicd_config_by_config_id(conn, &c.id)
}

pub fn delete_cicd_config(conn: &Connection, config_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM cicd_configs WHERE id = ?", [config_id])?;
    Ok(())
}

pub fn touch_cicd_config_deploy(conn: &Connection, config_id: &str) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE cicd_configs SET lastDeployedAt = ? WHERE id = ?",
        params![now, config_id],
    )?;
    Ok(())
}

// Deploy modules
pub fn get_deploy_modules(
    conn: &Connection,
    config_id: &str,
) -> Result<Vec<DeployModule>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT * FROM deploy_modules WHERE configId = ? ORDER BY deployOrder ASC")?;
    stmt.query_map([config_id], row_to_deploy_module)
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

pub fn add_deploy_module(
    conn: &Connection,
    m: &DeployModule,
) -> Result<DeployModule, rusqlite::Error> {
    conn.execute(
        "INSERT INTO deploy_modules (id, configId, moduleName, modulePath, buildPath, \
         buildCommand, buildTool, outputPath, artifactName, artifactType, libFilterRules, \
         deployOrder, deployPath, enabled, createdAt, updatedAt) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            m.id,
            m.config_id,
            m.module_name,
            m.module_path,
            m.build_path,
            m.build_command,
            m.build_tool,
            m.output_path,
            m.artifact_name,
            m.artifact_type,
            m.lib_filter_rules,
            m.deploy_order,
            m.deploy_path,
            if m.enabled { 1 } else { 0 },
            m.created_at,
            m.updated_at
        ],
    )?;
    get_deploy_module_by_id(conn, &m.id).map(|opt| opt.unwrap())
}

pub fn update_deploy_module(
    conn: &Connection,
    m: &DeployModule,
) -> Result<Option<DeployModule>, rusqlite::Error> {
    conn.execute(
        "UPDATE deploy_modules SET configId=?, moduleName=?, modulePath=?, buildPath=?, \
         buildCommand=?, buildTool=?, outputPath=?, artifactName=?, artifactType=?, \
         libFilterRules=?, deployOrder=?, deployPath=?, enabled=?, updatedAt=? WHERE id=?",
        params![
            m.config_id,
            m.module_name,
            m.module_path,
            m.build_path,
            m.build_command,
            m.build_tool,
            m.output_path,
            m.artifact_name,
            m.artifact_type,
            m.lib_filter_rules,
            m.deploy_order,
            m.deploy_path,
            if m.enabled { 1 } else { 0 },
            m.updated_at,
            m.id
        ],
    )?;
    get_deploy_module_by_id(conn, &m.id)
}

pub fn delete_deploy_module(conn: &Connection, module_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM deploy_modules WHERE id = ?", [module_id])?;
    Ok(())
}

fn get_deploy_module_by_id(
    conn: &Connection,
    module_id: &str,
) -> Result<Option<DeployModule>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM deploy_modules WHERE id = ?")?;
    stmt.query_row([module_id], row_to_deploy_module)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
}

// Deploy logs
pub fn add_deploy_log(conn: &Connection, log: &DeployLog) -> Result<DeployLog, rusqlite::Error> {
    conn.execute(
        "INSERT INTO deploy_logs (id, configId, status, startTime, endTime, \
         errorMessage, progress, triggeredBy, createdAt, logFilePath, artifactPaths, environment) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            log.id,
            log.config_id,
            log.status,
            log.start_time,
            log.end_time,
            log.error_message,
            log.progress,
            log.triggered_by,
            log.created_at,
            log.log_file_path,
            log.artifact_paths,
            log.environment
        ],
    )?;
    get_deploy_log_by_id(conn, &log.id).map(|opt| opt.unwrap())
}

pub fn update_deploy_log(
    conn: &Connection,
    log: &DeployLog,
) -> Result<Option<DeployLog>, rusqlite::Error> {
    conn.execute(
        "UPDATE deploy_logs SET configId=?, status=?, startTime=?, endTime=?, \
         errorMessage=?, progress=?, triggeredBy=?, createdAt=?, logFilePath=?, artifactPaths=?, environment=? \
         WHERE id=?",
        params![
            log.config_id,
            log.status,
            log.start_time,
            log.end_time,
            log.error_message,
            log.progress,
            log.triggered_by,
            log.created_at,
            log.log_file_path,
            log.artifact_paths,
            log.environment,
            log.id
        ],
    )?;
    get_deploy_log_by_id(conn, &log.id)
}

pub fn get_deploy_log_by_id(
    conn: &Connection,
    log_id: &str,
) -> Result<Option<DeployLog>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM deploy_logs WHERE id = ?")?;
    stmt.query_row([log_id], row_to_deploy_log)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
}

// Deploy step logs
#[allow(dead_code)]
pub fn add_deploy_step_log(conn: &Connection, step: &DeployStepLog) -> Result<(), rusqlite::Error> {
    // 不写 id：该列是 TEXT 主键，写入整数会被列亲和性变成文本，读取侧再按 i64 取值又会失败；
    // 保持 NULL（与几十万存量行一致），身份标识由 deployLogId + rowid 承担
    conn.execute(
        "INSERT INTO deploy_step_logs (deployLogId, stage, status, message, timestamp) \
         VALUES (?, ?, ?, ?, ?)",
        params![
            step.deploy_log_id,
            step.stage,
            step.status,
            step.message,
            step.timestamp
        ],
    )?;
    Ok(())
}

pub fn get_deploy_step_logs(
    conn: &Connection,
    deploy_log_id: &str,
) -> Result<Vec<DeployStepLog>, rusqlite::Error> {
    // 排序用 rowid：id 全为 NULL 时按 id 排序等于没排序，rowid 才等于写入（即部署阶段）顺序
    let mut stmt =
        conn.prepare("SELECT * FROM deploy_step_logs WHERE deployLogId = ? ORDER BY rowid ASC")?;
    stmt.query_map([deploy_log_id], row_to_deploy_step_log)
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

// Deploy history
pub fn add_deploy_history(
    conn: &Connection,
    h: &DeployHistory,
) -> Result<DeployHistory, rusqlite::Error> {
    conn.execute(
        "INSERT INTO deploy_history (id, configId, status, deployedAt, rolledBack, rolledBackAt) \
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            h.id,
            h.config_id,
            h.status,
            h.deployed_at,
            if h.rolled_back { 1 } else { 0 },
            h.rolled_back_at
        ],
    )?;
    get_deploy_history_by_id(conn, &h.id).map(|opt| opt.unwrap())
}

fn get_deploy_history_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<DeployHistory>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM deploy_history WHERE id = ?")?;
    stmt.query_row([id], row_to_deploy_history)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
}

#[cfg(test)]
mod step_log_tests {
    use super::*;
    use crate::db::Database;

    fn temp_db(tag: &str) -> Database {
        let dir = std::env::temp_dir().join(format!(
            "st_step_log_{}_{}",
            tag,
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        Database::new(&path).unwrap()
    }

    /// 回归：id 列是 TEXT 主键且插入从不写 id（全 NULL），按 i64 强转会静默丢行 → 步骤日志永远为空
    #[test]
    fn null_id_rows_are_still_returned_in_insert_order() {
        let mut db = temp_db("null_id");
        for (i, stage) in ["git", "build", "ssh"].iter().enumerate() {
            add_deploy_step_log(
                db.conn(),
                &DeployStepLog {
                    id: 0,
                    deploy_log_id: "d1".to_string(),
                    stage: stage.to_string(),
                    status: if i == 2 { "failed".to_string() } else { "success".to_string() },
                    message: Some(format!("第{}步", i + 1)),
                    timestamp: format!("2026-08-27T0{}:00:00Z", 6 + i),
                },
            )
            .unwrap();
        }
        let rows = get_deploy_step_logs(db.conn(), "d1").unwrap();
        assert_eq!(rows.len(), 3, "id 为 NULL 的行也必须读出来");
        assert_eq!(
            rows.iter().map(|r| r.stage.as_str()).collect::<Vec<_>>(),
            vec!["git", "build", "ssh"],
            "必须按写入顺序返回（阶段顺序是诊断依据）"
        );
        assert_eq!(rows[2].message.as_deref(), Some("第3步"));

        // 只取本部署记录的步骤
        add_deploy_step_log(
            db.conn(),
            &DeployStepLog {
                id: 0,
                deploy_log_id: "d2".to_string(),
                stage: "git".to_string(),
                status: "success".to_string(),
                message: None,
                timestamp: "x".to_string(),
            },
        )
        .unwrap();
        assert_eq!(get_deploy_step_logs(db.conn(), "d1").unwrap().len(), 3);
    }
}
