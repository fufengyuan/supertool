//! Hermes Profile Management for Multi-Agent Orchestration

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Run command through user's login shell to inherit full environment (PATH, etc.)
fn run_with_user_env(cmd: &str, args: &[&str]) -> Result<String, String> {
    // Build the full command string
    let full_cmd = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{} {}", cmd, args.join(" "))
    };

    // Use login shell (-l) to load user's full environment including PATH
    let output = Command::new("/bin/bash")
        .args(["-l", "-c", &full_cmd])
        .output()
        .map_err(|e| format!("Failed to run command via shell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run hermes profile CLI through user's shell environment
fn run_profile_cmd(args: &[String]) -> Result<String, String> {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_with_user_env("hermes profile", &args_str)
}

/// Run hermes kanban CLI through user's shell environment
fn run_kanban_cmd(args: &[String]) -> Result<String, String> {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_with_user_env("hermes kanban", &args_str)
}

/// Profile info
#[derive(Debug, Serialize, Deserialize)]
pub struct HermesProfile {
    pub name: String,
    pub model: Option<String>,
    pub gateway_status: Option<String>,
    pub alias: Option<String>,
    pub distribution: Option<String>,
    pub description: Option<String>,
    pub is_default: bool,
}

/// Parse profile list output into structured data
fn parse_profile_list(output: String) -> Vec<HermesProfile> {
    let mut profiles: Vec<HermesProfile> = vec![];

    // Parse tabular output
    for line in output.lines().skip(2) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].replace('◆', "").trim().to_string();
            let is_default = parts[0].contains('◆');
            let model = if parts.len() > 1 && parts[1] != "—" {
                Some(parts[1].to_string())
            } else {
                None
            };
            let gateway_status = if parts.len() > 2 && parts[2] != "—" {
                Some(parts[2].to_string())
            } else {
                None
            };
            let alias = if parts.len() > 3 && parts[3] != "—" {
                Some(parts[3].to_string())
            } else {
                None
            };
            let distribution = if parts.len() > 4 && parts[4] != "—" {
                Some(parts[4].to_string())
            } else {
                None
            };

            profiles.push(HermesProfile {
                name,
                model,
                gateway_status,
                alias,
                distribution,
                description: None,
                is_default,
            });
        }
    }

    profiles
}

/// List all profiles
#[tauri::command(rename_all = "camelCase")]
pub fn profile_list() -> Result<Vec<HermesProfile>, String> {
    let output = run_profile_cmd(&["list".into()])?;
    Ok(parse_profile_list(output))
}

/// Get profile details
#[tauri::command(rename_all = "camelCase")]
pub fn profile_show(name: String) -> Result<serde_json::Value, String> {
    // Run profile show and parse the output
    let output = run_profile_cmd(&["show".into(), name.clone()])?;

    // Parse into structured format
    let mut result = serde_json::Map::new();
    result.insert("name".into(), serde_json::Value::String(name));
    result.insert(
        "raw_output".into(),
        serde_json::Value::String(output.clone()),
    );

    // Parse key-value pairs from output
    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase().replace(' ', "_");
            let value = value.trim();
            result.insert(key, serde_json::Value::String(value.to_string()));
        }
    }

    Ok(serde_json::Value::Object(result))
}

/// Create a new profile
#[tauri::command(rename_all = "camelCase")]
pub fn profile_create(name: String, description: Option<String>) -> Result<(), String> {
    let mut args: Vec<String> = vec!["create".into(), name];
    if let Some(d) = description {
        args.push("--description".into());
        args.push(d);
    }
    run_profile_cmd(&args)?;
    Ok(())
}

/// Delete a profile
#[tauri::command(rename_all = "camelCase")]
pub fn profile_delete(name: String) -> Result<(), String> {
    run_profile_cmd(&["delete".into(), name])?;
    Ok(())
}

/// Set default profile
#[tauri::command(rename_all = "camelCase")]
pub fn profile_use(name: String) -> Result<(), String> {
    run_profile_cmd(&["use".into(), name])?;
    Ok(())
}

/// Set profile description (used by kanban orchestrator for routing)
#[tauri::command(rename_all = "camelCase")]
pub fn profile_describe(name: String, description: String) -> Result<(), String> {
    run_profile_cmd(&["describe".into(), name, "--text".into(), description])?;
    Ok(())
}

/// Get profile description
#[tauri::command(rename_all = "camelCase")]
pub fn profile_get_description(name: String) -> Result<String, String> {
    run_profile_cmd(&["describe".into(), name])
}

/// Set profile model
#[tauri::command(rename_all = "camelCase")]
pub fn profile_set_model(name: String, model: String) -> Result<(), String> {
    let cmd = format!("hermes --profile {} config set model {}", name, model);
    run_with_user_env(&cmd, &[])?;
    Ok(())
}

/// Install a profile distribution from git URL
#[tauri::command(rename_all = "camelCase")]
pub fn profile_install(name: String, source: String) -> Result<(), String> {
    run_profile_cmd(&["install".into(), source, "--profile".into(), name])?;
    Ok(())
}

/// Update a profile distribution
#[tauri::command(rename_all = "camelCase")]
pub fn profile_update(name: String) -> Result<(), String> {
    run_profile_cmd(&["update".into(), name])?;
    Ok(())
}

/// Trigger a dispatch pass (reclaim stale, promote ready, spawn workers)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_dispatch(
    dry_run: Option<bool>,
    max_spawns: Option<u32>,
) -> Result<serde_json::Value, String> {
    let mut args: Vec<String> = vec!["dispatch".into(), "--json".into()];
    if dry_run == Some(true) {
        args.push("--dry-run".into());
    }
    if let Some(m) = max_spawns {
        args.push("--max".into());
        args.push(m.to_string());
    }

    let stdout = run_kanban_cmd(&args)?;
    if stdout.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }

    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse dispatch output: {}", e))
}

/// Check dispatcher status (running in gateway or daemon)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_dispatcher_status() -> Result<serde_json::Value, String> {
    let stdout = run_with_user_env("hermes gateway status", &[])?;
    
    // Parse plain text output - check for "running" indicators
    let running = stdout.contains("Active: active (running)") 
        || stdout.contains("✓ User gateway service is running");
    
    Ok(serde_json::json!({
        "running": running,
        "raw_output": stdout.lines().take(5).collect::<Vec<_>>().join("\n")
    }))
}

/// Get current assignee workload (tasks per profile)
#[tauri::command(rename_all = "camelCase")]
pub fn kanban_workload() -> Result<Vec<serde_json::Value>, String> {
    let stdout = run_kanban_cmd(&["assignees".into(), "--json".into()])?;
    if stdout.trim().is_empty() {
        return Ok(vec![]);
    }

    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse assignees: {}", e))
}