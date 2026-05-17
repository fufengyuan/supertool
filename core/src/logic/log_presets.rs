use super::ssh;
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
                .map_err(|e| e.to_string())
        })?;

        let server_ids: Vec<String> =
            serde_json::from_str(preset["serverIds"].as_str().unwrap_or("[]")).unwrap_or_default();

        if server_ids.is_empty() {
            return Ok(
                json!({"presetId": preset_id, "lines": lines, "results": [], "note": "No servers configured"}),
            );
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

    pub async fn log_search(
        &self,
        preset_id: &str,
        keyword: &str,
        lines: usize,
    ) -> Result<Value, String> {
        if keyword.trim().is_empty() {
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
                .map_err(|e| e.to_string())
        })?;

        let server_ids: Vec<String> =
            serde_json::from_str(preset["serverIds"].as_str().unwrap_or("[]")).unwrap_or_default();

        let cmd = build_grep_command(&preset, keyword, lines);
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
                    let lines = parse_grep_output(&exec_result.output, keyword);
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
            let q = |p: &str| format!("'{}'", p.replace('\'', "'\\''"));
            format!(
                "tail -n {} {} 2>/dev/null",
                lines,
                paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" ")
            )
        }
    }
}

fn build_grep_command(preset: &Value, keyword: &str, context_lines: usize) -> String {
    let log_type = preset["logType"].as_str().unwrap_or("file");
    let log_path = preset["logPath"].as_str().unwrap_or("");
    let escaped_kw = keyword.replace('\'', "'\\''");
    let grep_ctx = if context_lines > 0 {
        format!(" -C {}", context_lines)
    } else {
        String::new()
    };
    let grep = format!("grep{} -i -n '{}'", grep_ctx, escaped_kw);

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
                        c,
                        format!("grep{} -i -n '{}'", grep_ctx, escaped_kw)
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
                format!("journalctl --grep='{}' --no-pager 2>/dev/null", escaped_kw)
            } else {
                let unit_args: Vec<String> = units
                    .iter()
                    .map(|u| format!("-u '{}'", u.replace('\'', "'\\''")))
                    .collect();
                format!(
                    "journalctl {} --grep='{}' --no-pager 2>/dev/null",
                    unit_args.join(" "),
                    escaped_kw
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
            let q = |p: &str| format!("'{}'", p.replace('\'', "'\\''"));
            format!(
                "{} {} 2>/dev/null",
                grep,
                paths.iter().map(|p| q(p)).collect::<Vec<_>>().join(" ")
            )
        }
    }
}

fn parse_grep_output(output: &str, keyword: &str) -> Vec<Value> {
    let kw_lower = keyword.to_lowercase();
    output
        .lines()
        .filter(|l| l.trim().is_empty() || *l == "--")
        .count(); // consume filter

    output
        .lines()
        .filter(|l| !l.trim().is_empty() && *l != "--")
        .filter_map(|line| {
            // grep -n output: "filename:lineNum:content" or "lineNum:content"
            // match line: lineNum:content, context line: lineNum-content
            let match_line = regex_match(line, r"^(?:[^:]*:)?(\d+):(.*)$");
            let context_line = regex_match(line, r"^(?:[^:]*:)?(\d+)-(.*)$");
            let parsed = match_line.or(context_line);

            parsed.map(|(line_num, content)| {
                // Strip ANSI color codes
                let content = content
                    .replace("\x1b[0m", "")
                    .replace("\x1b[31m", "")
                    .replace("\x1b[32m", "");
                let is_match = content.to_lowercase().contains(&kw_lower);
                json!({
                    "content": content,
                    "isMatch": is_match,
                    "lineNum": line_num
                })
            })
        })
        .collect()
}

fn regex_match(line: &str, pattern: &str) -> Option<(String, String)> {
    if pattern.contains(r"^(\d+):(.*)$") {
        if let Some(pos) = line.find(':') {
            let num_part = &line[..pos];
            if num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
    }
    if pattern.contains(r"^(\d+)-(.*)$") {
        if let Some(pos) = line.find('-') {
            let num_part = &line[..pos];
            if num_part.chars().all(|c| c.is_ascii_digit()) {
                return Some((num_part.to_string(), line[pos + 1..].to_string()));
            }
        }
    }
    if pattern.contains(r"^(?:[^:]*:)?(\d+):(.*)$") {
        // "filename:lineNum:content" or "lineNum:content"
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 2 {
            if parts.len() == 2 {
                // "lineNum:content"
                if parts[0].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            } else {
                // "filename:lineNum:content"
                if parts[1].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[1].to_string(), parts[2].to_string()));
                }
            }
        }
    }
    if pattern.contains(r"^(?:[^:]*:)?(\d+)-(.*)$") {
        // Context line with dash separator
        let parts: Vec<&str> = line.splitn(3, '-').collect();
        if parts.len() >= 2 {
            if parts.len() == 2 {
                if parts[0].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            } else {
                if parts[1].chars().all(|c| c.is_ascii_digit()) {
                    return Some((parts[1].to_string(), parts[2].to_string()));
                }
            }
        }
    }
    None
}
