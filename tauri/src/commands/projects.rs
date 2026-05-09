use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_projects(
    core: State<'_, CoreService>,
    only_active: bool,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_projects() called");
    core.get_all_projects(only_active).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_project(
    core: State<'_, CoreService>,
    project: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_project() called");
    core.add_project(project).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_project(
    core: State<'_, CoreService>,
    project: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_project() called");
    core.update_project(project).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_project(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_project() called");
    core.delete_project(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_project_stats(
    core: State<'_, CoreService>,
    project_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_project_stats() called");
    core.get_project_stats(&project_id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_project_todos(
    core: State<'_, CoreService>,
    project_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_project_todos() called");
    core.get_project_todos(&project_id).await
}
