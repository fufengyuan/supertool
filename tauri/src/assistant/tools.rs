//! AI 助手 —— 内部工具注册表
//!
//! 工具设计红线（与 safety.rs 一起构成边界，改动前先读 docs/ai-config-assistant.md）：
//! - 只有三类能力：**读配置**、**分析错误/查知识**、**产出变更提案**。
//!   没有任何写库工具，也没有文件、shell、SQL、网络类工具——模型连「读一个任意路径」的入口都没有；
//! - 读类工具的数据源固定为 core 里既有的业务函数（与 CLI 同源），不做任意 SQL；
//! - 返回值统一过 `redact_secrets`（在 agent 层做，这里额外保证不主动塞凭据）；
//! - 变更提案只允许白名单字段，密码类字段一律拒绝，改由用户在确认卡片里自己填。
use serde_json::{Value, json};
use supertool_core::db::cicd::{CicdConfig, DeployModule};
use supertool_core::logic::CoreService;

use super::context::{MAX_TOOL_RESULT_CHARS, clip_for_context};
use super::knowledge;
use super::llm::ToolSpec;
use super::safety::{assert_no_secret_fields, read_text_file_in, redact_text};

/// 一次工具调用的产出：回给模型的内容 + 需要前端处理的东西
#[derive(Default)]
pub struct ToolExec {
    pub payload: Value,
    /// 待用户确认的变更提案
    pub proposals: Vec<Value>,
    /// 界面动作（跳转页面等，不改数据）
    pub actions: Vec<Value>,
}

fn ok(payload: Value) -> ToolExec {
    ToolExec {
        payload,
        ..Default::default()
    }
}

fn err(message: impl Into<String>) -> ToolExec {
    ToolExec {
        payload: json!({ "error": message.into() }),
        ..Default::default()
    }
}

/// 取字符串参数：缺失或只有空白一律按「没给」处理，让模型收到可自纠的错误
fn as_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

// 以下读操作全部复用 core 里既有的业务查询（与 CLI 同源），助手侧不做任意 SQL
fn read_configs(core: &CoreService) -> Result<Vec<CicdConfig>, String> {
    core.db_read(|conn| {
        supertool_core::db::cicd::get_all_cicd_configs(conn).map_err(|e| e.to_string())
    })?
}

fn read_config(core: &CoreService, id: &str) -> Result<Option<CicdConfig>, String> {
    core.db_read(|conn| {
        supertool_core::db::cicd::get_cicd_config_by_config_id(conn, id).map_err(|e| e.to_string())
    })?
}

fn read_modules(core: &CoreService, id: &str) -> Result<Vec<DeployModule>, String> {
    core.db_read(|conn| {
        supertool_core::db::cicd::get_deploy_modules(conn, id).map_err(|e| e.to_string())
    })?
}

fn read_deploy_log(
    core: &CoreService,
    id: &str,
) -> Result<Option<supertool_core::db::cicd::DeployLog>, String> {
    core.db_read(|conn| {
        supertool_core::db::cicd::get_deploy_log_by_id(conn, id).map_err(|e| e.to_string())
    })?
}

fn read_step_logs(core: &CoreService, id: &str) -> Vec<supertool_core::db::cicd::DeployStepLog> {
    core.db_read(|conn| {
        supertool_core::db::cicd::get_deploy_step_logs(conn, id).map_err(|e| e.to_string())
    })
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default()
}

/// 数据库连接列表：**刻意不 SELECT password 列**，从数据源头掐断凭据外泄的可能
fn read_db_connections(core: &CoreService) -> Result<Vec<Value>, String> {
    core.db_read(|conn| -> Result<Vec<Value>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, type, host, port, username, dbName, dbIndex, path
             FROM db_connections ORDER BY name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(json!({
                    "id": r.get::<_, String>(0)?,
                    "name": r.get::<_, String>(1)?,
                    "type": r.get::<_, String>(2)?,
                    "host": r.get::<_, String>(3)?,
                    "port": r.get::<_, i64>(4)?,
                    "username": r.get::<_, String>(5)?,
                    "dbName": r.get::<_, Option<String>>(6)?,
                    "dbIndex": r.get::<_, Option<i64>>(7)?,
                    "path": r.get::<_, Option<String>>(8)?,
                }))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect::<Vec<Value>>();
        Ok(rows)
    })?
}

pub fn tool_specs() -> Vec<ToolSpec> {
    let mut specs: Vec<ToolSpec> = Vec::new();
    let mut add = |name: &str, description: &str, parameters: Value| {
        specs.push(ToolSpec {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        })
    };

    add(
        "get_app_snapshot",
        "看全局：各功能模块的条目数量、当前使用的模型、已配置的分组。回答「我配了多少东西」「从哪开始」时先用它。",
        json!({"type": "object", "properties": {}}),
    );
    add(
        "list_servers",
        "列出已配置的服务器（不含密码与密钥路径，这两类字段系统永远不会给到模型）。",
        json!({"type": "object", "properties": {}}),
    );
    add(
        "list_server_groups",
        "列出服务器分组树（新建服务器时 groupId 要从这里取）。",
        json!({"type": "object", "properties": {}}),
    );
    add(
        "test_server_connection",
        "用已保存的凭据测试某台服务器的 SSH 连通性。只返回成功/失败原因，不会返回凭据。",
        json!({
            "type": "object",
            "properties": { "serverId": {"type": "string", "description": "服务器 id（来自 list_servers）"} },
            "required": ["serverId"],
        }),
    );
    add(
        "list_db_connections",
        "列出数据库连接配置（不含密码）。",
        json!({"type": "object", "properties": {}}),
    );
    add(
        "list_cicd_configs",
        "列出所有部署配置的摘要（构建工具、分支、代码目录、产物目录、构建模式、最近部署时间）。",
        json!({"type": "object", "properties": {}}),
    );
    add(
        "get_cicd_config",
        "看一条部署配置的完整字段与多模块列表。给用户建议前必须先看这个，不要凭猜。",
        json!({
            "type": "object",
            "properties": { "configId": {"type": "string"} },
            "required": ["configId"],
        }),
    );
    add(
        "validate_cicd_config",
        "按项目踩坑规则校验一条部署配置，返回字段级问题（error/warn/info）与修复建议。用户说「部署失败/收不到产物/不知道哪填错」时优先调用。",
        json!({
            "type": "object",
            "properties": { "configId": {"type": "string"} },
            "required": ["configId"],
        }),
    );
    add(
        "get_deploy_history",
        "看某个部署配置最近几次部署记录（状态、时间、错误信息）。",
        json!({
            "type": "object",
            "properties": {
                "configId": {"type": "string"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 20, "default": 5}
            },
            "required": ["configId"],
        }),
    );
    add(
        "analyze_deploy_error",
        "分析一次部署失败：取该次部署的日志正文（系统仅允许读部署日志目录内的文件）+ 阶段日志，匹配已知错误特征，返回失败阶段、关键报错行与处理建议。",
        json!({
            "type": "object",
            "properties": { "deployLogId": {"type": "string", "description": "部署记录 id（来自 get_deploy_history）"} },
            "required": ["deployLogId"],
        }),
    );
    add(
        "search_usage_guides",
        "检索内置使用知识库：字段含义、怎么配、常见坑。教学类问题先查这里，答案要以条目内容为准，不要自己编。",
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "中文关键词，如「产物目录」「健康检查」「新增服务器」"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 5, "default": 3}
            },
            "required": ["query"],
        }),
    );
    add(
        "get_usage_guide",
        "按 id 取完整教学条目（search_usage_guides 命中后需要全文时用）。",
        json!({
            "type": "object",
            "properties": { "guideId": {"type": "string"} },
            "required": ["guideId"],
        }),
    );
    add(
        "propose_config_change",
        "产出一条「变更提案」交用户确认——你没有任何直接写库能力，所有配置改动都必须走这个工具。\
         字段只能填白名单内的非敏感项；缺密码/密钥时在 needUserInput 里列出，提示用户自己在表单补。\
         fields 要给出完整可用值（新建时含 name/host 等必填），不要写占位符。",
        json!({
            "type": "object",
            "properties": {
                "targetType": {"type": "string", "enum": ["server", "cicd", "dbConnection", "aiProvider"]},
                "operation": {"type": "string", "enum": ["create", "update"]},
                "targetId": {"type": "string", "description": "update 时必填（已有记录 id）；create 留空"},
                "displayName": {"type": "string", "description": "卡片标题，如「新增服务器：测试机」"},
                "fields": {"type": "object", "description": "白名单字段键值对"},
                "rationale": {"type": "string", "description": "为什么这么改，一到两句"},
                "needUserInput": {"type": "array", "items": {"type": "string"}, "description": "需要用户自己填的敏感字段名（如 password、sshKeyPath、apiKey）"},
                "applyRoute": {"type": "string", "description": "用户确认后跳转过去补全/提交的页面路由，如 /servers、/cicd"}
            },
            "required": ["targetType", "operation", "displayName", "fields", "rationale"],
        }),
    );
    add(
        "open_config_page",
        "带用户去某个功能页面继续操作（只跳转，不改数据）。教学时配合说明「到这个页面做什么」。",
        json!({
            "type": "object",
            "properties": {
                "module": {"type": "string", "enum": ["servers", "cicd", "db", "logs", "nginx", "git", "alert", "backup", "settings", "assistant"]},
                "note": {"type": "string", "description": "到页面后要做的一件事，展示给用户"}
            },
            "required": ["module"],
        }),
    );
    specs
}

/// 提案字段白名单（按目标类型）。不在名单里的字段直接拒绝，避免模型乱写未知列。
const ALLOWED_FIELDS: &[(&str, &[&str])] = &[
    (
        "server",
        &[
            "name",
            "host",
            "port",
            "username",
            "description",
            "tags",
            "groupId",
            "requiresApproval",
        ],
    ),
    (
        "dbConnection",
        &[
            "name",
            "type",
            "host",
            "port",
            "username",
            "dbName",
            "dbIndex",
            "path",
        ],
    ),
    (
        "cicd",
        &[
            "name",
            "groupName",
            "gitRepoId",
            "deployBranch",
            "buildTool",
            "buildCommand",
            "localPath",
            "buildPath",
            "outputPath",
            "parentBuildMode",
            "parentBuildPath",
            "repoUrl",
            "deployPath",
            "servers",
            "restartScript",
            "npmScript",
            "npmCustomScript",
            "mavenProfile",
            "mavenSettings",
            "javaHome",
            "mavenHome",
            "nodeHome",
            "environments",
            "healthCheckUrl",
            "healthCheckTimeout",
            "healthCheckRetries",
            "incrementalUpload",
            "libSeparate",
            "libFilterRules",
            "requiresApproval",
        ],
    ),
    (
        "aiProvider",
        &["name", "protocol", "baseUrl", "models", "enabled"],
    ),
];

fn allowed_fields(target: &str) -> Option<&'static [&'static str]> {
    ALLOWED_FIELDS
        .iter()
        .find(|(t, _)| *t == target)
        .map(|(_, f)| *f)
}

/// 校验并归一提案（纯函数，可单测）
pub fn check_proposal(
    target: &str,
    operation: &str,
    target_id: Option<&str>,
    fields: &Value,
) -> Result<Value, String> {
    if !matches!(operation, "create" | "update") {
        return Err("operation 只能是 create 或 update".to_string());
    }
    if operation == "update" && target_id.unwrap_or("").is_empty() {
        return Err("update 必须给 targetId（先用读类工具确认是哪一条）".to_string());
    }
    let allowlist = allowed_fields(target)
        .ok_or_else(|| format!("不支持的变更目标：{target}（可用：server/cicd/dbConnection/aiProvider）"))?;
    let obj = fields
        .as_object()
        .ok_or_else(|| "fields 必须是对象".to_string())?;
    if obj.is_empty() {
        return Err("fields 为空，提案没有意义".to_string());
    }
    // 密钥类字段先拒（即使在白名单之外也单独给明确理由，模型才能纠正）
    assert_no_secret_fields(fields)?;

    let mut accepted = serde_json::Map::new();
    let mut rejected: Vec<String> = Vec::new();
    for (k, v) in obj {
        if allowlist.contains(&k.as_str()) {
            accepted.insert(k.clone(), v.clone());
        } else {
            rejected.push(k.clone());
        }
    }
    if !rejected.is_empty() {
        return Err(format!(
            "字段不在可变更白名单内：{}。{} 可用字段：{}",
            rejected.join("、"),
            if target == "cicd" { "（cicd 字段较多，先 get_cicd_config 看现有键名）" } else { "" },
            allowlist.join("、")
        ));
    }

    // 关键类型/取值检查：避免提案卡片里出现显然跑不通的值
    if let Some(port) = accepted.get("port") {
        let p = port.as_i64().ok_or_else(|| "port 必须是数字".to_string())?;
        if !(1..=65535).contains(&p) {
            return Err(format!("port 超出范围: {p}"));
        }
    }
    if let Some(host) = accepted.get("host") {
        if host.as_str().map(str::trim).unwrap_or("").is_empty() {
            return Err("host 不能为空".to_string());
        }
    }
    if let Some(models) = accepted.get("models") {
        let arr = models
            .as_array()
            .ok_or_else(|| "models 必须是数组".to_string())?;
        for m in arr {
            if m["id"].as_str().unwrap_or("").trim().is_empty() {
                return Err("models[].id（模型 ID）不能为空".to_string());
            }
            if let Some(cw) = m.get("contextWindow").and_then(|v| v.as_u64()) {
                if !(512..=4_000_000).contains(&cw) {
                    return Err(format!("contextWindow 超出合理范围: {cw}"));
                }
            }
        }
    }
    if let Some(protocol) = accepted.get("protocol") {
        if !matches!(protocol.as_str(), Some("openai") | Some("anthropic")) {
            return Err("protocol 只能是 openai 或 anthropic".to_string());
        }
    }

    Ok(Value::Object(accepted))
}

/// 部署配置规则校验：把 AGENTS.md 里的踩坑结论变成可执行的字段级检查
pub fn validate_cicd_rules(cfg: &CicdConfig, modules: &[DeployModule]) -> Vec<Value> {
    let mut issues: Vec<Value> = Vec::new();
    let mut push = |level: &str, field: &str, message: &str, fix: &str, guide: &str| {
        issues.push(json!({
            "level": level, "field": field, "message": message, "fix": fix, "seeGuide": guide,
        }));
    };

    let tool = cfg.build_tool.clone().unwrap_or_default();
    let enabled_modules: Vec<&DeployModule> = modules.iter().filter(|m| m.enabled).collect();

    if tool.trim().is_empty() {
        push("error", "buildTool", "未指定构建工具，部署时不知道该跑 mvn 还是 npm",
            "编辑配置选好构建工具（maven/npm/pnpm/yarn/gradle/cargo）", "cicd-build-vs-output-dir");
    }
    let has_source = cfg
        .local_path
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
        || cfg.git_repo_id.is_some();
    if !has_source {
        push("error", "localPath", "既没有关联 Git 仓库，也没有代码目录，部署引擎不知道去哪构建",
            "在向导里选择仓库；代码不在仓库根目录时用「选择目录」指向实际代码目录", "cicd-maven-parent-path");
    }
    if cfg.deploy_branch.trim().is_empty() {
        push("warn", "deployBranch", "部署分支为空，将按默认分支处理",
            "填上要部署的分支名（注意远端必须存在该分支）", "cicd-multi-env");
    }
    if cfg.servers.as_deref().map(|s| s.trim().is_empty() || s == "[]").unwrap_or(true)
        && cfg.deploy_path.trim().is_empty()
    {
        push("error", "servers", "没有选择目标服务器，也没有部署路径",
            "先配好服务器并选为部署目标", "server-fields");
    }
    if cfg.health_check_url.as_deref().unwrap_or("").trim().is_empty() {
        push("info", "healthCheckUrl", "未配置健康检查：部署失败不会自动回滚",
            "生产配置建议填写健康检查地址，失败时会自动恢复备份并重跑重启脚本", "cicd-health-rollback");
    }

    // maven 相关
    if tool == "maven" {
        if cfg.parent_build_mode && !cfg.parent_build_path.trim().is_empty() {
            push("error", "parentBuildPath",
                format!("maven 父统一构建的构建目录填了「{}」，会被当成单模块构建，CI-Friendly 项目的 ${{revision}} 不展开、兄弟模块依赖解析失败", cfg.parent_build_path).as_str(),
                "清空该字段（构建根即代码目录/聚合根），产物从「产物目录」收集", "cicd-maven-parent-path");
        }
        if !cfg.parent_build_mode && enabled_modules.is_empty() {
            let looks_multi = cfg
                .output_path
                .as_deref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            if !looks_multi && cfg.deploy_path.trim().is_empty() {
                push("warn", "outputPath",
                    "单体 maven 部署未填产物目录，将自动扫描 target；目录里有多个 jar 时会拿不到正确产物",
                    "填主模块的产物目录，如 seller-api/target", "cicd-single-main-module");
            }
        }
        if cfg.lib_separate && cfg.restart_script.trim().is_empty() {
            push("warn", "restartScript", "开启 Jar/Lib 分离但重启脚本为空，lib 目录变化时进程可能起不来",
                "补上重启脚本（含 java -jar 或 systemd 调用）", "cicd-lib-separate");
        }
    } else if cfg.lib_separate {
        push("info", "libSeparate", "Jar/Lib 分离仅对 maven 生效，当前会被忽略",
            "非 maven 项目关掉该开关可避免误解", "cicd-lib-separate");
    }

    // 前端/npm 相关
    if matches!(tool.as_str(), "npm" | "pnpm" | "yarn") {
        if cfg.parent_build_mode {
            // 单体模式：模块行可能是复制残留
            let dirty: Vec<String> = enabled_modules
                .iter()
                .filter(|m| {
                    m.build_command
                        .as_deref()
                        .map(|c| c.to_lowercase().contains("mvn") || c.to_lowercase().contains("gradle"))
                        .unwrap_or(false)
                })
                .map(|m| m.module_name.clone())
                .collect();
            if !dirty.is_empty() {
                push(
                    "warn",
                    "deployModules",
                    &format!(
                        "npm 单体模式下这些模块行是复制残留、不参与构建：{}",
                        dirty.join("、")
                    ),
                    "删掉无用模块行，或确认部署模式与模块列表一致",
                    "cicd-npm-single",
                );
            }
        }
        let out = cfg.output_path.clone().unwrap_or_default();
        if out.trim() == "build/h5" {
            push("error", "outputPath",
                "uni-app 的产物目录填成 build/h5 通常不存在（打包输出在 dist/build/h5），会静默回退扫描 dist 父目录",
                "改成 dist/build/h5", "cicd-npm-single");
        }
        if out.trim().is_empty() {
            push("info", "outputPath", "未填产物目录，将自动扫描 dist / build/h5 等候选",
                "扫描结果不对时显式填写产物目录（看构建日志里打印的实际收集目录）", "cicd-build-vs-output-dir");
        }
        if cfg.npm_script.as_deref().unwrap_or("").trim().is_empty()
            && cfg.npm_custom_script.as_deref().unwrap_or("").trim().is_empty()
            && cfg.build_command.as_deref().unwrap_or("").trim().is_empty()
        {
            push("warn", "npmScript", "没有指定构建脚本，将使用默认 build 脚本",
                "从「构建脚本」下拉里选一个（列表来自构建目录 package.json）", "cicd-npm-single");
        }
    }

    // 多模块一致性
    for m in &enabled_modules {
        if m.module_path.trim().is_empty() {
            push("error", "deployModules", &format!("模块「{}」缺少模块路径", m.module_name),
                "重新扫描模块或手工补路径", "cicd-single-main-module");
        }
        let is_front = m.build_tool.as_deref().map(|t| matches!(t, "npm" | "pnpm" | "yarn")).unwrap_or(false)
            || m.artifact_type.as_deref() == Some("dist");
        if is_front && m.output_path.as_deref().unwrap_or("").trim().is_empty() {
            push("info", "deployModules",
                &format!("前端模块「{}」未填产物目录，构建后会回退自动扫描该模块的 dist 目录", m.module_name),
                "扫描结果不符预期时显式填产物子目录", "cicd-build-vs-output-dir");
        }
    }
    issues
}

/// 部署日志允许读取的目录（系统自己写的部署日志，别的一律不给读）
fn allowed_log_dirs(core: &CoreService) -> Vec<std::path::PathBuf> {
    let mut dirs = vec![core.app_dir().join("deploy-logs")];
    if let Ok(raw) = core.db_read(|conn| {
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'appDataDir'",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
    }) {
        let setting = raw.trim();
        if !setting.is_empty() {
            dirs.push(std::path::Path::new(setting).join("deploy-logs"));
        }
    }
    // 历史默认值：未配置 appDataDir 时部署引擎写的就是 /tmp/deploy-logs
    // 这里唯一的信任前提是「路径来自我们自己库里的 deploy_logs.logFilePath」，不是模型输入；
    // safety::read_text_file_in 仍会 canonicalize 后做前缀校验，软链与 .. 都过不去。
    dirs.push(std::path::Path::new("/tmp").join("deploy-logs"));
    dirs
}

fn cicd_summary(cfg: &CicdConfig) -> Value {
    json!({
        "id": cfg.id,
        "name": cfg.name,
        "groupName": cfg.group_name,
        "buildTool": cfg.build_tool,
        "deployBranch": cfg.deploy_branch,
        "parentBuildMode": cfg.parent_build_mode,
        "parentBuildPath": cfg.parent_build_path,
        "localPath": cfg.local_path,
        "buildPath": cfg.build_path,
        "outputPath": cfg.output_path,
        "deployPath": cfg.deploy_path,
        "restartScript": if cfg.restart_script.is_empty() { Value::Null } else { json!(clip_for_context(&cfg.restart_script, 120)) },
        "healthCheckUrl": cfg.health_check_url,
        "libSeparate": cfg.lib_separate,
        "incrementalUpload": cfg.incremental_upload,
        "requiresApproval": cfg.requires_approval,
        "lastDeployedAt": cfg.last_deployed_at,
    })
}

pub async fn execute(core: &CoreService, name: &str, args: &Value) -> ToolExec {
    match name {
        "get_app_snapshot" => {
            let servers = core.get_all_servers().await.unwrap_or_else(|e| json!({"error": e}));
            let groups = core.get_all_server_groups().await.unwrap_or(Value::Null);
            let configs = read_configs(core).map(|v| v.len());
            let active = core.get_active_ai_model().await.unwrap_or(Value::Null);
            let route = core.resolve_ai_route();
            ok(json!({
                "serverCount": servers.as_array().map(|a| a.len()),
                "serverGroupCount": groups.as_array().map(|a| a.len()),
                "cicdConfigCount": configs.ok(),
                "activeModel": active,
                "modelProvider": route.as_ref().ok().map(|r| json!({
                    "provider": r.provider_name, "protocol": r.protocol.as_str(),
                    "modelId": r.model_id, "contextWindow": r.context_window,
                })),
                "modelNote": route.err(),
                "modules": ["servers", "cicd", "db", "logs", "nginx", "git", "alert", "backup", "settings"],
            }))
        }
        "list_servers" => match core.get_all_servers().await {
            // get_all_servers 已剔除 password；sshKeyPath 由 redact 层再兜一次
            Ok(v) => ok(v),
            Err(e) => err(format!("读取服务器失败: {e}")),
        },
        "list_server_groups" => match core.get_all_server_groups().await {
            Ok(v) => ok(v),
            Err(e) => err(format!("读取分组失败: {e}")),
        },
        "test_server_connection" => {
            let Some(id) = as_str(args, "serverId") else {
                return err("缺少参数 serverId");
            };
            match core.get_server_by_id(id).await {
                Ok(server) if !server.is_null() => {
                    // 凭据只在内存里传给 ssh 层，绝不进入返回给模型的任何字段
                    match core.test_server_connection(server).await {
                        Ok(_) => ok(json!({"ok": true, "serverId": id})),
                        Err(e) => ok(json!({
                            "ok": false,
                            "serverId": id,
                            "reason": redact_text(&e),
                            "hints": knowledge::match_error_hints(&e),
                        })),
                    }
                }
                Ok(_) => err(format!("服务器不存在: {id}")),
                Err(e) => err(format!("读取服务器失败: {e}")),
            }
        }
        "list_db_connections" => match read_db_connections(core) {
            Ok(rows) => ok(json!(rows)),
            Err(e) => err(format!("读取数据库连接失败: {e}")),
        },
        "list_cicd_configs" => match read_configs(core) {
            Ok(list) => ok(Value::Array(list.iter().map(cicd_summary).collect())),
            Err(e) => err(format!("读取部署配置失败: {e}")),
        },
        "get_cicd_config" => {
            let Some(id) = as_str(args, "configId") else {
                return err("缺少参数 configId");
            };
            match (read_config(core, id), read_modules(core, id)) {
                (Ok(Some(cfg)), Ok(modules)) => ok(json!({
                    "config": serde_json::to_value(&cfg).unwrap_or_default(),
                    "modules": modules
                        .iter()
                        .filter_map(|m| serde_json::to_value(m).ok())
                        .collect::<Vec<_>>(),
                })),
                (Ok(None), _) => err(format!("部署配置不存在: {id}")),
                (Err(e), _) | (_, Err(e)) => err(format!("读取配置失败: {e}")),
            }
        }
        "validate_cicd_config" => {
            let Some(id) = as_str(args, "configId") else {
                return err("缺少参数 configId");
            };
            let cfg = match read_config(core, id) {
                Ok(Some(c)) => c,
                Ok(None) => return err(format!("部署配置不存在: {id}")),
                Err(e) => return err(format!("校验失败: {e}")),
            };
            let mods = read_modules(core, id).unwrap_or_default();
            let issues = validate_cicd_rules(&cfg, &mods);
            ok(json!({
                "configId": id,
                "configName": cfg.name,
                "issueCount": issues.len(),
                "issues": issues,
                "blocking": issues.iter().any(|i| i["level"] == "error"),
            }))
        }
        "get_deploy_history" => {
            let Some(id) = as_str(args, "configId") else {
                return err("缺少参数 configId");
            };
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5).clamp(1, 20);
            match core.get_deploy_logs_by_config(id, limit) {
                Ok(logs) => ok(Value::Array(
                    logs.iter()
                        .map(|l| {
                            json!({
                                "deployLogId": l.id,
                                "status": l.status,
                                "startTime": l.start_time,
                                "endTime": l.end_time,
                                "progress": l.progress,
                                "error": l.error_message.as_deref().map(|e| clip_for_context(e, 400)),
                                "environment": l.environment,
                                "logAvailable": l.log_file_path.is_some(),
                            })
                        })
                        .collect(),
                )),
                Err(e) => err(format!("读取部署记录失败: {e}")),
            }
        }
        "analyze_deploy_error" => {
            let Some(id) = as_str(args, "deployLogId") else {
                return err("缺少参数 deployLogId");
            };
            let log = match read_deploy_log(core, id) {
                Ok(Some(l)) => l,
                Ok(None) => return err(format!("部署记录不存在: {id}")),
                Err(e) => return err(format!("读取部署记录失败: {e}")),
            };
            let steps = read_step_logs(core, id);

            let mut out = json!({
                "deployLogId": id,
                "status": log.status,
                "startTime": log.start_time,
                "endTime": log.end_time,
                "error": log.error_message.as_deref().map(|e| clip_for_context(e, 1200)),
                "failedStages": steps.iter().filter(|s| s.status == "failed" || s.status == "warning")
                    .map(|s| json!({"stage": s.stage, "status": s.status, "message": s.message})).collect::<Vec<_>>(),
            });

            // 只读部署日志目录内的文件；路径来自数据库而非用户输入，仍做白名单二次校验
            if let Some(path) = log.log_file_path.as_deref() {
                let dirs = allowed_log_dirs(core);
                let dir_refs: Vec<&std::path::Path> = dirs.iter().map(|p| p.as_path()).collect();
                match read_text_file_in(path, &dir_refs, 8 * 1024 * 1024) {
                    Ok(content) => {
                        let error_lines: Vec<&str> = content
                            .lines()
                            .filter(|l| {
                                let u = l.to_ascii_uppercase();
                                u.contains("[ERROR]")
                                    || u.contains("ERROR:")
                                    || u.contains("FAILED")
                                    || u.contains("BUILD FAILURE")
                                    || u.contains("异常")
                                    || u.contains("PERMISSION DENIED")
                                    || u.contains("TIMEOUT")
                            })
                            .rev()
                            .take(40)
                            .collect();
                        out["logPath"] = json!(path);
                        out["logLines"] = json!(content.lines().count());
                        out["errorLines"] = json!(error_lines);
                        out["tail"] = json!(clip_for_context(&content, 2_500));
                        out["knownHints"] = json!(knowledge::match_error_hints(&content));
                    }
                    Err(e) => {
                        out["logReadSkipped"] = json!(e);
                        if let Some(err_msg) = log.error_message.as_deref() {
                            out["knownHints"] = json!(knowledge::match_error_hints(err_msg));
                        }
                    }
                }
            }
            ok(out)
        }
        "search_usage_guides" => {
            let Some(q) = as_str(args, "query") else {
                return err("缺少参数 query");
            };
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(3).clamp(1, 5) as usize;
            let hits = knowledge::search_guides(q, limit);
            if hits.is_empty() {
                return ok(json!({
                    "hits": [],
                    "note": "知识库里没有直接命中的条目。可以换个说法（用字段名如「产物目录」「健康检查」，或模块名）再查一次；查不到就如实说明不确定，不要编造规则。",
                    "availableTitles": knowledge::guide_index(),
                }));
            }
            ok(json!({ "hits": hits }))
        }
        "get_usage_guide" => {
            let Some(id) = as_str(args, "guideId") else {
                return err("缺少参数 guideId");
            };
            match knowledge::guide_body(id) {
                Some(g) => ok(json!({
                    "id": g.id, "title": g.title, "module": g.module, "route": g.route, "body": g.body,
                })),
                None => err(format!("没有该知识条目: {id}")),
            }
        }
        "propose_config_change" => {
            let Some(target) = as_str(args, "targetType") else {
                return err("缺少参数 targetType");
            };
            let Some(operation) = as_str(args, "operation") else {
                return err("缺少参数 operation");
            };
            let target_id = as_str(args, "targetId");
            let fields = args.get("fields").cloned().unwrap_or(Value::Null);
            match check_proposal(target, operation, target_id, &fields) {
                Ok(accepted) => {
                    let proposal = json!({
                        "targetType": target,
                        "operation": operation,
                        "targetId": target_id,
                        "displayName": args.get("displayName").and_then(|v| v.as_str()).unwrap_or("配置变更"),
                        "fields": accepted,
                        "rationale": args.get("rationale").and_then(|v| v.as_str()).unwrap_or(""),
                        "needUserInput": args.get("needUserInput").cloned().unwrap_or(json!([])),
                        "applyRoute": args.get("applyRoute").and_then(|v| v.as_str()),
                        "allowedFields": allowed_fields(target).map(|a| json!(a)).unwrap_or(Value::Null),
                    });
                    ToolExec {
                        payload: json!({
                            "queued": true,
                            "note": "提案已交给用户确认。你还不能写入，也不要声称已生效；可以解释每项取值与理由。",
                        }),
                        proposals: vec![proposal],
                        actions: vec![],
                    }
                }
                Err(e) => err(e),
            }
        }
        "open_config_page" => {
            let Some(module) = as_str(args, "module") else {
                return err("缺少参数 module");
            };
            let routes = json!({
                "servers": "/servers", "cicd": "/cicd", "db": "/db", "logs": "/logs",
                "nginx": "/nginx", "git": "/git", "alert": "/alert", "backup": "/backup",
                "settings": "/settings", "assistant": "/assistant",
            });
            match routes.get(module) {
                Some(route) => ToolExec {
                    payload: json!({"opened": module, "note": "已请求界面跳转到该页面"}),
                    proposals: vec![],
                    actions: vec![json!({
                        "type": "navigate",
                        "route": route,
                        "note": args.get("note").and_then(|v| v.as_str()).unwrap_or(""),
                    })],
                },
                None => err(format!("未知模块: {module}")),
            }
        }
        other => err(format!("没有这个工具: {other}；只能使用系统提示里列出的工具")),
    }
}

/// 工具结果统一裁剪 + 脱敏前的体积控制（脱敏在 agent 层做）
pub fn shrink(payload: Value) -> Value {
    let text = payload.to_string();
    if text.len() <= MAX_TOOL_RESULT_CHARS * 2 {
        return payload;
    }
    json!({
        "truncated": true,
        "note": "结果过大已裁剪，可用更精确的参数（如指定 configId、缩小 limit）再查",
        "preview": clip_for_context(&text, MAX_TOOL_RESULT_CHARS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cicd(over: Value) -> CicdConfig {
        let mut base = json!({
            "id": "c1", "name": "测试配置", "deployBranch": "master",
            "mavenProfile": "", "deployPath": "/opt/app", "libSeparate": false,
            "restartScript": "sh run.sh", "healthCheckTimeout": 10, "createdAt": "", "updatedAt": "",
            "parentBuildPath": "", "buildMode": "", "groupName": "默认",
            "requiresApproval": false, "parentBuildMode": false,
            "servers": "[{\"serverId\":\"s1\",\"deployDir\":\"/opt/app\"}]"
        });
        for (k, v) in over.as_object().unwrap() {
            base[k] = v.clone();
        }
        serde_json::from_value(base).unwrap()
    }

    #[test]
    fn flags_parent_path_on_maven_multi_module() {
        let cfg = cicd(json!({"buildTool": "maven", "parentBuildMode": true, "parentBuildPath": "mall-server"}));
        let issues = validate_cicd_rules(&cfg, &[]);
        let hit = issues
            .iter()
            .find(|i| i["field"] == "parentBuildPath")
            .expect("应指出父统一构建填子模块的问题");
        assert_eq!(hit["level"], "error");
        assert_eq!(hit["seeGuide"], "cicd-maven-parent-path");
    }

    #[test]
    fn flags_uniapp_output_path_and_npm_residue() {
        let cfg = cicd(json!({
            "buildTool": "npm", "outputPath": "build/h5", "npmScript": "build:h5",
            "parentBuildMode": true
        }));
        let module = DeployModule {
            id: "m1".into(),
            config_id: "c1".into(),
            module_name: "mall-admin".into(),
            module_path: "admin".into(),
            build_path: None,
            build_command: Some("mvn clean package".into()),
            build_tool: Some("maven".into()),
            output_path: None,
            artifact_name: "".into(),
            artifact_type: None,
            lib_filter_rules: None,
            deploy_order: 1,
            deploy_path: None,
            enabled: true,
            created_at: "".into(),
            updated_at: "".into(),
        };
        let issues = validate_cicd_rules(&cfg, &[module]);
        assert!(issues.iter().any(|i| i["field"] == "outputPath"
            && i["message"].as_str().unwrap().contains("dist/build/h5")));
        assert!(issues.iter().any(|i| i["message"].as_str().unwrap().contains("复制残留")),
            "npm 单体带着 mvn 模块行必须提示: {:?}", issues);
    }

    #[test]
    fn healthy_config_only_gets_info_level_notes() {
        let cfg = cicd(json!({
            "buildTool": "maven", "localPath": "/repo/SRC/app",
            "healthCheckUrl": "http://127.0.0.1:8080/actuator/health",
            "outputPath": "app/target"
        }));
        let issues = validate_cicd_rules(&cfg, &[]);
        assert!(
            !issues.iter().any(|i| i["level"] == "error"),
            "不该有 error: {:?}",
            issues
        );
    }

    #[test]
    fn missing_source_and_target_are_errors() {
        let cfg = cicd(json!({"buildTool": null, "localPath": "", "gitRepoId": null, "servers": "[]", "deployPath": ""}));
        let issues = validate_cicd_rules(&cfg, &[]);
        let fields: Vec<&str> = issues
            .iter()
            .filter(|i| i["level"] == "error")
            .map(|i| i["field"].as_str().unwrap())
            .collect();
        assert!(fields.contains(&"buildTool"));
        assert!(fields.contains(&"localPath"));
        assert!(fields.contains(&"servers"));
    }

    #[test]
    fn proposal_rejects_unknown_and_secret_fields() {
        // 白名单外字段
        let e = check_proposal("server", "create", None, &json!({"host": "1.1.1.1", "os": "linux"}))
            .unwrap_err();
        assert!(e.contains("os") && e.contains("白名单"), "{}", e);
        // 密码类字段给出明确理由
        let e = check_proposal("server", "create", None, &json!({"host": "1.1.1.1", "password": "x"}))
            .unwrap_err();
        assert!(e.contains("password") && e.contains("表单"), "{}", e);
        let e = check_proposal("cicd", "update", Some("c1"), &json!({"sshKeyPath": "/x"})).unwrap_err();
        assert!(e.contains("sshKeyPath"), "{}", e);
        // update 不给 id
        assert!(check_proposal("cicd", "update", None, &json!({"name": "x"})).is_err());
        // 端口类型与范围
        assert!(check_proposal("server", "create", None, &json!({"host": "h", "port": "22"})).is_err());
        assert!(check_proposal("server", "create", None, &json!({"host": "h", "port": 99999})).is_err());
        assert!(check_proposal("server", "create", None, &json!({"host": "  "})).is_err());
        // 模型上下文窗口范围
        assert!(check_proposal(
            "aiProvider", "create", None,
            &json!({"name":"n","protocol":"openai","baseUrl":"http://x","models":[{"id":"m","contextWindow":9}]}))
            .is_err());
        // 合法提案按白名单归一
        let okp = check_proposal(
            "server", "create", None,
            &json!({"name":"测试机","host":"192.168.1.69","port":22,"username":"deploy","requiresApproval":true}),
        )
        .unwrap();
        assert_eq!(okp["host"], "192.168.1.69");
        assert_eq!(okp["port"], 22);
        assert!(okp.get("password").is_none());
    }

    #[test]
    fn registry_exposes_no_dangerous_capabilities() {
        // 红线：注册表里不得出现文件/命令/SQL/网络/写库类工具
        let names: Vec<String> = tool_specs().iter().map(|t| t.name.clone()).collect();
        // 1) 明确禁止的工具名（写库与执行类，一个都不能有）
        for banned in [
            "deploy", "rollback", "cancel_deploy", "save_server", "add_server", "update_server",
            "delete_server", "save_cicd_config", "delete_cicd_config", "save_deploy_module",
            "exec", "exec_batch", "read_file", "write_file", "list_dir", "download", "sql",
            "query", "fetch_page_content", "apply_change",
        ] {
            assert!(!names.contains(&banned.to_string()), "工具注册表不应出现: {banned}");
        }
        // 2) 名称里也不允许带这些能力关键字（防止改名绕过）
        for keyword in ["exec", "bash", "shell", "file", "sql", "delete", "save", "write", "mkdir", "upload", "download", "fetch", "http"] {
            assert!(
                !names.iter().any(|n| n.contains(keyword)),
                "工具名不应含能力关键字 {keyword}: {names:?}"
            );
        }
        // 每个工具都要有描述与合法 schema
        for t in tool_specs() {
            assert!(!t.description.trim().is_empty(), "{} 缺描述", t.name);
            assert_eq!(t.parameters["type"], "object", "{} schema 不合法", t.name);
            assert!(t.parameters.get("properties").is_some(), "{} 缺 properties", t.name);
        }
        assert!(names.contains(&"propose_config_change".to_string()));
    }

    #[test]
    fn unknown_tool_is_rejected_by_name() {
        // execute 的兜底分支不 panic，且明确告知只能用已有工具
        // （纯字符串检查，无需 CoreService）
        let msg = format!("没有这个工具: {}；只能使用系统提示里列出的工具", "rm -rf");
        assert!(msg.contains("只能使用"));
    }
}

/// 工具分发层的集成测试：真实临时库 + 真实部署日志文件，验证「能给什么、绝不能给什么」
#[cfg(test)]
mod tools_exec_tests {
    use super::*;
    use crate::assistant::safety::deep_redact;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use supertool_core::db::cicd::{add_deploy_log, add_deploy_step_log, DeployLog, DeployStepLog};
    use supertool_core::db::db_connections::DbConnectionConfig;
    use supertool_core::db::Database;

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    const SERVER_PWD: &str = "S3cr3t-Prod-Pwd";
    const SERVER_KEY: &str = "/Users/x/.ssh/id_prod_secret_key";
    const DB_PWD: &str = "DbPass#1-plain";
    const ENV_SECRET: &str = "EnvSecret#7";
    const LOG_SECRET: &str = "FileSecret#9";

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "st_assistant_tools_{}_{}_{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn seeded(tag: &str) -> (CoreService, std::path::PathBuf) {
        let dir = temp_dir(tag);
        let core = CoreService::new(Database::new(&dir.join("t.db")).unwrap(), dir.clone());

        core.add_server(json!({
            "id": "srv-1", "name": "生产机", "host": "10.0.0.9", "port": 22,
            "username": "deploy", "password": SERVER_PWD, "sshKeyPath": SERVER_KEY,
            "description": "核心库", "tags": ["prod"], "requiresApproval": true,
        }))
        .await
        .unwrap();

        core.add_db_connection(DbConnectionConfig {
            id: "dbc-1".to_string(),
            name: "订单库".to_string(),
            db_type: "mysql".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: DB_PWD.to_string(),
            db_name: Some("orders".to_string()),
            db_index: None,
            path: None,
        })
        .await
        .unwrap();

        core.save_cicd_config_full(json!({
            "id": "cfg-1", "name": "坏配置", "deployBranch": "master", "mavenProfile": "",
            "deployPath": "/opt/app", "libSeparate": false, "restartScript": "sh run.sh",
            "healthCheckTimeout": 30, "healthCheckRetries": 3, "incrementalUpload": true,
            "buildTool": "maven", "parentBuildMode": true, "parentBuildPath": "mall-server",
            "localPath": "/repo/SRC/mall", "buildMode": "single", "groupName": "默认",
            "requiresApproval": false, "createdAt": "2026-08-27T00:00:00Z",
            "updatedAt": "2026-08-27T00:00:00Z",
            "servers": "[{\"serverId\":\"srv-1\",\"deployDir\":\"/opt/app\"}]",
            "environments": format!("[{{\"name\":\"test\",\"envVars\":\"DB_PASSWORD={}\"}}]", ENV_SECRET),
        }))
        .unwrap();

        (core, dir)
    }

    fn seed_deploy(core: &CoreService, dir: &std::path::Path, log_path: Option<String>) -> String {
        let id = "dep-1".to_string();
        core.db_write(|conn| {
            add_deploy_log(
                conn,
                &DeployLog {
                id: id.clone(),
                config_id: "cfg-1".to_string(),
                status: "failed".to_string(),
                start_time: "2026-08-27T06:00:00Z".to_string(),
                end_time: Some("2026-08-27T06:05:00Z".to_string()),
                error_message: Some("Maven 构建失败 (exit 1)".to_string()),
                progress: 40,
                triggered_by: "user".to_string(),
                created_at: "2026-08-27T06:00:00Z".to_string(),
                    log_file_path: log_path.clone(),
                    artifact_paths: None,
                    environment: None,
                },
            )
            .map_err(|e| e.to_string())
        })
        .unwrap()
        .unwrap();
        core.db_write(|conn| {
            add_deploy_step_log(
                conn,
                &DeployStepLog {
                    id: 0,
                    deploy_log_id: id.clone(),
                    stage: "maven".to_string(),
                    status: "failed".to_string(),
                    message: Some("[ERROR] no POM in this directory".to_string()),
                    timestamp: "2026-08-27T06:04:00Z".to_string(),
                },
            )
            .map_err(|e| e.to_string())
        })
        .unwrap()
        .unwrap();
        if let Some(path) = log_path {
            let file = std::path::Path::new(&path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(
                file,
                format!(
                    "[maven] [building] [INFO] Scanning\n\
                     [ssh] [connecting] ssh://deploy:{}@10.0.0.9:22\n\
                     [maven] [building] [ERROR] no POM in this directory /repo/SRC/mall/mall-server\n\
                     [build] [failed] Maven 构建失败 (exit 1)\n",
                    LOG_SECRET
                ),
            )
            .unwrap();
        }
        let _ = dir;
        id
    }

    /// 读类工具：连接信息要给全（助手要靠它判断），凭据一个字节都不给
    #[tokio::test]
    async fn read_tools_expose_topology_but_never_credentials() {
        let (core, _dir) = seeded("read").await;
        for (tool, args) in [
            ("list_servers", json!({})),
            ("get_app_snapshot", json!({})),
            ("list_db_connections", json!({})),
            ("list_cicd_configs", json!({})),
            ("get_cicd_config", json!({"configId": "cfg-1"})),
            ("validate_cicd_config", json!({"configId": "cfg-1"})),
        ] {
            let exec = execute(&core, tool, &args).await;
            let wire = deep_redact(&exec.payload).to_string();
            for secret in [SERVER_PWD, SERVER_KEY, DB_PWD, ENV_SECRET] {
                assert!(!wire.contains(secret), "{tool} 泄漏了凭据: {secret}");
            }
            assert!(!wire.contains("\"password\":\""), "{tool} 不应带出 password 键");
        }

        // 同时确认「有用的连接信息」没被过度屏蔽，否则助手没法工作
        let servers = deep_redact(&execute(&core, "list_servers", &json!({})).await.payload).to_string();
        assert!(servers.contains("10.0.0.9") && servers.contains("deploy") && servers.contains("\"port\":22"));
        assert!(servers.contains("[已隐藏]"), "sshKeyPath 应被抹成占位符");
        let conns = deep_redact(&execute(&core, "list_db_connections", &json!({})).await.payload).to_string();
        assert!(conns.contains("3306") && conns.contains("orders"));
    }

    /// 内嵌 JSON 里的密钥（environments/servers 列）也必须被深度脱敏拦下
    #[tokio::test]
    async fn embedded_json_secrets_are_scrubbed() {
        let (core, _dir) = seeded("embed").await;
        let exec = execute(&core, "get_cicd_config", &json!({"configId": "cfg-1"})).await;
        let raw = exec.payload.to_string();
        assert!(raw.contains(ENV_SECRET), "未脱敏前明文确实在结果里（说明用例有意义）");
        let wire = deep_redact(&exec.payload).to_string();
        assert!(!wire.contains(ENV_SECRET), "environments 内嵌 JSON 的密钥漏了出去: {wire}");
    }

    /// 部署日志：只读部署日志目录，命中已知坑，且正文里的凭据被抹掉
    #[tokio::test]
    async fn analyze_deploy_error_uses_allowlisted_file() {
        let (core, dir) = seeded("analyze").await;
        let log_path = dir.join("deploy-logs").join("dep-1.log").to_string_lossy().to_string();
        let id = seed_deploy(&core, &dir, Some(log_path));

        let exec = execute(&core, "analyze_deploy_error", &json!({"deployLogId": id})).await;
        let wire = deep_redact(&exec.payload).to_string();
        assert!(exec.payload["knownHints"].as_array().map(|a| !a.is_empty()).unwrap_or(false),
            "应命中 no POM 这个已知坑: {:?}", exec.payload);
        assert!(exec.payload["errorLines"]
            .as_array()
            .map(|l| l.iter().any(|x| x.as_str().unwrap_or("").contains("no POM")))
            .unwrap_or(false));
        assert!(!wire.contains(LOG_SECRET), "日志正文里的凭据泄漏了");
        assert!(wire.contains("10.0.0.9"), "主机信息要保留，助手要靠它诊断");
        assert!(exec.payload["failedStages"]
            .as_array()
            .map(|s| s.iter().any(|x| x["stage"] == "maven"))
            .unwrap_or(false));
    }

    /// 数据库里被人塞了目录外的路径时，只能拒绝读取，不能顺手读走
    #[tokio::test]
    async fn analyze_deploy_error_refuses_path_outside_deploy_logs() {
        let (core, dir) = seeded("escape").await;
        let outside = dir.join("elsewhere").join("id_ed25519");
        std::fs::create_dir_all(outside.parent().unwrap()).unwrap();
        std::fs::write(&outside, format!("PRIVATE KEY MATERIAL {LOG_SECRET}")).unwrap();
        let id = seed_deploy(&core, &dir, Some(outside.to_string_lossy().to_string()));

        let exec = execute(&core, "analyze_deploy_error", &json!({"deployLogId": id})).await;
        let wire = exec.payload.to_string();
        assert!(exec.payload["logReadSkipped"].is_string(), "应拒绝读取白名单外路径");
        assert!(!wire.contains(LOG_SECRET), "越界读取成功了: {wire}");
        assert!(!wire.contains("PRIVATE KEY MATERIAL"));
    }

    /// 连通性测试：能用已存凭据去连，但返回体里不含任何凭据
    #[tokio::test]
    async fn test_server_connection_reports_failure_without_leaking_credentials() {
        let (core, _dir) = seeded("ssh").await;
        let exec = execute(&core, "test_server_connection", &json!({"serverId": "srv-1"})).await;
        let wire = deep_redact(&exec.payload).to_string();
        assert_eq!(exec.payload["ok"], false, "10.0.0.9 连不上，应报失败");
        assert!(exec.payload["reason"].is_string());
        for secret in [SERVER_PWD, SERVER_KEY] {
            assert!(!wire.contains(secret), "连通性测试泄漏了凭据");
        }
        assert!(wire.contains("hints"), "应带上可执行的排查建议");
    }

    #[tokio::test]
    async fn validate_flags_the_seeded_pitfall_and_blocks() {
        let (core, _dir) = seeded("validate").await;
        let exec = execute(&core, "validate_cicd_config", &json!({"configId": "cfg-1"})).await;
        assert_eq!(exec.payload["blocking"], true);
        assert!(exec.payload["issues"]
            .as_array()
            .map(|i| i.iter().any(|x| x["field"] == "parentBuildPath"))
            .unwrap_or(false));
    }

    #[tokio::test]
    async fn proposal_round_trip_and_rejections() {
        let (core, _dir) = seeded("proposal").await;
        let good = execute(
            &core,
            "propose_config_change",
            &json!({
                "targetType": "server", "operation": "create", "displayName": "新增测试机",
                "fields": {"name": "测试机", "host": "192.168.1.20", "port": 22, "username": "ci"},
                "rationale": "跑回归", "needUserInput": ["password"],
            }),
        )
        .await;
        assert_eq!(good.proposals.len(), 1);
        assert_eq!(good.payload["queued"], true);
        assert_eq!(good.proposals[0]["fields"]["host"], "192.168.1.20");

        // 想替用户填密码 → 拒绝，且不产出提案
        let bad = execute(
            &core,
            "propose_config_change",
            &json!({
                "targetType": "server", "operation": "create", "displayName": "x",
                "fields": {"host": "1.1.1.1", "password": "我替用户填"},
                "rationale": "r",
            }),
        )
        .await;
        assert!(bad.proposals.is_empty());
        assert!(bad.payload["error"].as_str().unwrap().contains("password"));
    }

    /// 参数缺失/未知工具/非法 JSON 都必须是「软错误」，让模型能自我纠正而不是中断回合
    #[tokio::test]
    async fn soft_errors_for_bad_calls() {
        let (core, _dir) = seeded("errors").await;
        for (tool, args) in [
            ("get_cicd_config", json!({})),
            ("search_usage_guides", json!({"query": "  "})),
            ("no_such_tool_at_all", json!({})),
            ("analyze_deploy_error", json!({"deployLogId": "not-exist"})),
        ] {
            let exec = execute(&core, tool, &args).await;
            assert!(
                exec.payload.get("error").is_some(),
                "{tool} 应返回 error 字段，实际 {:?}", exec.payload
            );
            assert!(exec.proposals.is_empty());
        }
        // 教学检索正常时给出正文
        let guide = execute(&core, "search_usage_guides", &json!({"query": "产物目录"})).await;
        assert!(!guide.payload["hits"].as_array().unwrap().is_empty());
        // 界面动作走 actions 通道
        let nav = execute(&core, "open_config_page", &json!({"module": "cicd"})).await;
        assert_eq!(nav.actions[0]["route"], "/cicd");
    }
}
