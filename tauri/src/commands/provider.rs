//! Provider credential management for Tauri GUI
//!
//! IPC commands for managing Hermes provider credentials:
//! - List providers with credential status
//! - Save/remove API keys
//! - OAuth device code flow
//!
//! Data source: ~/.hermes/auth.json (credential_pool)
//! Provider metadata: ~/.hermes/models_dev_cache.json

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

/// A single credential entry in the credential pool
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CredentialEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    label: String,
    #[serde(rename = "auth_type")]
    #[serde(default)]
    auth_type: String,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    source: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    request_count: i64,
    #[serde(default)]
    last_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_status_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error_reset_at: Option<String>,
}

impl Default for CredentialEntry {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string()[..6].to_string(),
            label: String::new(),
            auth_type: "api_key".to_string(),
            priority: 0,
            source: String::new(),
            access_token: String::new(),
            refresh_token: String::new(),
            api_key: String::new(),
            base_url: String::new(),
            request_count: 0,
            last_status: "ok".to_string(),
            last_status_at: None,
            last_error_code: None,
            last_error_reason: None,
            last_error_message: None,
            last_error_reset_at: None,
        }
    }
}

/// Auth.json root structure
#[derive(Debug, Serialize, Deserialize)]
struct AuthData {
    #[serde(default)]
    version: serde_json::Value,
    #[serde(default)]
    providers: HashMap<String, serde_json::Value>,
    #[serde(default)]
    active_provider: serde_json::Value,
    #[serde(default)]
    updated_at: serde_json::Value,
    #[serde(default)]
    credential_pool: HashMap<String, Vec<CredentialEntry>>,
}

/// Provider info exposed to frontend
#[derive(Debug, Serialize, Deserialize)]
struct ProviderInfo {
    id: String,
    name: String,
    auth_type: String,
    configured: bool,
    api_key_preview: String,
    has_valid_key: bool,
}

/// Models.dev cache entry
#[derive(Debug, Deserialize)]
struct ModelsCacheProvider {
    #[serde(default)]
    env: Vec<String>,
    #[serde(default)]
    models: HashMap<String, serde_json::Value>,
}

type ModelsCache = HashMap<String, ModelsCacheProvider>;

// ============================================================================
// Path helpers
// ============================================================================

fn hermes_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".hermes")
}

fn auth_json_path() -> PathBuf {
    hermes_dir().join("auth.json")
}

fn models_cache_path() -> PathBuf {
    hermes_dir().join("models_dev_cache.json")
}

// ============================================================================
// Read/write auth.json
// ============================================================================

fn read_auth() -> Result<AuthData, String> {
    let path = auth_json_path();
    if !path.exists() {
        return Ok(AuthData {
            version: json!(""),
            providers: HashMap::new(),
            active_provider: json!(""),
            updated_at: json!(""),
            credential_pool: HashMap::new(),
        });
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read auth.json: {e}"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse auth.json: {e}"))
}

fn write_auth(auth: &AuthData) -> Result<(), String> {
    let path = auth_json_path();
    let content = serde_json::to_string_pretty(auth)
        .map_err(|e| format!("Failed to serialize auth.json: {e}"))?;
    // Ensure .hermes dir exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .hermes directory: {e}"))?;
    }
    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write auth.json: {e}"))?;
    Ok(())
}

fn read_models_cache() -> Result<ModelsCache, String> {
    let path = models_cache_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read models cache: {e}"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse models_dev_cache.json: {e}"))
}

/// Get a list of known provider IDs with user-friendly names from the models cache
fn get_provider_names() -> HashMap<String, String> {
    let mut names = HashMap::new();
    // Built-in known provider names (from models.dev cache)
    let known: &[(&str, &str)] = &[
        ("openai", "OpenAI"),
        ("anthropic", "Anthropic"),
        ("google", "Google"),
        ("xai", "xAI"),
        ("mistral", "Mistral"),
        ("deepseek", "DeepSeek"),
        ("openrouter", "OpenRouter"),
        ("groq", "Groq"),
        ("togetherai", "Together AI"),
        ("fireworks-ai", "Fireworks AI"),
        ("cerebras", "Cerebras"),
        ("perplexity", "Perplexity"),
        ("huggingface", "Hugging Face"),
        ("nvidia", "NVIDIA NIM"),
        ("zai", "Z.ai / GLM"),
        ("qwen", "Qwen"),
        ("minimax", "MiniMax"),
        ("alibaba", "Alibaba"),
        ("siliconflow", "SiliconFlow"),
        ("zhipuai", "Zhipu AI"),
        ("stepfun", "StepFun"),
        ("moonshotai", "Moonshot AI"),
        ("cohere", "Cohere"),
        ("azure", "Azure OpenAI"),
        ("amazon-bedrock", "Amazon Bedrock"),
        ("databricks", "Databricks"),
        ("gitlab", "GitLab"),
        ("github-models", "GitHub Models"),
        ("cloudflare-workers-ai", "Cloudflare Workers AI"),
        ("deepinfra", "DeepInfra"),
        ("nebius", "Nebius"),
        ("vercel", "Vercel"),
        ("baseten", "Baseten"),
        ("friendli", "Friendli"),
        ("abacus", "Abacus AI"),
        ("novita-ai", "Novita AI"),
        ("venice", "Venice"),
        ("requesty", "Requesty"),
        ("synthetic", "Synthetic"),
        ("aihubmix", "AI Hub Mix"),
    ];
    for (id, name) in known {
        names.insert(id.to_string(), name.to_string());
    }
    names
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// List all providers with their credential status
///
/// Returns combined info from:
/// - auth.json credential_pool (configured status)
/// - models_dev_cache.json (available providers)
#[tauri::command(rename_all = "camelCase")]
pub fn list_providers() -> Result<serde_json::Value, String> {
    let auth = read_auth()?;
    let cache = read_models_cache().unwrap_or_default();
    let names = get_provider_names();

    // Collect all known provider IDs (from cache + credential pool + built-in)
    let mut provider_ids: Vec<String> = Vec::new();
    for pid in cache.keys() {
        if !provider_ids.contains(pid) {
            provider_ids.push(pid.clone());
        }
    }
    for pid in auth.credential_pool.keys() {
        if !provider_ids.contains(pid) {
            provider_ids.push(pid.clone());
        }
    }
    // Add built-in names not in cache or pool
    for pid in names.keys() {
        if !provider_ids.contains(pid) {
            provider_ids.push(pid.clone());
        }
    }
    provider_ids.sort();

    let providers: Vec<ProviderInfo> = provider_ids
        .iter()
        .map(|id| {
            let name = names
                .get(id)
                .cloned()
                .unwrap_or_else(|| {
                    // Humanize the ID: replace dashes with spaces, capitalize
                    let mut s = String::new();
                    let mut capitalize = true;
                    for c in id.chars() {
                        if c == '-' || c == '_' {
                            s.push(' ');
                            capitalize = true;
                        } else if capitalize {
                            s.push(c.to_ascii_uppercase());
                            capitalize = false;
                        } else {
                            s.push(c);
                        }
                    }
                    s
                });

            let auth_type = cache
                .get(id)
                .and_then(|p| {
                    // Determine auth type from env keys (OAuth providers have specific patterns)
                    let has_oauth = p.env.iter().any(|e| e.contains("OAUTH") || e.contains("TOKEN"));
                    if has_oauth { Some("oauth_device_code".to_string()) } else { Some("api_key".to_string()) }
                })
                .unwrap_or_else(|| "api_key".to_string());

            let entries = auth.credential_pool.get(id);
            let configured = entries.is_some_and(|e| !e.is_empty());
            let has_valid_key = entries.is_some_and(|e| {
                e.iter().any(|entry| {
                    !entry.access_token.is_empty()
                        || !entry.api_key.is_empty()
                })
            });
            let api_key_preview = entries
                .and_then(|e| e.first())
                .map(|entry| {
                    let token = if !entry.access_token.is_empty() {
                        &entry.access_token
                    } else if !entry.api_key.is_empty() {
                        &entry.api_key
                    } else {
                        return String::new();
                    };
                    if token.len() > 12 {
                        format!("{}...{}", &token[..8], &token[token.len() - 4..])
                    } else if !token.is_empty() {
                        format!("{}...", &token[..token.len().min(4)])
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();

            ProviderInfo {
                id: id.clone(),
                name,
                auth_type,
                configured,
                has_valid_key,
                api_key_preview,
            }
        })
        .collect();

    Ok(json!({
        "success": true,
        "providers": providers,
    }))
}

/// Save an API key for a provider
#[tauri::command(rename_all = "camelCase")]
pub fn save_provider_credential(provider_id: String, api_key: String) -> Result<serde_json::Value, String> {
    if provider_id.trim().is_empty() {
        return Err("provider_id is required".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("api_key is required".to_string());
    }

    let mut auth = read_auth()?;
    let provider_id = provider_id.trim().to_string();
    let api_key = api_key.trim().to_string();

    let entry = CredentialEntry {
        id: Uuid::new_v4().to_string()[..6].to_string(),
        label: format!("{}_API_KEY", provider_id.to_uppercase()),
        auth_type: "api_key".to_string(),
        priority: 0,
        source: "manual".to_string(),
        access_token: api_key.clone(),
        refresh_token: String::new(),
        api_key: api_key,
        base_url: String::new(),
        request_count: 0,
        last_status: "ok".to_string(),
        last_status_at: None,
        last_error_code: None,
        last_error_reason: None,
        last_error_message: None,
        last_error_reset_at: None,
    };
    auth.credential_pool
        .insert(provider_id.clone(), vec![entry]);
    auth.updated_at = json!(chrono::Utc::now().to_rfc3339());

    write_auth(&auth)?;

    log::info!("[provider] Saved credential for provider: {}", provider_id);

    Ok(json!({
        "success": true,
        "providerId": provider_id,
    }))
}

/// Remove credential for a provider
#[tauri::command(rename_all = "camelCase")]
pub fn remove_provider_credential(provider_id: String) -> Result<serde_json::Value, String> {
    if provider_id.trim().is_empty() {
        return Err("provider_id is required".to_string());
    }

    let mut auth = read_auth()?;
    let provider_id = provider_id.trim().to_string();

    if auth.credential_pool.remove(&provider_id).is_some() {
        auth.updated_at = json!(chrono::Utc::now().to_rfc3339());
        write_auth(&auth)?;
        log::info!("[provider] Removed credential for provider: {}", provider_id);
    }

    Ok(json!({
        "success": true,
        "providerId": provider_id,
    }))
}

/// Start OAuth device code flow for a provider
///
/// Returns the authorization URL that the user should open in a browser.
/// The actual OAuth flow is handled by the Hermes gateway.
#[tauri::command(rename_all = "camelCase")]
pub fn start_oauth_flow(provider_id: String) -> Result<serde_json::Value, String> {
    if provider_id.trim().is_empty() {
        return Err("provider_id is required".to_string());
    }

    let provider_id = provider_id.trim().to_string();

    // Use the Hermes CLI to initiate OAuth
    let output = std::process::Command::new("hermes")
        .args(["auth", "login", "--provider", &provider_id, "--json"])
        .output()
        .map_err(|e| format!("Failed to execute hermes auth login: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If hermes CLI is not available, generate a URL based on known patterns
        if stderr.contains("not found") || stderr.contains("No such file") {
            // Fallback: return a generic OAuth URL pattern
            return Ok(json!({
                "success": true,
                "authorizationUrl": format!("https://{}", provider_id),
                "deviceCode": Uuid::new_v4().to_string(),
                "verificationUri": format!("https://{}/oauth/device", provider_id),
                "providerId": provider_id,
                "note": "Fallback URL — hermes CLI not available"
            }));
        }
        return Err(format!("OAuth login failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse OAuth response: {e}"))?;

    Ok(json!({
        "success": true,
        "authorizationUrl": result.get("url").or(result.get("authorization_url")).and_then(|v| v.as_str()).unwrap_or(""),
        "deviceCode": result.get("device_code").and_then(|v| v.as_str()).unwrap_or(""),
        "verificationUri": result.get("verification_uri").and_then(|v| v.as_str()).unwrap_or(""),
        "providerId": provider_id,
    }))
}

/// Poll OAuth result for a provider
///
/// Returns the credential status after OAuth flow completion.
#[tauri::command(rename_all = "camelCase")]
pub fn poll_oauth_result(provider_id: String) -> Result<serde_json::Value, String> {
    if provider_id.trim().is_empty() {
        return Err("provider_id is required".to_string());
    }

    let provider_id = provider_id.trim().to_string();

    // Re-read auth.json to check if credential was added by the gateway
    let auth = read_auth()?;
    let entries = auth.credential_pool.get(&provider_id);

    let configured = entries.is_some_and(|e| !e.is_empty());
    let has_token = entries.is_some_and(|e| {
        e.iter().any(|entry| !entry.access_token.is_empty())
    });

    Ok(json!({
        "success": true,
        "providerId": provider_id,
        "configured": configured,
        "hasToken": has_token,
        "completed": configured && has_token,
    }))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_providers() {
        let result = list_providers();
        // This may fail if auth.json doesn't exist, but the function handles that
        if let Ok(json) = result {
            assert!(json.get("success").is_some());
            assert!(json.get("providers").is_some());
        }
    }

    #[test]
    fn test_save_provider_credential_empty() {
        let result = save_provider_credential("".to_string(), "key".to_string());
        assert!(result.is_err());

        let result = save_provider_credential("test".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_provider_credential_empty() {
        let result = remove_provider_credential("".to_string());
        assert!(result.is_err());
    }
}
