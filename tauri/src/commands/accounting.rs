use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_accounting_records(
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_accounting_records() called");
    let result = core.get_accounting_records(params).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_accounting_record(
    core: State<'_, CoreService>,
    record: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_accounting_record() called");
    let result = core.add_accounting_record(record).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_accounting_record(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_accounting_record() called");
    let result = core.update_accounting_record(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_accounting_record(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_accounting_record() called");
    let result = core.delete_accounting_record(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_accounting_categories(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_accounting_categories() called");
    let result = core.get_accounting_categories().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_accounting_category(
    core: State<'_, CoreService>,
    category: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_accounting_category() called");
    let result = core.add_accounting_category(category).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_accounting_category(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_accounting_category() called");
    let result = core.update_accounting_category(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_accounting_category(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_accounting_category() called");
    let result = core.delete_accounting_category(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_accounting_stats(
    core: State<'_, CoreService>,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_accounting_stats() called");
    let result = core
        .get_accounting_stats(params.unwrap_or(serde_json::Value::Null))
        .await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_accounting_trend(
    core: State<'_, CoreService>,
    months: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_accounting_trend() called");
    let result = core.get_accounting_trend(months.unwrap_or(12)).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn check_budget_alerts(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] check_budget_alerts() called");
    let result = core.check_budget_alerts().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_accounting_csv(
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] export_accounting_csv() called");
    let result = core.export_accounting_csv(params).await?;
    log::info!(
        "[Tauri CMD] export_accounting_csv() result type: is_string={}, is_object={}",
        result.is_string(),
        result.is_object()
    );
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_budgets(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_budgets() called");
    let result = core.get_budgets().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_templates(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_templates() called");
    let result = core.get_templates().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_template(
    core: State<'_, CoreService>,
    template: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_template() called");
    let result = core.add_template(template).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_template(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_template() called");
    let result = core.update_template(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_template(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_template() called");
    let result = core.delete_template(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn use_template(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] use_template() called");
    let result = core.use_template(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_budget(
    core: State<'_, CoreService>,
    budget: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_budget() called");
    let result = core.add_budget(budget).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_budget(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_budget() called");
    let result = core.update_budget(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_budget(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_budget() called");
    let result = core.delete_budget(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn upload_accounting_receipt(
    core: State<'_, CoreService>,
    file_name: String,
    base64_data: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] upload_accounting_receipt() called: {}",
        file_name
    );
    let app_path = core.get_app_path().await.map_err(|e| e)?;
    let app_dir = std::path::PathBuf::from(app_path.as_str().unwrap_or("."));
    let receipt_dir = app_dir.join("accounting-receipts");
    std::fs::create_dir_all(&receipt_dir).map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_string();
    let ext = file_name.rsplit('.').next().unwrap_or("png");
    let safe_name = format!(
        "{}_{}.{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        id,
        ext
    );
    let file_path = receipt_dir.join(&safe_name);

    let clean_data = if let Some(pos) = base64_data.find(',') {
        base64_data[pos + 1..].to_string()
    } else {
        base64_data.clone()
    };
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &clean_data)
        .map_err(|e| e.to_string())?;

    std::fs::write(&file_path, &bytes).map_err(|e| e.to_string())?;

    let is_pdf = ext.eq_ignore_ascii_case("pdf");
    Ok(serde_json::json!({
        "path": file_path.to_string_lossy(),
        "name": file_name,
        "type": if is_pdf { "pdf" } else { "image" },
        "size": bytes.len() as i64,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_accounting_receipt_file(
    _core: State<'_, CoreService>,
    file_path: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] get_accounting_receipt_file() called: {}",
        file_path
    );

    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Ok(serde_json::json!({
            "success": false,
            "error": "文件不存在"
        }));
    }

    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime = match ext.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    let data_url = format!("data:{};base64,{}", mime, b64);

    Ok(serde_json::json!({
        "success": true,
        "dataUrl": data_url,
    }))
}
