/// Tauri IPC Commands — PTY 终端操作
///
/// 给 GUI 调用的终端相关命令，通过 CoreService 异步包装器调用 SSH 服务。
/// 所有 SSH 操作经 run_ssh_blocking → spawn_blocking 执行，不阻塞 async 运行时。

use supertool_core::logic::CoreService;
use serde_json::{Value, json};
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn create_terminal(
    core: State<'_, CoreService>,
    server_id: String,
    terminal_id: String,
    rows: u32,
    cols: u32,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] create_terminal() called");
    core.ssh_create_terminal(&server_id, &terminal_id, rows, cols).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn read_terminal(
    core: State<'_, CoreService>,
    terminal_id: String,
) -> Result<Value, String> {
    core.ssh_read_terminal(&terminal_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn resize_terminal(
    core: State<'_, CoreService>,
    terminal_id: String,
    rows: u32,
    cols: u32,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] resize_terminal() called");
    core.ssh_resize_terminal(&terminal_id, rows, cols).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn write_to_terminal(
    core: State<'_, CoreService>,
    terminal_id: String,
    data: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] write_to_terminal() called");
    core.ssh_write_to_terminal(&terminal_id, &data).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn close_terminal(
    core: State<'_, CoreService>,
    terminal_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] close_terminal() called");
    core.ssh_close_terminal(&terminal_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn is_terminal_active(
    core: State<'_, CoreService>,
    terminal_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] is_terminal_active() called");
    let active = core.ssh_is_terminal_active(&terminal_id).await;
    Ok(json!({"success": true, "active": active}))
}
