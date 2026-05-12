use supertool_core::db::git_repo as db_git_repo;
use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_git_repos(
    core: State<'_, CoreService>,
) -> Result<Vec<db_git_repo::GitRepo>, String> {
    log::info!("[Tauri CMD] get_git_repos() called");
    core.db_read(|conn| {
        db_git_repo::get_all(conn).map_err(|e| e.to_string())
    })?
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_git_repo(
    core: State<'_, CoreService>,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_git_repo() called");
    let id = data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let path = data.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let remote = data.get("remote").and_then(|v| v.as_str()).map(|s| s.to_string());
    let branch = data.get("branch").and_then(|v| v.as_str()).map(|s| s.to_string());

    if path.is_empty() {
        return Ok(serde_json::json!({ "success": false, "error": "仓库路径不能为空" }));
    }

    let _ = core.db_write(|conn| {
        db_git_repo::add(conn, &id, &name, &path, remote.as_deref(), branch.as_deref())
            .map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_git_repo(
    core: State<'_, CoreService>,
    id: String,
    data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_git_repo() called");
    let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let path = data.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let remote = data.get("remote").and_then(|v| v.as_str()).map(|s| s.to_string());
    let branch = data.get("branch").and_then(|v| v.as_str()).map(|s| s.to_string());

    let _ = core.db_write(|conn| {
        db_git_repo::update(conn, &id, &name, &path, remote.as_deref(), branch.as_deref())
            .map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_git_repo(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_git_repo() called");
    let _ = core.db_write(|conn| {
        db_git_repo::delete(conn, &id).map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "success": true }))
}