//! MCP (Model Context Protocol) stdio server — 让 AI 通过工具协议原生调用 stool 能力。
//! 协议：JSON-RPC 2.0 over stdio（每行一条消息），手写实现，无第三方依赖。
//! 工具内部直接调用 supertool-core（与 CLI 命令等价），输出 JSON 结构化结果。
//! 安全：危险工具（server_exec/cicd_deploy）在工具层检查高危命令与 requiresApproval。

use crate::runtime::CliRuntime;
use crate::utils::is_dangerous_command;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2025-06-18";

pub async fn cmd_mcp_serve(rt: &mut CliRuntime, _name: &str) -> anyhow::Result<()> {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                // JSON 解析失败：回 JSON-RPC -32700 Parse error（避免客户端挂起等待）
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                let _ = stdout
                    .write_all(serde_json::to_string(&resp).unwrap_or_default().as_bytes())
                    .await;
                let _ = stdout.write_all(b"\n").await;
                let _ = stdout.flush().await;
                continue;
            }
        };
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        // JSON-RPC 2.0：无 id 的消息视为通知，不响应
        let id = msg.get("id").cloned();
        let is_notification = id.is_none();

        let response: Option<Value> = match method {
            "initialize" => Some(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {}, "resources": {} },
                "serverInfo": { "name": "stool", "version": env!("CARGO_PKG_VERSION") }
            })),
            "notifications/initialized" | "notifications/cancelled" => None,
            "ping" => Some(json!({})),
            "tools/list" => Some(list_tools()),
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
                let name = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                Some(call_tool(rt, &name, &args).await)
            }
            "resources/list" => Some(list_resources()),
            "resources/read" => {
                let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
                let uri = params
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(read_resource(rt, &uri).await)
            }
            _ => Some(json!({
                "error": { "code": -32601, "message": format!("未知方法: {}", method) }
            })),
        };

        if let Some(res) = response {
            // JSON-RPC 2.0：缺 id 一律视为通知，不响应（tools/call 若忘带 id 由客户端超时自纠）
            if is_notification {
                continue;
            }
            let mut resp = json!({ "jsonrpc": "2.0" });
            if let Some(i) = id {
                resp["id"] = i;
            } else {
                resp["id"] = Value::Null;
            }
            if let Some(err) = res.get("error") {
                resp["error"] = err.clone();
            } else {
                resp["result"] = res;
            }
            let _ = stdout
                .write_all(serde_json::to_string(&resp).unwrap_or_default().as_bytes())
                .await;
            let _ = stdout.write_all(b"\n").await;
            let _ = stdout.flush().await;
        }
    }
    Ok(())
}

/// 打印工具清单（调试用：`stool mcp list-tools`）
pub fn cmd_mcp_list_tools() {
    println!("{}", serde_json::to_string_pretty(&list_tools()).unwrap_or_default());
}

// ============ tools ============

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    })
}

fn list_tools() -> Value {
    let tools = vec![
        // ---- server ----
        tool("server_list", "列出所有 SSH 服务器（含分组、requiresApproval 状态）", json!({}), &[]),
        tool(
            "server_exec",
            "在服务器上执行 shell 命令（危险命令与需审批服务器会被拦截）",
            json!({
                "id": { "type": "string", "description": "服务器 ID" },
                "command": { "type": "string", "description": "要执行的命令" },
                "timeout": { "type": "integer", "description": "超时秒数（默认 60）" },
            }),
            &["id", "command"],
        ),
        // ---- cicd ----
        tool("cicd_list", "列出全部 CI/CD 部署配置", json!({}), &[]),
        tool(
            "cicd_deploy",
            "触发一次部署（需审批的配置会被拦截，请提示用户在 GUI 确认）",
            json!({
                "config_id": { "type": "string", "description": "部署配置 ID" },
                "branch": { "type": "string", "description": "可选，覆盖配置中的部署分支" },
            }),
            &["config_id"],
        ),
        tool(
            "cicd_history",
            "查看某配置的部署历史",
            json!({
                "config_id": { "type": "string" },
                "limit": { "type": "integer", "description": "条数，默认 10" },
            }),
            &["config_id"],
        ),
        // ---- db ----
        tool("db_list", "列出已配置的数据库连接（MySQL/PostgreSQL/Redis，密码隐藏）", json!({}), &[]),
        tool(
            "db_query",
            "执行 SQL 查询并返回结果行（仅建议 SELECT 等只读查询）",
            json!({
                "db_id": { "type": "string", "description": "数据库连接 ID" },
                "sql": { "type": "string", "description": "SQL 语句" },
            }),
            &["db_id", "sql"],
        ),
        tool(
            "db_tables",
            "列出数据库中的表",
            json!({
                "db_id": { "type": "string" },
                "db": { "type": "string", "description": "可选，库名" },
            }),
            &["db_id"],
        ),
        tool(
            "redis_keys",
            "列出 Redis key（支持 pattern）",
            json!({
                "db_id": { "type": "string", "description": "Redis 连接 ID" },
                "pattern": { "type": "string", "description": "pattern，默认 *" },
            }),
            &["db_id"],
        ),
        tool(
            "redis_get",
            "读取 Redis key 的值与类型",
            json!({
                "db_id": { "type": "string" },
                "key": { "type": "string" },
            }),
            &["db_id", "key"],
        ),
        // ---- log ----
        tool("log_list", "列出日志查询预设（含分组）", json!({}), &[]),
        tool(
            "log_search",
            "在日志预设中搜索关键字（grep，支持 | 多选）",
            json!({
                "preset": { "type": "string", "description": "预设 ID 或序号" },
                "keyword": { "type": "string", "description": "搜索关键字，支持 | 多选" },
                "lines": { "type": "integer", "description": "搜索行数范围，默认 50" },
            }),
            &["preset", "keyword"],
        ),
        tool(
            "log_tail",
            "静态查看日志预设末尾 N 行",
            json!({
                "preset": { "type": "string", "description": "预设 ID 或序号" },
                "lines": { "type": "integer", "description": "行数，默认 100" },
            }),
            &["preset"],
        ),
        tool(
            "log_context",
            "查看日志某一行周边的上下文（定位问题）",
            json!({
                "preset": { "type": "string" },
                "server_id": { "type": "string" },
                "line_num": { "type": "integer" },
                "context_lines": { "type": "integer", "description": "上下文行数，默认 20" },
            }),
            &["preset", "server_id", "line_num"],
        ),
        // ---- mfa ----
        tool(
            "mfa_code",
            "生成 MFA TOTP 验证码（按 ID 或序号）",
            json!({ "identifier": { "type": "string", "description": "MFA 密钥 ID 或列表序号" } }),
            &["identifier"],
        ),
        // ---- todo ----
        tool("todo_list", "列出任务（支持按优先级/完成状态）", json!({}), &[]),
        tool(
            "todo_add",
            "添加任务",
            json!({
                "text": { "type": "string" },
                "priority": { "type": "string", "enum": ["high", "medium", "low"] },
                "due": { "type": "string", "description": "截止日期 YYYY-MM-DD" },
            }),
            &["text"],
        ),
        tool("todo_complete", "完成任务", json!({ "id": { "type": "string" } }), &["id"]),
        // ---- audit ----
        tool(
            "audit_list",
            "查询操作审计记录（CLI/GUI 的写操作，参数已脱敏）",
            json!({
                "actor": { "type": "string", "enum": ["cli", "gui", "ai", "user"] },
                "result": { "type": "string", "enum": ["success", "failed", "blocked"] },
                "limit": { "type": "integer", "description": "条数，默认 50" },
            }),
            &[],
        ),
    ];
    json!({ "tools": tools })
}

// ============ resources ============

fn list_resources() -> Value {
    json!({
        "resources": [
            { "uri": "stool://servers", "name": "服务器列表", "mimeType": "application/json" },
            { "uri": "stool://log-presets", "name": "日志查询预设", "mimeType": "application/json" },
            { "uri": "stool://cicd-configs", "name": "CI/CD 部署配置", "mimeType": "application/json" },
        ]
    })
}

async fn read_resource(rt: &mut CliRuntime, uri: &str) -> Value {
    let text = match uri {
        "stool://servers" => {
            match rt.core.get_all_servers().await {
                Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
                Err(e) => e,
            }
        }
        "stool://log-presets" => match rt.core.get_log_presets().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
            Err(e) => e,
        },
        "stool://cicd-configs" => match rt.core.get_all_cicd_data().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_default(),
            Err(e) => e,
        },
        _ => {
            return json!({
                "error": { "code": -32002, "message": format!("未知资源: {}", uri) }
            })
        }
    };
    json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": text,
        }]
    })
}

// ============ tools/call ============

async fn call_tool(rt: &mut CliRuntime, name: &str, args: &Value) -> Value {
    let result: Result<Value, String> = match name {
        "server_list" => rt.core.get_all_servers().await,
        "server_exec" => server_exec(rt, args).await,
        "cicd_list" => rt.core.get_all_cicd_data().await,
        "cicd_deploy" => cicd_deploy(rt, args).await,
        "cicd_history" => cicd_history(rt, args).await,
        "db_list" => db_list(rt).await,
        "db_query" => db_query(rt, args).await,
        "db_tables" => db_tables(rt, args).await,
        "redis_keys" => redis_keys(rt, args).await,
        "redis_get" => redis_get(rt, args).await,
        "log_list" => rt.core.get_log_presets().await,
        "log_search" => log_search(rt, args).await,
        "log_tail" => log_tail(rt, args).await,
        "log_context" => log_context(rt, args).await,
        "mfa_code" => mfa_code(rt, args).await,
        "todo_list" => rt.core.get_all_todos().await,
        "todo_add" => todo_add(rt, args).await,
        "todo_complete" => todo_complete(rt, args).await,
        "audit_list" => {
            rt.core
                .list_audit(
                    args.get("actor").and_then(|v| v.as_str()),
                    args.get("result").and_then(|v| v.as_str()),
                    args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize,
                )
                .map_err(|e| e.to_string())
        }
        _ => return json!({
            "content": [{ "type": "text", "text": format!("未知工具: {}", name) }],
            "isError": true,
        }),
    };

    // MCP 写工具审计（actor=ai）：与 CLI 分发层同表，参数脱敏
    const AI_WRITE_TOOLS: &[&str] = &["server_exec", "cicd_deploy", "todo_add", "todo_complete"];
    if AI_WRITE_TOOLS.contains(&name) {
        let desc = supertool_core::logic::log_sanitizer::sanitize_string(&format!("mcp:{} {}", name, serde_json::to_string(args).unwrap_or_default()));
        let desc: String = desc.chars().take(500).collect();
        let status = if result.is_ok() { "success" } else { "failed" };
        let _ = rt.core.record_audit("ai", "mcp", &desc, "", "", status, 0);
    }

    match result {
        Ok(v) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }]
        }),
        Err(e) => json!({
            "content": [{ "type": "text", "text": e }],
            "isError": true,
        }),
    }
}

// ---- server ----

async fn server_exec(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    let command = req_str(args, "command")?;
    // 高危命令拦截（与 CLI 一致）
    if is_dangerous_command(&command) {
        return Err("高危命令已被拦截，如需执行请使用 GUI 手动操作".into());
    }
    // requiresApproval 检查
    let servers = rt.core.get_all_servers().await?;
    for s in servers.as_array().unwrap_or(&vec![]) {
        if s.get("id").and_then(|v| v.as_str()) == Some(id) {
            if s.get("requiresApproval").and_then(|v| v.as_bool()).unwrap_or(false) {
                return Err("服务器已开启执行审核（requiresApproval），请提示用户到 GUI 确认后执行".into());
            }
        }
    }
    rt.core.exec_ssh_command(id, &command).await
}

// ---- cicd ----

async fn cicd_deploy(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let config_id = req_str(args, "config_id")?;
    let branch = args.get("branch").and_then(|v| v.as_str());
    let resp = rt
        .core
        .cicd_deploy_with_branch(config_id, branch.map(|s| s.to_string()))
        .await?;
    if resp.get("success").and_then(|v| v.as_bool()) == Some(false) {
        if resp.get("requiresApproval").and_then(|v| v.as_bool()) == Some(true) {
            return Err("该配置已开启部署审核，请提示用户到 GUI 手动确认部署".into());
        }
        return Err(format!(
            "部署失败: {}",
            resp.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")
        ));
    }
    Ok(resp)
}

async fn cicd_history(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let config_id = req_str(args, "config_id")?;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as i64;
    let history = rt.core.get_deploy_history_by_config(config_id, limit);
    Ok(serde_json::to_value(history).map_err(|e| e.to_string())?)
}

// ---- db ----

async fn db_list(rt: &mut CliRuntime) -> Result<Value, String> {
    let setting = rt.core.get_setting("db_connections").await?;
    let mut conns: Value = match setting {
        Value::String(s) => serde_json::from_str(&s).unwrap_or_else(|_| json!([])),
        Value::Array(arr) => Value::Array(arr),
        _ => json!([]),
    };
    // 脱敏：移除密码（与 CLI database.rs 一致），避免 AI 客户端拿到明文
    if let Some(arr) = conns.as_array_mut() {
        for c in arr.iter_mut() {
            if let Some(obj) = c.as_object_mut() {
                obj.remove("password");
            }
        }
    }
    Ok(conns)
}

async fn db_query(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let db_id = req_str(args, "db_id")?;
    let sql = req_str(args, "sql")?;
    // 只读白名单：AI 遭 prompt injection 时可防 DROP/TRUNCATE/DELETE 等破坏（CLI 不限制，MCP 层加强）
    let sql_upper = sql.trim_start().to_uppercase();
    let read_only_prefixes = [
        "SELECT ", "SHOW ", "EXPLAIN ", "DESC ", "DESCRIBE ", "PRAGMA ", "WITH ",
    ];
    if !read_only_prefixes.iter().any(|p| sql_upper.starts_with(p)) {
        return Err(
            "MCP 仅允许只读 SQL（SELECT/SHOW/EXPLAIN/DESC/PRAGMA/WITH），写操作请使用 GUI 或 CLI".into(),
        );
    }
    let config = get_db_config(rt, db_id).await?;
    rt.core.execute_db_query(config, sql).await
}

async fn db_tables(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let db_id = req_str(args, "db_id")?;
    let config = get_db_config(rt, db_id).await?;
    let db_name = args
        .get("db")
        .and_then(|v| v.as_str())
        .or(config.db_name.as_deref())
        .unwrap_or("");
    rt.core.db_get_tables(&config, db_name).await
}

async fn redis_keys(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let db_id = req_str(args, "db_id")?;
    let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");
    let config = get_db_config(rt, db_id).await?;
    let db_index = config.db_index.unwrap_or(0);
    rt.core.db_redis_list_keys(&config, db_index, pattern).await
}

async fn redis_get(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let db_id = req_str(args, "db_id")?;
    let key = req_str(args, "key")?;
    let config = get_db_config(rt, db_id).await?;
    let db_index = config.db_index.unwrap_or(0);
    rt.core.db_redis_get_value(&config, db_index, key).await
}

// ---- log ----

async fn resolve_preset(rt: &mut CliRuntime, preset: &str) -> Result<String, String> {
    if let Ok(idx) = preset.parse::<usize>() {
        let presets = rt.core.get_log_presets().await?;
        let arr = presets.as_array().ok_or("日志预设数据格式错误")?;
        if idx > 0 && idx <= arr.len() {
            if let Some(id) = arr[idx - 1].get("id").and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }
        return Err(format!("预设序号越界: {}", idx));
    }
    Ok(preset.to_string())
}

async fn log_search(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let preset = req_str(args, "preset")?;
    let keyword = req_str(args, "keyword")?;
    let lines = args.get("lines").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
    let id = resolve_preset(rt, preset).await?;
    rt.core.log_search(&id, &keyword, lines).await
}

async fn log_tail(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let preset = req_str(args, "preset")?;
    let lines = args.get("lines").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let id = resolve_preset(rt, preset).await?;
    rt.core.log_tail(&id, lines).await
}

async fn log_context(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let preset = req_str(args, "preset")?;
    let server_id = req_str(args, "server_id")?;
    let line_num = args.get("line_num").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let ctx = args.get("context_lines").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let id = resolve_preset(rt, preset).await?;
    rt.core.log_context(&id, server_id, line_num, ctx).await
}

// ---- mfa ----

async fn mfa_code(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let identifier = req_str(args, "identifier")?;
    let secrets = rt.core.get_all_mfa_secrets().await?;
    let arr = secrets.as_array().ok_or("MFA 数据格式错误")?;
    let target: Option<Value> = if let Ok(idx) = identifier.parse::<usize>() {
        if idx > 0 && idx <= arr.len() {
            Some(arr[idx - 1].clone())
        } else {
            None
        }
    } else {
        arr.iter()
            .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(identifier))
            .cloned()
    };
    let s = target.ok_or_else(|| format!("未找到 MFA 密钥: {}", identifier))?;
    let secret = s.get("secret").and_then(|v| v.as_str()).unwrap_or("");
    let digits = s.get("digits").and_then(|v| v.as_u64()).unwrap_or(6) as u32;
    let period = s.get("period").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
    let algorithm = s.get("algorithm").and_then(|v| v.as_str()).unwrap_or("SHA1");
    let result = rt.core.generate_totp(secret, digits, period, algorithm).await?;
    Ok(json!({
        "name": s.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
        "code": result.get("code").and_then(|v| v.as_str()).unwrap_or("?"),
        "remainingSeconds": result.get("remaining").and_then(|v| v.as_u64()).unwrap_or(0),
        "id": s.get("id").and_then(|v| v.as_str()).unwrap_or(""),
    }))
}

// ---- todo ----

async fn todo_add(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let text = req_str(args, "text")?;
    let params = json!({
        "text": text,
        "priority": args.get("priority").cloned().unwrap_or_else(|| json!("medium")),
        "due": args.get("due").cloned().unwrap_or_else(|| json!(null)),
    });
    rt.core.add_todo(params).await
}

async fn todo_complete(rt: &mut CliRuntime, args: &Value) -> Result<Value, String> {
    let id = req_str(args, "id")?;
    rt.core
        .update_todo(json!({ "id": id, "completed": true }))
        .await
}

// ---- helpers ----

fn req_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("缺少参数: {}", key))
}

/// 从 db_connections 设置中查找连接配置（含密码，与 CLI database.rs 等价）
async fn get_db_config(
    rt: &mut CliRuntime,
    db_id: &str,
) -> Result<supertool_core::db_pool::DbConnectionConfig, String> {
    let setting = rt.core.get_setting("db_connections").await?;
    let connections: Vec<Value> = match setting {
        Value::String(s) => serde_json::from_str(&s).unwrap_or_default(),
        Value::Array(arr) => arr,
        _ => vec![],
    };
    let conn = connections
        .iter()
        .find(|c| c.get("id").and_then(|v| v.as_str()) == Some(db_id))
        .ok_or_else(|| format!("未找到数据库连接: {}", db_id))?;
    serde_json::from_value(conn.clone()).map_err(|e| format!("解析连接配置失败: {}", e))
}

/// `stool mcp` 命令分发
pub async fn cmd_mcp(rt: &mut CliRuntime, action: &crate::types::McpCommands) -> anyhow::Result<()> {
    match action {
        crate::types::McpCommands::Serve { name } => cmd_mcp_serve(rt, name).await,
        crate::types::McpCommands::ListTools => {
            cmd_mcp_list_tools();
            Ok(())
        }
    }
}
