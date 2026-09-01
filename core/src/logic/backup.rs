use serde_json::{Value, json};

/// snake_case → camelCase（nginx_servers → nginxServers）
/// 仅用于备份导出/导入的固定表名集合转换（表名不含连续/首下划线，边界缺陷不会命中），
/// 不要用于任意 key（settings 等用户自定义 key 保持原样）
fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.push(c.to_ascii_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// SELECT * 整表导出为 JSON 数组（列名 = 数据库真实列名，永远与表结构同步）。
/// 表不存在返回 Ok(None)（调用方可静默跳过）；Blob 以 "[blob]" 占位。
fn export_table_rows(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<Option<Value>, String> {
    let sql = format!("SELECT * FROM {}", table);
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) if e.to_string().contains("no such table") => return Ok(None),
        Err(e) => return Err(format!("{}: {}", table, e)),
    };
    let col_names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();
    let rows = stmt
        .query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, col) in col_names.iter().enumerate() {
                match row.get::<_, rusqlite::types::Value>(i) {
                    Ok(rusqlite::types::Value::Null) => {}
                    Ok(rusqlite::types::Value::Integer(v)) => {
                        map.insert(col.clone(), json!(v));
                    }
                    Ok(rusqlite::types::Value::Real(v)) => {
                        map.insert(col.clone(), json!(v));
                    }
                    Ok(rusqlite::types::Value::Text(v)) => {
                        map.insert(col.clone(), json!(v));
                    }
                    Ok(rusqlite::types::Value::Blob(_v)) => {
                        map.insert(col.clone(), json!("[blob]"));
                    }
                    _ => {}
                }
            }
            Ok(json!(map))
        })
        .map_err(|e| format!("{}: {}", table, e))?;
    let arr: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
    Ok(Some(json!(arr)))
}

/// 跨机器路径改写：备份里形如 /Users/<其他用户>/... 的路径前缀替换为本机 home。
/// 仅改写确以「别的用户 home」开头的值，本机路径原样保留。
fn rewrite_home_path(val: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    rewrite_home_path_with(val, home.to_string_lossy().as_ref())
}

fn rewrite_home_path_with(val: &str, home_str: &str) -> Option<String> {
    let prefix = "/Users/";
    if val.starts_with(home_str) || !val.starts_with(prefix) {
        return None;
    }
    // /Users/<name>/rest → 本机 home + /rest；仅当 <name> 不是本机用户名
    let rest = &val[prefix.len()..];
    let first_seg_len = rest.find('/').unwrap_or(rest.len());
    if first_seg_len == 0 {
        return None;
    }
    Some(format!("{}{}", home_str, &rest[first_seg_len..]))
}

/// 对 JSON 值递归改写指定列名的路径前缀（server 配置里的 localPath/repoPath 等都是平铺字段，一层即可）
fn rewrite_path_fields(item: &mut Value, fields: &[&str]) -> usize {
    let mut count = 0;
    if let Some(obj) = item.as_object_mut() {
        for f in fields {
            if let Some(v) = obj.get_mut(*f).and_then(|v| v.as_str().map(|s| s.to_string())) {
                if let Some(new_path) = rewrite_home_path(&v) {
                    obj.insert(f.to_string(), json!(new_path));
                    count += 1;
                }
            }
        }
    }
    count
}

impl super::CoreService {
    /// 服务器整表导出（含 password 密文）——备份专用，恢复后无需重新录入密码
    fn export_servers_with_password(&self) -> impl Future<Output = Result<Value, String>> + Send + '_ {
        let this = self;
        async move {
            let r = this.db_read(|conn| {
                export_table_rows(conn, "servers")?
                    .ok_or_else(|| "servers 表不存在".to_string())
            })?;
            r
        }
    }

    pub async fn export_all_data(&self) -> Result<Value, String> {
        let todos = self.get_all_todos().await?;
        let projects = self.get_all_projects(true).await?;
        let servers = self.get_all_servers().await?;
        Ok(json!({
            "todos": todos,
            "projects": projects,
            "servers": servers,
        }))
    }

    pub async fn export_all_tables(&self) -> Result<Value, String> {
        // 收集导出过程中的错误，避免静默失败导致备份文件缺失数据
        let mut export_errors: Vec<String> = Vec::new();

        macro_rules! try_export {
            ($expr:expr, $name:literal) => {
                match $expr.await {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("[Backup Export] {} 失败: {}", $name, e);
                        export_errors.push(format!("{}: {}", $name, e));
                        json!([])
                    }
                }
            };
        }

        let todos = try_export!(self.get_all_todos(), "todos");
        let projects = try_export!(self.get_all_projects(true), "projects");
        // 服务器：直接 SELECT *（保留 password 密文，导入后无需重新录入）。
        // 不走 get_all_servers——它会剥离密码，导致恢复备份后所有服务器都要重新输密码。
        let servers = try_export!(self.export_servers_with_password(), "servers");
        let server_groups = try_export!(self.get_all_server_groups(), "serverGroups");
        let mfa_secrets = try_export!(self.get_all_mfa_secrets(), "mfaSecrets");
        let notes = try_export!(self.get_all_notes(None, None), "notes");
        let note_groups = try_export!(self.get_all_note_groups(), "noteGroups");
        let weekly_reports = try_export!(self.get_weekly_reports(9999), "weeklyReports");
        let accounting_categories = try_export!(self.get_accounting_categories(), "accountingCategories");
        let accounting_records_data = try_export!(self.get_accounting_records(json!({})), "accountingRecords");
        let accounting_records = accounting_records_data
            .get("records")
            .cloned()
            .unwrap_or(json!([]));
        let budgets = try_export!(self.get_budgets(), "accountingBudgets");
        let templates = try_export!(self.get_templates(), "accountingTemplates");
        let log_presets = try_export!(self.get_log_presets(), "logPresets");

        let mut all_subtasks = vec![];
        if let Some(todos_arr) = todos.as_array() {
            for todo in todos_arr {
                if let Some(id) = todo.get("id").and_then(|v| v.as_str()) {
                    if let Ok(st) = self.get_subtasks_for_todo(id).await {
                        if let Some(arr) = st.as_array() {
                            all_subtasks.extend(arr.clone());
                        }
                    }
                }
            }
        }

        let tags = self.get_all_tags().await.unwrap_or(json!([]));
        let lan_users = self.get_all_lan_users().await.unwrap_or(json!([]));
        let lan_msgs = self.get_all_lan_messages().await.unwrap_or(json!({}));

        // ---- Missing modules: fetch all via raw SQL ----
        let extra = self.export_extra_tables().await?;

        Ok({
            let mut map = serde_json::Map::new();
            map.insert("todos".into(), todos);
            map.insert("subtasks".into(), json!(all_subtasks));
            map.insert("tags".into(), tags);
            map.insert("projects".into(), projects);
            map.insert("servers".into(), servers);
            map.insert("serverGroups".into(), server_groups);
            map.insert("mfaSecrets".into(), mfa_secrets);
            map.insert("notes".into(), notes);
            map.insert("noteGroups".into(), note_groups);
            map.insert("weeklyReports".into(), weekly_reports);
            map.insert("accountingCategories".into(), accounting_categories);
            map.insert("accountingRecords".into(), accounting_records);
            map.insert("accountingBudgets".into(), budgets);
            map.insert("accountingTemplates".into(), templates);
            map.insert("logPresets".into(), log_presets);
            // CICD 五表全部走 SELECT * 原样导出（含 environments/outputPath 等新列，
            // 不再用手写列清单 getter——此前漏列导致多环境/增量上传配置静默丢失）
            for &t in &["cicd_configs", "deploy_modules", "deploy_logs", "deploy_history", "deploy_step_logs"] {
                if let Ok(Ok(Some(rows))) = self.db_read(|conn| export_table_rows(conn, t)) {
                    map.insert(snake_to_camel(t), rows);
                }
            }
            map.insert("users".into(), lan_users);
            map.insert("messages".into(), lan_msgs.get("messages").cloned().unwrap_or(json!([])));
            map.insert("chatMessages".into(), lan_msgs.get("chatMessages").cloned().unwrap_or(json!([])));
            map.insert("fileTransfers".into(), lan_msgs.get("fileTransfers").cloned().unwrap_or(json!([])));
            // Extra tables from missing modules
            for (k, v) in extra.as_object().unwrap_or(&serde_json::Map::new()).clone() {
                map.insert(k, v);
            }
            // 嵌入导出错误（导入时会自然忽略此字段）
            if !export_errors.is_empty() {
                map.insert("_exportErrors".into(), json!(export_errors));
            }
            json!(map)
        })
    }

    // Helper: bulk-export tables that don't have dedicated CoreService getters
    async fn export_extra_tables(&self) -> Result<Value, String> {
        let extra = self.db_read(|conn| -> Result<Value, String> {
            let mut result = serde_json::Map::new();

            // servers：含 password 密文原样导出（恢复后无需重新录入密码）
            if let Some(v) = export_table_rows(conn, "servers")? {
                result.insert("servers".to_string(), v);
            }

            // Settings as a JSON object (key-value pairs)
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM settings") {
                let mut settings = serde_json::Map::new();
                if let Ok(rows) = stmt.query_map([], |row| {
                    let key: String = row.get(0)?;
                    let val: String = row.get(1)?;
                    Ok((key, val))
                }) {
                    for row in rows.flatten() {
                        settings.insert(row.0, json!(row.1));
                    }
                }
                result.insert("settings".to_string(), json!(settings));
            }

            // Generic table export
            let tables = [
                "wireguard_configs", "git_repos",
                "calculator_history", "api_requests",
                "nginx_presets", "nginx_config_versions", "nginx_servers",
                "nginx_locations", "nginx_upstreams", "nginx_upstream_servers",
                "nginx_http_params", "nginx_streams", "nginx_certs",
                "nginx_templates", "nginx_basic_settings", "nginx_params",
                "nginx_deny_allows", "nginx_passwords",
                "alert_email_config", "alert_services", "alert_resources", "alert_history",
            ];

            for &table in &tables {
                if let Some(v) = export_table_rows(conn, table)? {
                    // key 统一驼峰（与导入端一致；旧格式下划线由导入端归一化兼容）
                    result.insert(snake_to_camel(table).to_string(), v);
                }
            }

            Ok(json!(result))
        })?; // unwrap outer Result from db_read
        extra
    }

    // ============ 自动备份（后端真实调度，替代前端空壳） ============

    /// 静默自动备份：导出 all-data.json + receipts 打包为 zip，写入 dir，
    /// 文件名 supertool-auto-YYYY-MM-DD.stbackup（同日覆盖），并轮转保留最近 keep 份。
    /// 成功返回备份文件完整路径。
    pub async fn run_auto_backup(&self, dir: &str, keep: usize) -> Result<String, String> {
        let data = self.export_all_tables().await?;
        let data_json = serde_json::to_string(&data).map_err(|e| format!("序列化失败: {}", e))?;

        let dir_path = if dir.trim().is_empty() {
            crate::logic::data_dir::resolve_data_dir().join("backups")
        } else {
            std::path::PathBuf::from(shellexpand_home(dir))
        };
        std::fs::create_dir_all(&dir_path).map_err(|e| format!("创建备份目录失败: {}", e))?;

        let file_name = format!(
            "supertool-auto-{}.stbackup",
            chrono::Local::now().format("%Y-%m-%d")
        );
        let dest = dir_path.join(&file_name);

        write_backup_zip(&data_json, &dest)?;

        // 轮转：删除最旧的 auto 备份，保留 keep 份
        rotate_backups(&dir_path, keep);

        Ok(dest.to_string_lossy().to_string())
    }

    /// 返回 (imported, skipped, errors, path_rewritten)
    pub async fn import_all_tables(
        &self,
        data: Value,
        mode: &str,
    ) -> Result<(usize, usize, Vec<String>, usize), String> {
        let mut imported = 0usize;
        let mut skipped = 0usize;
        let mut errors: Vec<String> = Vec::new();

        // 兼容早期备份文件：导出端曾用下划线 key（nginx_servers），
        // 导入端统一驼峰（nginxServers）。归一化一次，后续 data.get 全用驼峰。
        let mut data = data;
        if let Some(obj) = data.as_object_mut() {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if k.contains('_') {
                    let camel = snake_to_camel(&k);
                    if !obj.contains_key(&camel) {
                        if let Some(v) = obj.remove(&k) {
                            obj.insert(camel, v);
                        }
                    }
                }
            }
        }

        // 统一表清单：key（备份 JSON 顶层驼峰名）→ (表名, 主键, 路径字段)。
        // 导入引擎按目标表 PRAGMA table_info 动态取列：备份列多余则忽略、目标列缺失填默认，
        // 旧/新备份文件与表结构演化彻底解耦（不再硬编码列清单）。
        // mode: replace=备份覆盖本地（同 id INSERT OR REPLACE）；merge=本地优先跳过重复。
        const GENERIC_TABLES: &[(&str, &str, &str, &[&str])] = &[
            // (jsonKey, tableName, pk, pathFields)
            ("projects", "projects", "id", &["repoPath", "repoPath2"]),
            ("servers", "servers", "id", &["sshKeyPath"]),
            ("serverGroups", "server_groups", "id", &[]),
            ("todos", "todos", "id", &[]),
            ("subtasks", "subtasks", "id", &[]),
            ("tags", "tags", "id", &[]),
            ("mfaSecrets", "mfa_secrets", "id", &[]),
            ("notes", "notes", "id", &[]),
            ("noteGroups", "note_groups", "id", &[]),
            ("weeklyReports", "weekly_reports", "id", &[]),
            ("accountingCategories", "accounting_categories", "id", &[]),
            ("accountingRecords", "accounting_records", "id", &[]),
            ("accountingBudgets", "budgets", "id", &[]),
            ("accountingTemplates", "templates", "id", &[]),
            ("logPresets", "log_presets", "id", &[]),
            ("users", "users", "id", &[]),
            ("messages", "messages", "id", &[]),
            ("chatMessages", "chat_messages", "id", &[]),
            ("fileTransfers", "file_transfers", "id", &[]),
            ("wireguardConfigs", "wireguard_configs", "id", &[]),
            ("gitRepos", "git_repos", "id", &["path"]),
            ("calculatorHistory", "calculator_history", "id", &[]),
            ("apiRequests", "api_requests", "id", &[]),
            ("nginxPresets", "nginx_presets", "id", &[]),
            ("nginxConfigVersions", "nginx_config_versions", "id", &[]),
            ("nginxServers", "nginx_servers", "id", &[]),
            ("nginxLocations", "nginx_locations", "id", &[]),
            ("nginxUpstreams", "nginx_upstreams", "id", &[]),
            ("nginxUpstreamServers", "nginx_upstream_servers", "id", &[]),
            ("nginxHttpParams", "nginx_http_params", "id", &[]),
            ("nginxStreams", "nginx_streams", "id", &[]),
            ("nginxCerts", "nginx_certs", "id", &[]),
            ("nginxTemplates", "nginx_templates", "id", &[]),
            ("nginxBasicSettings", "nginx_basic_settings", "id", &[]),
            ("nginxParams", "nginx_params", "id", &[]),
            ("nginxDenyAllows", "nginx_deny_allows", "id", &[]),
            ("nginxPasswords", "nginx_passwords", "id", &[]),
            ("alertEmailConfig", "alert_email_config", "id", &[]),
            ("alertServices", "alert_services", "id", &[]),
            ("alertResources", "alert_resources", "id", &[]),
            ("alertHistory", "alert_history", "id", &[]),
            // CICD 五表（此前走独立 import_cicd_data，硬编码列清单漏 5 个新列静默丢配置，现统一走通用引擎）
            ("cicdConfigs", "cicd_configs", "id", &["localPath"]),
            ("deployModules", "deploy_modules", "id", &[]),
            ("deployLogs", "deploy_logs", "id", &[]),
            ("deployHistory", "deploy_history", "id", &[]),
            ("deployStepLogs", "deploy_step_logs", "id", &[]),
        ];

        let mode_owned = if mode == "replace" { "replace" } else { "merge" };

        let result: Result<Result<(usize, usize, Vec<String>, usize), String>, String> = self.db_write_tx(|conn| {
            let mut path_rewritten = 0usize;
            // 事务内先做路径改写（纯 JSON 变换，再写库）
            if let Some(items) = data.get_mut("servers").and_then(|v| v.as_array_mut()) {
                for item in items.iter_mut() {
                    path_rewritten += rewrite_path_fields(item, &["sshKeyPath"]);
                }
            }
            if let Some(items) = data.get_mut("projects").and_then(|v| v.as_array_mut()) {
                for item in items.iter_mut() {
                    path_rewritten += rewrite_path_fields(item, &["repoPath", "repoPath2"]);
                }
            }
            if let Some(items) = data.get_mut("gitRepos").and_then(|v| v.as_array_mut()) {
                for item in items.iter_mut() {
                    path_rewritten += rewrite_path_fields(item, &["path"]);
                }
            }
            if let Some(items) = data.get_mut("cicdConfigs").and_then(|v| v.as_array_mut()) {
                for item in items.iter_mut() {
                    path_rewritten += rewrite_path_fields(item, &["localPath"]);
                }
            }
            if let Some(settings) = data.get_mut("settings").and_then(|v| v.as_object_mut()) {
                for (_, v) in settings.iter_mut() {
                    if let Some(s) = v.as_str() {
                        if let Some(p) = rewrite_home_path(s) {
                            *v = json!(p);
                            path_rewritten += 1;
                        }
                    }
                }
            }

            // replace：清空全部备份涉及的表（一个事务内，失败整体回滚）
            if mode_owned == "replace" {
                for (_, table, _, _) in GENERIC_TABLES {
                    conn.execute(&format!("DELETE FROM {}", table), [])
                        .map_err(|e| format!("清空 {}: {}", table, e))?;
                }
                conn.execute("DELETE FROM settings", [])
                    .map_err(|e| format!("清空 settings: {}", e))?;
            }

            // Settings：key-value 对象
            if let Some(settings) = data.get("settings").and_then(|v| v.as_object()) {
                for (key, value) in settings {
                    let val_str = if value.is_string() {
                        value.as_str().unwrap_or("").to_string()
                    } else {
                        serde_json::to_string(value).unwrap_or_default()
                    };
                    let sql = if mode_owned == "replace" {
                        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)"
                    } else {
                        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)"
                    };
                    conn.execute(sql, rusqlite::params![key, val_str])
                        .map_err(|e| format!("settings({}): {}", key, e))?;
                    imported += 1;
                }
            }

            for (json_key, table, pk, _path_fields) in GENERIC_TABLES {
                let items = match data.get(*json_key).and_then(|v| v.as_array()) {
                    Some(a) if !a.is_empty() => a,
                    _ => continue,
                };
                // 目标表真实列：PRAGMA table_info（不存在列名/类型/非空/默认值）
                let mut pragma = conn
                    .prepare(&format!("PRAGMA table_info({})", table))
                    .map_err(|e| format!("{}: {}", table, e))?;
                let cols: Vec<(String, String, i64, Option<String>)> = pragma
                    .query_map([], |r| {
                        Ok((
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, i64>(3)?,
                            r.get::<_, Option<String>>(4)?,
                        ))
                    })
                    .map_err(|e| format!("{}: {}", table, e))?
                    .filter_map(|r| r.ok())
                    .collect();
                drop(pragma);
                if cols.is_empty() {
                    continue; // 表不存在，跳过
                }

                // 列名一律加双引号：group/order/index/key 等是 SQLite 保留字或常见列名，
                // 裸写会导致 "near \"group\": syntax error"（log_presets 等表命中）
                let col_list: Vec<String> = cols
                    .iter()
                    .map(|(n, _, _, _)| format!("\"{}\"", n.replace('"', "\"\"")))
                    .collect();
                let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{}", i)).collect();
                let upsert = if mode_owned == "replace" {
                    "INSERT OR REPLACE"
                } else {
                    "INSERT OR IGNORE"
                };
                let sql = format!(
                    "{} INTO {} ({}) VALUES ({})",
                    upsert,
                    table,
                    col_list.join(", "),
                    placeholders.join(", ")
                );

                let mut table_imported = 0usize;
                let mut table_skipped = 0usize;
                let mut table_errors = 0usize;
                for item in items {
                    // 值绑定：备份 JSON 有该列则用备份值（类型自适应），没有则用列默认值
                    let mut param_values: Vec<rusqlite::types::Value> = Vec::with_capacity(cols.len());
                    for (name, ctype, notnull, default) in &cols {
                        let v = item.get(name.as_str());
                        let bound = match v {
                            Some(Value::String(s)) => rusqlite::types::Value::Text(s.clone()),
                            Some(Value::Number(n)) => {
                                if let Some(i) = n.as_i64() {
                                    rusqlite::types::Value::Integer(i)
                                } else if let Some(f) = n.as_f64() {
                                    rusqlite::types::Value::Real(f)
                                } else {
                                    default_value(ctype, *notnull, default)
                                }
                            }
                            Some(Value::Bool(b)) => rusqlite::types::Value::Integer(*b as i64),
                            Some(Value::Null) | None => default_value(ctype, *notnull, default),
                            Some(other) => {
                                // 对象/数组 → JSON 字符串
                                match serde_json::to_string(other) {
                                    Ok(s) => rusqlite::types::Value::Text(s),
                                    Err(_) => default_value(ctype, *notnull, default),
                                }
                            }
                        };
                        param_values.push(bound);
                    }
                    // merge 模式主键判重（显式 skip，与 IGNORE 语义一致但可计数）
                    if mode_owned == "merge" {
                        if let Some(pk_val) = item.get(*pk) {
                            let exists: i64 = conn
                                .query_row(
                                    &format!(
                                        "SELECT COUNT(*) FROM {} WHERE \"{}\" = ?1",
                                        table, pk
                                    ),
                                    [pk_val.to_string()],
                                    |r| r.get(0),
                                )
                                .unwrap_or(0);
                            if exists > 0 {
                                table_skipped += 1;
                                continue;
                            }
                        }
                    }
                    match conn.execute(&sql, rusqlite::params_from_iter(param_values.iter())) {
                        Ok(_) => table_imported += 1,
                        Err(e) => {
                            table_errors += 1;
                            let id_disp = item.get(*pk).map(|v| v.to_string()).unwrap_or_default();
                            errors.push(format!("{}({}): {}", table, id_disp, e));
                        }
                    }
                }
                imported += table_imported;
                skipped += table_skipped;
                log::info!(
                    "[Backup] {}: imported={}, skipped={}, errors={}",
                    table, table_imported, table_skipped, table_errors
                );
            }

            if path_rewritten > 0 {
                log::info!("[Backup] 跨机器路径改写 {} 处（/Users/<源机器用户> → 本机 home）", path_rewritten);
            }
            if !errors.is_empty() {
                log::warn!("[Backup Import] {} errors occurred", errors.len());
                for e in errors.iter().take(5) {
                    log::warn!("  - {}", e);
                }
            }
            Ok((imported, skipped, errors, path_rewritten))
        });
        Ok(result.map_err(|e| format!("db_write failed: {}", e))??)
    }
}

/// 列默认值推导：备份 JSON 缺列时按目标表列定义兜底
fn default_value(ctype: &str, notnull: i64, default: &Option<String>) -> rusqlite::types::Value {
    if let Some(d) = default {
        // SQLite 默认值形态：'text' / 数字 / CURRENT_TIMESTAMP
        let d = d.trim();
        if let Some(inner) = d.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            return rusqlite::types::Value::Text(inner.to_string());
        }
        if let Ok(i) = d.parse::<i64>() {
            return rusqlite::types::Value::Integer(i);
        }
        if let Ok(f) = d.parse::<f64>() {
            return rusqlite::types::Value::Real(f);
        }
        if d.eq_ignore_ascii_case("CURRENT_TIMESTAMP") {
            return rusqlite::types::Value::Text(chrono::Utc::now().to_rfc3339());
        }
    }
    if notnull != 0 {
        // NOT NULL 无默认：按类型给零值，避免整行插入失败
        let t = ctype.to_ascii_uppercase();
        if t.contains("INT") || t.contains("REAL") || t.contains("FLOA") || t.contains("DOUB") || t.contains("NUM") {
            return rusqlite::types::Value::Integer(0);
        }
        return rusqlite::types::Value::Text(String::new());
    }
    rusqlite::types::Value::Null
}

/// 展开 ~ 前缀到本机 home
fn shellexpand_home(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    p.to_string()
}

/// 打包 all-data.json + receipts/ 为 .stbackup zip
fn write_backup_zip(data_json: &str, dest: &std::path::Path) -> Result<(), String> {
    use std::io::{Cursor, Write as IoWrite};
    let buf = std::sync::Mutex::new(Cursor::new(Vec::new()));
    {
        let mut guard = buf.lock().map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(&mut *guard);
        let opts: zip::write::FileOptions<()> =
            zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("all-data.json", opts)
            .map_err(|e| format!("ZIP创建失败: {}", e))?;
        zip.write_all(data_json.as_bytes())
            .map_err(|e| format!("写入ZIP失败: {}", e))?;

        // 收据附件
        let receipt_dir = crate::logic::data_dir::resolve_data_dir().join("accounting-receipts");
        if receipt_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&receipt_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                        let filename = entry.file_name();
                        if let Ok(content) = std::fs::read(entry.path()) {
                            let zip_path = format!("receipts/{}", filename.to_string_lossy());
                            let _ = zip.start_file(&zip_path, opts);
                            let _ = zip.write_all(&content);
                        }
                    }
                }
            }
        }
        zip.finish().map_err(|e| format!("ZIP完成失败: {}", e))?;
    }
    let bytes = buf.into_inner().map_err(|e| e.to_string())?.into_inner();
    // 先写临时文件再原子改名，避免写一半崩溃留下损坏备份
    let tmp = dest.with_extension("tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("写入临时文件失败: {}", e))?;
    std::fs::rename(&tmp, dest).map_err(|e| format!("落盘失败: {}", e))?;
    Ok(())
}

/// 轮转：只保留最近 keep 份 supertool-auto-*.stbackup
fn rotate_backups(dir: &std::path::Path, keep: usize) {
    use std::fs;
    let mut autos: Vec<(std::path::PathBuf, std::time::SystemTime)> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("supertool-auto-")
            })
            .filter_map(|e| {
                let mtime = e.metadata().ok()?.modified().ok()?;
                Some((e.path(), mtime))
            })
            .collect(),
        Err(_) => return,
    };
    if autos.len() <= keep {
        return;
    }
    autos.sort_by_key(|(_, m)| *m);
    let to_delete = autos.len() - keep;
    for (path, _) in autos.iter().take(to_delete) {
        if let Err(e) = fs::remove_file(path) {
            log::warn!("[AutoBackup] 轮转删除失败 {}: {}", path.display(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_home_path_cross_machine() {
        let rewritten = rewrite_home_path_with("/Users/duormi/workspace/ehuipay", "/Users/fufengyuan");
        assert_eq!(rewritten.as_deref(), Some("/Users/fufengyuan/workspace/ehuipay"));
    }

    #[test]
    fn test_rewrite_home_path_keeps_local() {
        // 本机路径不改写
        assert_eq!(rewrite_home_path_with("/Users/fufengyuan/IdeaProjects/x", "/Users/fufengyuan"), None);
        // 非 /Users 前缀不改写
        assert_eq!(rewrite_home_path_with("/opt/apphome", "/Users/fufengyuan"), None);
        assert_eq!(rewrite_home_path_with("/Users/", "/Users/fufengyuan"), None);
        // 无后续段不改写
        assert_eq!(rewrite_home_path_with("/Users/duormi", "/Users/fufengyuan"), Some("/Users/fufengyuan".to_string()));
    }

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("git_repos"), "gitRepos");
        assert_eq!(snake_to_camel("deploy_step_logs"), "deployStepLogs");
        assert_eq!(snake_to_camel("servers"), "servers");
    }

    /// 列名含 SQLite 保留字（group/key/order）时必须加引号，否则 INSERT 语法错误。
    /// 回归用例：旧实现裸写列名导致 log_presets 14 条全部导入失败。
    #[test]
    fn test_quote_identifier() {
        let quoted: String = "\"group\"".to_string();
        assert_eq!(quoted, "\"group\"");
        // 内含双引号需转义
        let name = "we\"ird";
        assert_eq!(format!("\"{}\"", name.replace('"', "\"\"")), "\"we\"\"ird\"");
    }

    /// 密钥轮换 roundtrip：轮换后存量服务器密码必须仍可解密（否则等于密码丢失）
    #[tokio::test]
    async fn rotate_key_preserves_server_passwords() {
        // 本测试会临时切换全局 active key，必须持锁独占，
        // 否则并行测试可能在窗口内用错密钥（见 encryption::TEST_KEY_LOCK）
        let _key_guard = crate::encryption::lock_test_key().await;
        use base64::Engine as _;
        let dir = std::env::temp_dir().join(format!("st_rotate_{}_{}", std::process::id(), rand_suffix()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("t.db");
        let _ = std::fs::remove_file(&db_path);
        let db = crate::db::Database::new(&db_path).unwrap();
        let core = crate::logic::CoreService::new(db, dir.clone());

        let pw = "SuperSecret123!";
        let enc = crate::encryption::encrypt_password(pw).await.unwrap();
        core.db_write(move |conn| {
            conn.execute(
                "INSERT INTO servers (id,name,host,port,username,password,description,tags,createdAt,updatedAt) \
                 VALUES ('s1','test','1.2.3.4',22,'root',?1,'','','2026-01-01','2026-01-01')",
                rusqlite::params![&enc],
            )
            .unwrap();
        })
        .unwrap();

        // 轮换前可解密
        let before = core.get_server_by_id("s1").await.unwrap();
        assert_eq!(
            before.get("password").and_then(|v| v.as_str()),
            Some(pw),
            "轮换前解密失败"
        );

        // 轮换：prepare（旧密钥解密）→ commit(新密钥重加密，active key 仍旧）→ 再切换 key
        let (total, failed) = core.rotate_encryption_key_prepare().await.unwrap();
        assert!(failed.is_empty(), "prepare 出现解密失败: {:?}", failed);
        assert_eq!(total, 1, "应扫描到 1 条密文");

        let new_key_bytes = [7u8; 32];
        let n = core.commit_rotation(&new_key_bytes).await.unwrap();
        assert_eq!(n, 1, "应重加密 1 条");

        // commit 后、切 key 前：active key 仍是默认，此时用默认 key 读应失败（密文已是新 key）
        // （体现"先写回后切换"的安全中间态）
        let nk = base64::engine::general_purpose::STANDARD.encode(new_key_bytes);
        // ⚠️ 必须用 _for_test 版本（只切内存缓存）：生产 set_custom_key 会写真实数据目录的
        // .encryption_key，曾把测试密钥 [7u8;32] 写进开发者本机导致全部真实密文解不开（2026-09-01）
        crate::encryption::set_custom_key_for_test(&nk).await;
        let after = core.get_server_by_id("s1").await.unwrap();
        assert_eq!(
            after.get("password").and_then(|v| v.as_str()),
            Some(pw),
            "轮换后解密失败——密码丢失"
        );

        // 还原全局密钥为「未设置」，避免污染其他测试（默认密钥假设）
        crate::encryption::clear_custom_key_for_test().await;
        let _ = std::fs::remove_dir_all(&dir);

        // 防回归断言：测试绝不能把密钥写进真实数据目录（2026-09-01 事故防线）。
        // 若真实 .encryption_key 存在，其内容绝不能等于测试密钥 [7u8;32] 的 base64。
        let real_key = crate::logic::data_dir::encryption_key_path();
        if let Ok(content) = std::fs::read_to_string(&real_key) {
            assert_ne!(
                content.trim(),
                nk,
                "测试密钥泄漏到了真实数据目录 {} —— 单测不得调用 set_custom_key",
                real_key.display()
            );
        }
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
