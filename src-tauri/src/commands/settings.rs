use crate::core::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_setting(
    core: State<'_, CoreService>,
    key: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_setting() called");
    let result = core.get_setting(&key).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_setting(
    core: State<'_, CoreService>,
    key: String,
    value: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_setting() called");
    let result = core.set_setting(&key, &value).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_notification_settings(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_notification_settings() called");
    let result = core.get_notification_settings().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_notification_settings(
    core: State<'_, CoreService>,
    settings: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_notification_settings() called");
    let result = core.set_notification_settings(settings).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_app_version() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_app_version() called");
    let version = env!("CARGO_PKG_VERSION").to_string();
    Ok(serde_json::json!(version))
}


// =================== Additional Commands ===================

#[tauri::command(rename_all = "camelCase")]
pub fn check_network_permission(host: String, port: i64) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] check_network_permission() called");
    use std::net::UdpSocket;
    
    // Test UDP broadcast capability — this is the real macOS Local Network Privacy check
    // TCP connect to 0.0.0.0:0 would always fail; UDP bind+send is the correct test
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            let _ = socket.set_broadcast(true);
            // Try sending a zero-byte packet to broadcast address
            let test_addr = format!("255.255.255.255:{}", if port > 0 { port } else { 49152 });
            match socket.send_to(&[], &test_addr) {
                Ok(_) => Ok(serde_json::json!({ "success": true, "data": true })),
                Err(e) => {
                    log::warn!("[check_network_permission] UDP broadcast failed: {}", e);
                    Ok(serde_json::json!({ "success": true, "data": false }))
                }
            }
        }
        Err(e) => {
            log::warn!("[check_network_permission] UDP bind failed: {}", e);
            Ok(serde_json::json!({ "success": true, "data": false }))
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_menu_icon(_icon_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_menu_icon() called");
    // Tauri doesn't have the same menu icon system as Electron
    Ok(serde_json::json!({ "success": true, "data": null }))
}

/// Send a test desktop notification
#[tauri::command(rename_all = "camelCase")]
pub fn notification_test() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] notification_test() called");
    use notify_rust::Notification;
    Notification::new()
        .summary("SuperTool 测试通知")
        .body("如果你看到这条消息，说明通知功能正常工作！")
        .show()
        .map_err(|e| format!("发送通知失败: {}", e))?;
    Ok(serde_json::json!({ "success": true, "data": "通知已发送" }))
}
