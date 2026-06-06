//! Hermes Sessions management commands.

#[tauri::command(rename_all = "camelCase")]
pub fn sessions_export(
    output: String,
    source: Option<String>,
    session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut args: Vec<String> = vec!["sessions".into(), "export".into()];
    if let Some(ref s) = source {
        args.push("--source".into());
        args.push(s.clone());
    }
    if let Some(ref sid) = session_id {
        args.push("--session-id".into());
        args.push(sid.clone());
    }
    args.push(output);

    let cmd = std::process::Command::new("hermes")
        .args(&args)
        .output()
        .map_err(|e| format!("执行 hermes sessions export 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&cmd.stdout);
    let stderr = String::from_utf8_lossy(&cmd.stderr);

    Ok(serde_json::json!({
        "success": cmd.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn sessions_prune(
    older_than: Option<i32>,
    source: Option<String>,
    yes: Option<bool>,
) -> Result<serde_json::Value, String> {
    let mut args: Vec<String> = vec!["sessions".into(), "prune".into()];
    if let Some(days) = older_than {
        args.push("--older-than".into());
        args.push(days.to_string());
    }
    if let Some(ref s) = source {
        args.push("--source".into());
        args.push(s.clone());
    }
    if yes.unwrap_or(false) {
        args.push("--yes".into());
    }

    let cmd = std::process::Command::new("hermes")
        .args(&args)
        .output()
        .map_err(|e| format!("执行 hermes sessions prune 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&cmd.stdout);
    let stderr = String::from_utf8_lossy(&cmd.stderr);

    Ok(serde_json::json!({
        "success": cmd.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}
