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

/// Get the current data directory path
#[tauri::command(rename_all = "camelCase")]
pub fn get_data_dir() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_data_dir() called");
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let config_file = home_dir.join(".supertool_dir");

    let (current_path, is_custom) = if config_file.exists() {
        match std::fs::read_to_string(&config_file) {
            Ok(content) => {
                let custom_path = content.trim().to_string();
                if !custom_path.is_empty() {
                    (custom_path, true)
                } else {
                    (home_dir.join(".supertool").to_string_lossy().to_string(), false)
                }
            }
            Err(_) => (home_dir.join(".supertool").to_string_lossy().to_string(), false),
        }
    } else {
        (home_dir.join(".supertool").to_string_lossy().to_string(), false)
    };

    Ok(serde_json::json!({
        "success": true,
        "path": current_path,
        "isCustom": is_custom,
        "defaultPath": home_dir.join(".supertool").to_string_lossy()
    }))
}

/// Set a custom data directory path (stored in ~/.supertool_dir)
/// App restart is required for changes to take effect
#[tauri::command(rename_all = "camelCase")]
pub fn set_data_dir(path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_data_dir() called with path: {}", path);

    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let config_file = home_dir.join(".supertool_dir");

    // Validate path
    let target = std::path::PathBuf::from(&path);
    if path.is_empty() {
        // Reset to default: remove config file
        if config_file.exists() {
            std::fs::remove_file(&config_file)
                .map_err(|e| format!("删除配置文件失败: {}", e))?;
        }
        return Ok(serde_json::json!({
            "success": true,
            "message": "已恢复默认数据目录，重启应用后生效",
            "needRestart": true
        }));
    }

    // Check if path is valid and writable
    if !target.is_absolute() {
        return Err("路径必须是绝对路径".to_string());
    }

    // Create directory if it doesn't exist
    if !target.exists() {
        std::fs::create_dir_all(&target)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // Write config file
    std::fs::write(&config_file, &path)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(serde_json::json!({
        "success": true,
        "message": "数据目录已更新，重启应用后生效",
        "needRestart": true,
        "path": path
    }))
}
