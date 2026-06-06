//! Hermes Gateway management commands — using hermes-config paths.
//!
//! Status reads gateway PID file; start/stop/restart spawn the CLI binary.

use hermes_config::paths;

fn run_hermes(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("hermes")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run hermes: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let err_line = stderr.lines().next().unwrap_or(&stderr);
        return Err(format!("hermes failed: {err_line}"));
    }

    Ok(stdout)
}

/// Read gateway PID file to check if running.
fn is_gateway_running() -> bool {
    let pid_path = paths::gateway_pid_path();
    if !pid_path.exists() {
        return false;
    }
    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return false,
    };
    let pid: i32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => return false,
    };
    // Check if process exists
    let output = std::process::Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .output();
    matches!(output, Ok(o) if o.status.success())
}

#[tauri::command(rename_all = "camelCase")]
pub fn gateway_status() -> Result<serde_json::Value, String> {
    let running = is_gateway_running();

    let pid_path = paths::gateway_pid_path();
    let pid = if running {
        std::fs::read_to_string(&pid_path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    Ok(serde_json::json!({
        "success": true,
        "running": running,
        "pid": pid,
        "status": if running { "running" } else { "stopped" },
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn gateway_start() -> Result<serde_json::Value, String> {
    let output = run_hermes(&["gateway", "start"])?;
    Ok(serde_json::json!({
        "success": true,
        "output": output,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn gateway_stop() -> Result<serde_json::Value, String> {
    let output = run_hermes(&["gateway", "stop"])?;
    Ok(serde_json::json!({
        "success": true,
        "output": output,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn gateway_restart() -> Result<serde_json::Value, String> {
    let output = run_hermes(&["gateway", "restart"])?;
    Ok(serde_json::json!({
        "success": true,
        "output": output,
    }))
}
