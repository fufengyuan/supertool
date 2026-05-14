use crate::db::alert;
#[allow(unused_imports)]
use crate::logic::CoreService;
use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Result of a single alert check — used for both service checks and resource expiry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertResult {
    #[serde(rename = "alertType")]
    pub alert_type: String, // "service_down", "service_recovered", "resource_expiry"
    #[serde(rename = "refId")]
    pub ref_id: String,
    #[serde(rename = "refName")]
    pub ref_name: String,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub category: Option<String>,
    pub message: String,
}

/// Check if a TCP port is reachable within the given timeout.
fn check_tcp_port(host: &str, port: u16, timeout_secs: u64) -> bool {
    let addr_str = format!("{}:{}", host, port);
    let addr = match addr_str.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => return false,
        },
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)).is_ok()
}

impl super::CoreService {
    /// Check all enabled alert services for port connectivity.
    /// Returns a list of alert results that should trigger notifications.
    pub async fn check_alert_services(&self) -> Vec<AlertResult> {
        let mut results = Vec::new();

        // Get all enabled services
        let services: Vec<alert::AlertService> = self.db_read(|conn| {
            let mut stmt = match conn.prepare("SELECT * FROM alert_services WHERE enabled = 1") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Alert] Failed to prepare query: {}", e);
                    return Vec::new();
                }
            };
            let rows = match stmt.query_map([], |row| {
                let enabled: i64 = row.get("enabled")?;
                Ok(alert::AlertService {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    host: row.get("host")?,
                    port: row.get("port")?,
                    check_interval: row.get("check_interval")?,
                    timeout_seconds: row.get("timeout_seconds")?,
                    max_retries: row.get("max_retries")?,
                    enabled: enabled == 1,
                    last_check_at: row.get("last_check_at").ok(),
                    last_status: row.get("last_status").ok(),
                    consecutive_failures: row.get("consecutive_failures")?,
                    alert_sent_at: row.get("alert_sent_at").ok(),
                    created_at: row.get("created_at")?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[Alert] Failed to query services: {}", e);
                    return Vec::new();
                }
            };
            rows.filter_map(|r| r.ok()).collect()
        }).unwrap_or_default();

        let now = chrono::Utc::now();

        for service in services {
            // Skip if not enough time has passed since last check
            if let Some(ref last_check) = service.last_check_at {
                if let Ok(last) = chrono::DateTime::parse_from_rfc3339(last_check) {
                    let last_utc = last.with_timezone(&chrono::Utc);
                    let elapsed = now.signed_duration_since(last_utc).num_seconds();
                    log::debug!(
                        "[Alert] Service {} last_check {}s ago (interval {}s)",
                        service.name, elapsed, service.check_interval
                    );
                    if elapsed < service.check_interval {
                        log::debug!("[Alert] Skipping service {} - not due yet", service.name);
                        continue;
                    }
                } else {
                    log::warn!(
                        "[Alert] Failed to parse last_check_at for service {}: {}",
                        service.name, last_check
                    );
                }
            }

            let host_clone = service.host.clone();
            let port_val = service.port as u16;
            let timeout_val = service.timeout_seconds as u64;
            let port_up = tokio::task::spawn_blocking(move || {
                check_tcp_port(&host_clone, port_val, timeout_val)
            }).await.unwrap_or(false);
            let current_failures = if port_up { 0 } else { service.consecutive_failures + 1 };

            // Update last_status and consecutive_failures in DB
            let last_status_val: i64 = if port_up { 1 } else { 0 };
            let now_str = now.to_rfc3339();
            if let Err(e) = self.db_write(|conn| {
                conn.execute(
                    "UPDATE alert_services SET last_status=?1, consecutive_failures=?2, last_check_at=?3 WHERE id=?4",
                    rusqlite::params![last_status_val, current_failures, now_str, service.id],
                ).map(|_| ()).map_err(|e| format!("{}", e))
            }) {
                log::warn!("[Alert] Failed to update service status {}: {}", service.id, e);
            }

            if port_up {
                if service.last_status == Some(0) {
                    // Was down, now recovered
                    let recent = self.db_read(|conn| -> bool {
                        let mut stmt = match conn.prepare(
                            "SELECT COUNT(*) FROM alert_history WHERE type='service_recovered' AND ref_id=?1 AND sent_at > datetime('now', '-1 hours')"
                        ) {
                            Ok(s) => s,
                            Err(_) => return true,
                        };
                        stmt.query_row(rusqlite::params![service.id], |row| row.get::<_, i64>(0))
                            .map(|c| c > 0)
                            .unwrap_or(true)
                    }).unwrap_or(true);

                    if !recent {
                        results.push(AlertResult {
                            alert_type: "service_recovered".to_string(),
                            ref_id: service.id.clone(),
                            ref_name: service.name.clone(),
                            host: Some(service.host.clone()),
                            port: Some(service.port),
                            category: None,
                            message: format!("服务 {} ({}:{}) 已恢复", service.name, service.host, service.port),
                        });
                    }
                }
                if let Err(e) = self.db_write(|conn| {
                    conn.execute(
                        "UPDATE alert_services SET alert_sent_at=NULL WHERE id=?1",
                        rusqlite::params![service.id],
                    ).map(|_| ()).map_err(|e| format!("{}", e))
                }) {
                    log::warn!("[Alert] Failed to clear alert_sent_at {}: {}", service.id, e);
                }
            } else {
                let should_alert = current_failures >= service.max_retries;
                if should_alert {
                    let recent_alert = self.db_read(|conn| -> bool {
                        let mut stmt = match conn.prepare(
                            "SELECT COUNT(*) FROM alert_history WHERE type='service_down' AND ref_id=?1 AND sent_at > datetime('now', '-1 hours')"
                        ) {
                            Ok(s) => s,
                            Err(_) => return true,
                        };
                        stmt.query_row(rusqlite::params![service.id], |row| row.get::<_, i64>(0))
                            .map(|c| c > 0)
                            .unwrap_or(true)
                    }).unwrap_or(true);

                    if !recent_alert {
                        results.push(AlertResult {
                            alert_type: "service_down".to_string(),
                            ref_id: service.id.clone(),
                            ref_name: service.name.clone(),
                            host: Some(service.host.clone()),
                            port: Some(service.port),
                            category: None,
                            message: format!("服务 {} ({}:{}) 无法连接（连续 {} 次失败）", service.name, service.host, service.port, current_failures),
                        });
                        if let Err(e) = self.db_write(|conn| {
                            conn.execute(
                                "UPDATE alert_services SET alert_sent_at=?1 WHERE id=?2",
                                rusqlite::params![now_str, service.id],
                            ).map(|_| ()).map_err(|e| format!("{}", e))
                        }) {
                            log::warn!("[Alert] Failed to update alert_sent_at {}: {}", service.id, e);
                        }
                    }
                }
            }
        }
        results
    }

    /// Check all enabled resources for upcoming expiry dates.
    pub async fn check_expiring_resources(&self) -> Vec<AlertResult> {
        let mut results = Vec::new();

        let resources: Vec<alert::AlertResource> = self.db_read(|conn| {
            let mut stmt = match conn.prepare("SELECT * FROM alert_resources WHERE enabled = 1") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Alert] Failed to prepare resource query: {}", e);
                    return Vec::new();
                }
            };
            let rows = match stmt.query_map([], |row| {
                let enabled: i64 = row.get("enabled")?;
                Ok(alert::AlertResource {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    category: row.get("category").ok(),
                    remark: row.get("remark").ok(),
                    expire_at: row.get("expire_at").ok(),
                    alert_advance_days: row.get("alert_advance_days")?,
                    enabled: enabled == 1,
                    last_alert_sent_at: row.get("last_alert_sent_at").ok(),
                    created_at: row.get("created_at")?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[Alert] Failed to query resources: {}", e);
                    return Vec::new();
                }
            };
            rows.filter_map(|r| r.ok()).collect()
        }).unwrap_or_default();

        let now = chrono::Utc::now();

        for resource in resources {
            let expire_at_str = match &resource.expire_at {
                Some(d) => d,
                None => continue,
            };

            let expire_at = chrono::DateTime::parse_from_rfc3339(expire_at_str)
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(expire_at_str, "%Y-%m-%d")
                        .map(|d| {
                            d.and_hms_opt(23, 59, 59).unwrap().and_utc().fixed_offset()
                        })
                });

            let expire_at = match expire_at {
                Ok(dt) => dt,
                Err(_) => {
                    log::warn!("[Alert] Failed to parse expire_at for resource {}: {}", resource.name, expire_at_str);
                    continue;
                }
            };

            let days_until_expiry = expire_at.signed_duration_since(now).num_days();

            if days_until_expiry < 0 || days_until_expiry <= resource.alert_advance_days as i64 {
                let recent_alert = self.db_read(|conn| -> bool {
                    let mut stmt = match conn.prepare(
                        "SELECT COUNT(*) FROM alert_history WHERE type='resource_expiry' AND ref_id=?1 AND sent_at > datetime('now', '-24 hours')"
                    ) {
                        Ok(s) => s,
                        Err(_) => return true,
                    };
                    stmt.query_row(rusqlite::params![resource.id], |row| row.get::<_, i64>(0))
                        .map(|c| c > 0)
                        .unwrap_or(true)
                }).unwrap_or(true);

                if !recent_alert {
                    let msg = if days_until_expiry < 0 {
                        format!("资源 {} ({}) 已过期！到期日: {}", resource.name, resource.category.as_deref().unwrap_or("未分类"), expire_at_str)
                    } else {
                        format!("资源 {} ({}) 将在 {} 天后到期（{}），请及时处理", resource.name, resource.category.as_deref().unwrap_or("未分类"), days_until_expiry, expire_at_str)
                    };

                    results.push(AlertResult {
                        alert_type: "resource_expiry".to_string(),
                        ref_id: resource.id.clone(),
                        ref_name: resource.name.clone(),
                        host: None,
                        port: None,
                        category: resource.category.clone(),
                        message: msg,
                    });

                    let now_str = now.to_rfc3339();
                    if let Err(e) = self.db_write(|conn| {
                        conn.execute(
                            "UPDATE alert_resources SET last_alert_sent_at=?1 WHERE id=?2",
                            rusqlite::params![now_str, resource.id],
                        ).map(|_| ()).map_err(|e| format!("{}", e))
                    }) {
                        log::warn!("[Alert] Failed to update resource last_alert {}: {}", resource.id, e);
                    }
                }
            }
        }
        results
    }

    /// Get email config
    pub async fn get_email_config(&self) -> Result<Option<alert::AlertEmailConfig>, String> {
        self.db_read(|conn| {
            let mut stmt = match conn.prepare(
                "SELECT id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, from_email, to_email, updated_at FROM alert_email_config WHERE id = 1"
            ) {
                Ok(s) => s,
                Err(_) => return None,
            };
            match stmt.query_row([], |row| {
                Ok(alert::AlertEmailConfig {
                    id: row.get("id")?,
                    smtp_host: row.get("smtp_host").ok(),
                    smtp_port: row.get("smtp_port")?,
                    smtp_username: row.get("smtp_username").ok(),
                    smtp_password: row.get("smtp_password").ok(),
                    smtp_encryption: row.get::<_, Option<String>>("smtp_encryption").ok().flatten().unwrap_or_else(|| "starttls".to_string()),
                    from_email: row.get("from_email").ok(),
                    to_email: row.get("to_email").ok(),
                    updated_at: row.get("updated_at")?,
                })
            }) {
                Ok(c) => Some(c),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(_) => None,
            }
        })
    }

    /// Upsert email config
    pub async fn save_email_config(&self, config: alert::AlertEmailConfig) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO alert_email_config (id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, from_email, to_email, updated_at)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                rusqlite::params![
                    config.smtp_host,
                    config.smtp_port,
                    config.smtp_username,
                    config.smtp_password,
                    config.smtp_encryption,
                    config.from_email,
                    config.to_email,
                ],
            ).map(|_| ()).map_err(|e| format!("保存邮件配置失败: {}", e))
        })?
    }

    /// Get all alert services
    pub async fn get_alert_services(&self) -> Result<Vec<alert::AlertService>, String> {
        self.db_read(|conn| {
            let mut stmt = match conn.prepare("SELECT * FROM alert_services ORDER BY name") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Alert] prepare services: {}", e);
                    return Vec::new();
                }
            };
            let rows = match stmt.query_map([], |row| {
                let enabled: i64 = row.get("enabled")?;
                Ok(alert::AlertService {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    host: row.get("host")?,
                    port: row.get("port")?,
                    check_interval: row.get("check_interval")?,
                    timeout_seconds: row.get("timeout_seconds")?,
                    max_retries: row.get("max_retries")?,
                    enabled: enabled == 1,
                    last_check_at: row.get("last_check_at").ok(),
                    last_status: row.get("last_status").ok(),
                    consecutive_failures: row.get("consecutive_failures")?,
                    alert_sent_at: row.get("alert_sent_at").ok(),
                    created_at: row.get("created_at")?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[Alert] query services: {}", e);
                    return Vec::new();
                }
            };
            rows.filter_map(|r| r.ok()).collect()
        })
    }

    /// Add alert service
    pub async fn add_alert_service(&self, service: alert::AlertService) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "INSERT INTO alert_services (id, name, host, port, check_interval, timeout_seconds, max_retries, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
                rusqlite::params![
                    service.id, service.name, service.host, service.port,
                    service.check_interval, service.timeout_seconds, service.max_retries,
                    if service.enabled { 1 } else { 0 },
                ],
            ).map(|_| ()).map_err(|e| format!("添加服务失败: {}", e))
        })?
    }

    /// Update alert service
    pub async fn update_alert_service(&self, service: alert::AlertService) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "UPDATE alert_services SET name=?2, host=?3, port=?4, check_interval=?5, timeout_seconds=?6, max_retries=?7, enabled=?8 WHERE id=?1",
                rusqlite::params![
                    service.id, service.name, service.host, service.port,
                    service.check_interval, service.timeout_seconds, service.max_retries,
                    if service.enabled { 1 } else { 0 },
                ],
            ).map(|_| ()).map_err(|e| format!("更新服务失败: {}", e))
        })?
    }

    /// Delete alert service
    pub async fn delete_alert_service(&self, id: &str) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute("DELETE FROM alert_services WHERE id = ?1", rusqlite::params![id])
                .map(|_| ()).map_err(|e| format!("删除服务失败: {}", e))
        })?
    }

    /// Get all alert resources
    pub async fn get_alert_resources(&self) -> Result<Vec<alert::AlertResource>, String> {
        self.db_read(|conn| {
            let mut stmt = match conn.prepare("SELECT * FROM alert_resources ORDER BY name") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Alert] prepare resources: {}", e);
                    return Vec::new();
                }
            };
            let rows = match stmt.query_map([], |row| {
                let enabled: i64 = row.get("enabled")?;
                Ok(alert::AlertResource {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    category: row.get("category").ok(),
                    remark: row.get("remark").ok(),
                    expire_at: row.get("expire_at").ok(),
                    alert_advance_days: row.get("alert_advance_days")?,
                    enabled: enabled == 1,
                    last_alert_sent_at: row.get("last_alert_sent_at").ok(),
                    created_at: row.get("created_at")?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[Alert] query resources: {}", e);
                    return Vec::new();
                }
            };
            rows.filter_map(|r| r.ok()).collect()
        })
    }

    /// Add alert resource
    pub async fn add_alert_resource(&self, resource: alert::AlertResource) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "INSERT INTO alert_resources (id, name, category, remark, expire_at, alert_advance_days, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                rusqlite::params![
                    resource.id, resource.name, resource.category, resource.remark, resource.expire_at,
                    resource.alert_advance_days, if resource.enabled { 1 } else { 0 },
                ],
            ).map(|_| ()).map_err(|e| format!("添加资源失败: {}", e))
        })?
    }

    /// Update alert resource
    pub async fn update_alert_resource(&self, resource: alert::AlertResource) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "UPDATE alert_resources SET name=?2, category=?3, remark=?4, expire_at=?5, alert_advance_days=?6, enabled=?7 WHERE id=?1",
                rusqlite::params![
                    resource.id, resource.name, resource.category, resource.remark, resource.expire_at,
                    resource.alert_advance_days, if resource.enabled { 1 } else { 0 },
                ],
            ).map(|_| ()).map_err(|e| format!("更新资源失败: {}", e))
        })?
    }

    /// Delete alert resource
    pub async fn delete_alert_resource(&self, id: &str) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute("DELETE FROM alert_resources WHERE id = ?1", rusqlite::params![id])
                .map(|_| ()).map_err(|e| format!("删除资源失败: {}", e))
        })?
    }

    /// Get alert history (last 100)
    pub async fn get_alert_history(&self) -> Result<Vec<alert::AlertHistory>, String> {
        self.db_read(|conn| {
            let mut stmt = match conn.prepare("SELECT id, type, ref_id, ref_name, message, sent_at FROM alert_history ORDER BY sent_at DESC LIMIT 100") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Alert] prepare history: {}", e);
                    return Vec::new();
                }
            };
            let rows = match stmt.query_map([], |row| {
                Ok(alert::AlertHistory {
                    id: row.get("id")?,
                    alert_type: row.get("type")?,
                    ref_id: row.get("ref_id")?,
                    ref_name: row.get("ref_name")?,
                    message: row.get("message")?,
                    sent_at: row.get("sent_at")?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("[Alert] query history: {}", e);
                    return Vec::new();
                }
            };
            rows.filter_map(|r| r.ok()).collect()
        })
    }

    /// Insert alert history
    pub async fn insert_alert_history(&self, h: alert::AlertHistory) -> Result<(), String> {
        self.db_write(|conn| {
            conn.execute(
                "INSERT INTO alert_history (id, type, ref_id, ref_name, message, sent_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
                rusqlite::params![h.id, h.alert_type, h.ref_id, h.ref_name, h.message],
            ).map(|_| ()).map_err(|e| format!("插入告警历史失败: {}", e))
        })?
    }
}
