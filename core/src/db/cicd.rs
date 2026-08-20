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
}

fn default_true() -> bool {
    true
}

fn default_retries() -> i64 {
    3
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
        id: row.get(0)?,
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
         parentBuildMode, parentBuildPath, requiresApproval, gitRepoId, buildMode, environments, incrementalUpload, healthCheckRetries) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            c.id, &c.name, c.deploy_branch, c.maven_settings, c.maven_profile,
            c.deploy_path, if c.lib_separate { 1 } else { 0 }, c.restart_script, c.health_check_url,
            c.health_check_timeout, c.created_at, c.updated_at, c.build_tool, c.build_command,
            c.build_path, c.repo_url, c.local_path, c.npm_script, c.npm_custom_script,
            c.maven_home, c.npm_home, c.pnpm_home, c.yarn_home, c.java_home, c.node_home, c.servers, c.group_name,
            if c.parent_build_mode { 1 } else { 0 }, c.parent_build_path,
            if c.requires_approval { 1 } else { 0 },
            c.git_repo_id, c.build_mode, c.environments,
            if c.incremental_upload { 1 } else { 0 }, c.health_check_retries
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
         parentBuildPath=?, requiresApproval=?, gitRepoId=?, buildMode=?, environments=?, incrementalUpload=?, healthCheckRetries=? WHERE id=?",
        params![
            &c.name, c.deploy_branch, c.maven_settings, c.maven_profile,
            c.deploy_path, if c.lib_separate { 1 } else { 0 }, c.restart_script,
            c.health_check_url, c.health_check_timeout, c.updated_at, c.build_tool,
            c.build_command, c.build_path, c.repo_url, c.local_path, c.npm_script,
            c.npm_custom_script, c.maven_home, c.npm_home, c.pnpm_home, c.yarn_home,
            c.java_home, c.node_home, c.servers, c.group_name, if c.parent_build_mode { 1 } else { 0 },
            c.parent_build_path, if c.requires_approval { 1 } else { 0 },
            c.git_repo_id, c.build_mode, c.environments,
            if c.incremental_upload { 1 } else { 0 }, c.health_check_retries, c.id
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
    let mut stmt =
        conn.prepare("SELECT * FROM deploy_step_logs WHERE deployLogId = ? ORDER BY id ASC")?;
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
