//! Hermes Insights / usage analytics command (pure Rust, no Python bridge).
//!
//! Wraps `hermes insights [--days DAYS] [--source SOURCE]`.

use serde_json::json;
use std::process::Command;

/// Get usage insights and analytics.
///
/// Wraps `hermes insights [--days DAYS] [--source SOURCE]`.
/// Returns the full text output from the CLI for frontend display.
#[tauri::command(rename_all = "camelCase")]
pub fn get_insights(
    days: Option<i32>,
    source: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut args: Vec<String> = vec!["insights".into()];
    if let Some(d) = days {
        args.push("--days".into());
        args.push(d.to_string());
    }
    if let Some(ref s) = source {
        args.push("--source".into());
        args.push(s.clone());
    }

    let cmd = Command::new("hermes")
        .args(&args)
        .output()
        .map_err(|e| format!("执行 hermes insights 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&cmd.stdout);
    let stderr = String::from_utf8_lossy(&cmd.stderr);

    Ok(json!({
        "success": cmd.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}
