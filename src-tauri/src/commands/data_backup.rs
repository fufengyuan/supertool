use crate::core::CoreService;
use tauri_plugin_dialog::DialogExt;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use serde_json::json;

#[tauri::command(rename_all = "camelCase")]
pub async fn export_all_data(
    app: tauri::AppHandle,
    core: tauri::State<'_, crate::core::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_all_data() called");
    let data = export_all_tables(core.inner()).await?;
    let data_json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化失败: {}", e))?;

    // Compute stats for user feedback
    let table_count = data.as_object().map(|o| o.len()).unwrap_or(0);
    let total_items: usize = data.as_object().map(|o| {
        o.values().map(|v| v.as_array().map(|a| a.len()).unwrap_or(0)).sum()
    }).unwrap_or(0);

    let default_name = format!(
        "supertool-backup-{}.stbackup",
        chrono::Local::now().format("%Y-%m-%d")
    );

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file()
        .set_title("导出完整备份")
        .set_file_name(&default_name)
        .add_filter("SuperTool Backup", &["stbackup"])
        .save_file(move |file_path: Option<tauri_plugin_dialog::FilePath>| {
            let _ = tx.send(file_path);
        });

    let file_path = rx.recv().map_err(|e| format!("Dialog error: {}", e))?;
    let file_path = file_path.ok_or("用户取消了导出")?;
    let path_str = file_path.as_path().ok_or("无法获取文件路径")?
        .to_string_lossy().to_string();

    let mut zip_buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buf);
        let opts: FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("all-data.json", opts)
            .map_err(|e| format!("ZIP创建失败: {}", e))?;
        zip.write_all(data_json.as_bytes())
            .map_err(|e| format!("写入ZIP失败: {}", e))?;

        let data_dir = crate::core::data_dir::resolve_data_dir();
        let receipt_dir = data_dir.join("accounting-receipts");
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

    std::fs::write(&path_str, zip_buf.into_inner())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(json!({ "success": true, "path": path_str, "tableCount": table_count, "totalItems": total_items }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_data(
    app: tauri::AppHandle,
    core: tauri::State<'_, crate::core::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_data() called");
    export_all_data(app, core).await
}

#[tauri::command]
pub async fn import_json(
    app: tauri::AppHandle,
    core: tauri::State<'_, crate::core::CoreService>,
    import_mode: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] import_json() called");
    let mode = import_mode.unwrap_or_else(|| "merge".to_string());
    log::info!("[Tauri CMD] import_mode = {}", mode);

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file()
        .add_filter("SuperTool Backup", &["stbackup"])
        .pick_file(move |file_path: Option<tauri_plugin_dialog::FilePath>| {
            let _ = tx.send(file_path);
        });

    let file_path = rx.recv().map_err(|e| format!("Dialog error: {}", e))?;
    let file_path = file_path.ok_or("用户取消了导入")?;
    let path_str = file_path.as_path().ok_or("无法获取文件路径")?
        .to_string_lossy().to_string();

    log::info!("[Backup] Importing from: {}", path_str);
    
    let zip_data = std::fs::read(&path_str)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    log::info!("[Backup] ZIP file size: {} bytes", zip_data.len());
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_data))
        .map_err(|e| format!("ZIP解析失败: {}", e))?;
    log::info!("[Backup] ZIP entries: {}", archive.len());
    log::info!("[Backup] Import mode: {}", mode);

    let all_data_json = {
        let mut file = archive.by_name("all-data.json")
            .map_err(|_| "备份文件格式错误：缺少 all-data.json")?;
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut content)
            .map_err(|e| format!("读取all-data.json失败: {}", e))?;
        String::from_utf8(content)
            .map_err(|e| format!("解码失败: {}", e))?
    };

    let data: serde_json::Value = serde_json::from_str(&all_data_json)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    // Extract receipt files
    let data_dir = crate::core::data_dir::resolve_data_dir();
    let receipt_dir = data_dir.join("accounting-receipts");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("ZIP读取失败: {}", e))?;
        let name = file.name().to_string();
        if name.starts_with("receipts/") && !name.ends_with("/") {
            let filename = std::path::Path::new(&name)
                .file_name().ok_or("无效的收据文件路径")?;
            if !receipt_dir.exists() {
                std::fs::create_dir_all(&receipt_dir)
                    .map_err(|e| format!("创建收据目录失败: {}", e))?;
            }
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut content)
                .map_err(|e| format!("读取收据文件失败: {}", e))?;
            std::fs::write(receipt_dir.join(filename), content)
                .map_err(|e| format!("写入收据文件失败: {}", e))?;
        }
    }

    let (imported, skipped, import_errors) = import_all_tables(core.inner(), data, &mode).await?;

    log::info!("[Backup] Import complete: imported={}, skipped={}", imported, skipped);
    if !import_errors.is_empty() {
        log::warn!("[Backup] Import completed with {} errors:", import_errors.len());
        for e in &import_errors {
            log::warn!("  - {}", e);
        }
    }
    Ok(json!({
        "success": import_errors.is_empty(),
        "importedCount": imported,
        "skippedCount": skipped,
        "errors": import_errors,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn import_all_data(
    core: tauri::State<'_, crate::core::CoreService>,
    data: serde_json::Value,
    mode: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] import_all_data() called");
    let (imported, skipped, import_errors) = import_all_tables(core.inner(), data, &mode).await?;
    Ok(json!({
        "success": import_errors.is_empty(),
        "importedCount": imported,
        "skippedCount": skipped,
        "errors": import_errors,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_app_path(
    core: tauri::State<'_, crate::core::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_app_path() called");
    let result = core.get_app_path().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_csv(
    core: tauri::State<'_, crate::core::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_csv() called");
    let result = core.export_all_data().await?;
    let todos = result.get("todos").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    let mut csv = String::from("id,text,completed,priority,createdAt,dueDate\n");
    for todo in todos {
        let id = todo.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let text = todo.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let completed = todo.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
        let priority = todo.get("priority").and_then(|v| v.as_str()).unwrap_or("");
        let created = todo.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        let due = todo.get("dueDate").and_then(|v| v.as_str()).unwrap_or("");
        let text_escaped = text.replace('"', "\"\"");
        csv.push_str(&format!("{},\"{}\",{},{},{},{}\n", id, text_escaped, completed, priority, created, due));
    }
    Ok(json!({ "success": true, "csv": csv }))
}

// ============ Internal helpers ============

async fn export_all_tables(core: &CoreService) -> Result<serde_json::Value, String> {
    let todos = core.get_all_todos().await.unwrap_or(json!([]));
    let projects = core.get_all_projects(true).await.unwrap_or(json!([]));
    let servers = core.get_all_servers().await.unwrap_or(json!([]));
    let server_groups = core.get_all_server_groups().await.unwrap_or(json!([]));
    let mfa_secrets = core.get_all_mfa_secrets().await.unwrap_or(json!([]));
    let notes = core.get_all_notes(None, None).await.unwrap_or(json!([]));
    let note_groups = core.get_all_note_groups().await.unwrap_or(json!([]));
    let weekly_reports = core.get_weekly_reports(9999).await.unwrap_or(json!([]));
    let accounting_categories = core.get_accounting_categories().await.unwrap_or(json!([]));
    let accounting_records_data = core.get_accounting_records(json!({})).await.unwrap_or(json!({"records": []}));
    let accounting_records = accounting_records_data.get("records").cloned().unwrap_or(json!([]));
    let budgets = core.get_budgets().await.unwrap_or(json!([]));
    let templates = core.get_templates().await.unwrap_or(json!([]));
    let log_presets = core.get_log_presets().await.unwrap_or(json!([]));

    let mut all_subtasks = vec![];
    if let Some(todos_arr) = todos.as_array() {
        for todo in todos_arr {
            if let Some(id) = todo.get("id").and_then(|v| v.as_str()) {
                if let Ok(st) = core.get_subtasks_for_todo(id).await {
                    if let Some(arr) = st.as_array() {
                        all_subtasks.extend(arr.clone());
                    }
                }
            }
        }
    }

    let tags = core.get_all_tags().await.unwrap_or(json!([]));
    let cicd_data = core.get_all_cicd_data().await.unwrap_or(json!({}));
    let lan_users = core.get_all_lan_users().await.unwrap_or(json!([]));
    let lan_msgs = core.get_all_lan_messages().await.unwrap_or(json!({}));

    Ok(json!({
        "todos": todos,
        "subtasks": all_subtasks,
        "tags": tags,
        "projects": projects,
        "servers": servers,
        "serverGroups": server_groups,
        "mfaSecrets": mfa_secrets,
        "notes": notes,
        "noteGroups": note_groups,
        "weeklyReports": weekly_reports,
        "accountingCategories": accounting_categories,
        "accountingRecords": accounting_records,
        "accountingBudgets": budgets,
        "accountingTemplates": templates,
        "logPresets": log_presets,
        "cicdConfigs": cicd_data.get("cicdConfigs").cloned().unwrap_or(json!([])),
        "deployModules": cicd_data.get("deployModules").cloned().unwrap_or(json!([])),
        "deployLogs": cicd_data.get("deployLogs").cloned().unwrap_or(json!([])),
        "deployHistory": cicd_data.get("deployHistory").cloned().unwrap_or(json!([])),
        "deployStepLogs": cicd_data.get("deployStepLogs").cloned().unwrap_or(json!([])),
        "users": lan_users,
        "messages": lan_msgs.get("messages").cloned().unwrap_or(json!([])),
        "chatMessages": lan_msgs.get("chatMessages").cloned().unwrap_or(json!([])),
        "fileTransfers": lan_msgs.get("fileTransfers").cloned().unwrap_or(json!([])),
    }))
}

// Helper: get DB connection from CoreService for direct SQL inserts
#[allow(dead_code)]
fn get_conn(_core: &CoreService) -> Result<rusqlite::Connection, String> {
    // Use the CoreService's internal DB — we need direct SQL access
    // Since CoreService holds a Mutex<Database>, we use db_read/db_write for access
    // But for imports we need write access, so we use a workaround:
    // Clone the Arc<Mutex<Database>> and lock it
    // Actually, we'll use the core.db_read/db_write pattern but for batch inserts
    // we need to hold the lock for the entire import operation
    Err("Use direct_sql_import instead".to_string())
}

async fn import_all_tables(
    core: &CoreService,
    data: serde_json::Value,
    mode: &str,
) -> Result<(usize, usize, Vec<String>), String> {
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    // Get direct DB access for batch import
    core.db_write(|conn| {
        if mode == "replace" {
            if let Err(e) = conn.execute_batch("
                DELETE FROM deploy_step_logs;
                DELETE FROM deploy_logs;
                DELETE FROM deploy_modules;
                DELETE FROM cicd_configs;
                DELETE FROM deploy_history;
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
            ") {
                errors.push(format!("清空表失败: {}", e));
            }
        }

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
                    "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
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
                    "INSERT OR REPLACE INTO projects (id, name, description, color, repoPath, branch, repoPath2, branch2, gitUrl1, gitUrl2, category, createdAt, updatedAt, archived)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                    "INSERT OR REPLACE INTO servers (id, name, host, port, username, sshKeyPath, password, description, tags, groupId, requiresApproval, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                    "INSERT OR REPLACE INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, completedAt, assignedTo, assignedBy, assignedAt, owner, orderNum, repeatType, repeatInterval, repeatEndDate, repeatCount, parentTodoId, projectId)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
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
                    "INSERT OR REPLACE INTO subtasks (id, todoId, text, description, completed, createdAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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
                    "INSERT OR REPLACE INTO server_groups (id, name, description, parentId, color, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
                    "INSERT OR REPLACE INTO mfa_secrets (id, name, secret, issuer, digits, period, algorithm, account, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        m.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        m.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        m.get("secret").and_then(|v| v.as_str()).unwrap_or(""),
                        m.get("issuer").and_then(|v| v.as_str()),
                        m.get("digits").and_then(|v| v.as_i64()).unwrap_or(6),
                        m.get("period").and_then(|v| v.as_i64()).unwrap_or(30),
                        m.get("algorithm").and_then(|v| v.as_str()).unwrap_or("SHA1"),
                        m.get("account").and_then(|v| v.as_str()),
                        m.get("createdAt").and_then(|v| v.as_str()).unwrap_or(""),
                        m.get("updatedAt").or(m.get("createdAt")).and_then(|v| v.as_str()).unwrap_or(""),
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
                    "INSERT OR REPLACE INTO note_groups (id, name) VALUES (?1, ?2)",
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
                    "INSERT OR REPLACE INTO notes (id, title, content, groupId, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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

        // Weekly reports
        if let Some(reports) = data.get("weeklyReports").and_then(|v| v.as_array()) {
            log::info!("[Backup] Importing {} weekly_reports", reports.len());
            for r in reports {
                let data_str = r.get("data").map(|v| {
                    if v.is_string() { v.as_str().unwrap().to_string() }
                    else { serde_json::to_string(v).unwrap_or_default() }
                }).unwrap_or_default();
                match conn.execute(
                    "INSERT OR REPLACE INTO weekly_reports (id, startDate, endDate, data, createdAt)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        r.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("startDate").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("endDate").and_then(|v| v.as_str()).unwrap_or(""),
                        data_str,
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
                    "INSERT OR REPLACE INTO accounting_categories (id, name, type, icon, sortOrder, createdAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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

        // Accounting records
        if let Some(records) = data.get("accountingRecords").and_then(|v| v.as_array()) {
            log::info!("[Backup] Importing {} accounting_records", records.len());
            for r in records {
                match conn.execute(
                    "INSERT OR REPLACE INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
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
                        r.get("voucherNumber").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("receiptType").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("receiptPath").and_then(|v| v.as_str()),
                        r.get("entity").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("project").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("supplier").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("invoiceNumber").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("taxAmount").and_then(|v| v.as_f64()),
                        r.get("paymentMethod").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("approver").and_then(|v| v.as_str()).unwrap_or(""),
                        r.get("attachmentsJson").and_then(|v| v.as_str()),
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
                    "INSERT OR REPLACE INTO budgets (id, name, \"limit\", period)
                     VALUES (?1, ?2, ?3, ?4)",
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
                    "INSERT OR REPLACE INTO templates (id, name, content)
                     VALUES (?1, ?2, ?3)",
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
                    "INSERT OR REPLACE INTO log_presets (id, name, serverIds, logPath, logType, maxLines, presetGroup, keywords, createdAt, updatedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                    "INSERT OR REPLACE INTO users (id, name, ip, port, lastSeen, isOnline)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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
                    "INSERT OR REPLACE INTO messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, createdAt, read)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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
                    "INSERT OR REPLACE INTO chat_messages (id, fromUserId, fromUserName, toUserId, toUserName, content, type, fileName, fileSize, filePath, status, progress, createdAt, read)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                    "INSERT OR REPLACE INTO file_transfers (id, fromUserId, fromUserName, toUserId, toUserName, fileName, fileSize, filePath, status, progress, createdAt, completedAt)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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

        // Store errors in a thread-safe way for the caller
        if !errors.is_empty() {
            log::warn!("[Backup Import] {} errors occurred:", errors.len());
            for e in &errors {
                log::warn!("  - {}", e);
            }
        }
    }).map_err(|e| format!("db_write failed: {}", e))?;

    log::info!("[Backup] === Summary: imported={}, skipped={}, errors={} ===", imported, skipped, errors.len());
    if !errors.is_empty() {
        log::warn!("[Backup] First 5 errors:");
        for e in errors.iter().take(5) {
            log::warn!("  - {}", e);
        }
    }

    // CICD data
    log::info!("[Backup] Importing CICD data (mode={})...", mode);
    match core.import_cicd_data(&data, mode).await {
        Ok((cicd_imported, cicd_skipped)) => {
            log::info!("[Backup] CICD done: imported={}, skipped={}", cicd_imported, cicd_skipped);
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
