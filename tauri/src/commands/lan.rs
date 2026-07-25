/// LAN 协作 Tauri Commands — 全部绑定到实际 LanService 方法和 DB 查询
use crate::lan::LanService;
use std::process::Command;
use std::sync::{Arc, Mutex};
use supertool_core::db::lan;
use tauri::AppHandle;

use std::sync::OnceLock;
static LAN_SERVICE: OnceLock<Arc<Mutex<Option<LanService>>>> = OnceLock::new();
static LAN_DB_PATH: OnceLock<String> = OnceLock::new();

#[allow(dead_code)]
pub fn init_lan_service(
    _db_conn: Arc<Mutex<rusqlite::Connection>>,
) -> Arc<Mutex<Option<LanService>>> {
    LAN_SERVICE
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .clone()
}

pub fn init_lan_service_with_db(db_path: &str) {
    let _ = LAN_DB_PATH.set(db_path.to_string());
    let _ = LAN_SERVICE.get_or_init(|| Arc::new(Mutex::new(None)));
}

/// Auto-start LAN service on app launch — reads/creates user identity from DB.
pub fn auto_start_lan(app: &tauri::AppHandle) {
    let db_path = match LAN_DB_PATH.get() {
        Some(p) => p.clone(),
        None => {
            log::warn!("[LAN] auto_start_lan: DB path not set");
            return;
        }
    };

    // Check if already running
    if let Some(svc) = get_lan_service() {
        if let Ok(guard) = svc.lock() {
            if guard.is_some() {
                log::info!("[LAN] Already running, skip auto-start");
                return;
            }
        }
    }

    // Open DB to read/create user identity
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("[LAN] auto_start_lan: DB open failed: {}", e);
            return;
        }
    };

    // Read or create user identity
    let user_id = match lan::get_lan_setting(&conn, "my_user_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            // Generate new user ID
            let id = format!("user_{}", uuid::Uuid::new_v4().simple());
            let _ = lan::save_lan_setting(&conn, "my_user_id", &id);
            id
        }
        Err(e) => {
            log::error!("[LAN] auto_start_lan: read user_id failed: {}", e);
            return;
        }
    };

    let user_name = match lan::get_lan_setting(&conn, "my_user_name") {
        Ok(Some(name)) => name,
        Ok(None) => {
            // Use hostname as default name
            let name = hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "User".to_string());
            let _ = lan::save_lan_setting(&conn, "my_user_name", &name);
            name
        }
        Err(e) => {
            log::error!("[LAN] auto_start_lan: read user_name failed: {}", e);
            return;
        }
    };

    log::info!(
        "[LAN] auto_start_lan: user_id={}, user_name={}",
        user_id,
        user_name
    );

    // Create and start LAN service
    let db_conn = Arc::new(Mutex::new(conn));
    let lan = LanService::new(user_id, user_name, db_conn);
    lan.set_app_handle(app.clone());

    match lan.start() {
        Ok(()) => {
            if let Some(svc) = get_lan_service() {
                if let Ok(mut guard) = svc.lock() {
                    *guard = Some(lan);
                }
            }
            log::info!("[LAN] auto_start_lan: success");
        }
        Err(e) => {
            log::error!("[LAN] auto_start_lan: start failed: {}", e);
        }
    }
}

pub fn get_lan_service() -> Option<Arc<Mutex<Option<LanService>>>> {
    LAN_SERVICE.get().cloned()
}

/// Helper: lock the LAN service and run a closure on it.
/// Returns None if LAN service is not initialized (not running).
fn with_lan<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&LanService) -> T,
{
    if let Some(svc) = get_lan_service() {
        let guard = svc.lock().ok()?;
        if let Some(lan) = guard.as_ref() {
            return Some(f(lan));
        }
    }
    None
}

/// Require LAN to be running, otherwise return an error.
fn require_lan() -> Result<Arc<Mutex<Option<LanService>>>, String> {
    get_lan_service().ok_or("LAN 服务未启动".to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_user_info() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_user_info() called");
    let defaults = serde_json::json!({
        "id": "",
        "name": "",
        "userName": "",
        "avatar": "😀",
    });
    Ok(with_lan(|lan| lan.get_user_info()).unwrap_or(defaults))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_start(
    app: AppHandle,
    user_id: String,
    user_name: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_start() called, user={}", user_name);
    let db_path = LAN_DB_PATH
        .get()
        .ok_or("DB path not set. Call init_lan_service_with_db first.")?;
    let svc = require_lan()?;
    let mut guard = svc.lock().map_err(|e| format!("Lock error: {}", e))?;
    if guard.is_some() {
        return Ok(serde_json::json!({ "success": true, "message": "LAN 已在运行" }));
    }
    let conn = rusqlite::Connection::open(db_path).map_err(|e| format!("打开数据库失败: {}", e))?;
    let db_conn = Arc::new(Mutex::new(conn));
    let lan = LanService::new(user_id, user_name, db_conn);
    lan.set_app_handle(app.clone());
    if let Err(e) = lan.start() {
        log::error!("[Tauri CMD] LAN start failed: {}", e);
        return Err(format!("LAN 启动失败: {}", e));
    }
    *guard = Some(lan);
    log::info!("[Tauri CMD] LAN service started successfully");
    Ok(serde_json::json!({ "success": true, "message": "LAN 服务已启动" }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_stop() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_stop() called");
    if let Some(svc) = get_lan_service() {
        let mut guard = svc.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(lan) = guard.take() {
            lan.stop();
            return Ok(serde_json::json!({ "success": true, "message": "LAN 服务已停止" }));
        }
    }
    Ok(serde_json::json!({ "success": false, "error": "LAN 服务未启动" }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_status(status: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_status() called");
    require_lan()?;
    with_lan(|lan| lan.set_status(status));
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_assign_task(peer_id: String, task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_assign_task() called, peer={}", peer_id);
    let task_val: serde_json::Value =
        serde_json::from_str(&task).unwrap_or_else(|_| serde_json::json!({ "task": task }));
    require_lan()?;
    let result = with_lan(|lan| match lan.assign_task(&peer_id, &task_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_sync_task_status(task_json: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_sync_task_status() called");
    let task_val: serde_json::Value = serde_json::from_str(&task_json)
        .unwrap_or_else(|_| serde_json::json!({ "task": task_json }));
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_task_status_change(&task_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_refresh_discovery() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_refresh_discovery() called");
    require_lan()?;
    with_lan(|lan| lan.refresh_discovery());
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_message(message: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_message() called");
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_message(&message) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_update(task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_update() called");
    let task_val: serde_json::Value =
        serde_json::from_str(&task).unwrap_or_else(|_| serde_json::json!({ "task": task }));
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_task_update(&task_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_status_change(task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_status_change() called");
    let task_val: serde_json::Value =
        serde_json::from_str(&task).unwrap_or_else(|_| serde_json::json!({ "task": task }));
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_task_status_change(&task_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_comment(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_comment() called");
    let data_val: serde_json::Value =
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({ "data": data }));
    if let Some(peer_id) = data_val.get("peerId").and_then(|v| v.as_str()) {
        require_lan()?;
        let result = with_lan(|lan| match lan.broadcast_task_comment(peer_id, &data_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        });
        Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
    } else {
        Err("缺少 peerId".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_collaboration_started(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_collaboration_started() called");
    let data_val: serde_json::Value =
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({ "data": data }));
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_collaboration_started(&data_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_collaboration_ended(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_collaboration_ended() called");
    let data_val: serde_json::Value =
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({ "data": data }));
    require_lan()?;
    let result = with_lan(|lan| match lan.broadcast_collaboration_ended(&data_val) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_all_unread_counts(user_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_all_unread_counts() called");
    require_lan()?;
    let result = with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_unread_counts(&conn, &user_id) {
                Ok(counts) => {
                    let data: serde_json::Map<String, serde_json::Value> = counts
                        .into_iter()
                        .map(|(peer_id, peer_name, count)| {
                            (
                                peer_id.clone(),
                                serde_json::json!({
                                    "name": peer_name,
                                    "count": count,
                                }),
                            )
                        })
                        .collect();
                    serde_json::json!({ "success": true, "data": data })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_status() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_status() called");
    let result = with_lan(|lan| {
        serde_json::json!({
            "running": lan.is_running(),
            "status": lan.get_status(),
        })
    });
    Ok(result.unwrap_or(serde_json::json!({
        "running": false,
        "status": "stopped",
    })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_network_info() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_network_info() called");
    let result = with_lan(|lan| {
        serde_json::json!({
            "address": lan.get_local_ip(),
            "ports": "49152/49154",
            "version": env!("CARGO_PKG_VERSION"),
        })
    });
    Ok(result.unwrap_or(serde_json::json!({
        "address": "",
        "ports": "49152/49154",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_receive_path() -> Result<String, String> {
    log::info!("[Tauri CMD] lan_get_receive_path() called");
    with_lan(|lan| lan.get_receive_path()).ok_or("LAN 服务未启动".to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_peers() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_peers() called");
    let peers = with_lan(|lan| lan.get_all_peers()).unwrap_or_default();
    Ok(serde_json::to_value(peers).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_nick_name(name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_nick_name() called");
    require_lan()?;
    with_lan(|lan| lan.set_nickname(name));
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_avatar(avatar: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_avatar() called");
    require_lan()?;
    with_lan(|lan| lan.set_avatar(avatar));
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_upload_avatar(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_upload_avatar() called, path={}", file_path);
    require_lan()?;

    use std::fs;
    use std::path::Path;

    // 获取 supertool 数据目录
    let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
    let avatars_dir = data_dir.join("avatars");

    // 创建 avatars 目录
    if !avatars_dir.exists() {
        fs::create_dir_all(&avatars_dir).map_err(|e| e.to_string())?;
    }

    // 读取源文件
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err("文件不存在".to_string());
    }

    // 生成目标文件名（使用时间戳 + 原始扩展名）
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let dest_filename = format!("avatar_{}.{}", timestamp, ext);
    let dest_path = avatars_dir.join(&dest_filename);

    // 复制文件
    fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;

    // 返回相对路径（用于存储到数据库）
    let avatar_path = format!("avatar:{}", dest_filename);

    // 更新 avatar
    with_lan(|lan| lan.set_avatar(avatar_path.clone()));

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "path": avatar_path,
            "fullPath": dest_path.to_string_lossy().to_string()
        }
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_avatar_path(avatar: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_avatar_path() called");

    // 如果是 emoji（短字符串），直接返回
    if avatar.len() <= 4 || !avatar.starts_with("avatar:") {
        return Ok(
            serde_json::json!({ "success": true, "data": { "isEmoji": true, "path": avatar } }),
        );
    }

    // 解析 avatar:filename 格式
    let filename = avatar.strip_prefix("avatar:").unwrap_or(&avatar);
    let data_dir = supertool_core::logic::data_dir::resolve_data_dir();
    let full_path = data_dir.join("avatars").join(filename);

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "isEmoji": false,
            "path": full_path.to_string_lossy().to_string()
        }
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_receive_path(path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_receive_path() called");
    require_lan()?;
    with_lan(|lan| lan.set_receive_path(path));
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_show_open_dialog_for_dirs(_app: AppHandle) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_show_open_dialog_for_dirs() called");
    Ok(serde_json::json!({
        "success": true,
        "message": "请使用前端 tauri.dialog.open({ directory: true }) 选择目录"
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_send_message(peer_id: String, content: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_send_message() called");
    require_lan()?;
    let result = with_lan(|lan| match lan.send_message(&peer_id, &content) {
        Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_send_file(
    peer_id: String,
    file_path: String,
    file_name: String,
    resume_offset: Option<u64>,
    file_id: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] lan_send_file() called, peer={}, file={}",
        peer_id,
        file_name
    );
    require_lan()?;
    let result = with_lan(|lan| {
        match lan.send_file(
            &peer_id,
            &file_path,
            &file_name,
            resume_offset.unwrap_or(0),
            file_id,
        ) {
            Ok(file_id) => serde_json::json!({ "success": true, "data": { "fileId": file_id } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_screenshot() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_screenshot() called");
    require_lan()?;
    let result = with_lan(|lan| match lan.screenshot() {
        Ok(path) => serde_json::json!({ "success": true, "data": { "path": path } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_save_temp_file(
    base64_data: String,
    file_name: String,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] lan_save_temp_file() called, file={}",
        file_name
    );
    require_lan()?;
    let result = with_lan(|lan| match lan.save_temp_file(&base64_data, &file_name) {
        Ok(path) => serde_json::json!({ "success": true, "data": { "path": path } }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_load_local_file_as_base64(file_path: String) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] lan_load_local_file_as_base64() called, path={}",
        file_path
    );
    require_lan()?;
    let result = with_lan(|lan| match lan.load_file_as_base64(&file_path) {
        Ok(encoded) => serde_json::json!({ "success": true, "data": encoded }),
        Err(e) => serde_json::json!({ "success": false, "error": e }),
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_open_file(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_open_file() called, path={}", file_path);
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(&file_path).spawn()
    } else {
        Command::new("xdg-open").arg(&file_path).spawn()
    };
    match result {
        Ok(_) => Ok(serde_json::json!({ "success": true })),
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_open_file_folder(file_path: String) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] lan_open_file_folder() called, path={}",
        file_path
    );
    let folder_path = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.clone());
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(&folder_path).spawn()
    } else {
        Command::new("xdg-open").arg(&folder_path).spawn()
    };
    match result {
        Ok(_) => Ok(serde_json::json!({ "success": true })),
        Err(e) => Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_messages_between(
    user_id1: String,
    user_id2: String,
    limit: usize,
    offset: usize,
) -> Result<serde_json::Value, String> {
    log::info!(
        "[Tauri CMD] lan_get_messages_between() called, user1={}, user2={}",
        user_id1,
        user_id2
    );
    require_lan()?;
    let result = with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_messages_between(&conn, &user_id1, &user_id2, limit, offset) {
                Ok(msgs) => {
                    let data: Vec<serde_json::Value> = msgs
                        .into_iter()
                        .map(|m| serde_json::to_value(m).unwrap_or_default())
                        .collect();
                    serde_json::json!({ "success": true, "data": data })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_check_network_permission() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_check_network_permission() called");
    use std::net::UdpSocket;

    // 收集本地 IPv4 地址，用于计算定向广播地址（x.y.z.255）
    let local_ips: Vec<std::net::Ipv4Addr> = match get_local_ipv4_addrs() {
        Ok(ips) => ips,
        Err(e) => {
            log::warn!("[lan_check_network_permission] get_local_ipv4_addrs failed: {}", e);
            Vec::new()
        }
    };

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            log::warn!("[lan_check_network_permission] UDP bind failed: {}", e);
            return Ok(serde_json::json!({ "success": true, "data": { "granted": false, "error": format!("UDP socket bind failed: {}", e), "kind": "bind_failed" } }));
        }
    };
    if let Err(e) = socket.set_broadcast(true) {
        log::warn!("[lan_check_network_permission] set_broadcast failed: {}", e);
    }

    // 候选广播目标：限制广播 + 各网卡的定向广播
    let mut targets: Vec<String> = vec!["255.255.255.255:49152".to_string()];
    for ip in &local_ips {
        let octets = ip.octets();
        // 跳过回环和链路本地
        if octets[0] == 127 || (octets[0] == 169 && octets[1] == 254) {
            continue;
        }
        targets.push(format!("{}.{}.{}.255:49152", octets[0], octets[1], octets[2]));
    }

    let mut last_err: Option<String> = None;
    let mut last_kind: String = "unknown".to_string();
    for addr in &targets {
        match socket.send_to(&[0u8], addr) {
            Ok(_) => {
                log::info!("[lan_check_network_permission] broadcast OK via {}", addr);
                return Ok(serde_json::json!({ "success": true, "data": { "granted": true } }));
            }
            Err(e) => {
                let kind = e.kind();
                log::warn!("[lan_check_network_permission] broadcast to {} failed: {} (kind={:?})", addr, e, kind);
                // 权限拒绝是确定性失败，无需继续尝试
                if kind == std::io::ErrorKind::PermissionDenied {
                    return Ok(serde_json::json!({ "success": true, "data": { "granted": false, "error": format!("Operation not permitted (PermissionDenied): {}", e), "kind": "tcc_blocked" } }));
                }
                last_err = Some(format!("{}", e));
                last_kind = format!("{:?}", kind);
            }
        }
    }

    let err_msg = last_err.unwrap_or_else(|| "UDP broadcast not permitted".to_string());
    Ok(serde_json::json!({ "success": true, "data": { "granted": false, "error": err_msg, "kind": last_kind } }))
}

/// 获取本机所有非回环 IPv4 地址
fn get_local_ipv4_addrs() -> Result<Vec<std::net::Ipv4Addr>, String> {
    use std::process::Command;
    // macOS: ifconfig，Linux: ip addr。优先 ifconfig（macOS 默认有），失败回退 ip
    let out = Command::new("ifconfig").output();
    let text = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => {
            let o = Command::new("ip").args(["-4", "addr"]).output()
                .map_err(|e| format!("ip addr failed: {}", e))?;
            String::from_utf8_lossy(&o.stdout).to_string()
        }
    };
    let mut ips = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // ifconfig: "inet 192.168.1.5 netmask ..."
        // ip addr:  "inet 192.168.1.5/24 brd ..."
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let addr_str = rest.split_whitespace().next().unwrap_or("");
            let addr_clean = addr_str.split('/').next().unwrap_or("");
            if let Ok(ip) = addr_clean.parse::<std::net::Ipv4Addr>() {
                if !ip.is_loopback() && !ip.is_link_local() {
                    ips.push(ip);
                }
            }
        }
    }
    Ok(ips)
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_permission_status() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_permission_status() called");
    use std::net::UdpSocket;

    let (granted, detail) = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            if let Err(e) = socket.set_broadcast(true) {
                log::warn!("[lan_get_permission_status] set_broadcast failed: {}", e);
            }
            match socket.send_to(&[0u8], "255.255.255.255:49152") {
                Ok(_) => ("granted", "UDP broadcast OK".to_string()),
                Err(e) => ("denied", format!("UDP broadcast failed: {}", e)),
            }
        }
        Err(e) => ("denied", format!("UDP bind failed: {}", e)),
    };
    Ok(serde_json::json!({ "success": true, "data": { "status": granted, "detail": detail } }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_message_history(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_message_history() called");
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    require_lan()?;
    let result = with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_chat_messages(&conn) {
                Ok(msgs) => {
                    let page: Vec<serde_json::Value> = msgs
                        .into_iter()
                        .skip(offset)
                        .take(limit)
                        .map(|m| serde_json::to_value(m).unwrap_or_default())
                        .collect();
                    serde_json::json!({ "success": true, "data": page })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            let all_history = lan.get_message_history(limit + offset);
            let page: Vec<serde_json::Value> = all_history
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            serde_json::json!({ "success": true, "data": page })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_file_transfer_history(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_file_transfer_history() called");
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    require_lan()?;
    let result = with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_file_transfers(&conn) {
                Ok(transfers) => {
                    let page: Vec<serde_json::Value> = transfers
                        .into_iter()
                        .skip(offset)
                        .take(limit)
                        .map(|t| serde_json::to_value(t).unwrap_or_default())
                        .collect();
                    serde_json::json!({ "success": true, "data": page })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            let transfers = lan.get_file_transfer_history(limit + offset, offset);
            let data: Vec<serde_json::Value> = transfers
                .into_iter()
                .map(|t| serde_json::to_value(t).unwrap_or_default())
                .collect();
            serde_json::json!({ "success": true, "data": data })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_logs(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_logs() called");
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    require_lan()?;
    let result = with_lan(|lan| {
        let all_logs = lan.get_logs(limit + offset);
        let page: Vec<serde_json::Value> = all_logs
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect();
        serde_json::json!({ "success": true, "data": page })
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_mark_messages_read(peer_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_mark_messages_read() called");
    require_lan()?;
    let result = with_lan(|lan| {
        let my_user_id = lan.get_user_id();
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::mark_messages_read(&conn, &my_user_id, &peer_id) {
                Ok(count) => serde_json::json!({ "success": true, "data": { "marked": count } }),
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_unread_count(peer_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_unread_count() called");
    require_lan()?;
    let result = with_lan(|lan| {
        let my_user_id = lan.get_user_id();
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_unread_count(&conn, &my_user_id, &peer_id) {
                Ok(count) => serde_json::json!({ "success": true, "data": { "count": count } }),
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

#[tauri::command(rename_all = "camelCase")]
/// 读取图片文件并返回 base64 数据（用于前端预览）
pub fn lan_read_image_file(file_path: String) -> Result<serde_json::Value, String> {
    use std::fs;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    // 检查文件扩展名，确定 MIME 类型
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();
    let mime_types = [
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("png", "image/png"),
        ("gif", "image/gif"),
        ("webp", "image/webp"),
        ("bmp", "image/bmp"),
        ("svg", "image/svg+xml"),
    ];
    let mime = mime_types
        .iter()
        .find(|(k, _)| k == &ext)
        .map(|(_, v)| *v)
        .unwrap_or("image/jpeg");
    
    // 读取文件
    let data = match fs::read(&file_path) {
        Ok(d) => d,
        Err(e) => return Ok(serde_json::json!({ "success": false, "error": e.to_string() })),
    };
    
    // 转换为 base64
    let base64_str = STANDARD.encode(&data);
    
    log::info!("[LAN] Read image file: {} ({} bytes, mime={})", file_path, data.len(), mime);
    
    Ok(serde_json::json!({
        "success": true,
        "data": {
            "base64": base64_str,
            "mime": mime,
            "size": data.len(),
            "url": format!("data:{};base64,{}", mime, base64_str)
        }
    }))
}

/// 设置局域网用户的备注名（本地存储，不会同步到其他设备）
#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_peer_remark(peer_id: String, remark: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_peer_remark() called, peer={}, remark={}", peer_id, remark);
    let result = with_lan(|lan| {
        let key = format!("peer_remark:{}", peer_id);
        if let Ok(conn) = lan.get_db_conn().lock() {
            let trimmed = remark.trim();
            if trimmed.is_empty() {
                // 空备注 = 删除
                let _ = conn.execute("DELETE FROM lan_settings WHERE key = ?1", rusqlite::params![key]);
            } else {
                let _ = lan::save_lan_setting(&conn, &key, trimmed);
            }
            serde_json::json!({ "success": true })
        } else {
            serde_json::json!({ "success": false, "error": "数据库锁定失败" })
        }
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": false, "error": "LAN 服务未启动" })))
}

/// 获取所有局域网用户的备注名（返回 { peerId: remark } 映射）
#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_peer_remarks() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_peer_remarks() called");
    let result = with_lan(|lan| {
        let mut remarks = serde_json::Map::new();
        if let Ok(conn) = lan.get_db_conn().lock() {
            // 查询所有 peer_remark: 开头的设置项
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM lan_settings WHERE key LIKE 'peer_remark:%'") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    for row in rows.flatten() {
                        let peer_id = row.0.strip_prefix("peer_remark:").unwrap_or(&row.0).to_string();
                        remarks.insert(peer_id, serde_json::Value::String(row.1));
                    }
                }
            }
        }
        serde_json::json!({ "success": true, "data": { "remarks": remarks } })
    });
    Ok(result.unwrap_or(serde_json::json!({ "success": true, "data": { "remarks": {} } })))
}
