use crate::output::{print_error, print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};

pub async fn cmd_mfa(runtime: &mut CliRuntime, action: &crate::types::MfaCommands) -> Result<()> {
    use crate::types::MfaCommands;
    match action {
        MfaCommands::List { json } => {
            let result = runtime
                .core
                .get_all_mfa_secrets()
                .await
                .map_err(|e| anyhow!(e))?;
            if *json || runtime.json_mode {
                print_json(&result);
            } else {
                print_mfa_list(&result);
            }
        }
        MfaCommands::Add {
            name,
            secret,
            issuer,
            digits,
            period,
            algorithm,
        } => {
            let data = serde_json::json!({
                "name": name, "secret": secret,
                "issuer": issuer, "digits": digits,
                "period": period, "algorithm": algorithm,
            });
            let result = runtime
                .core
                .add_mfa_secret(data)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id, name} on success
            if result.get("id").is_some() {
                print_success(&format!("MFA 密钥已添加: {}", name));
                if let Some(id) = result.get("id") {
                    println!("ID: {}", id);
                }
            } else {
                print_error(&format!("添加失败: {}", result));
            }
        }
        MfaCommands::Delete { id } => {
            let result = runtime
                .core
                .delete_mfa_secret(id)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success("MFA 密钥已删除");
            } else {
                print_error(&format!("删除失败: {}", result));
            }
        }
        MfaCommands::Code { identifier, json } => {
            runtime.set_json(*json);
            let secrets = runtime
                .core
                .get_all_mfa_secrets()
                .await
                .map_err(|e| anyhow!(e))?;
            let target = if let Some(arr) = secrets.as_array() {
                if let Ok(idx) = identifier.parse::<usize>() {
                    if idx > 0 && idx <= arr.len() {
                        Some(arr[idx - 1].clone())
                    } else {
                        None
                    }
                } else {
                    arr.iter()
                        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(identifier.as_str()))
                        .cloned()
                }
            } else {
                None
            };

            match target {
                Some(s) => {
                    let secret = s.get("secret").and_then(|v| v.as_str()).unwrap_or("");
                    let digits = s.get("digits").and_then(|v| v.as_u64()).unwrap_or(6) as u32;
                    let period = s.get("period").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
                    let algorithm = s
                        .get("algorithm")
                        .and_then(|v| v.as_str())
                        .unwrap_or("SHA1");
                    let result = runtime
                        .core
                        .generate_totp(secret, digits, period, algorithm)
                        .await
                        .map_err(|e| anyhow!(e))?;
                    let code = result.get("code").and_then(|v| v.as_str()).unwrap_or("?");
                    let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let remaining = result
                        .get("remaining")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if runtime.json_mode {
                        print_json(&serde_json::json!({
                            "name": name,
                            "code": code,
                            "remainingSeconds": remaining,
                            "id": s.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        }));
                        return Ok(());
                    }
                    println!(
                        "[1;36m{}[0m  [1;32m{}[0m  ({}s 后过期)",
                        name, code, remaining
                    );
                }
                None => print_error(&format!("未找到 MFA 密钥: {}", identifier)),
            }
        }
        MfaCommands::ParseUri { uri, json } => {
            runtime.set_json(*json);
            if !uri.starts_with("otpauth://") {
                print_error("Invalid URI: must start with otpauth://");
                return Ok(());
            }
            let rest = uri.strip_prefix("otpauth://").unwrap();
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            if parts.len() < 2 {
                print_error("Invalid URI format");
                return Ok(());
            }
            let auth_type = parts[0];
            let (path, query_string) = if let Some(idx) = parts[1].find('?') {
                (&parts[1][..idx], &parts[1][idx + 1..])
            } else {
                (parts[1], "")
            };
            let (issuer_from_path, name) = if let Some(colon_idx) = path.find(':') {
                (&path[..colon_idx], &path[colon_idx + 1..])
            } else {
                ("", path)
            };
            let mut secret = String::new();
            let mut issuer = issuer_from_path.to_string();
            for param in query_string.split('&') {
                if let Some(eq_idx) = param.find('=') {
                    let key = &param[..eq_idx];
                    let value = &param[eq_idx + 1..];
                    if key == "secret" {
                        secret = value.to_uppercase();
                    } else if key == "issuer" {
                        issuer = value.to_string();
                    }
                }
            }
            if runtime.json_mode {
                print_json(&serde_json::json!({
                    "type": auth_type,
                    "name": name,
                    "issuer": issuer,
                    "secret": secret,
                }));
                return Ok(());
            }
            println!("类型: {}", auth_type);
            println!("名称: {}", name);
            println!("Issuer: {}", issuer);
            println!("Secret: {}", secret);
        }
    }
    Ok(())
}

fn print_mfa_list(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无 MFA 密钥");
            return;
        }
        println!(
            "[1;36m{:<4} {:<20} {:<15} {:<8} {:<10}[0m",
            "#", "名称", "Issuer", "位数", "周期"
        );
        for (i, s) in arr.iter().enumerate() {
            let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let issuer = s.get("issuer").and_then(|v| v.as_str()).unwrap_or("");
            let digits = s.get("digits").and_then(|v| v.as_u64()).unwrap_or(6);
            let period = s.get("period").and_then(|v| v.as_u64()).unwrap_or(30);
            println!(
                "{:<4} {:<20} {:<15} {:<8} {:<10}s",
                i + 1,
                name,
                issuer,
                digits,
                period
            );
        }
    }
}
