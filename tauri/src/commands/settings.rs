use supertool_core::logic::CoreService;
use tauri::State;

// =================== 加密密钥管理 ===================

/// 查看当前加密密钥（base64）。未设置自定义密钥时 isCustom=false（用内置默认密钥）
#[tauri::command(rename_all = "camelCase")]
pub async fn get_encryption_key() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_encryption_key() called");
    let key = supertool_core::encryption::get_custom_key().await;
    Ok(serde_json::json!({
        "success": true,
        "key": key,
        "isCustom": key.is_some(),
    }))
}

/// 修改加密密钥：① 旧密钥解密全部存量密文（prepare）
/// ② 写入新密钥并切换 ③ 新密钥重加密写回（commit）。
/// 任一步失败即中止；prepare 已发现解密失败条目时中止并返回明细（不破坏数据）。
#[tauri::command(rename_all = "camelCase")]
pub async fn rotate_encryption_key(
    core: State<'_, CoreService>,
    new_key: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] rotate_encryption_key() called");
    // ① 旧密钥解密全部密文
    let (total, failed) = core.rotate_encryption_key_prepare().await?;
    if !failed.is_empty() {
        core.clear_pending_rotation().await;
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("{} 条密文解密失败（可能来自其他机器的备份或密钥已变），未做任何修改", failed.len()),
            "failed": failed,
        }));
    }

    // ② 生成新密钥（先不切换 active key）
    let key = match new_key {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => {
            use base64::engine::general_purpose::STANDARD as B64;
            use base64::Engine as _;
            use rand::RngCore;
            let mut b = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut b);
            B64.encode(b)
        }
    };
    // 解码为新密钥字节（供 commit_rotation 用显式密钥加密）
    let new_key_bytes: [u8; 32] = {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&key)
            .map_err(|e| format!("新密钥解码失败: {}", e))?;
        if bytes.len() != 32 {
            core.clear_pending_rotation().await;
            return Err("新密钥长度错误：需要 32 字节".to_string());
        }
        let mut b = [0u8; 32];
        b.copy_from_slice(&bytes);
        b
    };

    // ③ 先用**新密钥**重加密写回（单事务，active key 仍是旧密钥）。
    // 这一步失败 → active key 未变、旧密文仍可解，重试安全。
    let n = core.commit_rotation(&new_key_bytes).await?;

    // ④ 全部重加密成功后再切换 active key（落盘 + 缓存）。
    // 至此密文已是新密钥加密，切换后读写一致。
    if let Err(e) = supertool_core::encryption::set_custom_key(&key).await {
        // 极低概率：磁盘写入失败。密文已按新密钥加密，但 active key 仍是旧的，
        // 需提示用户手动重试或重新录入，避免新密文用旧 key 读。
        return Err(format!("重加密已完成但密钥文件写入失败（请勿重启，立即重试或备份密钥）: {}", e));
    }

    Ok(serde_json::json!({
        "success": true,
        "key": key,
        "reencrypted": n,
        "total": total,
    }))
}

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
pub async fn get_db_connections(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_db_connections() called");
    let result = core.get_db_connections().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_db_connections(
    core: State<'_, CoreService>,
    connections: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] set_db_connections() called");
    let result = core.set_db_connections(connections).await?;
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
pub fn check_network_permission(_host: String, port: i64) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] check_network_permission() called");
    use std::net::UdpSocket;

    // Test UDP broadcast capability — this is the real macOS Local Network Privacy check
    // TCP connect to 0.0.0.0:0 would always fail; UDP bind+send is the correct test
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if let Err(e) = socket.set_broadcast(true) {
                log::warn!("[check_network_permission] set_broadcast failed: {}", e);
            }
            // Try sending a 1-byte probe to broadcast address (empty slice may not trigger I/O on some stacks)
            let test_addr = format!("255.255.255.255:{}", if port > 0 { port } else { 49152 });
            match socket.send_to(&[0u8], &test_addr) {
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

/// Send a test desktop notification (with sound)
#[tauri::command(rename_all = "camelCase")]
pub fn notification_test() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] notification_test() called");
    let manager = crate::tray_notification::NotificationManager::new();
    let result = manager.test_notification();
    Ok(serde_json::json!({ "success": result.success, "data": result.message }))
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
                    (
                        home_dir.join(".supertool").to_string_lossy().to_string(),
                        false,
                    )
                }
            }
            Err(_) => (
                home_dir.join(".supertool").to_string_lossy().to_string(),
                false,
            ),
        }
    } else {
        (
            home_dir.join(".supertool").to_string_lossy().to_string(),
            false,
        )
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
            std::fs::remove_file(&config_file).map_err(|e| format!("删除配置文件失败: {}", e))?;
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
        std::fs::create_dir_all(&target).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // Write config file
    std::fs::write(&config_file, &path).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(serde_json::json!({
        "success": true,
        "message": "数据目录已更新，重启应用后生效",
        "needRestart": true,
        "path": path
    }))
}
