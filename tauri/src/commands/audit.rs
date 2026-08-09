use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub fn audit_list(
    core: State<'_, CoreService>,
    actor: Option<String>,
    result: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] audit_list() called");
    core.list_audit(
        actor.as_deref(),
        result.as_deref(),
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
}
