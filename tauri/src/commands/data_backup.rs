use serde_json::json;
use std::io::{Cursor, Write};
use tauri_plugin_dialog::DialogExt;
use zip::write::FileOptions;

#[tauri::command(rename_all = "camelCase")]
pub async fn export_all_data(
    app: tauri::AppHandle,
    core: tauri::State<'_, supertool_core::logic::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_all_data() called");
    let data = core.export_all_tables().await?;
    let data_json =
        serde_json::to_string_pretty(&data).map_err(|e| format!("序列化失败: {}", e))?;

    // Compute stats for user feedback
    let table_count = data.as_object().map(|o| o.len()).unwrap_or(0);
    let total_items: usize = data
        .as_object()
        .map(|o| {
            o.values()
                .filter_map(|v| v.as_array().map(|a| a.len()))
                .sum()
        })
        .unwrap_or(0);

    // 提取导出错误作为警告（不计入 table_count）
    let warnings: Vec<String> = data
        .get("_exportErrors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let default_name = format!(
        "supertool-backup-{}.stbackup",
        chrono::Local::now().format("%Y-%m-%d")
    );

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_title("导出完整备份")
        .set_file_name(&default_name)
        .add_filter("SuperTool Backup", &["stbackup"])
        .save_file(move |file_path: Option<tauri_plugin_dialog::FilePath>| {
            let _ = tx.send(file_path);
        });

    let file_path = rx.recv().map_err(|e| format!("Dialog error: {}", e))?;
    let file_path = file_path.ok_or("用户取消了导出")?;
    let path_str = file_path
        .as_path()
        .ok_or("无法获取文件路径")?
        .to_string_lossy()
        .to_string();

    let mut zip_buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buf);
        let opts: FileOptions<()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("all-data.json", opts)
            .map_err(|e| format!("ZIP创建失败: {}", e))?;
        zip.write_all(data_json.as_bytes())
            .map_err(|e| format!("写入ZIP失败: {}", e))?;

        let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
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

    std::fs::write(&path_str, zip_buf.into_inner()).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(
        json!({ "success": true, "path": path_str, "tableCount": table_count, "totalItems": total_items, "warnings": warnings }),
    )
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_data(
    app: tauri::AppHandle,
    core: tauri::State<'_, supertool_core::logic::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_data() called");
    export_all_data(app, core).await
}

#[tauri::command]
pub async fn import_json(
    app: tauri::AppHandle,
    core: tauri::State<'_, supertool_core::logic::CoreService>,
    import_mode: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] import_json() called");
    let mode = import_mode.unwrap_or_else(|| "merge".to_string());
    log::info!("[Tauri CMD] import_mode = {}", mode);

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("SuperTool Backup", &["stbackup"])
        .pick_file(move |file_path: Option<tauri_plugin_dialog::FilePath>| {
            let _ = tx.send(file_path);
        });

    let file_path = rx.recv().map_err(|e| format!("Dialog error: {}", e))?;
    let file_path = file_path.ok_or("用户取消了导入")?;
    let path_str = file_path
        .as_path()
        .ok_or("无法获取文件路径")?
        .to_string_lossy()
        .to_string();

    log::info!("[Backup] Importing from: {}", path_str);

    let zip_data = std::fs::read(&path_str).map_err(|e| format!("读取文件失败: {}", e))?;
    log::info!("[Backup] ZIP file size: {} bytes", zip_data.len());
    let mut archive =
        zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| format!("ZIP解析失败: {}", e))?;
    log::info!("[Backup] ZIP entries: {}", archive.len());
    log::info!("[Backup] Import mode: {}", mode);

    let all_data_json = {
        let mut file = archive
            .by_name("all-data.json")
            .map_err(|_| "备份文件格式错误：缺少 all-data.json")?;
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut content)
            .map_err(|e| format!("读取all-data.json失败: {}", e))?;
        String::from_utf8(content).map_err(|e| format!("解码失败: {}", e))?
    };

    let data: serde_json::Value =
        serde_json::from_str(&all_data_json).map_err(|e| format!("JSON解析失败: {}", e))?;

    // Extract receipt files
    let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
    let receipt_dir = data_dir.join("accounting-receipts");
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP读取失败: {}", e))?;
        let name = file.name().to_string();
        if name.starts_with("receipts/") && !name.ends_with("/") {
            let filename = std::path::Path::new(&name)
                .file_name()
                .ok_or("无效的收据文件路径")?;
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

    let (imported, skipped, import_errors, path_rewritten) = core.import_all_tables(data, &mode).await?;

    log::info!(
        "[Backup] Import complete: imported={}, skipped={}, pathRewritten={}",
        imported,
        skipped,
        path_rewritten
    );
    if !import_errors.is_empty() {
        log::warn!(
            "[Backup] Import completed with {} errors:",
            import_errors.len()
        );
        for e in &import_errors {
            log::warn!("  - {}", e);
        }
    }
    Ok(json!({
        "success": import_errors.is_empty(),
        "importedCount": imported,
        "skippedCount": skipped,
        "errors": import_errors,
        "pathRewritten": path_rewritten,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn import_all_data(
    core: tauri::State<'_, supertool_core::logic::CoreService>,
    data: serde_json::Value,
    mode: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] import_all_data() called");
    let (imported, skipped, import_errors, path_rewritten) = core.import_all_tables(data, &mode).await?;
    Ok(json!({
        "success": import_errors.is_empty(),
        "importedCount": imported,
        "skippedCount": skipped,
        "errors": import_errors,
        "pathRewritten": path_rewritten,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_app_path(
    core: tauri::State<'_, supertool_core::logic::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_app_path() called");
    let result = core.get_app_path().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_csv(
    core: tauri::State<'_, supertool_core::logic::CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_csv() called");
    let result = core.export_all_data().await?;
    let todos = result
        .get("todos")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();
    let mut csv = String::from("id,text,completed,priority,createdAt,dueDate\n");
    for todo in todos {
        let id = todo.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let text = todo.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let completed = todo
            .get("completed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let priority = todo.get("priority").and_then(|v| v.as_str()).unwrap_or("");
        let created = todo.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
        let due = todo.get("dueDate").and_then(|v| v.as_str()).unwrap_or("");
        let text_escaped = text.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},\"{}\",{},{},{},{}\n",
            id, text_escaped, completed, priority, created, due
        ));
    }
    Ok(json!({ "success": true, "csv": csv }))
}
