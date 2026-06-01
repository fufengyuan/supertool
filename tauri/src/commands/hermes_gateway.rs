//! Hermes Gateway management commands (pure Rust, no Python bridge).
//!
//! Wraps `hermes gateway status/start/stop/restart` commands.

use serde_json::json;
use std::process::Command;

/// Run a hermes gateway subcommand and parse JSON output.
fn gateway_cmd(args: &[&str]) -> Result<serde_json::Value, String> {
    let output = Command::new("hermes")
        .args(args)
        .output()
        .map_err(|e| format!("执行 hermes gateway 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hermes gateway 失败: {}", stderr.lines().next().unwrap_or("")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|_| stdout.to_string())
}

/// Get gateway status (PID, uptime, health).
#[tauri::command(rename_all = "camelCase")]
pub fn gateway_status() -> Result<serde_json::Value, String> {
    let output = Command::new("hermes")
        .args(["gateway", "status"])
        .output()
        .map_err(|e| format!("执行 hermes gateway status 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse status output — hermes gateway status returns text, not JSON
    let is_running = output.status.success() && stdout.contains("PID");
    let pid = if is_running {
        stdout
            .lines()
            .find(|l| l.contains("PID"))
            .and_then(|l| l.split_whitespace().last())
            .unwrap_or("unknown")
            .to_string()
    } else {
        String::new()
    };

    Ok(json!({
        "success": true,
        "running": is_running,
        "pid": if is_running { &pid } else { "" },
        "status": if is_running { "running" } else { "stopped" },
        "output": stdout.lines().take(5).collect::<Vec<_>>().join("\n"),
        "error": if !output.status.success() { stderr.to_string() } else { String::new() },
    }))
}

/// Start the gateway.
#[tauri::command(rename_all = "camelCase")]
pub fn gateway_start() -> Result<serde_json::Value, String> {
    let output = Command::new("hermes")
        .args(["gateway", "start"])
        .output()
        .map_err(|e| format!("执行 hermes gateway start 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(json!({
        "success": output.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}

/// Stop the gateway.
#[tauri::command(rename_all = "camelCase")]
pub fn gateway_stop() -> Result<serde_json::Value, String> {
    let output = Command::new("hermes")
        .args(["gateway", "stop"])
        .output()
        .map_err(|e| format!("执行 hermes gateway stop 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(json!({
        "success": output.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}

/// Restart the gateway.
#[tauri::command(rename_all = "camelCase")]
pub fn gateway_restart() -> Result<serde_json::Value, String> {
    let output = Command::new("hermes")
        .args(["gateway", "restart"])
        .output()
        .map_err(|e| format!("执行 hermes gateway restart 失败: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(json!({
        "success": output.status.success(),
        "output": stdout.to_string(),
        "error": stderr.to_string(),
    }))
}
