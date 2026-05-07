use crate::core::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_notes(
    core: State<'_, CoreService>,
    query: Option<String>,
    group_id: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_notes() called");
    let result = core.get_all_notes(query, group_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_note(
    core: State<'_, CoreService>,
    note: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_note() called");
    let result = core.add_note(note).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_note(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_note() called");
    let result = core.update_note(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_note(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_note() called");
    let result = core.delete_note(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_note_groups(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_note_groups() called");
    let result = core.get_all_note_groups().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_note_group(
    core: State<'_, CoreService>,
    group: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_note_group() called");
    let result = core.add_note_group(group).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_note_group(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_note_group() called");
    let result = core.update_note_group(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_note_group(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_note_group() called");
    let result = core.delete_note_group(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}
