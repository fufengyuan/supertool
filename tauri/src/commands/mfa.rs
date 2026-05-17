use supertool_core::logic::CoreService;
use tauri::State;

/// Parse an otpauth:// URI and extract TOTP parameters
#[tauri::command(rename_all = "camelCase")]
pub fn mfa_parse_uri(uri: String) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] mfa_parse_uri() called");
    // Expected format: otpauth://totp/ISSUER:NAME?secret=XXX&issuer=ISSUER&digits=6&period=30&algorithm=SHA1
    if !uri.starts_with("otpauth://") {
        return Err("Invalid URI: must start with otpauth://".to_string());
    }

    // Parse type (totp/hotp)
    let rest = uri.strip_prefix("otpauth://").unwrap();
    let parts: Vec<&str> = rest.splitn(2, '/').collect();
    if parts.len() < 2 {
        return Err("Invalid URI format".to_string());
    }

    let auth_type = parts[0]; // "totp" or "hotp"
    let path_and_query = parts[1];

    // Split path from query string
    let (path, query_string) = if let Some(idx) = path_and_query.find('?') {
        (&path_and_query[..idx], &path_and_query[idx + 1..])
    } else {
        (path_and_query, "")
    };

    // Parse the name part (may contain ISSUER:NAME or just NAME)
    let (issuer_from_path, name) = if let Some(colon_idx) = path.find(':') {
        (
            url_decode(&path[..colon_idx]),
            url_decode(&path[colon_idx + 1..]),
        )
    } else {
        (String::new(), url_decode(path))
    };

    // Parse query parameters
    let mut secret = String::new();
    let mut issuer = issuer_from_path.clone();
    let mut digits: u32 = 6;
    let mut period: u32 = 30;
    let mut algorithm = String::from("SHA1");

    for param in query_string.split('&') {
        if let Some(eq_idx) = param.find('=') {
            let key = &param[..eq_idx];
            let value = url_decode(&param[eq_idx + 1..]);
            match key {
                "secret" => secret = value.to_uppercase(),
                "issuer" => issuer = value,
                "digits" => digits = value.parse().unwrap_or(6),
                "period" => period = value.parse().unwrap_or(30),
                "algorithm" => algorithm = value,
                _ => {}
            }
        }
    }

    if secret.is_empty() {
        return Err("Missing secret in URI".to_string());
    }

    Ok(serde_json::json!({
        "success": true,
        "data": {
            "type": auth_type,
            "secret": secret,
            "name": name,
            "issuer": issuer,
            "digits": digits,
            "period": period,
            "algorithm": algorithm.to_uppercase()
        }
    }))
}

/// URL-decode a string (handles %XX encoding)
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h = chars.next().unwrap_or(b'0');
            let l = chars.next().unwrap_or(b'0');
            let hex = format!("{}{}", h as char, l as char);
            if let Ok(val) = u8::from_str_radix(&hex, 16) {
                result.push(val as char);
            } else {
                result.push('%');
                result.push(h as char);
                result.push(l as char);
            }
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }
    result
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_mfa_secrets(
    core: State<'_, CoreService>,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] get_all_mfa_secrets() called");
    let result = core.get_all_mfa_secrets().await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_mfa_secret(
    core: State<'_, CoreService>,
    secret: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] add_mfa_secret() called");
    let result = core.add_mfa_secret(secret).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_mfa_secret(
    core: State<'_, CoreService>,
    id: String,
    updates: serde_json::Value,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] update_mfa_secret() called");
    let result = core.update_mfa_secret(&id, updates).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_mfa_secret(
    core: State<'_, CoreService>,
    id: String,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] delete_mfa_secret() called");
    let result = core.delete_mfa_secret(&id).await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn generate_totp(
    core: State<'_, CoreService>,
    secret: String,
    digits: u32,
    period: u32,
    algorithm: String,
) -> Result<serde_json::Value, String> {
    let result = core
        .generate_totp(&secret, digits, period, &algorithm)
        .await?;
    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}
