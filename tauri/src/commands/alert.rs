use chrono::Timelike;
use lettre::AsyncTransport;
use supertool_core::db::alert::{AlertEmailConfig, AlertHistory, AlertResource, AlertService};
use supertool_core::encryption::{encrypt_password, try_decrypt_password};
use supertool_core::logic::CoreService;
use tauri::{Emitter, Manager, State};
use uuid::Uuid;

/// Build SMTP transport based on encryption mode
fn build_smtp_transport(
    host: &str,
    port: u16,
    encryption: &str,
    username: &str,
    password: &str,
) -> Result<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>, String> {
    use lettre::transport::smtp::client::Tls;
    use lettre::transport::smtp::client::TlsParameters;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        username.to_string(),
        password.to_string(),
    );
    let tls_params =
        TlsParameters::new(host.to_string()).map_err(|e| format!("创建 TLS 参数失败: {}", e))?;

    // Auto-detect encryption from port: 465=SSL, 587=STARTTLS, 25=none
    // Port takes priority over user config to prevent mismatches
    let effective_enc = match port {
        465 => "ssl",
        587 => "starttls",
        _ => encryption,
    };

    let builder = match effective_enc {
        "ssl" => {
            // Port 465: implicit TLS — TLS handshake happens before any SMTP dialogue
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .tls(Tls::Wrapper(tls_params))
                .timeout(Some(std::time::Duration::from_secs(30)))
                .credentials(creds)
        }
        "starttls" => {
            // Port 587: STARTTLS — plain connection first, upgrades to TLS via STARTTLS command
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(host)
                .map_err(|e| format!("创建 SMTP 传输失败: {}", e))?
                .port(port)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .credentials(creds)
        }
        _ => {
            // No encryption (port 25)
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .tls(Tls::None)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .credentials(creds)
        }
    };
    Ok(builder.build())
}

/// Build email message with multiple recipients (comma-separated)
fn build_email(from: &str, to: &str, subject: &str, body: &str) -> Result<lettre::Message, String> {
    let mut builder = lettre::message::Message::builder().from(
        from.parse()
            .map_err(|e| format!("发件人邮箱格式错误: {}", e))?,
    );
    for addr in to.split(',') {
        let addr = addr.trim();
        if !addr.is_empty() {
            builder = builder.to(addr
                .parse()
                .map_err(|e| format!("收件人邮箱格式错误 '{}': {}", addr, e))?);
        }
    }
    builder
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("构建邮件失败: {}", e))
}

// ==================== Email Sending (Tauri layer) ====================

/// Send an alert email using SMTP config from DB.
async fn send_alert_email(core: &CoreService, subject: &str, body: &str) -> Result<(), String> {
    let config = core
        .get_email_config()
        .await?
        .ok_or_else(|| "邮件配置未设置".to_string())?;

    let host = config
        .smtp_host
        .ok_or_else(|| "SMTP 主机未配置".to_string())?;
    let username = config.smtp_username.unwrap_or_default();
    let from = config
        .from_email
        .ok_or_else(|| "发件人邮箱未配置".to_string())?;
    let to = config
        .to_email
        .ok_or_else(|| "收件人邮箱未配置".to_string())?;
    let port = config.smtp_port as u16;
    let encryption = &config.smtp_encryption;
    let encrypted_pw = config.smtp_password.unwrap_or_default();
    let password = try_decrypt_password(&encrypted_pw);

    let email = build_email(&from, &to, subject, body)?;
    let mailer = build_smtp_transport(&host, port, encryption, &username, &password)?;

    mailer
        .send(email)
        .await
        .map_err(|e| format!("发送邮件失败: {}", e))?;

    log::info!("[Alert] Alert email sent: {}", subject);
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_email_config(
    core: State<'_, CoreService>,
) -> Result<Option<AlertEmailConfig>, String> {
    log::info!("[Tauri CMD] get_email_config() called");
    core.get_email_config().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_email_config(
    _core: State<'_, CoreService>,
    smtp_host: String,
    smtp_port: i64,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    from_email: String,
    to_email: String,
    smtp_encryption: String,
) -> Result<String, String> {
    log::info!("[Tauri CMD] test_email_config() called");

    let pwd = smtp_password.unwrap_or_default();
    let password = if pwd.starts_with("enc:")
        || pwd.starts_with("$argon")
        || (pwd.len() > 20
            && pwd
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='))
    {
        try_decrypt_password(&pwd)
    } else {
        pwd
    };

    let email = build_email(
        &from_email,
        &to_email,
        "SuperTool 告警测试",
        "这是一封测试邮件，来自 SuperTool 告警系统。",
    )?;
    let mailer = build_smtp_transport(
        &smtp_host,
        smtp_port as u16,
        &smtp_encryption,
        &smtp_username.unwrap_or_default(),
        &password,
    )?;

    mailer
        .send(email)
        .await
        .map_err(|e| format!("发送邮件失败: {}", e))?;

    log::info!("[Alert] Test email sent successfully");
    Ok("测试邮件发送成功".to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_email_config(
    core: State<'_, CoreService>,
    smtp_host: Option<String>,
    smtp_port: i64,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    from_email: Option<String>,
    to_email: Option<String>,
    smtp_encryption: String,
) -> Result<(), String> {
    log::info!("[Tauri CMD] save_email_config() called");
    let mut password = smtp_password.unwrap_or_default();
    // Encrypt password if it's plaintext
    // Known encrypted formats: "enc:" prefix (legacy), "$argon" (Argon2), or our AES-256-GCM base64
    // Our encrypted format: base64 of 12-byte nonce + ciphertext, typically 40-100 chars
    // Safe approach: try to decrypt. If it succeeds and differs from input, it was already encrypted.
    if !password.is_empty() && !password.starts_with("enc:") && !password.starts_with("$argon") {
        let decrypted = try_decrypt_password(&password);
        if decrypted == password {
            // Decryption returned same string → it's plaintext → encrypt it
            password = encrypt_password(&password).map_err(|e| e.to_string())?;
        }
        // else: already encrypted, keep as-is
    }
    let config = AlertEmailConfig {
        id: 1,
        smtp_host,
        smtp_port,
        smtp_username,
        smtp_password: Some(password),
        smtp_encryption: smtp_encryption,
        from_email,
        to_email,
        updated_at: String::new(),
    };
    core.save_email_config(config).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_alert_services(core: State<'_, CoreService>) -> Result<Vec<AlertService>, String> {
    log::info!("[Tauri CMD] get_alert_services() called");
    core.get_alert_services().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_alert_service(
    core: State<'_, CoreService>,
    service: AlertService,
) -> Result<(), String> {
    log::info!("[Tauri CMD] add_alert_service() called");
    let mut service = service;
    if service.id.is_empty() {
        service.id = Uuid::new_v4().to_string();
    }
    core.add_alert_service(service).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_alert_service(
    core: State<'_, CoreService>,
    service: AlertService,
) -> Result<(), String> {
    log::info!("[Tauri CMD] update_alert_service() called");
    core.update_alert_service(service).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_alert_service(core: State<'_, CoreService>, id: String) -> Result<(), String> {
    log::info!("[Tauri CMD] delete_alert_service() called");
    core.delete_alert_service(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_alert_resources(
    core: State<'_, CoreService>,
) -> Result<Vec<AlertResource>, String> {
    log::info!("[Tauri CMD] get_alert_resources() called");
    core.get_alert_resources().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_alert_resource(
    core: State<'_, CoreService>,
    resource: AlertResource,
) -> Result<(), String> {
    log::info!("[Tauri CMD] add_alert_resource() called");
    let mut resource = resource;
    if resource.id.is_empty() {
        resource.id = Uuid::new_v4().to_string();
    }
    core.add_alert_resource(resource).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_alert_resource(
    core: State<'_, CoreService>,
    resource: AlertResource,
) -> Result<(), String> {
    log::info!("[Tauri CMD] update_alert_resource() called");
    core.update_alert_resource(resource).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_alert_resource(core: State<'_, CoreService>, id: String) -> Result<(), String> {
    log::info!("[Tauri CMD] delete_alert_resource() called");
    core.delete_alert_resource(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_alert_history(core: State<'_, CoreService>) -> Result<Vec<AlertHistory>, String> {
    log::info!("[Tauri CMD] get_alert_history() called");
    core.get_alert_history().await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn trigger_alert_check(
    core: State<'_, CoreService>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    log::info!("[Tauri CMD] trigger_alert_check() called");
    let results = run_alert_check(&core).await;
    let count = results.len();
    // Emit event to frontend
    if !results.is_empty() {
        let _ = app_handle.emit(
            "alert:notification",
            serde_json::json!({
                "count": count,
                "results": results,
            }),
        );
    }
    Ok(serde_json::json!({
        "checked": true,
        "alerts": count,
    }))
}

// ==================== Background Scheduler ====================

/// Run a single alert check cycle — checks services and resources, sends emails, logs history.
async fn run_alert_check(core: &CoreService) -> Vec<serde_json::Value> {
    let mut results = Vec::new();

    // Check services
    let service_results = core.check_alert_services().await;
    for result in service_results {
        let json_result = serde_json::json!({
            "alertType": result.alert_type,
            "refId": result.ref_id,
            "refName": result.ref_name,
            "host": result.host,
            "port": result.port,
            "category": result.category,
            "message": result.message,
        });

        // Write to alert_history
        let history = AlertHistory {
            id: Uuid::new_v4().to_string(),
            alert_type: result.alert_type.clone(),
            ref_id: result.ref_id.clone(),
            ref_name: result.ref_name.clone(),
            message: result.message.clone(),
            sent_at: String::new(), // will be set by DB default
        };
        if let Err(e) = core.insert_alert_history(history).await {
            log::error!("[Alert] Failed to insert alert history: {}", e);
        }

        // Send email
        let subject = match result.alert_type.as_str() {
            "service_down" => format!("[告警] 服务 {} 无法连接", result.ref_name),
            "service_recovered" => format!("[恢复] 服务 {} 已恢复", result.ref_name),
            _ => format!("[告警] {}", result.ref_name),
        };
        if let Err(e) = send_alert_email(core, &subject, &result.message).await {
            log::error!("[Alert] Failed to send email: {}", e);
        }

        results.push(json_result);
    }

    // Check resources 3x per day (9:00, 13:00, 20:00)
    let hour = chrono::Local::now().hour();
    let should_check_resources = matches!(hour, 9 | 13 | 20);
    let resource_results = if should_check_resources {
        core.check_expiring_resources().await
    } else {
        Vec::new()
    };
    for result in resource_results {
        let json_result = serde_json::json!({
            "alertType": result.alert_type,
            "refId": result.ref_id,
            "refName": result.ref_name,
            "host": result.host,
            "port": result.port,
            "category": result.category,
            "message": result.message,
        });

        // Write to alert_history
        let history = AlertHistory {
            id: Uuid::new_v4().to_string(),
            alert_type: result.alert_type.clone(),
            ref_id: result.ref_id.clone(),
            ref_name: result.ref_name.clone(),
            message: result.message.clone(),
            sent_at: String::new(),
        };
        if let Err(e) = core.insert_alert_history(history).await {
            log::error!("[Alert] Failed to insert alert history: {}", e);
        }

        // Send email
        let subject = format!("[到期告警] {}", result.ref_name);
        if let Err(e) = send_alert_email(core, &subject, &result.message).await {
            log::error!("[Alert] Failed to send email: {}", e);
        }

        results.push(json_result);
    }

    results
}

/// Start the alert background scheduler.
/// Checks every 1 minute for service port issues and resource expiry.
pub fn start_alert_scheduler(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        // Initial check on startup
        {
            if let Some(core) = app_handle.try_state::<CoreService>() {
                let results = run_alert_check(&core).await;
                if !results.is_empty() {
                    log::info!(
                        "[Alert] Initial check completed with {} alert(s)",
                        results.len()
                    );
                    let _ = app_handle.emit(
                        "alert:notification",
                        serde_json::json!({
                            "count": results.len(),
                            "results": results,
                        }),
                    );
                }
            }
        }

        // Periodic check
        loop {
            interval.tick().await;

            if let Some(core) = app_handle.try_state::<CoreService>() {
                let results = run_alert_check(&core).await;
                if !results.is_empty() {
                    log::info!(
                        "[Alert] Periodic check completed with {} alert(s)",
                        results.len()
                    );
                    let _ = app_handle.emit(
                        "alert:notification",
                        serde_json::json!({
                            "count": results.len(),
                            "results": results,
                        }),
                    );
                }
            }
        }
    });

    log::info!("[Alert] Background alert scheduler started (every 60 seconds)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        test::{get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };
    use supertool_core::Database;

    // ── Test app builder ────────────────────────────────────────────────

    fn build_test_app() -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        // Create in-memory database and CoreService for stateful commands
        let db = Database::new(&Path::new(":memory:")).expect("in-memory DB should open");
        let core = CoreService::new(db, std::env::temp_dir());

        let app = mock_builder()
            .manage(core)
            .invoke_handler(tauri::generate_handler![
                crate::commands::alert::get_email_config,
                crate::commands::alert::save_email_config,
                crate::commands::alert::test_email_config,
                crate::commands::alert::get_alert_services,
                crate::commands::alert::add_alert_service,
                crate::commands::alert::update_alert_service,
                crate::commands::alert::delete_alert_service,
                crate::commands::alert::get_alert_resources,
                crate::commands::alert::add_alert_resource,
                crate::commands::alert::update_alert_resource,
                crate::commands::alert::delete_alert_resource,
                crate::commands::alert::get_alert_history,
                // NOTE: trigger_alert_check takes AppHandle — cannot be
                // registered with MockRuntime. Omitted from test handler.
            ])
            .build(mock_context(noop_assets()))
            .expect("mock app should build");
        let ww = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("webview window should build");
        (app, ww)
    }

    // ── IPC invoke helper ───────────────────────────────────────────────

    fn invoke_ipc<R: serde::de::DeserializeOwned>(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: serde_json::Value,
    ) -> Result<R, String> {
        let res = get_ipc_response(
            webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        match res {
            Ok(response) => response
                .deserialize::<R>()
                .map_err(|e| format!("deserialize error: {e:?}")),
            Err(e) => Err(format!("IPC error: {e:?}")),
        }
    }

    // ── Pure-logic helper tests (no Tauri) ──────────────────────────────

    #[test]
    fn test_build_email_success() {
        let email = build_email("from@test.com", "to@test.com", "Subject", "Body")
            .expect("valid addresses should build");
        // Just confirm the message was built — subject() API may vary by lettre version
    }

    #[test]
    fn test_build_email_invalid_from() {
        let result = build_email("not-an-email", "to@test.com", "Subject", "Body");
        assert!(result.is_err(), "invalid from should fail");
    }

    #[test]
    fn test_build_email_invalid_to() {
        let result = build_email("from@test.com", "not-an-email", "Subject", "Body");
        assert!(result.is_err(), "invalid to should fail");
    }

    #[test]
    fn test_build_email_multiple_recipients() {
        let email = build_email(
            "from@test.com",
            "a@test.com, b@test.com",
            "Subject",
            "Body",
        )
        .expect("multiple valid recipients should build");
        // Just confirm the message was built — subject() API may vary by lettre version
    }

    #[test]
    fn test_build_smtp_transport_no_encryption_succeeds() {
        // With "none" encryption, transport construction always succeeds;
        // actual failure only happens at send time.
        let result = build_smtp_transport("smtp.example.com", 25, "none", "user", "pass");
        assert!(result.is_ok(), "transport construction should succeed");
    }

    // ── IPC tests: mock_app build smoke test ────────────────────────────

    #[test]
    fn test_ipc_mock_builder_creates_app() {
        let (_app, _ww) = build_test_app();
    }

    // ── IPC tests: get_email_config / save_email_config ─────────────────

    #[test]
    fn test_ipc_get_email_config_returns_none_initially() {
        let (_app, ww) = build_test_app();
        let result: Result<Option<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_email_config", serde_json::json!({}));
        assert!(result.is_ok(), "get_email_config should not fail");
        assert!(
            result.unwrap().is_none(),
            "no config should exist initially"
        );
    }

    #[test]
    fn test_ipc_save_and_get_email_config() {
        let (_app, ww) = build_test_app();

        // Save a config
        let save: Result<(), String> = invoke_ipc(
            &ww,
            "save_email_config",
            serde_json::json!({
                "smtpHost": "smtp.example.com",
                "smtpPort": 587,
                "smtpUsername": "alice",
                "smtpPassword": "secret",
                "fromEmail": "alice@example.com",
                "toEmail": "bob@example.com",
                "smtpEncryption": "starttls",
            }),
        );
        assert!(save.is_ok(), "save_email_config should succeed");

        // Read it back
        let get: Result<Option<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_email_config", serde_json::json!({}));
        let config = get.expect("get_email_config should succeed")
            .expect("config should now exist");
        assert_eq!(config["smtpHost"], "smtp.example.com");
        assert_eq!(config["smtpPort"], 587);
        assert_eq!(config["smtpUsername"], "alice");
        assert_eq!(config["fromEmail"], "alice@example.com");
        assert_eq!(config["toEmail"], "bob@example.com");
        assert_eq!(config["smtpEncryption"], "starttls");
    }

    // ── IPC tests: test_email_config (expected to fail without SMTP) ────

    #[test]
    fn test_ipc_test_email_config_fails_without_smtp() {
        let (_app, ww) = build_test_app();
        // No real SMTP server available → should return an Err
        let result: Result<String, String> = invoke_ipc(
            &ww,
            "test_email_config",
            serde_json::json!({
                "smtpHost": "nonexistent.local",
                "smtpPort": 587,
                "smtpUsername": null,
                "smtpPassword": null,
                "fromEmail": "test@test.com",
                "toEmail": "test@test.com",
                "smtpEncryption": "starttls",
            }),
        );
        assert!(result.is_err(), "test_email_config should fail without SMTP");
    }

    // ── IPC tests: alert_services CRUD ──────────────────────────────────

    #[test]
    fn test_ipc_get_alert_services_returns_empty_initially() {
        let (_app, ww) = build_test_app();
        let result: Result<Vec<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({}));
        let services = result.expect("get_alert_services should succeed");
        assert!(services.is_empty(), "no services yet");
    }

    #[test]
    fn test_ipc_add_alert_service() {
        let (_app, ww) = build_test_app();

        let add: Result<(), String> = invoke_ipc(
            &ww,
            "add_alert_service",
            serde_json::json!({
                "service": {
                    "name": "My Service",
                    "host": "192.168.1.1",
                    "port": 443,
                    "checkInterval": 60,
                    "timeoutSeconds": 5,
                    "maxRetries": 3,
                    "enabled": true,
                },
            }),
        );
        assert!(add.is_ok(), "add_alert_service should succeed");

        // Verify it shows up
        let list: Result<Vec<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({}));
        let services = list.expect("get_alert_services should succeed");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0]["name"], "My Service");
        assert_eq!(services[0]["host"], "192.168.1.1");
        assert_eq!(services[0]["port"], 443);
    }

    #[test]
    fn test_ipc_update_alert_service() {
        let (_app, ww) = build_test_app();

        // Add first
        let add: Result<(), String> = invoke_ipc(
            &ww,
            "add_alert_service",
            serde_json::json!({
                "service": {
                    "name": "Original",
                    "host": "10.0.0.1",
                    "port": 80,
                    "checkInterval": 60,
                    "timeoutSeconds": 5,
                    "maxRetries": 3,
                    "enabled": true,
                },
            }),
        );
        assert!(add.is_ok());

        // Get the generated ID
        let list: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
        let service_id = list[0]["id"].as_str().unwrap().to_string();

        // Update
        let update: Result<(), String> = invoke_ipc(
            &ww,
            "update_alert_service",
            serde_json::json!({
                "service": {
                    "id": service_id,
                    "name": "Updated",
                    "host": "10.0.0.1",
                    "port": 8080,
                    "checkInterval": 120,
                    "timeoutSeconds": 10,
                    "maxRetries": 5,
                    "enabled": false,
                },
            }),
        );
        assert!(update.is_ok(), "update_alert_service should succeed");

        // Verify
        let list2: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0]["name"], "Updated");
        assert_eq!(list2[0]["port"], 8080);
        assert!(!list2[0]["enabled"].as_bool().unwrap_or(true));
    }

    #[test]
    fn test_ipc_delete_alert_service() {
        let (_app, ww) = build_test_app();

        // Add
        let _: () = invoke_ipc(
            &ww,
            "add_alert_service",
            serde_json::json!({
                "service": {
                    "name": "ToDelete",
                    "host": "10.0.0.1",
                    "port": 80,
                    "checkInterval": 60,
                    "timeoutSeconds": 5,
                    "maxRetries": 3,
                    "enabled": true,
                },
            }),
        ).unwrap();

        // Get ID
        let list: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
        let sid = list[0]["id"].as_str().unwrap().to_string();

        // Delete
        let del: Result<(), String> = invoke_ipc(
            &ww,
            "delete_alert_service",
            serde_json::json!({ "id": sid }),
        );
        assert!(del.is_ok(), "delete_alert_service should succeed");

        // Verify empty
        let list2: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_services", serde_json::json!({})).unwrap();
        assert!(list2.is_empty());
    }

    // ── IPC tests: alert_resources CRUD ─────────────────────────────────

    #[test]
    fn test_ipc_get_alert_resources_returns_empty_initially() {
        let (_app, ww) = build_test_app();
        let result: Result<Vec<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({}));
        let resources = result.expect("get_alert_resources should succeed");
        assert!(resources.is_empty(), "no resources yet");
    }

    #[test]
    fn test_ipc_add_alert_resource() {
        let (_app, ww) = build_test_app();

        let add: Result<(), String> = invoke_ipc(
            &ww,
            "add_alert_resource",
            serde_json::json!({
                "resource": {
                    "name": "SSL Cert",
                    "category": "certificate",
                    "remark": "example.com cert",
                    "expireAt": "2026-12-31",
                    "alertAdvanceDays": 30,
                    "enabled": true,
                },
            }),
        );
        assert!(add.is_ok(), "add_alert_resource should succeed");

        let list: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["name"], "SSL Cert");
        assert_eq!(list[0]["category"], "certificate");
    }

    #[test]
    fn test_ipc_update_alert_resource() {
        let (_app, ww) = build_test_app();

        // Add
        let _: () = invoke_ipc(
            &ww,
            "add_alert_resource",
            serde_json::json!({
                "resource": {
                    "name": "Original",
                    "category": "certificate",
                    "remark": "",
                    "expireAt": "2026-12-31",
                    "alertAdvanceDays": 30,
                    "enabled": true,
                },
            }),
        ).unwrap();

        let list: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
        let rid = list[0]["id"].as_str().unwrap().to_string();

        let update: Result<(), String> = invoke_ipc(
            &ww,
            "update_alert_resource",
            serde_json::json!({
                "resource": {
                    "id": rid,
                    "name": "Updated Cert",
                    "category": "certificate",
                    "remark": "updated",
                    "expireAt": "2027-01-15",
                    "alertAdvanceDays": 14,
                    "enabled": false,
                },
            }),
        );
        assert!(update.is_ok(), "update_alert_resource should succeed");

        let list2: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0]["name"], "Updated Cert");
        assert_eq!(list2[0]["alertAdvanceDays"], 14);
    }

    #[test]
    fn test_ipc_delete_alert_resource() {
        let (_app, ww) = build_test_app();

        let _: () = invoke_ipc(
            &ww,
            "add_alert_resource",
            serde_json::json!({
                "resource": {
                    "name": "ToDelete",
                    "category": "domain",
                    "remark": "",
                    "expireAt": "2026-12-31",
                    "alertAdvanceDays": 30,
                    "enabled": true,
                },
            }),
        ).unwrap();

        let list: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
        let rid = list[0]["id"].as_str().unwrap().to_string();

        let del: Result<(), String> = invoke_ipc(
            &ww,
            "delete_alert_resource",
            serde_json::json!({ "id": rid }),
        );
        assert!(del.is_ok(), "delete_alert_resource should succeed");

        let list2: Vec<serde_json::Value> =
            invoke_ipc(&ww, "get_alert_resources", serde_json::json!({})).unwrap();
        assert!(list2.is_empty());
    }

    // ── IPC tests: get_alert_history ────────────────────────────────────

    #[test]
    fn test_ipc_get_alert_history_returns_empty_initially() {
        let (_app, ww) = build_test_app();
        let result: Result<Vec<serde_json::Value>, String> =
            invoke_ipc(&ww, "get_alert_history", serde_json::json!({}));
        let history = result.expect("get_alert_history should succeed");
        assert!(history.is_empty(), "no history yet");
    }

    // trigger_alert_check takes AppHandle — cannot be compiled with MockRuntime,
    // so it is omitted from the test invoke_handler entirely.
}
