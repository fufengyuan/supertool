use super::ssh;
use regex::Regex;
use rusqlite::params;
use serde_json::{Value, json};

impl super::CoreService {
    pub async fn get_log_presets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM log_presets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    let server_ids: String = row.get("serverIds")?;
                    let keywords_str: String =
                        row.get("keywords").unwrap_or_else(|_| "[]".to_string());
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "serverIds": serde_json::from_str::<Value>(&server_ids)
                            .unwrap_or(Value::Array(vec![])),
                        "logPath": row.get::<_, String>("logPath")?,
                        "logType": row.get::<_, String>("logType")?,
                        "maxLines": row.get::<_, i64>("maxLines")?,
                        "presetGroup": row.get::<_, Option<String>>("presetGroup")?,
                        "keywords": serde_json::from_str::<Value>(&keywords_str)
                            .unwrap_or(Value::Array(vec![])),
                    }))
                })
                .map_err(|e| e.to_string())?;
            let presets: Result<Vec<Value>, _> = rows.collect();
            presets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_log_preset(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let server_ids =
            serde_json::to_string(&params["serverIds"]).unwrap_or_else(|_| "[]".to_string());
        let log_path = params["logPath"].as_str().unwrap_or("").to_string();
        let log_type = params["logType"].as_str().unwrap_or("file").to_string();
        let max_lines = params["maxLines"].as_i64().unwrap_or(100);
        let preset_group = params.get("presetGroup").and_then(|v| v.as_str());
        let keywords =
            serde_json::to_string(&params["keywords"]).unwrap_or_else(|_| "[]".to_string());
        self.with_db(|db| {
            db.conn_mut().execute(
                "INSERT INTO log_presets (id, name, serverIds, logPath, logType, maxLines, presetGroup, keywords, createdAt, updatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))",
                params![id, name, server_ids, log_path, log_type, max_lines, preset_group, keywords],
            ).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_log_preset(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let server_ids =
            serde_json::to_string(&params["serverIds"]).unwrap_or_else(|_| "[]".to_string());
        let log_path = params["logPath"].as_str().unwrap_or("").to_string();
        let log_type = params["logType"].as_str().unwrap_or("file").to_string();
        let max_lines = params["maxLines"].as_i64().unwrap_or(100);
        let preset_group = params.get("presetGroup").and_then(|v| v.as_str());
        let keywords =
            serde_json::to_string(&params["keywords"]).unwrap_or_else(|_| "[]".to_string());
        self.with_db(|db| {
            db.conn_mut().execute(
                "UPDATE log_presets SET name=?2, serverIds=?3, logPath=?4, logType=?5, maxLines=?6, presetGroup=?7, keywords=?8 WHERE id=?1",
                params![id, name, server_ids, log_path, log_type, max_lines, preset_group, keywords],
            ).map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_log_preset(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM log_presets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    // ============ Log Aggregator ============

    pub async fn log_tail(&self, preset_id: &str, lines: usize) -> Result<Value, String> {
        // Get preset from database
        let preset = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM log_presets WHERE id = ?1",
                    params![preset_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "name": row.get::<_, String>("name")?,
                            "serverIds": row.get::<_, String>("serverIds")?,
                            "logPath": row.get::<_, String>("logPath")?,
                            "logType": row.get::<_, String>("logType")?,
                            "maxLines": row.get::<_, i64>("maxLines")?,
                        }))
                    },
                )
                .map_err(preset_not_found_err(preset_id))
        })?;

        let server_ids: Vec<String> = match &preset["serverIds"] {
            Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            Value::String(s) => serde_json::from_str(s).unwrap_or_default(),
            _ => Vec::new(),
        };
        if server_ids.is_empty() {
            return Err(format!("预设 {} 没有配置服务器", preset_id));
        }

        let log_path = preset["logPath"].as_str().unwrap_or("");
        if log_path.trim().is_empty() {
            return Err(format!("预设 {} 没有配置日志路径", preset_id));
        }

        // Build tail command based on log type
        let cmd = build_tail_command(&preset, lines);
        let mut results = Vec::new();

        for server_id in &server_ids {
            // Get server info
            let server = self.with_db(|db| {
                db.conn()
                    .query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        params![server_id],
                        |row| {
                            Ok(json!({
                                "id": row.get::<_, String>("id")?,
                                "name": row.get::<_, String>("name")?,
                                "host": row.get::<_, String>("host")?,
                                "port": row.get::<_, i64>("port")?,
                                "username": row.get::<_, String>("username")?,
                                "password": row.get::<_, Option<String>>("password")?,
                                "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                            }))
                        },
                    )
                    .map_err(|e| e.to_string())
            });

            let Ok(s) = server else {
                continue;
            };

            let host = s["host"].as_str().unwrap_or("").to_string();
            let port = s["port"].as_u64().unwrap_or(22) as u32;
            let username = s["username"].as_str().unwrap_or("").to_string();
            let raw_password = s
                .get("password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let ssh_key_path = s
                .get("sshKeyPath")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            // 解密密码
            let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

            let config = ssh::SshServerConfig {
                id: server_id.clone(),
                name: s["name"].as_str().unwrap_or("").to_string(),
                host,
                port,
                username,
                password,
                ssh_key_path,
            };

            // Connect if not already connected
            if !self.ssh.is_connected(server_id) {
                if let Err(e) = self.ssh.connect(&config) {
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "error": e,
                        "lines": []
                    }));
                    continue;
                }
            }

            // Execute tail command
            let output = self.ssh.exec_command(server_id, &cmd);
            match output {
                Ok(exec_result) => {
                    let line_list: Vec<String> = exec_result
                        .output
                        .lines()
                        .map(|l| l.to_string())
                        .take(lines)
                        .collect();
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "lines": line_list,
                        "error": if !exec_result.success { Some(exec_result.error_output) } else { None }
                    }));
                }
                Err(e) => {
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "error": e,
                        "lines": []
                    }));
                }
            }
        }

        Ok(json!({"presetId": preset_id, "lines": results.len(), "results": results}))
    }

    /// Load more historical log lines for an active stream.
    /// Queries each server for older lines beyond the current count
    /// and returns them as a list of {serverId, serverName, lines[]}.
    pub async fn load_more_logs(
        &self,
        preset_id: &str,
        current_count: usize,
        batch_size: usize,
    ) -> Result<Value, String> {
        let presets = self.get_log_presets().await?;
        let empty_vec = vec![];
        let preset_list = presets.as_array().unwrap_or(&empty_vec);
        if preset_list.is_empty() {
            return Ok(json!({"lines": 0, "results": []}));
        }
	let preset = preset_list
            .iter()
            .find(|p| p["id"].as_str() == Some(preset_id))
            .unwrap_or_else(|| &preset_list[0]); // fallback to first preset

	let server_ids: Vec<String> =
	    serde_json::from_value(preset["serverIds"].clone()).unwrap_or_default();

        if server_ids.is_empty() {
            return Ok(json!({"lines": 0, "results": [], "note": "No servers configured"}));
        }

        let log_type = preset["logType"].as_str().unwrap_or("file");
        let log_path = preset["logPath"].as_str().unwrap_or("");
        let mut results = Vec::new();

        for server_id in &server_ids {
            let server = self.with_db(|db| {
                db.conn()
                    .query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        params![server_id],
                        |row| {
                            Ok(json!({
                                "id": row.get::<_, String>("id")?,
                                "name": row.get::<_, String>("name")?,
                                "host": row.get::<_, String>("host")?,
                                "port": row.get::<_, i64>("port")?,
                                "username": row.get::<_, String>("username")?,
                                "password": row.get::<_, Option<String>>("password")?,
                                "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                            }))
                        },
                    )
                    .map_err(|e| e.to_string())
            });

            let Ok(s) = server else { continue };

            let host = s["host"].as_str().unwrap_or("").to_string();
            let port = s["port"].as_u64().unwrap_or(22) as u32;
            let username = s["username"].as_str().unwrap_or("").to_string();
            let raw_password = s.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ssh_key_path = s
                .get("sshKeyPath")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

            let config = ssh::SshServerConfig {
                id: server_id.clone(),
                name: s["name"].as_str().unwrap_or("").to_string(),
                host, port, username, password, ssh_key_path,
            };

            if !self.ssh.is_connected(server_id) {
                if let Err(e) = self.ssh.connect(&config) {
                    results.push(json!({"serverId": server_id, "serverName": s["name"], "error": e, "lines": []}));
                    continue;
                }
            }

            // Build a command that reads (current_count + batch_size) lines from the end,
            // then takes only the first `batch_size` (the oldest portion of the window).
            // This gives us lines that come BEFORE the currently displayed log lines.
            let cmd = match log_type {
                "docker" => {
                    let containers: Vec<String> = log_path.split('\n')
                        .filter(|c| !c.trim().is_empty()).map(|c| c.trim().to_string()).collect();
                    if containers.is_empty() { continue; }
                    let c = &containers[0];
                    format!("docker logs --tail {} {} 2>&1 | head -n {}", current_count + batch_size, c, batch_size)
                }
                "journal" => {
                    format!("journalctl -n {} --no-pager -o cat | head -n {}", current_count + batch_size, batch_size)
                }
                _ => {
                    // For file logs: get last N+M lines, then take first M
                    format!("tail -n {} {} 2>/dev/null | head -n {}", current_count + batch_size, log_path, batch_size)
                }
            };

            match self.ssh.exec_command(server_id, &cmd) {
                Ok(output) => {
                    let lines: Vec<String> = output.output.lines()
                        .map(|l| l.to_string())
                        .filter(|l| !l.trim().is_empty())
                        .collect();
                    results.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "lines": lines,
                    }));
                }
                Err(e) => {
                    results.push(json!({"serverId": server_id, "serverName": s["name"], "error": e, "lines": []}));
                }
            }
        }

        Ok(json!({"lines": results.len(), "results": results}))
    }

    pub async fn log_search(
        &self,
        preset_id: &str,
        keyword: &str,
        lines: usize,
        date: Option<&str>,
        days: Option<u64>,
        regex_mode: bool,
    ) -> Result<Value, String> {
        let spec = LogKeywordSpec::parse(keyword, regex_mode);
        if spec.is_empty() {
            return Ok(json!({"presetId": preset_id, "keyword": keyword, "matches": []}));
        }

        // Get preset from database
        let preset = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM log_presets WHERE id = ?1",
                    params![preset_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "name": row.get::<_, String>("name")?,
                            "serverIds": row.get::<_, String>("serverIds")?,
                            "logPath": row.get::<_, String>("logPath")?,
                            "logType": row.get::<_, String>("logType")?,
                        }))
                    },
                )
                .map_err(preset_not_found_err(preset_id))
        })?;

        let server_ids: Vec<String> = match &preset["serverIds"] {
            Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            Value::String(s) => serde_json::from_str(s).unwrap_or_default(),
            _ => Vec::new(),
        };
        if server_ids.is_empty() {
            return Err(format!("预设 {} 没有配置服务器", preset_id));
        }

        let log_path = preset["logPath"].as_str().unwrap_or("");
        if log_path.trim().is_empty() {
            return Err(format!("预设 {} 没有配置日志路径", preset_id));
        }

        let cmd = build_grep_command(&preset, keyword, lines, date, days, regex_mode);
        let mut matches = Vec::new();

        for server_id in &server_ids {
            let server = self.with_db(|db| {
                db.conn()
                    .query_row(
                        "SELECT * FROM servers WHERE id = ?1",
                        params![server_id],
                        |row| {
                            Ok(json!({
                                "id": row.get::<_, String>("id")?,
                                "name": row.get::<_, String>("name")?,
                                "host": row.get::<_, String>("host")?,
                                "port": row.get::<_, i64>("port")?,
                                "username": row.get::<_, String>("username")?,
                                "password": row.get::<_, Option<String>>("password")?,
                                "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                            }))
                        },
                    )
                    .map_err(|e| e.to_string())
            });

            let Ok(s) = server else {
                continue;
            };

            let host = s["host"].as_str().unwrap_or("").to_string();
            let port = s["port"].as_u64().unwrap_or(22) as u32;
            let username = s["username"].as_str().unwrap_or("").to_string();
            let raw_password = s
                .get("password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let ssh_key_path = s
                .get("sshKeyPath")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            // 解密密码（与 logs_start_stream 一致）
            let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

            let config = ssh::SshServerConfig {
                id: server_id.clone(),
                name: s["name"].as_str().unwrap_or("").to_string(),
                host,
                port,
                username,
                password,
                ssh_key_path,
            };

            if !self.ssh.is_connected(server_id) {
                if let Err(e) = self.ssh.connect(&config) {
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": server_id,
                        "matchCount": 0,
                        "lines": [],
                        "error": e
                    }));
                    continue;
                }
            }

            let output = self.ssh.exec_command(server_id, &cmd);
            let output = match output {
                Ok(r) => Ok(r),
                Err(e) => {
                    // 连接可能已断开，重连一次
                    log::warn!(
                        "[log_search] exec_command failed for {}, retrying: {}",
                        server_id,
                        e
                    );
                    self.ssh.disconnect(server_id);
                    if let Err(re) = self.ssh.connect(&config) {
                        Err(format!("重连失败: {}", re))
                    } else {
                        self.ssh.exec_command(server_id, &cmd)
                    }
                }
            };
            match output {
                Ok(exec_result) => {
                    let lines = parse_grep_output(&exec_result.output, &spec);
                    let match_count = lines
                        .iter()
                        .filter(|l| l["isMatch"].as_bool().unwrap_or(false))
                        .count();
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": s["name"],
                        "matchCount": match_count,
                        "lines": lines
                    }));
                }
                Err(e) => {
                    matches.push(json!({
                        "serverId": server_id,
                        "serverName": server_id,
                        "matchCount": 0,
                        "lines": [],
                        "error": e
                    }));
                }
            }
        }

        Ok(json!({"presetId": preset_id, "keyword": keyword, "matches": matches}))
    }

    /// Load context lines around a specific line number from a specific server.
    /// Returns `context_lines` lines (half before, half after the given line_num).
    pub async fn log_context(
        &self,
        preset_id: &str,
        server_id: &str,
        line_num: usize,
        context_lines: usize,
    ) -> Result<Value, String> {
        let half = context_lines / 2;
        let start = if line_num > half { line_num - half } else { 1 };
        let end = line_num + half;

        let preset = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM log_presets WHERE id = ?1",
                    params![preset_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "logPath": row.get::<_, String>("logPath")?,
                            "logType": row.get::<_, String>("logType")?,
                        }))
                    },
                )
                .map_err(preset_not_found_err(preset_id))
        })?;

        let log_type = preset["logType"].as_str().unwrap_or("file");
        let log_path = preset["logPath"].as_str().unwrap_or("");

        let server = self.with_db(|db| {
            db.conn()
                .query_row(
                    "SELECT * FROM servers WHERE id = ?1",
                    params![server_id],
                    |row| {
                        Ok(json!({
                            "id": row.get::<_, String>("id")?,
                            "name": row.get::<_, String>("name")?,
                            "host": row.get::<_, String>("host")?,
                            "port": row.get::<_, i64>("port")?,
                            "username": row.get::<_, String>("username")?,
                            "password": row.get::<_, Option<String>>("password")?,
                            "sshKeyPath": row.get::<_, Option<String>>("sshKeyPath")?,
                        }))
                    },
                )
                .map_err(|e| e.to_string())
        })?;

        let host = server["host"].as_str().unwrap_or("").to_string();
        let port = server["port"].as_u64().unwrap_or(22) as u32;
        let username = server["username"].as_str().unwrap_or("").to_string();
        let raw_password = server.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
        let ssh_key_path = server
            .get("sshKeyPath")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let password = raw_password.map(|pw| crate::encryption::try_decrypt_password(&pw));

        let config = ssh::SshServerConfig {
            id: server_id.to_string(),
            name: server["name"].as_str().unwrap_or("").to_string(),
            host, port, username, password, ssh_key_path,
        };

        if !self.ssh.is_connected(server_id) {
            self.ssh.connect(&config).map_err(|e| e.to_string())?;
        }

        let cmd = build_context_command(log_type, log_path, start, end);
        let output = self.ssh.exec_command(server_id, &cmd).map_err(|e| e.to_string())?;

        let lines: Vec<Value> = output.output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, content)| {
                json!({"lineNum": start + i, "content": content.to_string()})
            })
            .collect();

        Ok(json!({
            "serverId": server_id,
            "serverName": server["name"],
            "lines": lines,
            "start": start,
            "end": end,
        }))
    }
}

/// Map a SQLite "no rows" error into a clear "preset not found" hint,
/// so callers (GUI/CLI) can tell the user to re-check the preset id.
fn preset_not_found_err(preset_id: &str) -> impl Fn(rusqlite::Error) -> String {
    let id = preset_id.to_string();
    move |e: rusqlite::Error| {
        if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
            format!("日志预设 {} 不存在（可用 stool log list 核对 id）", id)
        } else {
            e.to_string()
        }
    }
}

fn build_tail_command(preset: &Value, lines: usize) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");

    match log_type {
        "docker" => {
            let containers: Vec<String> = log_path
                .split('\n')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();
            if containers.is_empty() {
                return "echo 'No containers configured'".to_string();
            }
            containers
                .iter()
                .map(|c| format!("docker logs --tail {} '{}'", lines, c))
                .collect::<Vec<_>>()
                .join(" ; ")
        }
        "journalctl" => {
            let units: Vec<String> = log_path
                .split('\n')
                .filter(|u| !u.trim().is_empty())
                .map(|u| u.trim().to_string())
                .collect();
            if units.is_empty() {
                format!("journalctl -n {} --no-pager 2>/dev/null", lines)
            } else {
                let unit_args: Vec<String> = units.iter().map(|u| format!("-u '{}'", u)).collect();
                format!(
                    "journalctl {} -n {} --no-pager 2>/dev/null",
                    unit_args.join(" "),
                    lines
                )
            }
        }
        _ => {
            let paths: Vec<String> = log_path
                .split('\n')
                .filter(|p| !p.trim().is_empty())
                .map(|p| p.trim().to_string())
                .collect();
            if paths.is_empty() {
                return "echo 'No log paths configured'".to_string();
            }
            let q = |p: &str| shell_quote_path(p);
            format!(
                "tail -n {} {} 2>/dev/null",
                lines,
                paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" ")
            )
        }
    }
}

/// Shell-引用路径，将 ~ 展开为 $HOME（在引号外）
/// 与前端 quotePath() 逻辑一致
fn shell_quote_path(p: &str) -> String {
    if let Some(rest) = p.strip_prefix('~') {
        if rest.is_empty() {
            "$HOME".to_string()
        } else {
            let escaped = rest.replace('\'', "'\\''");
            format!("$HOME'{}'", escaped)
        }
    } else {
        format!("'{}'", p.replace('\'', "'\\''"))
    }
}

/// 生成历史轮转日志的 find -name 匹配条件（按文件名中的日期后缀对齐，而非 mtime）。
///
/// 背景：gzip 轮转日志的文件名后缀等于「数据日期」，而 mtime 是「压缩/轮转时间」（多为次日），
/// 旧实现用 `-newermt` 按 mtime 匹配会有一天的数据偏移（搜 08-19 命中 08-18 数据）。
/// 这里改为匹配文件名后缀，常见两种命名：`app.log.2026-08-06(.gz)` 与 `app.log-20260806(.gz)`；
/// 当日日期同时出现在未轮转（无日期后缀）的活动日志中，故当天额外纳入 `$BASE`。
fn date_name_filters(date: Option<&str>, days: Option<u64>) -> String {
    let today = chrono::Local::now().date_naive();
    let today_iso = today.format("%Y-%m-%d").to_string();
    let mut dates: Vec<chrono::NaiveDate> = Vec::new();
    if let Some(d) = date {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            dates.push(parsed);
        }
    } else {
        let n = days.unwrap_or(1).max(1).min(30) as i64;
        for i in 0..n {
            dates.push(today - chrono::Duration::days(i));
        }
    }
    let mut filters: Vec<String> = Vec::new();
    for d in &dates {
        let iso = d.format("%Y-%m-%d").to_string();
        let compact = d.format("%Y%m%d").to_string();
        filters.push(format!("-name \"*{}*\"", iso));
        if compact != iso {
            filters.push(format!("-name \"*{}*\"", compact));
        }
        if iso == today_iso {
            filters.push("-name \"$BASE\"".to_string());
        }
    }
    if filters.is_empty() {
        String::new()
    } else if filters.len() == 1 {
        filters[0].clone()
    } else {
        format!("\\( {} \\)", filters.join(" -o "))
    }
}

/// 搜索关键字的匹配规格：把用户输入的单个 keyword 编译成两份东西
///   1. 远端 grep 表达式（`grep_flag` + `pattern`）
///   2. 本地判定器（`is_match`），用于给上下文行回填 `isMatch`
///
/// 语义：
///   - 默认整串字面量匹配（`grep -F`），与历史行为一致
///   - 含 `|`（或 shell 转义写法 `\|`）时按 OR 拆分，每支仍按字面量处理（`grep -E`）
///   - `--regex/-E` 时整串作为 ERE 正则原样下传
pub struct LogKeywordSpec {
    /// grep 选项：`-F` 字面量 / `-E` 扩展正则
    pub grep_flag: &'static str,
    /// 交给 grep 的模式串（**未做** shell 单引号转义）
    pub pattern: String,
    /// 字面量判定分支（已小写），多关键词时为拆分后的各支
    branches: Vec<String>,
    /// `--regex` 模式下的已编译正则；编译失败为 None（退化为子串包含）
    regex: Option<Regex>,
}

impl LogKeywordSpec {
    pub fn parse(keyword: &str, regex_mode: bool) -> Self {
        if regex_mode {
            let regex = Regex::new(keyword).ok();
            // Rust regex 与远端 GNU/BSD grep 的 ERE 语法并非完全等价，编译失败时
            // 不拦截（仍原样下发给 grep），本地判定退回子串包含，避免所有行都丢高亮
            let branches = if regex.is_some() {
                Vec::new()
            } else {
                vec![keyword.to_lowercase()]
            };
            return Self {
                grep_flag: "-E",
                pattern: keyword.to_string(),
                branches,
                regex,
            };
        }
        let parts = split_alternatives_raw(keyword);
        if parts.len() > 1 {
            let branches: Vec<String> = parts
                .iter()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            // 只有分隔符（如 `|`）：空模式会匹配所有行，直接置空让调用方短路返回
            if branches.is_empty() {
                return Self {
                    grep_flag: "-F",
                    pattern: String::new(),
                    branches: Vec::new(),
                    regex: None,
                };
            }
            let pattern = branches
                .iter()
                .map(|b| escape_ere_literal(b))
                .collect::<Vec<_>>()
                .join("|");
            return Self {
                grep_flag: "-E",
                pattern,
                branches: branches.iter().map(|b| b.to_lowercase()).collect(),
                regex: None,
            };
        }
        Self {
            grep_flag: "-F",
            pattern: keyword.to_string(),
            branches: vec![keyword.to_lowercase()],
            regex: None,
        }
    }

    /// 空模式（如输入只有分隔符）不应下发 grep——空模式会匹配所有行
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    pub fn is_match(&self, content: &str) -> bool {
        if let Some(re) = &self.regex {
            return re.is_match(content);
        }
        let lower = content.to_lowercase();
        self.branches
            .iter()
            .any(|b| !b.is_empty() && lower.contains(b))
    }
}

/// 按 OR 分隔符拆分关键字（未做 trim / 空支过滤）。`|` 与 shell 常见的 `\|`
/// 写法都视为分隔符，`\\` 折叠为单个字面反斜杠。
fn split_alternatives_raw(keyword: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut chars = keyword.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.peek() {
                Some('|') => {
                    chars.next();
                    parts.push(std::mem::take(&mut cur));
                }
                Some('\\') => {
                    chars.next();
                    cur.push('\\');
                }
                _ => cur.push('\\'),
            },
            '|' => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    parts.push(std::mem::take(&mut cur));
    parts
}

/// 把一个分支转义成 ERE 字面量：只转义真正的 ERE 元字符，
/// 保留 `/`、`-` 等普通字符的原样写法（GNU grep 对未知转义会告警）。
fn escape_ere_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(
            c,
            '\\' | '.' | '[' | ']' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '{' | '}' | '|'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn build_grep_command(
    preset: &Value,
    keyword: &str,
    context_lines: usize,
    date: Option<&str>,
    days: Option<u64>,
    regex_mode: bool,
) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");
    let spec = LogKeywordSpec::parse(keyword, regex_mode);
    let escaped_kw = spec.pattern.replace('\'', "'\\''");
    let grep_ctx = if context_lines > 0 {
        format!(" -C {}", context_lines)
    } else {
        String::new()
    };
    // 默认 `-F` 固定字符串匹配：traceId / 完整关键字原样按字面子串匹配，
    // 避免 grep 把关键字当作 BRE 正则（如 traceId 含 `.`/`-` 等元字符时
    // 用完整 traceId 搜不到，而后缀纯数字能搜到——正是正则解析差异导致）。
    // 多关键词（`a|b`）或 --regex 时切 `-E`，各分支已按字面量转义。
    let grep = format!(
        "grep{} -i {} -n -e '{}'",
        grep_ctx, spec.grep_flag, escaped_kw
    );
    // 历史轮转日志多为 gzip，按扩展名分支解压：
    //   *.gz  → gzip -cd | grep（管道自动解压，不依赖 `zgrep`——部分服务器 zgrep
    //            对非 gzip 文件静默不出结果，与 gzip 版本相关）
    //   其余  → grep 直读。仅历史分支使用。
    // 参数布局（sh -c 'SCRIPT' sh <KW> <FILE>，find --exec \; 每次传一个文件）：
    //   $1 = 关键字；$2 = 当前文件名。用 $2 定位文件，避免 `$@` 把 $1 关键字也带进循环，
    //   导致对不存在的"名为关键字的路径"多做一次无效 grep。
    let gz_grep_script = format!(
        "case \"$2\" in *.gz) gzip -cd -- \"$2\" 2>/dev/null | grep{} -i {} -n -e \"$1\" ;; *) grep{} -i {} -n -e \"$1\" -- \"$2\" 2>/dev/null ;; esac",
        grep_ctx, spec.grep_flag, grep_ctx, spec.grep_flag,
    );
    // 关键字以位置参数 "$1" 传入（脚本内除 "$1"/"$2" 外不含任何 shell 元字符引用，规避嵌套引号）
    // 用 `\;`（每文件一次 sh）而非 `+`（批量）：busybox find/sh 对 `-exec ... +` 支持不完整
    let gz_exec = format!("-exec sh -c '{}' sh '{}' {{}} \\;", gz_grep_script, escaped_kw);
    // journalctl --grep 走 ERE：字面量模式需整体正则转义，多关键词/--regex 直接下传
    let journal_kw = if spec.grep_flag == "-E" {
        spec.pattern.clone()
    } else {
        regex::escape(&spec.pattern)
    }
    .replace('\'', "'\\''");

    // 历史查询目前仅支持 file 类型（docker/journalctl 的历史时间范围语法差异大）
    if (date.is_some() || days.is_some()) && (log_type == "docker" || log_type == "journalctl") {
        return format!(
            "echo '{} 类型暂不支持 --date/--days 历史查询，请查看当前日志'",
            log_type
        );
    }

    match log_type {
        "docker" => {
            let containers: Vec<String> = log_path
                .split('\n')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();
            containers
                .iter()
                .map(|c| {
                    format!(
                        "docker logs '{}' 2>&1 | {} 2>/dev/null",
                        c, grep
                    )
                })
                .collect::<Vec<_>>()
                .join(" ; ")
        }
        "journalctl" => {
            let units: Vec<String> = log_path
                .split('\n')
                .filter(|u| !u.trim().is_empty())
                .map(|u| u.trim().to_string())
                .collect();
            if units.is_empty() {
                format!("journalctl --grep='{}' --no-pager 2>/dev/null", journal_kw)
            } else {
                let unit_args: Vec<String> = units
                    .iter()
                    .map(|u| format!("-u '{}'", u.replace('\'', "'\\''")))
                    .collect();
                format!(
                    "journalctl {} --grep='{}' --no-pager 2>/dev/null",
                    unit_args.join(" "),
                    journal_kw
                )
            }
        }
        _ => {
            let paths: Vec<String> = log_path
                .split('\n')
                .filter(|p| !p.trim().is_empty())
                .map(|p| p.trim().to_string())
                .collect();
            if paths.is_empty() {
                return "echo 'No log paths configured'".to_string();
            }
            let q = |p: &str| shell_quote_path(p);
            // 历史日志：按文件名中的日期后缀对齐（app.log.2026-08-06 / app.log-20260806），
            // 取代按 mtime（-newermt）匹配，避免 gz 轮转日志 mtime=压缩次日导致的日期偏移。
            // 解压分支（gz_exec）自解压 *.gz，其余经 sh -c 透传 grep 直读，
            //  2>/dev/null 抑制 gzip 的 "not in gzip format" 噪音
            if date.is_some() || days.is_some() {
                let date_filter = date_name_filters(date, days);
                if !date_filter.is_empty() {
                    let cmds: Vec<String> = paths
                        .iter()
                        .map(|p| {
                            format!(
                                "DIR=$(dirname {}); BASE=$(basename {}); find \"$DIR\" -maxdepth 1 -type f \\( -name \"$BASE\" -o -name \"$BASE.*\" -o -name \"$BASE-*\" \\) -a {} -a {} 2>/dev/null",
                                q(p), q(p), date_filter, gz_exec,
                            )
                        })
                        .collect();
                    return cmds.join(" ; ");
                }
            }
            format!(
                "{} {} 2>/dev/null",
                grep,
                paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" ")
            )
        }
    }
}

fn build_context_command(log_type: &str, log_path: &str, start: usize, end: usize) -> String {
    let count = end - start + 1;
    match log_type {
        "docker" => {
            let containers: Vec<String> = log_path
                .split('\n')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();
            if containers.is_empty() {
                return "echo 'No containers configured'".to_string();
            }
            containers.iter()
                .map(|c| format!("docker logs '{}' 2>&1 | tail -n +{} | head -n {}", c.replace('\'', "'\\''"), start, count))
                .collect::<Vec<_>>()
                .join(" ; ")
        }
        "journalctl" => {
            let units: Vec<String> = log_path
                .split('\n')
                .filter(|u| !u.trim().is_empty())
                .map(|u| u.trim().to_string())
                .collect();
            let base_cmd = if units.is_empty() {
                "journalctl --no-pager -o cat 2>/dev/null".to_string()
            } else {
                let unit_args: Vec<String> = units.iter().map(|u| format!("-u '{}'", u.replace('\'', "'\\''"))).collect();
                format!("journalctl {} --no-pager -o cat 2>/dev/null", unit_args.join(" "))
            };
            format!("{} | tail -n +{} | head -n {}", base_cmd, start, count)
        }
        _ => {
            let paths: Vec<String> = log_path
                .split('\n')
                .filter(|p| !p.trim().is_empty())
                .map(|p| p.trim().to_string())
                .collect();
            if paths.is_empty() {
                return "echo 'No log paths configured'".to_string();
            }
            let q = |p: &str| shell_quote_path(p);
            paths.iter()
                .map(|p| format!("tail -n +{} {} 2>/dev/null | head -n {}", start, q(p), count))
                .collect::<Vec<_>>()
                .join(" ; ")
        }
    }
}

fn parse_grep_output(output: &str, spec: &LogKeywordSpec) -> Vec<Value> {
    // 按 lineNum 去重：grep -C 上下文行在多匹配邻近时会重叠，且 docker 多容器/多文件
    // 场景同一物理行可能被多个 grep 重复匹配。同一 lineNum 只保留一条，
    // 匹配行(isMatch=true)优先于上下文行。
    let mut seen: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() || line == "--" {
            continue;
        }
        let match_line = regex_match(line, r"^(?:[^:]*:)?(\d+):(.*)$");
        let context_line = regex_match(line, r"^(?:[^:]*:)?(\d+)-(.*)$");
        let parsed = match_line.or(context_line);
        if let Some((line_num, content)) = parsed {
            // Strip ANSI color codes
            let content = content
                .replace("\x1b[0m", "")
                .replace("\x1b[31m", "")
                .replace("\x1b[32m", "");
            let is_match = spec.is_match(&content);
            if let Some(existing) = seen.get(&line_num) {
                // 已存在：仅当新行是匹配行、旧行不是时才替换（匹配行优先）
                let existing_match = existing["isMatch"].as_bool().unwrap_or(false);
                if is_match && !existing_match {
                    seen.insert(line_num.clone(), json!({
                        "content": content,
                        "isMatch": is_match,
                        "lineNum": line_num
                    }));
                }
            } else {
                seen.insert(line_num.clone(), json!({
                    "content": content,
                    "isMatch": is_match,
                    "lineNum": line_num
                }));
                order.push(line_num);
            }
        }
    }

    order.into_iter().filter_map(|k| seen.remove(&k)).collect()
}

fn regex_match(line: &str, pattern: &str) -> Option<(String, String)> {
    // Match line: "lineNum:content" or "filename:lineNum:content"
    if pattern.contains(r"^(\d+):(.*)$")
        || pattern.contains(r"^(?:[^:]*:)?(\d+):(.*)$")
    {
        // Try "lineNum:content" (no filename)
        if let Some(pos) = line.find(':') {
            let num_part = &line[..pos];
            if !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
        // Fallback: "filename:lineNum:content" — split at most 3 parts
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() == 3 && parts[1].chars().all(|c| c.is_ascii_digit()) {
            return Some((parts[1].to_string(), parts[2].to_string()));
        }
    }

    // Context line: "lineNum-content" or "filename:lineNum-content"
    if pattern.contains(r"^(\d+)-(.*)$")
        || pattern.contains(r"^(?:[^:]*:)?(\d+)-(.*)$")
    {
        // 策略1: 带文件名前缀 "filename:lineNum-content"
        // 用 rfind(':') 找最后一个 ':'，然后检查后面的 "digits-content" 格式
        if let Some(colon_pos) = line.rfind(':') {
            let after_colon = &line[colon_pos + 1..];
            if let Some(dash_pos) = after_colon.find('-') {
                let line_num = &after_colon[..dash_pos];
                if !line_num.is_empty() && line_num.chars().all(|c| c.is_ascii_digit()) {
                    let content = &line[colon_pos + 1 + dash_pos + 1..];
                    return Some((line_num.to_string(), content.to_string()));
                }
            }
        }
        // 策略2: 直接 "lineNum-content"（无文件名前缀）
        if let Some(pos) = line.find('-') {
            let num_part = &line[..pos];
            if !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_grep_output_dedupes_overlapping_context_lines() {
        // grep -C 输出：匹配行 10 的上下文(8,9,10,11,12) 与匹配行 12 的上下文(10,11,12,13,14)
        // 重叠，10/11/12 各出现两次。同一 lineNum 只保留一条，匹配行优先。
        let output = "\
8-before line 8
9-before line 9
10:match keyword here
11-context line 11
12-match keyword too
--
10-context line 10 dup
11-context line 11 dup
12:match keyword too
13-after line 13
14-after line 14
";
        let lines = parse_grep_output(output, &LogKeywordSpec::parse("keyword", false));
        let nums: Vec<String> = lines
            .iter()
            .map(|l| l["lineNum"].as_str().unwrap_or("").to_string())
            .collect();
        // 每个 lineNum 只出现一次
        assert_eq!(
            nums,
            vec!["8", "9", "10", "11", "12", "13", "14"],
            "lineNum 不应重复"
        );
        // lineNum=10 和 12 应标记为匹配行（isMatch=true）
        let l10 = lines.iter().find(|l| l["lineNum"] == "10").unwrap();
        let l12 = lines.iter().find(|l| l["lineNum"] == "12").unwrap();
        assert_eq!(l10["isMatch"], true, "lineNum=10 应为匹配行");
        assert_eq!(l12["isMatch"], true, "lineNum=12 应为匹配行");
    }

    #[test]
    fn parse_grep_output_prefers_match_over_context_for_same_lineno() {
        // 同一 lineNum 既是上下文行又是匹配行时，保留匹配行版本
        let output = "\
5-keyword appears here
--
5-context only line
";
        let lines = parse_grep_output(output, &LogKeywordSpec::parse("keyword", false));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["lineNum"], "5");
        assert_eq!(lines[0]["isMatch"], true);
    }

    #[test]
    fn date_name_filters_matches_iso_and_compact_suffix() {
        // 指定某一天：应同时生成 ISO(-) 与紧凑(纯数字) 两种 -name 后缀条件
        let f = date_name_filters(Some("2026-08-19"), None);
        assert!(f.contains("2026-08-19"), "应包含 ISO 日期后缀: {}", f);
        assert!(f.contains("20260819"), "应包含紧凑日期后缀: {}", f);
        // 非当天，不应纳入无日期后缀的活动日志 BASE
        assert!(!f.contains("$BASE"), "非当天不应匹配活动日志: {}", f);
    }

    #[test]
    fn date_name_filters_today_includes_active_log() {
        // 当天（date 未指定、days=1 即今天）：除日期后缀外应额外纳入 $BASE
        // 以保证搜索当天时能命中尚未轮转（无日期后缀）的活动日志
        let f = date_name_filters(None, Some(1));
        assert!(f.contains("$BASE"), "当天应匹配活动日志 BASE: {}", f);
    }

    #[test]
    fn date_name_filters_days_builds_backward_range() {
        // days=3 应覆盖今天 + 前 2 天，且彼此用 -o 连接
        let today = chrono::Local::now().date_naive();
        let y1 = (today - chrono::Duration::days(1)).format("%Y%m%d").to_string();
        let y2 = (today - chrono::Duration::days(2)).format("%Y%m%d").to_string();
        let f = date_name_filters(None, Some(3));
        assert!(f.contains(&y1), "应包含前一天紧凑后缀: {}", f);
        assert!(f.contains(&y2), "应包含前两天紧凑后缀: {}", f);
        assert!(f.contains("$BASE"), "今天应含活动日志 BASE: {}", f);
        assert!(f.contains(" -o "), "多日期应 OR 连接: {}", f);
    }

    #[test]
    fn build_grep_file_historical_uses_date_filter_and_gz_branch() {
        // file 类型 + 指定 date：命令应包含按文件名后缀的日期过滤与 gz 解压分支
        // （不含 mtime 的 -newermt，说明已按文件名后缀对齐日期的实现生效）
        let preset = serde_json::json!({
            "logType": "file",
            "logPath": "/opt/logs/mall-server.log",
        });
        let cmd = build_grep_command(&preset, "支付下单", 0, Some("2026-08-18"), None, false);
        assert!(!cmd.contains("newermt"), "不应再按 mtime 匹配: {}", cmd);
        assert!(cmd.contains("gzip -cd"), "应含 gz 解压分支: {}", cmd);
        assert!(cmd.contains("-name \"*2026-08-18*\"") || cmd.contains("-name \"*20260818*\""),
            "应按文件名后缀日期过滤: {}", cmd);
        // 关键字以位置参数 "$1" 传入（历史分支脚本内不直接内嵌关键字）
        assert!(cmd.contains("sh -c ") && cmd.contains("'支付下单'"),
            "应使用 sh -c 分支且关键字以位置参数传入: {}", cmd);
    }

    #[test]
    fn keyword_spec_single_keyword_stays_fixed_string() {
        // 单关键词保持历史行为：-F 字面量，不做任何正则解析
        let spec = LogKeywordSpec::parse("trace-1.2.3", false);
        assert_eq!(spec.grep_flag, "-F");
        assert_eq!(spec.pattern, "trace-1.2.3");
        assert!(spec.is_match("xx trace-1.2.3 yy"));
    }

    #[test]
    fn keyword_spec_splits_escaped_pipe_into_or() {
        // shell 里常见的 `购卡\|buy-card\|PrepaidCard` 写法（反斜杠保留）应拆成多支 OR
        let spec = LogKeywordSpec::parse(r"购卡\|buy-card\|buy-card/cards\|生成卡\|面值\|PrepaidCard", false);
        assert_eq!(spec.grep_flag, "-E", "多关键词应切 -E: {}", spec.pattern);
        assert_eq!(
            spec.pattern,
            "购卡|buy-card|buy-card/cards|生成卡|面值|PrepaidCard"
        );
        assert!(spec.is_match("用户 buy-card/cards 请求"));
        assert!(spec.is_match("PrepaidCard 面值不足"));
        assert!(!spec.is_match("无关日志一行"));
    }

    #[test]
    fn keyword_spec_splits_bare_pipe_into_or() {
        // 未转义的 `|` 同样按 OR 处理
        let spec = LogKeywordSpec::parse("ERROR|WARN", false);
        assert_eq!(spec.grep_flag, "-E");
        assert_eq!(spec.pattern, "ERROR|WARN");
    }

    #[test]
    fn keyword_spec_branch_keeps_literal_semantics() {
        // 分支内的正则元字符按字面量转义：a.b 不应匹配 axb
        let spec = LogKeywordSpec::parse("a.b|c+d", false);
        assert_eq!(spec.pattern, r"a\.b|c\+d", "分支内应转义元字符: {}", spec.pattern);
        assert!(spec.is_match("xx a.b yy"));
        assert!(!spec.is_match("xx axb yy"), "分支内不应按正则解析");
    }

    #[test]
    fn keyword_spec_regex_mode_passes_pattern_through() {
        let spec = LogKeywordSpec::parse(r"ERROR.*timeout|panic", true);
        assert_eq!(spec.grep_flag, "-E");
        assert_eq!(spec.pattern, r"ERROR.*timeout|panic");
        assert!(spec.is_match("ERROR connect timeout"));
        assert!(!spec.is_match("ERROR ok"));
    }

    #[test]
    fn keyword_spec_regex_mode_falls_back_when_rust_regex_rejects() {
        // Rust regex 与远端 grep 的 ERE 语法不完全等价：编译失败不应拦截下发，
        // 但本地判定需退回子串包含，否则匹配行全部丢失高亮
        let spec = LogKeywordSpec::parse(r"a\{2\}", true);
        assert_eq!(spec.grep_flag, "-E");
        assert_eq!(spec.pattern, r"a\{2\}", "模式串应原样下发给 grep");
        assert!(spec.is_match("xx a{2} yy"), "编译失败时应退回子串包含判定");
    }

    #[test]
    fn keyword_spec_is_empty_for_separator_only_input() {
        // 只有分隔符时不应下发 grep（空模式会匹配所有行）
        assert!(LogKeywordSpec::parse("|", false).is_empty());
        assert!(LogKeywordSpec::parse(r"\|", false).is_empty());
        assert!(!LogKeywordSpec::parse("a", false).is_empty());
    }

    #[test]
    fn build_grep_command_multi_keyword_uses_extended_regex() {
        let preset = serde_json::json!({"logType": "file", "logPath": "/opt/logs/app.log"});
        let cmd = build_grep_command(&preset, "购卡|PrepaidCard", 5, None, None, false);
        assert!(cmd.contains("grep -C 5 -i -E -n -e '购卡|PrepaidCard'"),
            "多关键词应走 -E 并保留上下文行数: {}", cmd);
        assert!(!cmd.contains(" -F "), "不应再用 -F: {}", cmd);
    }

    #[test]
    fn build_grep_command_single_keyword_uses_fixed_string() {
        let preset = serde_json::json!({"logType": "file", "logPath": "/opt/logs/app.log"});
        let cmd = build_grep_command(&preset, "购卡", 0, None, None, false);
        assert!(cmd.contains("grep -i -F -n -e '购卡'"), "单关键词应保持 -F: {}", cmd);
    }

    #[test]
    fn parse_grep_output_marks_match_when_any_branch_hits() {
        let output = "\
10-上下文行
11:用户 购卡 请求
12:PrepaidCard 面值不足
13:无关行
";
        let lines = parse_grep_output(output, &LogKeywordSpec::parse("购卡|PrepaidCard", false));
        let l11 = lines.iter().find(|l| l["lineNum"] == "11").unwrap();
        let l12 = lines.iter().find(|l| l["lineNum"] == "12").unwrap();
        assert_eq!(l11["isMatch"], true, "命中任一分支应标记匹配行");
        assert_eq!(l12["isMatch"], true, "命中第二分支也应标记匹配行");
        let l10 = lines.iter().find(|l| l["lineNum"] == "10").unwrap();
        let l13 = lines.iter().find(|l| l["lineNum"] == "13").unwrap();
        assert_eq!(l10["isMatch"], false, "上下文行不应标记匹配");
        assert_eq!(l13["isMatch"], false, "未命中行不应标记匹配");
    }
}
