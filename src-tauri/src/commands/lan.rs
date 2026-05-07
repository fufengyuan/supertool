/// LAN 协作 Tauri Commands — 全部绑定到实际 LanService 方法和 DB 查询
use crate::core::lan::LanService;
use crate::db::lan;
use std::sync::{Arc, Mutex};
use std::process::Command;
use tauri::AppHandle;

use std::sync::OnceLock;
static LAN_SERVICE: OnceLock<Arc<Mutex<Option<LanService>>>> = OnceLock::new();
static LAN_DB_PATH: OnceLock<String> = OnceLock::new();

pub fn init_lan_service(db_conn: Arc<Mutex<rusqlite::Connection>>) -> Arc<Mutex<Option<LanService>>> {
    LAN_SERVICE.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

/// Store DB path for later use by lan_start (called from main.rs setup)
pub fn init_lan_service_with_db(db_path: &str) {
    let _ = LAN_DB_PATH.set(db_path.to_string());
    let _ = LAN_SERVICE.get_or_init(|| Arc::new(Mutex::new(None)));
}

pub fn get_lan_service() -> Option<Arc<Mutex<Option<LanService>>>> {
    LAN_SERVICE.get().cloned()
}

/// Helper: lock the LAN service and run a closure on it
fn with_lan<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&LanService) -> T,
{
    if let Some(svc) = get_lan_service() {
        let guard = svc.lock().map_err(|e| format!("Lock error: {}", e))?;
        if let Some(lan) = guard.as_ref() {
            return Ok(f(lan));
        }
    }
    Err("LAN 服务未启动".to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_start(app: AppHandle, user_id: String, user_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_start() called, user={}", user_name);
    let db_path = LAN_DB_PATH.get().ok_or("DB path not set. Call init_lan_service_with_db first.")?;
    let svc = get_lan_service().ok_or("LAN service not initialized")?;
    let mut guard = svc.lock().map_err(|e| format!("Lock error: {}", e))?;
    if guard.is_some() {
        return Ok(serde_json::json!({ "success": true, "message": "LAN 已在运行" }));
    }
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("打开数据库失败: {}", e))?;
    let db_conn = Arc::new(Mutex::new(conn));
    let lan = LanService::new(user_id, user_name, db_conn);
    lan.set_app_handle(app.clone());
    lan.start().map_err(|e| format!("启动失败: {}", e))?;
    *guard = Some(lan);
    Ok(serde_json::json!({ "success": true, "message": "LAN 服务已启动" }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_start_with_db(app: AppHandle, user_id: String, user_name: String, db_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_start_with_db() called, user={}", user_name);
    let svc = get_lan_service().ok_or("LAN service not initialized")?;
    let mut guard = svc.lock().map_err(|e| format!("Lock error: {}", e))?;
    if guard.is_some() {
        return Ok(serde_json::json!({ "success": true, "message": "LAN 已在运行" }));
    }

    // Open a dedicated LAN DB connection (or reuse the main one)
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("打开数据库失败: {}", e))?;
    let db_conn = Arc::new(Mutex::new(conn));

    let lan = LanService::new(user_id, user_name, db_conn);
    lan.set_app_handle(app.clone());
    lan.start().map_err(|e| format!("启动失败: {}", e))?;
    *guard = Some(lan);
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
    with_lan(|lan| {
        lan.set_status(status);
        serde_json::json!({ "success": true })
    }).map(|v| v)
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_assign_task(peer_id: String, task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_assign_task() called, peer={}", peer_id);
    let task_val: serde_json::Value = serde_json::from_str(&task)
        .unwrap_or_else(|_| serde_json::json!({ "task": task }));
    with_lan(|lan| {
        match lan.assign_task(&peer_id, &task_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_sync_task_status(task_json: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_sync_task_status() called");
    let task_val: serde_json::Value = serde_json::from_str(&task_json)
        .unwrap_or_else(|_| serde_json::json!({ "task": task_json }));
    with_lan(|lan| {
        match lan.broadcast_task_status_change(&task_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_refresh_discovery() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_refresh_discovery() called");
    with_lan(|lan| {
        lan.refresh_discovery();
        serde_json::json!({ "success": true })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_message(message: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_message() called");
    with_lan(|lan| {
        match lan.broadcast_message(&message) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_update(task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_update() called");
    let task_val: serde_json::Value = serde_json::from_str(&task)
        .unwrap_or_else(|_| serde_json::json!({ "task": task }));
    with_lan(|lan| {
        match lan.broadcast_task_update(&task_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_status_change(task: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_status_change() called");
    let task_val: serde_json::Value = serde_json::from_str(&task)
        .unwrap_or_else(|_| serde_json::json!({ "task": task }));
    with_lan(|lan| {
        match lan.broadcast_task_status_change(&task_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_task_comment(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_task_comment() called");
    // Extract peer_id from data if possible, otherwise broadcast
    let data_val: serde_json::Value = serde_json::from_str(&data)
        .unwrap_or_else(|_| serde_json::json!({ "data": data }));
    if let Some(peer_id) = data_val.get("peerId").and_then(|v| v.as_str()) {
        with_lan(|lan| {
            match lan.broadcast_task_comment(peer_id, &data_val) {
                Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
                Err(e) => serde_json::json!({ "success": false, "error": e }),
            }
        })
    } else {
        Err("缺少 peerId".to_string())
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_collaboration_started(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_collaboration_started() called");
    let data_val: serde_json::Value = serde_json::from_str(&data)
        .unwrap_or_else(|_| serde_json::json!({ "data": data }));
    with_lan(|lan| {
        match lan.broadcast_collaboration_started(&data_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_broadcast_collaboration_ended(data: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_broadcast_collaboration_ended() called");
    let data_val: serde_json::Value = serde_json::from_str(&data)
        .unwrap_or_else(|_| serde_json::json!({ "data": data }));
    with_lan(|lan| {
        match lan.broadcast_collaboration_ended(&data_val) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_user_info() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_user_info() called");
    with_lan(|lan| {
        serde_json::json!({ "success": true, "data": lan.get_user_info() })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_all_unread_counts(user_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_all_unread_counts() called");
    with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_unread_counts(&conn, &user_id) {
                Ok(counts) => {
                    let data: serde_json::Map<String, serde_json::Value> = counts
                        .into_iter()
                        .map(|(peer_id, peer_name, count)| {
                            (peer_id.clone(), serde_json::json!({
                                "name": peer_name,
                                "count": count,
                            }))
                        })
                        .collect();
                    serde_json::json!({ "success": true, "data": data })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_status() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_status() called");
    with_lan(|lan| {
        serde_json::json!({
            "success": true,
            "data": {
                "running": lan.is_running(),
                "status": lan.get_status(),
            }
        })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_network_info() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_network_info() called");
    with_lan(|lan| {
        serde_json::json!({
            "address": lan.get_local_ip(),
            "ports": "49152/49154",
            "version": "2.0",
        })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_receive_path() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_receive_path() called");
    with_lan(|lan| {
        serde_json::json!({ "success": true, "data": lan.get_receive_path() })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_peers() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_peers() called");
    let peers = with_lan(|lan| lan.get_online_peers())?;
    Ok(serde_json::to_value(peers).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_nick_name(name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_nick_name() called");
    with_lan(|lan| {
        lan.set_nickname(name);
        serde_json::json!({ "success": true })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_avatar(avatar: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_avatar() called");
    with_lan(|lan| {
        lan.set_avatar(avatar);
        serde_json::json!({ "success": true })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_set_receive_path(path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_set_receive_path() called");
    with_lan(|lan| {
        lan.set_receive_path(path);
        serde_json::json!({ "success": true })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_show_open_dialog_for_dirs(app: AppHandle) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_show_open_dialog_for_dirs() called");
    // Use Tauri's dialog plugin — but we can't block here in a command easily.
    // We'll return an error telling the frontend to use the JS dialog API instead,
    // since tauri-plugin-dialog doesn't expose a programmatic Rust-only directory picker
    // without the event loop.
    // Actually, we can use tauri::api::dialog if available. For simplicity:
    Ok(serde_json::json!({
        "success": true,
        "message": "请使用前端 tauri.dialog.open({ directory: true }) 选择目录"
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_send_message(peer_id: String, content: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_send_message() called");
    with_lan(|lan| {
        match lan.send_message(&peer_id, &content) {
            Ok(sent) => serde_json::json!({ "success": true, "data": { "sent": sent } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_send_file(peer_id: String, file_path: String, file_name: String, resume_offset: Option<u64>, file_id: Option<String>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_send_file() called, peer={}, file={}", peer_id, file_name);
    with_lan(|lan| {
        match lan.send_file(&peer_id, &file_path, &file_name, resume_offset.unwrap_or(0), file_id) {
            Ok(file_id) => serde_json::json!({ "success": true, "data": { "fileId": file_id } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_screenshot() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_screenshot() called");
    with_lan(|lan| {
        match lan.screenshot() {
            Ok(path) => serde_json::json!({ "success": true, "data": { "path": path } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_save_temp_file(base64_data: String, file_name: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_save_temp_file() called, file={}", file_name);
    with_lan(|lan| {
        match lan.save_temp_file(&base64_data, &file_name) {
            Ok(path) => serde_json::json!({ "success": true, "data": { "path": path } }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_load_local_file_as_base64(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_load_local_file_as_base64() called, path={}", file_path);
    with_lan(|lan| {
        match lan.load_file_as_base64(&file_path) {
            Ok(encoded) => serde_json::json!({ "success": true, "data": encoded }),
            Err(e) => serde_json::json!({ "success": false, "error": e }),
        }
    })
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
    log::info!("[Tauri CMD] lan_open_file_folder() called, path={}", file_path);
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
pub fn lan_get_messages_between(user_id1: String, user_id2: String, limit: usize, offset: usize) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_messages_between() called, user1={}, user2={}", user_id1, user_id2);
    with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_messages_between(&conn, &user_id1, &user_id2, limit, offset) {
                Ok(msgs) => {
                    let data: Vec<serde_json::Value> = msgs.into_iter()
                        .map(|m| serde_json::to_value(m).unwrap_or_default())
                        .collect();
                    serde_json::json!({ "success": true, "data": data })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            serde_json::json!({ "success": false, "error": "DB lock failed" })
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_check_network_permission() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_check_network_permission() called");
    if cfg!(target_os = "linux") {
        return Ok(serde_json::json!({ "success": true, "data": { "granted": true } }));
    }
    Ok(serde_json::json!({ "success": true, "data": { "granted": true } }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_permission_status() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_permission_status() called");
    if cfg!(target_os = "linux") {
        return Ok(serde_json::json!({ "success": true, "data": { "status": "granted" } }));
    }
    Ok(serde_json::json!({ "success": true, "data": { "status": "granted" } }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_message_history(limit: Option<usize>, offset: Option<usize>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_message_history() called");
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_chat_messages(&conn) {
                Ok(msgs) => {
                    let page: Vec<serde_json::Value> = msgs.into_iter()
                        .skip(offset)
                        .take(limit)
                        .map(|m| serde_json::to_value(m).unwrap_or_default())
                        .collect();
                    serde_json::json!({ "success": true, "data": page })
                }
                Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }),
            }
        } else {
            // Fallback to in-memory
            let all_history = lan.get_message_history(limit + offset);
            let page: Vec<serde_json::Value> = all_history.into_iter()
                .skip(offset)
                .take(limit)
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();
            serde_json::json!({ "success": true, "data": page })
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_file_transfer_history(limit: Option<usize>, offset: Option<usize>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_file_transfer_history() called");
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    with_lan(|lan| {
        let db_conn = lan.get_db_conn();
        if let Ok(conn) = db_conn.lock() {
            match lan::get_all_file_transfers(&conn) {
                Ok(transfers) => {
                    let page: Vec<serde_json::Value> = transfers.into_iter()
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
            let data: Vec<serde_json::Value> = transfers.into_iter()
                .map(|t| serde_json::to_value(t).unwrap_or_default())
                .collect();
            serde_json::json!({ "success": true, "data": data })
        }
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_logs(limit: Option<usize>, offset: Option<usize>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_logs() called");
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    with_lan(|lan| {
        let all_logs = lan.get_logs(limit + offset);
        let page: Vec<serde_json::Value> = all_logs.into_iter()
            .skip(offset)
            .take(limit)
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect();
        serde_json::json!({ "success": true, "data": page })
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_mark_messages_read(peer_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_mark_messages_read() called");
    with_lan(|lan| {
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
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn lan_get_unread_count(peer_id: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] lan_get_unread_count() called");
    with_lan(|lan| {
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
    })
}
