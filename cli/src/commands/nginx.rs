use crate::output::{print_error, print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};

pub async fn cmd_nginx(
    runtime: &mut CliRuntime,
    action: &crate::types::NginxCommands,
) -> Result<()> {
    use crate::types::NginxCommands;
    match action {
        NginxCommands::List { json } => {
            let resp = runtime
                .core
                .get_all_nginx_presets()
                .await
                .map_err(|e| anyhow!(e))?;
            let data = serde_json::to_value(&resp.data.unwrap_or_default()).unwrap_or_default();
            if *json || runtime.json_mode {
                print_json(&data);
            } else {
                print_nginx_list(&data);
            }
        }
        NginxCommands::Add {
            name,
            server_id,
            config_path,
            content,
        } => {
            let now = chrono::Utc::now().to_rfc3339();
            let preset = supertool_core::db::nginx::NginxPreset {
                id: String::new(),
                name: name.clone(),
                server_id: server_id.clone().unwrap_or_default(),
                config_path: config_path.clone().unwrap_or_default(),
                description: content.clone().unwrap_or_default(),
                group_name: String::new(),
                is_active: false,
                created_at: now.clone(),
                updated_at: now,
            };
            let resp = runtime
                .core
                .add_nginx_preset(preset)
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                print_success(&format!("Nginx 预设已添加: {}", name));
            } else {
                print_error(&format!("添加失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Update {
            id,
            name,
            server_id,
            config_path,
        } => {
            let resp = runtime
                .core
                .update_nginx_preset(supertool_core::db::nginx::NginxPreset {
                    id: id.clone(),
                    name: name.clone().unwrap_or_default(),
                    server_id: server_id.clone().unwrap_or_default(),
                    config_path: config_path.clone().unwrap_or_default(),
                    description: String::new(),
                    group_name: String::new(),
                    is_active: false,
                    created_at: String::new(),
                    updated_at: String::new(),
                })
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                print_success("Nginx 预设已更新");
            } else {
                print_error(&format!("更新失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Delete { id } => {
            let resp = runtime
                .core
                .delete_nginx_preset(id)
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                print_success("Nginx 预设已删除");
            } else {
                print_error(&format!("删除失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Fetch {
            server_id,
            config_path,
            json,
        } => {
            runtime.set_json(*json);
            let resp = runtime
                .core
                .fetch_nginx_config(server_id, config_path)
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                if runtime.json_mode {
                    print_json(&serde_json::json!({
                        "ok": true,
                        "serverId": server_id,
                        "configPath": config_path,
                        "content": resp.data.unwrap_or_default(),
                    }));
                } else if let Some(content) = resp.data {
                    println!("{}", content);
                }
            } else {
                print_error(&format!("拉取失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Test {
            server_id,
            config_path,
            json,
        } => {
            runtime.set_json(*json);
            let resp = runtime
                .core
                .test_nginx_config(server_id, config_path)
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                if runtime.json_mode {
                    print_json(&serde_json::json!({"ok": true, "serverId": server_id, "configPath": config_path}));
                } else {
                    print_success("Nginx 配置测试通过");
                }
            } else {
                print_error(&format!("测试失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Deploy {
            server_id,
            config_path,
            content,
            json,
        } => {
            runtime.set_json(*json);
            let resp = runtime
                .core
                .deploy_nginx_config(server_id, config_path, content)
                .await
                .map_err(|e| anyhow!(e))?;
            if resp.success {
                if runtime.json_mode {
                    print_json(&serde_json::json!({"ok": true, "serverId": server_id, "configPath": config_path}));
                } else {
                    print_success("Nginx 配置已部署");
                }
            } else {
                print_error(&format!("部署失败: {}", resp.error.unwrap_or_default()));
            }
        }
        NginxCommands::Versions { preset_id, json } => {
            let resp = runtime
                .core
                .get_nginx_config_versions(preset_id)
                .await
                .map_err(|e| anyhow!(e))?;
            let data = serde_json::to_value(&resp.data.unwrap_or_default()).unwrap_or_default();
            if *json || runtime.json_mode {
                print_json(&data);
            } else {
                print_nginx_versions(&data);
            }
        }
    }
    Ok(())
}

fn print_nginx_list(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无 Nginx 配置预设");
            return;
        }
        println!(
            "\x1b[1;36m{:<4} {:<20} {:<15} {}\x1b[0m",
            "#", "名称", "服务器", "配置路径"
        );
        for (i, p) in arr.iter().enumerate() {
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let server = p.get("serverId").and_then(|v| v.as_str()).unwrap_or("-");
            let path = p.get("configPath").and_then(|v| v.as_str()).unwrap_or("-");
            println!("{:<4} {:<20} {:<15} {}", i + 1, name, server, path);
        }
    }
}

fn print_nginx_versions(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无版本历史");
            return;
        }
        for (i, v) in arr.iter().enumerate() {
            let version = v.get("version").and_then(|v| v.as_str()).unwrap_or("");
            let date = v.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
            let is_active = v
                .get("isCurrent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let marker = if is_active {
                " \x1b[32m[当前]\x1b[0m"
            } else {
                ""
            };
            println!("{}  {}  {}{}", i + 1, version, date, marker);
        }
    }
}
