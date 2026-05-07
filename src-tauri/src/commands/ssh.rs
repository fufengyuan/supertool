/// Tauri IPC Commands — SSH 连接 & 服务器监控
use crate::core::CoreService;
use serde_json::{json, Value};
use tauri::State;

/// 通过服务器 ID 建立 SSH 连接（先查 DB 拿配置，再连接）
#[tauri::command(rename_all = "camelCase")]
pub async fn connect_server(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] connect_server({})", server_id);

    // 从 DB 查找服务器配置
    let server = core.get_server_by_id(&server_id).await?;
    let data = server.as_object().ok_or("服务器数据格式错误")?;

    // 构建 SSH 配置
    let config = json!({
        "id": data.get("id").and_then(|v| v.as_str()).unwrap_or(""),
        "name": data.get("name").and_then(|v| v.as_str()).unwrap_or(""),
        "host": data.get("host").and_then(|v| v.as_str()).unwrap_or(""),
        "port": data.get("port").and_then(|v| v.as_u64()).unwrap_or(22),
        "username": data.get("username").and_then(|v| v.as_str()).unwrap_or(""),
        "password": data.get("password").and_then(|v| v.as_str()),
        "sshKeyPath": data.get("sshKeyPath").and_then(|v| v.as_str()),
    });

    core.ssh_connect(config).await?;
    Ok(json!({ "success": true, "serverId": server_id }))
}

/// 断开 SSH 连接
#[tauri::command(rename_all = "camelCase")]
pub async fn disconnect_server(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] disconnect_server({})", server_id);
    core.ssh_disconnect(&server_id).await?;
    Ok(json!({ "success": true }))
}

/// 检查服务器是否已连接
#[tauri::command(rename_all = "camelCase")]
pub async fn is_server_connected(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<Value, String> {
    let connected = core.ssh_is_connected(&server_id).await?;
    Ok(connected)
}

/// 在服务器上批量执行命令（用于监控面板）
/// 接受 commands: Vec<String>，依次执行并返回 {命令: 输出} 的映射
#[tauri::command(rename_all = "camelCase")]
pub async fn get_server_monitor(
    core: State<'_, CoreService>,
    server_id: String,
    commands: Vec<String>,
) -> Result<Value, String> {
    log::info!("[Tauri CMD] get_server_monitor({}), {} commands", server_id, commands.len());

    // 确保已连接
    let connected = core.ssh_is_connected(&server_id).await?;
    let is_conn = connected
        .get("connected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !is_conn {
        // 自动连接
        let server = core.get_server_by_id(&server_id).await?;
        let data = server.as_object().ok_or("服务器数据格式错误")?;
        let config = json!({
            "id": data.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "name": data.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "host": data.get("host").and_then(|v| v.as_str()).unwrap_or(""),
            "port": data.get("port").and_then(|v| v.as_u64()).unwrap_or(22),
            "username": data.get("username").and_then(|v| v.as_str()).unwrap_or(""),
            "password": data.get("password").and_then(|v| v.as_str()),
            "sshKeyPath": data.get("sshKeyPath").and_then(|v| v.as_str()),
        });
        core.ssh_connect(config).await?;
    }

    let mut results = serde_json::Map::new();
    for cmd in &commands {
        match core.exec_ssh_command(&server_id, cmd).await {
            Ok(result) => {
                if let Some(output) = result.get("output").and_then(|v| v.as_str()) {
                    results.insert(cmd.clone(), json!(output));
                } else {
                    results.insert(cmd.clone(), json!(result.to_string()));
                }
            }
            Err(e) => {
                results.insert(cmd.clone(), json!(format!("ERROR: {}", e)));
            }
        }
    }

    Ok(json!({
        "success": true,
        "results": results,
    }))
}
