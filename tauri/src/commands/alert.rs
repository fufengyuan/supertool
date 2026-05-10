use supertool_core::db::alert::{AlertEmailConfig, AlertHistory, AlertResource, AlertService};
use supertool_core::encryption::{encrypt_password, try_decrypt_password};
use supertool_core::logic::CoreService;
use tauri::{Emitter, Manager, State};
use uuid::Uuid;
use lettre::AsyncTransport;

// ==================== Email Sending (Tauri layer) ====================

/// Send an alert email using SMTP config from DB.
async fn send_alert_email(core: &CoreService, subject: &str, body: &str) -> Result<(), String> {
    let config = core.get_email_config().await?
        .ok_or_else(|| "邮件配置未设置".to_string())?;

    let smtp_host = config.smtp_host.ok_or_else(|| "SMTP 主机未配置".to_string())?;
    let smtp_username = config.smtp_username.ok_or_else(|| "SMTP 用户名未配置".to_string())?;
    let from_email = config.from_email.ok_or_else(|| "发件人邮箱未配置".to_string())?;
    let to_email = config.to_email.ok_or_else(|| "收件人邮箱未配置".to_string())?;
    let smtp_port = config.smtp_port as u16;
    let smtp_use_tls = config.smtp_use_tls;
    let encrypted_pw = config.smtp_password.unwrap_or_default();
    let smtp_password = try_decrypt_password(&encrypted_pw);

    // Build email
    let email = lettre::message::Message::builder()
        .from(from_email.parse().map_err(|e| format!("发件人邮箱格式错误: {}", e))?)
        .to(to_email.parse().map_err(|e| format!("收件人邮箱格式错误: {}", e))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("构建邮件失败: {}", e))?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(smtp_username, smtp_password);

    let transport_builder = if smtp_use_tls {
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&smtp_host)
    } else {
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp_host)
    };

    let mailer = transport_builder
        .map_err(|e| format!("创建 SMTP 传输失败: {}", e))?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer.send(email).await
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
pub async fn save_email_config(
    core: State<'_, CoreService>,
    config: AlertEmailConfig,
) -> Result<(), String> {
    log::info!("[Tauri CMD] save_email_config() called");
    let mut config = config;
    // Encrypt password if it's plaintext
    if let Some(ref pwd) = config.smtp_password.clone() {
        if !pwd.is_empty() && !pwd.starts_with("enc:") && !pwd.starts_with("$argon") {
            // Check if it's base64-looking (already encrypted) — skip if so
            let maybe_encrypted = pwd.len() > 20 && pwd.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
            if !maybe_encrypted {
                config.smtp_password = Some(encrypt_password(pwd).map_err(|e| e.to_string())?);
            }
        }
    }
    core.save_email_config(config).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_alert_services(
    core: State<'_, CoreService>,
) -> Result<Vec<AlertService>, String> {
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
pub async fn delete_alert_service(
    core: State<'_, CoreService>,
    id: String,
) -> Result<(), String> {
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
pub async fn delete_alert_resource(
    core: State<'_, CoreService>,
    id: String,
) -> Result<(), String> {
    log::info!("[Tauri CMD] delete_alert_resource() called");
    core.delete_alert_resource(&id).await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_alert_history(
    core: State<'_, CoreService>,
) -> Result<Vec<AlertHistory>, String> {
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

    // Check resources
    let resource_results = core.check_expiring_resources().await;
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
