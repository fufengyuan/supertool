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

impl super::CoreService {
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
    pub async fn import_all_data(&self, data: Value, mode: &str) -> Result<Value, String> {
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let data_clone = data.clone();
        let mode_owned = mode.to_string();
        if let Some(obj) = data.as_object() {
            // Import todos
            if let Some(items) = obj.get("todos").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_todo(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("todos".into(), c);
            }
            // Import projects
            if let Some(items) = obj.get("projects").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_project(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("projects".into(), c);
            }
            // Import servers
            if let Some(items) = obj.get("servers").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("servers".into(), c);
            }
            // Import tags
            if let Some(items) = obj.get("tags").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        if self.add_tag(name).await.is_ok() {
                            c += 1;
                        }
                    }
                }
                counts.insert("tags".into(), c);
            }
            // Import subtasks
            if let Some(items) = obj.get("subtasks").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_subtask(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("subtasks".into(), c);
            }
            // Import serverGroups
            if let Some(items) = obj.get("serverGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_server_group(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("serverGroups".into(), c);
            }
            // Import mfaSecrets
            if let Some(items) = obj.get("mfaSecrets").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_mfa_secret(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("mfaSecrets".into(), c);
            }
            // Import notes
            if let Some(items) = obj.get("notes").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("notes".into(), c);
            }
            // Import noteGroups
            if let Some(items) = obj.get("noteGroups").and_then(|v| v.as_array()) {
                let mut c = 0u32;
                for item in items {
                    if self.add_note_group(item.clone()).await.is_ok() {
                        c += 1;
                    }
                }
                counts.insert("noteGroups".into(), c);
            }
        }
        // CICD data handled separately
        let (cicd_c, cicd_s) = self
            .import_cicd_data(&data_clone, &mode_owned)
            .await
            .unwrap_or((0, 0));
        counts.insert("cicdConfigs".into(), cicd_c as u32 + cicd_s as u32);
        Ok(json!(counts))
    }
    // ============ Full Backup (unified with Tauri) ============

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
        let servers = try_export!(self.get_all_servers(), "servers");
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
        let cicd_data = self.get_all_cicd_data().await.unwrap_or(json!({}));
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
            map.insert("cicdConfigs".into(), cicd_data.get("cicdConfigs").cloned().unwrap_or(json!([])));
            map.insert("deployModules".into(), cicd_data.get("deployModules").cloned().unwrap_or(json!([])));
            map.insert("deployLogs".into(), cicd_data.get("deployLogs").cloned().unwrap_or(json!([])));
            map.insert("deployHistory".into(), cicd_data.get("deployHistory").cloned().unwrap_or(json!([])));
            map.insert("deployStepLogs".into(), cicd_data.get("deployStepLogs").cloned().unwrap_or(json!([])));
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
                let sql = format!("SELECT * FROM {}", table);
                if let Ok(mut stmt) = conn.prepare(&sql) {
                    let col_count = stmt.column_count();
                    let col_names: Vec<String> = (0..col_count)
                        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
                        .collect();
                    if let Ok(rows) = stmt.query_map([], |row| {
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
                    }) {
                        let arr: Vec<Value> = rows.flatten().collect();
                        // key 统一驼峰（与导入端一致；旧格式下划线由导入端归一化兼容）
                        result.insert(snake_to_camel(table).to_string(), json!(arr));
                    }
                }
            }

            Ok(json!(result))
        })?; // unwrap outer Result from db_read
        extra
    }

    pub async fn import_all_tables(
        &self,
        data: Value,
        mode: &str,
    ) -> Result<(usize, usize, Vec<String>), String> {
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

        // Get direct DB access for batch import
        self.db_write(|conn| {
            if mode == "replace" {
                // CICD 表（cicd_configs/deploy_modules/deploy_logs/deploy_history/deploy_step_logs）
                // 由 core.import_cicd_data 内部自行清空，避免在此预删后若 CICD 导入失败导致数据丢失
                if let Err(e) = conn.execute_batch("
                    DELETE FROM chat_messages;
                    DELETE FROM file_transfers;
                    DELETE FROM messages;
                    DELETE FROM subtasks;
                    DELETE FROM notes;
                    DELETE FROM note_groups;
                    DELETE FROM mfa_secrets;
                    DELETE FROM servers;
                    DELETE FROM server_groups;
                    DELETE FROM weekly_reports;
                    DELETE FROM todos;
                    DELETE FROM projects;
                    DELETE FROM tags;
                    DELETE FROM users;
                    DELETE FROM settings;
                    DELETE FROM accounting_records;
                    DELETE FROM accounting_categories;
                    DELETE FROM budgets;
                    DELETE FROM templates;
                    DELETE FROM log_presets;
                    DELETE FROM wireguard_configs;
                    DELETE FROM git_repos;
                    DELETE FROM calculator_history;
                    DELETE FROM api_requests;
                    DELETE FROM nginx_passwords;
                    DELETE FROM nginx_deny_allows;
                    DELETE FROM nginx_params;
                    DELETE FROM nginx_basic_settings;
                    DELETE FROM nginx_templates;
                    DELETE FROM nginx_certs;
                    DELETE FROM nginx_streams;
                    DELETE FROM nginx_http_params;
                    DELETE FROM nginx_upstream_servers;
                    DELETE FROM nginx_upstreams;
                    DELETE FROM nginx_locations;
                    DELETE FROM nginx_servers;
                    DELETE FROM nginx_config_versions;
                    DELETE FROM nginx_presets;
                    DELETE FROM alert_history;
                    DELETE FROM alert_resources;
                    DELETE FROM alert_services;
                    DELETE FROM alert_email_config;
                ") {
                    errors.push(format!("清空表失败: {}", e));
                }
            }

            // merge 模式用 INSERT OR IGNORE（跳过已存在的行），replace 模式用 INSERT OR REPLACE（覆盖）
            // 这统一实现了 UI 承诺的"合并（跳过重复数据）"语义
            let upsert = if mode == "merge" { "INSERT OR IGNORE" } else { "INSERT OR REPLACE" };

            // Settings (key-value pairs in JSON object)
            if let Some(settings) = data.get("settings").and_then(|v| v.as_object()) {
                log::info!("[Backup] Importing {} settings", settings.len());
                for (key, value) in settings {
                    let val_str = if value.is_string() {
                        value.as_str().unwrap_or("").to_string()
                    } else {
                        serde_json::to_string(value).unwrap_or_default()
                    };
                    match conn.execute(
                        &format!("{} INTO settings (key, value) VALUES (?1, ?2)", upsert),
                        rusqlite::params![key, val_str],
                    ) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("settings({}): {}", key, e)),
                    }
                }
                log::info!("[Backup] Settings done: imported={}", imported);
            }

            // Projects
            if let Some(projects) = data.get("projects").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} projects", projects.len());
                for p in projects {
                    let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if mode == "merge" {
                        let exists: Result<Option<String>, _> = conn.query_row(
                            "SELECT id FROM projects WHERE id = ?", [id], |r| r.get(0));
                        if exists.ok().flatten().is_some() { skipped += 1; continue; }
                    }
                    match conn.execute(
                        &format!("{} INTO projects (id, name, description, color, repoPath, branch, repoPath2, branch2, gitUrl1, gitUrl2, gitRepoId, gitRepoId2, category, createdAt, updatedAt, archived)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)", upsert),
                        rusqlite::params![
                            id,
                            p.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("color").and_then(|v| v.as_str()).unwrap_or("#6366f1"),
                            p.get("repoPath").and_then(|v| v.as_str()),
                            p.get("branch").and_then(|v| v.as_str()),
                            p.get("repoPath2").and_then(|v| v.as_str()),
                            p.get("branch2").and_then(|v| v.as_str()),
                            p.get("gitUrl1").and_then(|v| v.as_str()),
                            p.get("gitUrl2").and_then(|v| v.as_str()),
                            p.get("gitRepoId").and_then(|v| v.as_str()),
                            p.get("gitRepoId2").and_then(|v| v.as_str()),
                            p.get("category").and_then(|v| v.as_str()),
                            p.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("archived").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => {
                            log::error!("[Backup] FAILED to insert project '{}': {}", id, e);
                            errors.push(format!("projects({}): {}", id, e));
                        }
                    }
                }
                log::info!("[Backup] Projects done: imported={}, skipped={}", imported, skipped);
            }

            // Servers
            if let Some(servers) = data.get("servers").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} servers", servers.len());
                for s in servers {
                    let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if mode == "merge" {
                        let exists: Result<Option<String>, _> = conn.query_row(
                            "SELECT id FROM servers WHERE id = ?", [id], |r| r.get(0));
                        if exists.ok().flatten().is_some() { skipped += 1; continue; }
                    }
                    let tags_json = s.get("tags").and_then(|v| v.as_str()).unwrap_or("[]");
                    match conn.execute(
                        &format!("{} INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, groupId, requiresApproval, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", upsert),
                        rusqlite::params![
                            id,
                            s.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            s.get("host").and_then(|v| v.as_str()).unwrap_or(""),
                            s.get("port").and_then(|v| v.as_i64()).unwrap_or(22),
                            s.get("username").and_then(|v| v.as_str()).unwrap_or(""),
                            s.get("sshKeyPath").and_then(|v| v.as_str()),
                            s.get("password").and_then(|v| v.as_str()),
                            s.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            tags_json,
                            s.get("groupId").and_then(|v| v.as_str()),
                            s.get("requiresApproval").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                            s.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            s.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => {
                            log::error!("[Backup] FAILED to insert server '{}': {}", id, e);
                            errors.push(format!("servers({}): {}", id, e));
                        }
                    }
                }
                log::info!("[Backup] Servers done: imported={}", imported);
            }

            // Todos
            if let Some(todos) = data.get("todos").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} todos", todos.len());
                for t in todos {
                    let id = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if mode == "merge" {
                        let exists: Result<Option<String>, _> = conn.query_row(
                            "SELECT id FROM todos WHERE id = ?", [id], |r| r.get(0));
                        if exists.ok().flatten().is_some() { skipped += 1; continue; }
                    }
                    match conn.execute(
                        &format!("{} INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, completedAt, assignedTo, assignedBy, assignedAt, owner, orderNum, repeatType, repeatInterval, repeatEndDate, repeatCount, parentTodoId, projectId)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)", upsert),
                        rusqlite::params![
                            id,
                            t.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("completed").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                            t.get("priority").and_then(|v| v.as_str()).unwrap_or("medium"),
                            t.get("dueDate").and_then(|v| v.as_str()),
                            t.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("markdownDescription").and_then(|v| v.as_str()),
                            t.get("tag").and_then(|v| v.as_str()),
                            t.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("completedAt").and_then(|v| v.as_str()),
                            t.get("assignedTo").and_then(|v| v.as_str()),
                            t.get("assignedBy").and_then(|v| v.as_str()),
                            t.get("assignedAt").and_then(|v| v.as_str()),
                            t.get("owner").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("orderNum").and_then(|v| v.as_i64()).unwrap_or(0),
                            t.get("repeatType").and_then(|v| v.as_str()),
                            t.get("repeatInterval").and_then(|v| v.as_i64()).unwrap_or(0),
                            t.get("repeatEndDate").and_then(|v| v.as_str()),
                            t.get("repeatCount").and_then(|v| v.as_i64()).unwrap_or(0),
                            t.get("parentTodoId").and_then(|v| v.as_str()),
                            t.get("projectId").and_then(|v| v.as_str()),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => {
                            log::error!("[Backup] FAILED to insert todo '{}': {}", id, e);
                            errors.push(format!("todos({}): {}", id, e));
                        }
                    }
                }
                log::info!("[Backup] Todos done: imported={}", imported);
            }

            // Subtasks
            if let Some(subtasks) = data.get("subtasks").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} subtasks", subtasks.len());
                for st in subtasks {
                    match conn.execute(
                        &format!("{} INTO subtasks (id, todoId, text, description, completed, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            st.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            st.get("todoId").and_then(|v| v.as_str()).unwrap_or(""),
                            st.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                            st.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            st.get("completed").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                            st.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("subtasks: {}", e)),
                    }
                }
                log::info!("[Backup] Subtasks done: imported={}", imported);
            }

            // Tags (independent table in SQLite)
            if let Some(tags) = data.get("tags").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} tags", tags.len());
                for t in tags {
                    if let Some(name) = t.as_str() {
                        match conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", [name]) {
                            Ok(_) => imported += 1,
                            Err(e) => errors.push(format!("tags: {}", e)),
                        }
                    }
                }
                log::info!("[Backup] Tags done: imported={}", imported);
            }

            // Server groups
            if let Some(groups) = data.get("serverGroups").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} server_groups", groups.len());
                for g in groups {
                    match conn.execute(
                        &format!("{} INTO server_groups (id, name, description, parentId, color, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", upsert),
                        rusqlite::params![
                            g.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            g.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            g.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            g.get("parentId").and_then(|v| v.as_str()),
                            g.get("color").and_then(|v| v.as_str()).unwrap_or("#6c63ff"),
                            g.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            g.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("server_groups: {}", e)),
                    }
                }
                log::info!("[Backup] Server groups done: imported={}", imported);
            }

            // MFA
            if let Some(mfas) = data.get("mfaSecrets").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} mfa_secrets", mfas.len());
                for m in mfas {
                    match conn.execute(
                        &format!("{} INTO mfa_secrets (id, name, secret, issuer, digits, period, algorithm, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", upsert),
                        rusqlite::params![
                            m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("secret").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("issuer").and_then(|v| v.as_str()),
                            m.get("digits").and_then(|v| v.as_i64()).unwrap_or(6),
                            m.get("period").and_then(|v| v.as_i64()).unwrap_or(30),
                            m.get("algorithm").and_then(|v| v.as_str()).unwrap_or("SHA1"),
                            m.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("mfa_secrets: {}", e)),
                    }
                }
                log::info!("[Backup] MFA secrets done: imported={}", imported);
            }

            // Note groups
            if let Some(ng) = data.get("noteGroups").and_then(|v| v.as_array()) {
                for g in ng {
                    match conn.execute(
                        &format!("{} INTO note_groups (id, name) VALUES (?1, ?2)", upsert),
                        rusqlite::params![
                            g.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            g.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("note_groups: {}", e)),
                    }
                }
            }

            // Notes
            if let Some(notes) = data.get("notes").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} notes", notes.len());
                for n in notes {
                    match conn.execute(
                        &format!("{} INTO notes (id, title, content, groupId, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            n.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            n.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                            n.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                            n.get("groupId").and_then(|v| v.as_str()),
                            n.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            n.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("notes: {}", e)),
                    }
                }
                log::info!("[Backup] Notes done: imported={}", imported);
            }

            // Weekly reports — schema: id(INTEGER), weekStart, weekEnd, content, createdAt
            if let Some(reports) = data.get("weeklyReports").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} weekly_reports", reports.len());
                for r in reports {
                    // 优先 content（当前导出格式），回退 data（旧备份兼容）
                    let content_str = r.get("content").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .or_else(|| r.get("data").map(|v| {
                            if v.is_string() { v.as_str().unwrap().to_string() }
                            else { serde_json::to_string(v).unwrap_or_default() }
                        }))
                        .unwrap_or_default();
                    match conn.execute(
                        &format!("{} INTO weekly_reports (id, weekStart, weekEnd, content, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5)", upsert),
                        rusqlite::params![
                            r.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
                            r.get("weekStart").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("weekEnd").and_then(|v| v.as_str()).unwrap_or(""),
                            content_str,
                            r.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("weekly_reports: {}", e)),
                    }
                }
                log::info!("[Backup] Weekly reports done: imported={}", imported);
            }

            // Accounting categories
            if let Some(cats) = data.get("accountingCategories").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} accounting_categories", cats.len());
                for c in cats {
                    match conn.execute(
                        &format!("{} INTO accounting_categories (id, name, type, icon, sortOrder, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            c.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            c.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            c.get("type").and_then(|v| v.as_str()).unwrap_or("expense"),
                            c.get("icon").and_then(|v| v.as_str()).unwrap_or(""),
                            c.get("sortOrder").and_then(|v| v.as_i64()).unwrap_or(0),
                            c.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("accounting_categories: {}", e)),
                    }
                }
            }

            // Accounting records — 导出字段为 snake_case（与 accounting.rs 一致）
            if let Some(records) = data.get("accountingRecords").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} accounting_records", records.len());
                for r in records {
                    // attachments_json 导出时为 JSON 数组对象，导入需序列化回字符串
                    let attachments_json_str = r.get("attachments_json").map(|v| {
                        if v.is_string() { v.as_str().unwrap_or("[]").to_string() }
                        else { serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()) }
                    }).unwrap_or_else(|| "[]".to_string());
                    match conn.execute(
                        &format!("{} INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)", upsert),
                        rusqlite::params![
                            r.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("date").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("type").and_then(|v| v.as_str()).unwrap_or("expense"),
                            r.get("category").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            r.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("status").and_then(|v| v.as_str()).unwrap_or("completed"),
                            r.get("attachmentPath").and_then(|v| v.as_str()),
                            r.get("createdBy").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("voucher_number").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("receipt_type").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("receipt_path").and_then(|v| v.as_str()),
                            r.get("entity").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("project").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("supplier").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("invoice_number").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("tax_amount").and_then(|v| v.as_f64()),
                            r.get("payment_method").and_then(|v| v.as_str()).unwrap_or(""),
                            r.get("approver").and_then(|v| v.as_str()).unwrap_or(""),
                            attachments_json_str,
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("accounting_records: {}", e)),
                    }
                }
            }

            // Budgets
            if let Some(budgets) = data.get("accountingBudgets").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} budgets", budgets.len());
                for b in budgets {
                    match conn.execute(
                        &format!("{} INTO budgets (id, name, \"limit\", period)
                         VALUES (?1, ?2, ?3, ?4)", upsert),
                        rusqlite::params![
                            b.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            b.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            b.get("amount").or(b.get("limit")).and_then(|v| v.as_f64()).unwrap_or(0.0),
                            b.get("period").and_then(|v| v.as_str()).unwrap_or("monthly"),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("budgets: {}", e)),
                    }
                }
            }

            // Templates
            if let Some(templates) = data.get("accountingTemplates").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} templates", templates.len());
                for t in templates {
                    match conn.execute(
                        &format!("{} INTO templates (id, name, content)
                         VALUES (?1, ?2, ?3)", upsert),
                        rusqlite::params![
                            t.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("templates: {}", e)),
                    }
                }
            }

            // Log presets
            if let Some(presets) = data.get("logPresets").and_then(|v| v.as_array()) {
                let now = chrono::Utc::now().to_rfc3339();
                for p in presets {
                    match conn.execute(
                        &format!("{} INTO log_presets (id, name, serverIds, logPath, logType, maxLines, presetGroup, keywords, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", upsert),
                        rusqlite::params![
                            p.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("serverIds").and_then(|v| v.as_str()).unwrap_or("[]"),
                            p.get("logPath").and_then(|v| v.as_str()).unwrap_or(""),
                            p.get("logType").and_then(|v| v.as_str()).unwrap_or("file"),
                            p.get("maxLines").and_then(|v| v.as_i64()).unwrap_or(500),
                            p.get("presetGroup").or_else(|| p.get("group")).and_then(|v| v.as_str()).unwrap_or("未分组"),
                            p.get("keywords").and_then(|v| v.as_str()).unwrap_or("[]"),
                            p.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now),
                            p.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("log_presets: {}", e)),
                    }
                }
            }

            // LAN users
            if let Some(users) = data.get("users").and_then(|v| v.as_array()) {
                for u in users {
                    match conn.execute(
                        &format!("{} INTO users (id, name, ip, port, lastSeen, isOnline)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            u.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            u.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            u.get("ip").and_then(|v| v.as_str()).unwrap_or(""),
                            u.get("port").and_then(|v| v.as_i64()).unwrap_or(0),
                            u.get("lastSeen").and_then(|v| v.as_str()).unwrap_or(""),
                            u.get("isOnline").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("users: {}", e)),
                    }
                }
            }

            // LAN messages
            if let Some(msgs) = data.get("messages").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} messages", msgs.len());
                for m in msgs {
                    match conn.execute(
                        &format!("{} INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", upsert),
                        rusqlite::params![
                            m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("fromUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("fromUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("toUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("toUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("type").and_then(|v| v.as_str()).unwrap_or("text"),
                            m.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("read").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("messages: {}", e)),
                    }
                }
            }

            // Chat messages
            if let Some(msgs) = data.get("chatMessages").and_then(|v| v.as_array()) {
                log::info!("[Backup] Importing {} chat_messages", msgs.len());
                for m in msgs {
                    match conn.execute(
                        &format!("{} INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, fileName, fileSize, filePath, status, progress, createdAt, read)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)", upsert),
                        rusqlite::params![
                            m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("fromUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("fromUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("toUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("toUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("content").and_then(|v| v.as_str()),
                            m.get("type").and_then(|v| v.as_str()).unwrap_or("text"),
                            m.get("fileName").and_then(|v| v.as_str()),
                            m.get("fileSize").and_then(|v| v.as_i64()).unwrap_or(0),
                            m.get("filePath").and_then(|v| v.as_str()),
                            m.get("status").and_then(|v| v.as_str()).unwrap_or("sent"),
                            m.get("progress").and_then(|v| v.as_i64()).unwrap_or(0),
                            m.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            m.get("read").and_then(|v| v.as_bool()).unwrap_or(false) as i64,
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("chat_messages: {}", e)),
                    }
                }
            }

            // File transfers
            if let Some(ftransfers) = data.get("fileTransfers").and_then(|v| v.as_array()) {
                for ft in ftransfers {
                    match conn.execute(
                        &format!("{} INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, completedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)", upsert),
                        rusqlite::params![
                            ft.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("fromUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("fromUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("toUserId").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("toUserName").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("fileName").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("fileSize").and_then(|v| v.as_i64()).unwrap_or(0),
                            ft.get("filePath").and_then(|v| v.as_str()),
                            ft.get("status").and_then(|v| v.as_str()).unwrap_or("pending"),
                            ft.get("progress").and_then(|v| v.as_i64()).unwrap_or(0),
                            ft.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                            ft.get("completedAt").and_then(|v| v.as_str()),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("file_transfers: {}", e)),
                    }
                }
            }

            // ======== Extra modules (previously missing from backup) ========

            let now_ts = chrono::Utc::now().to_rfc3339();

            // WireGuard configs
            if let Some(items) = data.get("wireguardConfigs").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO wireguard_configs (id, name, privateKey, publicKey, address, dns, mtu, peerPublicKey, peerEndpoint, peerAllowedIPs, peerPersistentKeepalive, presharedKey, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("privateKey").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("publicKey").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("address").and_then(|v| v.as_str()).unwrap_or("10.0.0.2/32"),
                            item.get("dns").and_then(|v| v.as_str()),
                            item.get("mtu").and_then(|v| v.as_i64()).unwrap_or(1420),
                            item.get("peerPublicKey").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("peerEndpoint").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("peerAllowedIPs").and_then(|v| v.as_str()).unwrap_or("0.0.0.0/0"),
                            item.get("peerPersistentKeepalive").and_then(|v| v.as_i64()).unwrap_or(25),
                            item.get("presharedKey").and_then(|v| v.as_str()),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("wireguard_configs: {}", e)),
                    }
                }
            }

            // Git repos
            if let Some(items) = data.get("gitRepos").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO git_repos (id, name, path, remote, branch, lastCommit, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("remote").and_then(|v| v.as_str()),
                            item.get("branch").and_then(|v| v.as_str()),
                            item.get("lastCommit").and_then(|v| v.as_str()),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("git_repos: {}", e)),
                    }
                }
            }

            // Calculator history
            if let Some(items) = data.get("calculatorHistory").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO calculator_history (id, expression, result, createdAt)
                         VALUES (?1, ?2, ?3, ?4)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("expression").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("result").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("calculator_history: {}", e)),
                    }
                }
            }

            // API requests
            if let Some(items) = data.get("apiRequests").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO api_requests (id, method, url, headers, body, statusCode, responseTime, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("method").and_then(|v| v.as_str()).unwrap_or("GET"),
                            item.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("headers").and_then(|v| v.as_str()).unwrap_or("{}"),
                            item.get("body").and_then(|v| v.as_str()),
                            item.get("statusCode").and_then(|v| v.as_i64()),
                            item.get("responseTime").and_then(|v| v.as_i64()),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("api_requests: {}", e)),
                    }
                }
            }

            // --- Nginx tables (order by FK dependency) ---

            // nginx_presets 是 nginx_servers/nginx_upstreams 等子表的 FK 父表（ON DELETE CASCADE）
            // merge 模式下用 INSERT OR IGNORE，避免 INSERT OR REPLACE 触发 DELETE→级联删除子表
            // （由上方统一的 upsert 变量提供）

            if let Some(items) = data.get("nginxPresets").and_then(|v| v.as_array()) {
                for item in items {
                    let sql = format!("{} INTO nginx_presets (id, name, serverId, configPath, description, groupName, isActive, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", upsert);
                    match conn.execute(&sql,
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("serverId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("configPath").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("groupName").and_then(|v| v.as_str()).unwrap_or("未分组"),
                            item.get("isActive").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_presets: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxConfigVersions").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_config_versions (id, presetId, content, checksum, comment, isCurrent, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("checksum").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("comment").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("isCurrent").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_config_versions: {}", e)),
                    }
                }
            }

            // nginx_servers 是 nginx_locations 的 FK 父表（ON DELETE CASCADE）
            if let Some(items) = data.get("nginxServers").and_then(|v| v.as_array()) {
                for item in items {
                    let sql = format!("{} INTO nginx_servers (id, presetId, proxyType, listen, ip, def, ipv6, proxyProtocol, serverName, ssl, certId, rewrite, rewriteListen, http2, protocols, passwordId, denyAllow, denyId, allowId, proxyUpstreamId, descr, enabled, sort, paramJson, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)", upsert);
                    match conn.execute(&sql,
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("proxyType").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("listen").and_then(|v| v.as_str()).unwrap_or("80"),
                            item.get("ip").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("def").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("ipv6").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("proxyProtocol").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("serverName").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("ssl").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("certId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("rewrite").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("rewriteListen").and_then(|v| v.as_str()).unwrap_or("80"),
                            item.get("http2").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("protocols").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("passwordId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("denyAllow").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("denyId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("allowId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("proxyUpstreamId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("descr").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("paramJson").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_servers: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxLocations").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_locations (id, serverId, enabled, path, locType, value, upstreamType, upstreamId, upstreamPath, rootPath, rootPage, rootType, header, websocket, cros, headerHost, returnUrl, returnPath, paramJson, sort, descr, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("serverId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("path").and_then(|v| v.as_str()).unwrap_or("/"),
                            item.get("locType").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("upstreamType").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("upstreamId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("upstreamPath").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("rootPath").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("rootPage").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("rootType").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("header").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("websocket").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("cros").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("headerHost").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("returnUrl").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("returnPath").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("paramJson").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("descr").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_locations: {}", e)),
                    }
                }
            }

            // nginx_upstreams 是 nginx_upstream_servers 的 FK 父表（ON DELETE CASCADE）
            if let Some(items) = data.get("nginxUpstreams").and_then(|v| v.as_array()) {
                for item in items {
                    let sql = format!("{} INTO nginx_upstreams (id, presetId, name, proxyType, strategy, descr, paramJson, sort, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", upsert);
                    match conn.execute(&sql,
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("proxyType").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("strategy").and_then(|v| v.as_str()).unwrap_or("polling"),
                            item.get("descr").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("paramJson").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_upstreams: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxUpstreamServers").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_upstream_servers (id, upstreamId, address, port, weight, maxFails, failTimeout, maxConns, backup, down, sort, enabled, param)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("upstreamId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("address").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("port").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("weight").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("maxFails").and_then(|v| v.as_i64()).unwrap_or(3),
                            item.get("failTimeout").and_then(|v| v.as_str()).unwrap_or("10s"),
                            item.get("maxConns").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("backup").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("down").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("param").and_then(|v| v.as_str()).unwrap_or(""),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_upstream_servers: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxHttpParams").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_http_params (id, presetId, name, value, enabled, sort, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_http_params: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxStreams").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_streams (id, presetId, listen, proxyUpstreamId, proxyPass, ssl, certId, protocol, descr, enabled, paramJson, sort, createdAt, updatedAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("listen").and_then(|v| v.as_str()).unwrap_or("0.0.0.0:80"),
                            item.get("proxyUpstreamId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("proxyPass").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("ssl").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("certId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("protocol").and_then(|v| v.as_str()).unwrap_or("TCP"),
                            item.get("descr").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("paramJson").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                            item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_streams: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxCerts").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_certs (id, presetId, name, pem, key, domain, sort, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("pem").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("key").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("domain").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_certs: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxTemplates").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_templates (id, presetId, name, content, sort, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_templates: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxBasicSettings").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_basic_settings (id, presetId, name, value, sort, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_basic_settings: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxParams").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_params (id, presetId, serverId, locationId, upstreamId, name, value, position, templateValue, sort, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("serverId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("locationId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("upstreamId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("position").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("templateValue").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sort").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_params: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxDenyAllows").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_deny_allows (id, presetId, name, ip, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("ip").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_deny_allows: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("nginxPasswords").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO nginx_passwords (id, presetId, name, pass, descr, path, createdAt)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("presetId").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("pass").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("descr").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("path").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("createdAt").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("nginx_passwords: {}", e)),
                    }
                }
            }

            // --- Alert tables ---

            if let Some(items) = data.get("alertEmailConfig").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO alert_email_config (id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, from_email, to_email, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("smtp_host").and_then(|v| v.as_str()),
                            item.get("smtp_port").and_then(|v| v.as_i64()).unwrap_or(465),
                            item.get("smtp_username").and_then(|v| v.as_str()),
                            item.get("smtp_password").and_then(|v| v.as_str()),
                            item.get("smtp_encryption").and_then(|v| v.as_str()).unwrap_or("starttls"),
                            item.get("from_email").and_then(|v| v.as_str()),
                            item.get("to_email").and_then(|v| v.as_str()),
                            item.get("updated_at").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("alert_email_config: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("alertServices").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO alert_services (id, name, host, port, check_interval, timeout_seconds, max_retries, enabled, last_check_at, last_status, consecutive_failures, alert_sent_at, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("host").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("port").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("check_interval").and_then(|v| v.as_i64()).unwrap_or(60),
                            item.get("timeout_seconds").and_then(|v| v.as_i64()).unwrap_or(5),
                            item.get("max_retries").and_then(|v| v.as_i64()).unwrap_or(3),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("last_check_at").and_then(|v| v.as_str()),
                            item.get("last_status").and_then(|v| v.as_i64()),
                            item.get("consecutive_failures").and_then(|v| v.as_i64()).unwrap_or(0),
                            item.get("alert_sent_at").and_then(|v| v.as_str()),
                            item.get("created_at").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("alert_services: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("alertResources").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO alert_resources (id, name, category, remark, expire_at, alert_advance_days, enabled, last_alert_sent_at, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("category").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("remark").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("expire_at").and_then(|v| v.as_str()),
                            item.get("alert_advance_days").and_then(|v| v.as_i64()).unwrap_or(30),
                            item.get("enabled").and_then(|v| v.as_i64()).unwrap_or(1),
                            item.get("last_alert_sent_at").and_then(|v| v.as_str()),
                            item.get("created_at").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("alert_resources: {}", e)),
                    }
                }
            }

            if let Some(items) = data.get("alertHistory").and_then(|v| v.as_array()) {
                for item in items {
                    match conn.execute(
                        &format!("{} INTO alert_history (id, type, ref_id, ref_name, message, sent_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)", upsert),
                        rusqlite::params![
                            item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("type").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("ref_id").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("ref_name").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("message").and_then(|v| v.as_str()).unwrap_or(""),
                            item.get("sent_at").and_then(|v| v.as_str()).unwrap_or(&now_ts),
                        ]) {
                        Ok(_) => imported += 1,
                        Err(e) => errors.push(format!("alert_history: {}", e)),
                    }
                }
            }

            // Store errors in a thread-safe way for the caller
            if !errors.is_empty() {
                log::warn!("[Backup Import] {} errors occurred:", errors.len());
                for e in &errors {
                    log::warn!("  - {}", e);
                }
            }
        }).map_err(|e| format!("db_write failed: {}", e))?;

        log::info!(
            "[Backup] === Summary: imported={}, skipped={}, errors={} ===",
            imported,
            skipped,
            errors.len()
        );
        if !errors.is_empty() {
            log::warn!("[Backup] First 5 errors:");
            for e in errors.iter().take(5) {
                log::warn!("  - {}", e);
            }
        }

        // CICD data
        log::info!("[Backup] Importing CICD data (mode={})...", mode);
        match self.import_cicd_data(&data, mode).await {
            Ok((cicd_imported, cicd_skipped)) => {
                log::info!(
                    "[Backup] CICD done: imported={}, skipped={}",
                    cicd_imported,
                    cicd_skipped
                );
                imported += cicd_imported;
                skipped += cicd_skipped;
            }
            Err(e) => {
                log::error!("[Backup] CICD import failed: {}", e);
                errors.push(format!("cicd: {}", e));
            }
        }

        Ok((imported, skipped, errors))
    }
}
