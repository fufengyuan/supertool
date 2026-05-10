use supertool_core::logic::{openvpn::OpenVPNManager, CoreService};
use supertool_core::db::openvpn as db_openvpn;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_get_all(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_get_all() called");
    let configs = core.db_read(|conn| {
        db_openvpn::get_all(conn).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::to_value(configs).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_add(
    core: State<'_, CoreService>,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_add() called");
    let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let file_path = data.get("filePath").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = data.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let id = core.db_write(|conn| {
        db_openvpn::add(conn, &name, &file_path, &content).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "id": id }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_delete(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_delete() called");
    let _ = core.db_write(|conn| {
        db_openvpn::delete(conn, &id).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_connect(
    openvpn: State<'_, OpenVPNManager>,
    config_id: String,
    config_name: String,
    content: String,
    sudo_password: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_connect() called");
    openvpn.connect(config_id, config_name, content, sudo_password)
        .map(|_| serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_retry_with_password(
    openvpn: State<'_, OpenVPNManager>,
    password: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_retry_with_password() called");
    openvpn.retry_with_password(password)
        .map(|success| serde_json::json!({ "success": success }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_disconnect(
    openvpn: State<'_, OpenVPNManager>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_disconnect() called");
    openvpn.disconnect()
        .map(|_| serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_get_status(
    openvpn: State<'_, OpenVPNManager>,
) -> Result<serde_json::Value, String> {
    let status = openvpn.get_status();
    Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_get_logs(
    openvpn: State<'_, OpenVPNManager>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_get_logs() called");
    let status = openvpn.get_status();
    Ok(serde_json::json!(status.log.join("\n")))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_check_available() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_check_available() called");
    let mgr = supertool_core::logic::openvpn::OpenVPNManager::new();
    match mgr.check_available() {
        Ok(version) => Ok(serde_json::json!({ "available": true, "version": version })),
        Err(e) => Ok(serde_json::json!({ "available": false, "error": e })),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_validate_config(
    openvpn: State<'_, OpenVPNManager>,
    content: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_validate_config() called");
    match openvpn.validate_config(&content) {
        Ok(()) => Ok(serde_json::json!({ "valid": true })),
        Err(e) => Ok(serde_json::json!({ "valid": false, "error": e })),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn openvpn_get_traffic_stats(
    openvpn: State<'_, OpenVPNManager>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] openvpn_get_traffic_stats() called");
    match openvpn.get_traffic_stats() {
        Some(stats) => Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?),
        None => Ok(serde_json::json!({})),
    }
}
