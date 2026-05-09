use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_weekly_reports(
    core: State<'_, CoreService>,
    limit: usize,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_weekly_reports() called");
    let result = core.get_weekly_reports(limit).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_weekly_report(
    core: State<'_, CoreService>,
    id: i64,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_weekly_report() called");
    let result = core.get_weekly_report(id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_weekly_report(
    core: State<'_, CoreService>,
    report: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_weekly_report() called");
    let result = core.save_weekly_report(report).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}
