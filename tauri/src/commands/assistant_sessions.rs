use supertool_core::logic::CoreService;
use tauri::State;

/// AI 配置助手历史会话的 CRUD 命令。
/// 会话数据统一存本地 SQLite，主窗口页与悬浮窗（不同 webview）共用同一批会话。
#[tauri::command(rename_all = "camelCase")]
pub async fn list_assistant_sessions(
    core: State<'_, CoreService>,
    with_messages: Option<bool>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] list_assistant_sessions() called");
    core.list_assistant_sessions_full(with_messages.unwrap_or(false)).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_assistant_session(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_assistant_session() called");
    core.get_assistant_session(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_assistant_session(
    core: State<'_, CoreService>,
    id: String,
    title: String,
    messages: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] save_assistant_session() called");
    core.save_assistant_session(&id, &title, &messages).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_assistant_session(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_assistant_session() called");
    core.delete_assistant_session(&id).await
}