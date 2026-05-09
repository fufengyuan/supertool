use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_servers(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_servers() called");
    core.get_all_servers().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_server_by_id(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_server_by_id() called");
    core.get_server_by_id(&server_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_server(
    core: State<'_, CoreService>,
    server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_server() called");
    core.add_server(server).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_server(
    core: State<'_, CoreService>,
    server: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_server() called");
    core.update_server(server).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_server(
    core: State<'_, CoreService>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_server() called");
    core.delete_server(&server_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_server_groups(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_server_groups() called");
    core.get_all_server_groups().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_server_group(
    core: State<'_, CoreService>,
    group: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_server_group() called");
    core.add_server_group(group).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_server_group(
    core: State<'_, CoreService>,
    group_id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    color: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_server_group() called");
    let params = serde_json::json!({
        "name": name,
        "description": description,
        "parentId": parent_id,
        "color": color,
    });
    core.update_server_group(&group_id, params).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_server_group(
    core: State<'_, CoreService>,
    group_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_server_group() called");
    core.delete_server_group(&group_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_connection(
    core: State<'_, CoreService>,
    host: String,
    port: i64,
    username: String,
    password: Option<String>,
    ssh_key_path: Option<String>,
    server_id: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] test_connection() called");
    let params = serde_json::json!({
        "id": server_id.unwrap_or_default(),
        "host": host,
        "port": port,
        "username": username,
        "password": password,
        "sshKeyPath": ssh_key_path,
    });
    core.ssh_test_connection(params).await
}

// =================== SFTP Commands (all via async wrappers, never blocking async runtime) ===================

#[tauri::command(rename_all = "camelCase")]
pub async fn sftp_upload_file(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    remote_path: String,
    local_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] sftp_upload_file() called");
    core.ensure_ssh_connected(&server_id).await?;
    core.sftp_upload_to_remote(&server_id, &local_path, &remote_path).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn sftp_download_file(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    remote_path: String,
    local_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] sftp_download_file() called");
    core.ensure_ssh_connected(&server_id).await?;
    core.sftp_download_to_local(&server_id, &remote_path, &local_path).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn sftp_upload_folder(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    remote_path: String,
    local_path: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] sftp_upload_folder() called");
    core.ensure_ssh_connected(&server_id).await?;
    core.sftp_upload_dir_recursive(&server_id, &local_path, &remote_path).await
}

#[tauri::command(rename_all = "camelCase")]
pub fn sftp_get_downloads_dir() -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] sftp_get_downloads_dir() called");
    let download_dir = dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .ok_or_else(|| "Cannot determine downloads directory".to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "data": download_dir.to_string_lossy().to_string()
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_sftp_dir(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    path: String,
) -> Result<serde_json::Value, String> {
    core.ensure_ssh_connected(&server_id).await?;
    core.sftp_list_dir(&server_id, &path).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn open_sftp_file_editor(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    file_path: String,
) -> Result<serde_json::Value, String> {
    core.ensure_ssh_connected(&server_id).await?;
    core.sftp_download_file(&server_id, &file_path).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_sftp_file(
    core: State<'_, supertool_core::logic::CoreService>,
    server_id: String,
    file_path: String,
    is_dir: bool,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_sftp_file() called");
    if is_dir {
        // Fallback to shell rm -rf for reliable directory deletion
        let escaped = file_path.replace('\'', "'\\\\''");
        let result = core.exec_ssh_command(&server_id, &format!("rm -rf '{}'", escaped)).await?;
        if !result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Err(format!("删除失败: {}", result.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误")));
        }
        Ok(serde_json::json!({ "success": true }))
    } else {
        core.sftp_delete_file(&server_id, &file_path).await
    }
}
