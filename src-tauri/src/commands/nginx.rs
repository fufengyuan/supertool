use crate::core::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_nginx_presets(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_nginx_presets() called");
    let result = core.get_all_nginx_presets().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_nginx_preset(
    core: State<'_, CoreService>,
    preset: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_nginx_preset() called");
    let p: crate::db::nginx::NginxPreset =
        serde_json::from_value(preset).map_err(|e| e.to_string())?;
    let result = core.add_nginx_preset(p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_nginx_preset(
    core: State<'_, CoreService>,
    preset: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_nginx_preset() called");
    let p: crate::db::nginx::NginxPreset =
        serde_json::from_value(preset).map_err(|e| e.to_string())?;
    let result = core.update_nginx_preset(p).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_nginx_preset(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_nginx_preset() called");
    let result = core.delete_nginx_preset(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn fetch_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] fetch_nginx_config() called");
    let result = core.fetch_nginx_config(&server_id, &config_path).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] test_nginx_config() called");
    let result = core.test_nginx_config(&server_id, &config_path).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn deploy_nginx_config(
    core: State<'_, CoreService>,
    server_id: String,
    config_path: String,
    content: String,
    _comment: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] deploy_nginx_config() called");
    let result = core
        .deploy_nginx_config(&server_id, &config_path, &content)
        .await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_nginx_config_versions(
    core: State<'_, CoreService>,
    preset_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_nginx_config_versions() called");
    let result = core.get_nginx_config_versions(&preset_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_nginx_config_version(
    core: State<'_, CoreService>,
    version: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_nginx_config_version() called");
    let v: crate::db::nginx::NginxConfigVersion =
        serde_json::from_value(version).map_err(|e| e.to_string())?;
    let result = core.add_nginx_config_version(v).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_active_nginx_version(
    core: State<'_, CoreService>,
    preset_id: String,
    version_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_active_nginx_version() called");
    let result = core.set_current_nginx_version(&preset_id, &version_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}
