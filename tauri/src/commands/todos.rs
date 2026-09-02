use supertool_core::logic::CoreService;
use tauri::{Emitter, State};

#[tauri::command(rename_all = "camelCase")]
pub async fn get_todos(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_todos() called");
    let result = core.get_all_todos().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_todo(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    todo: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_todo() called");
    let result = core.add_todo(todo).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_todo(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_todo() called");
    let result = core.update_todo(params).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_todo(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_todo() called");
    let result = core.delete_todo(&id).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_tag(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    name: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_tag() called");
    let result = core.add_tag(&name).await?;
    let _ = app.emit("tags-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_tags(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_tags() called");
    let result = core.get_all_tags().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_tag(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    name: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_tag() called");
    let result = core.delete_tag(&name).await?;
    let _ = app.emit("tags-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_subtask(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    subtask: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_subtask() called");
    let result = core.add_subtask(subtask).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_subtask(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_subtask() called");
    let result = core.update_subtask(params).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_subtask(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_subtask() called");
    let result = core.delete_subtask(&id).await?;
    let _ = app.emit("todos-changed", ());
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_subtasks_for_todo(
    core: State<'_, CoreService>,
    todo_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_subtasks_for_todo() called");
    let result = core.get_subtasks_for_todo(&todo_id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_many(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_many() called");
    let resp = core.delete_todos_many(ids).await?;
    let _ = app.emit("todos-changed", ());
    Ok(resp)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_order(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    items: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_order() called");
    let resp = core.update_todos_order(items).await?;
    let _ = app.emit("todos-changed", ());
    Ok(resp)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_repeat_instance(
    app: tauri::AppHandle,
    core: State<'_, CoreService>,
    todo_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] create_repeat_instance() called");
    // 统一走 core 的完整实现（读取 repeatType/interval/endDate 计算下次执行日期）。
    // 原 GUI 内联 SQL 是简化版（不处理重复周期、直接复制文本）；core 版是权威且更能
    // 反映「重复待办」语义。返回多出的 dueDate 字段对前端无害。
    let resp = core.create_repeat_instance(&todo_id).await?;
    let _ = app.emit("todos-changed", ());
    Ok(resp)
}

