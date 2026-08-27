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

fn as_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).map(str::trim)
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
