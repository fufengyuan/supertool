use supertool_core::logic::CoreService;
use tauri::State;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_todos(core: State<'_, CoreService>) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_todos() called");
    let result = core.get_all_todos().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_todo(
    core: State<'_, CoreService>,
    todo: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_todo() called");
    let result = core.add_todo(todo).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_todo(
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_todo() called");
    let result = core.update_todo(params).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_todo(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_todo() called");
    let result = core.delete_todo(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_tag(
    core: State<'_, CoreService>,
    name: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_tag() called");
    let result = core.add_tag(&name).await?;
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
    core: State<'_, CoreService>,
    name: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_tag() called");
    let result = core.delete_tag(&name).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_subtask(
    core: State<'_, CoreService>,
    subtask: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_subtask() called");
    let result = core.add_subtask(subtask).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_subtask(
    core: State<'_, CoreService>,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_subtask() called");
    let result = core.update_subtask(params).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_subtask(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_subtask() called");
    let result = core.delete_subtask(&id).await?;
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
    core: State<'_, CoreService>,
    ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_many() called");
    if ids.is_empty() {
        return Ok(serde_json::json!({ "deleted": 0 }));
    }
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!("DELETE FROM todos WHERE id IN ({})", placeholders.join(","));
    let deleted = core.db_write(|conn| {
        conn.execute(&sql, rusqlite::params_from_iter(&ids))
            .map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "deleted": deleted }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_order(
    core: State<'_, CoreService>,
    items: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_order() called");
    if items.is_empty() {
        return Ok(serde_json::json!({ "updated": 0 }));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let mut count = 0;
    for item in &items {
        let id = item["id"].as_str().unwrap_or("").to_string();
        let order_num = item["orderNum"].as_i64().unwrap_or(0);
        let _ = core.db_write(|conn| {
            conn.execute(
                "UPDATE todos SET orderNum = ?1, updatedAt = ?2 WHERE id = ?3",
                rusqlite::params![order_num, now, id],
            )
            .map_err(|e| e.to_string())
        })?;
        count += 1;
    }
    Ok(serde_json::json!({ "updated": count }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_repeat_instance(
    core: State<'_, CoreService>,
    todo_id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] create_repeat_instance() called");
    use rusqlite::params;
    let now = chrono::Utc::now().to_rfc3339();
    let new_id = uuid::Uuid::new_v4().to_string();
    let cloned_id = todo_id.clone();
    let todo_row = core.db_write(|conn| {
        let mut stmt = conn
            .prepare("SELECT text, priority, dueDate, description, markdownDescription, tag, projectId FROM todos WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_map(params![cloned_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2).unwrap_or_default(),
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5).unwrap_or_default(),
                    row.get::<_, Option<String>>(6).unwrap_or_default(),
                ))
            })
            .map_err(|e| e.to_string())?
            .next()
            .transpose()
            .map_err(|e: rusqlite::Error| e.to_string())?;
        Ok::<_, String>(result)
    })?;

    let (text, priority, due_date, description, markdown_desc, tag, project_id) =
        todo_row?.ok_or("Todo not found".to_string())?;

    let parent_id = todo_id.clone();
    let _ = core.db_write(|conn| {
        conn.execute(
            "INSERT INTO todos (id, text, completed, priority, dueDate, description, markdownDescription, tag, createdAt, updatedAt, orderNum, repeatCount, parentTodoId, projectId) VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10, ?11)",
            params![
                new_id,
                text,
                priority,
                due_date,
                description,
                markdown_desc,
                tag,
                now,
                now,
                parent_id,
                project_id,
            ],
        )
        .map_err(|e| e.to_string())
    })?;
    Ok(serde_json::json!({ "id": new_id, "parentTodoId": todo_id }))
}
