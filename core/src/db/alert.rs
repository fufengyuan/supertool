use crate::db::{ApiResponse, Database};
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ==================== Data Types ====================

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertEmailConfig {
    pub id: i32,
    #[serde(rename = "smtpHost")]
    pub smtp_host: Option<String>,
    #[serde(rename = "smtpPort")]
    pub smtp_port: i64,
    #[serde(rename = "smtpUsername")]
    pub smtp_username: Option<String>,
    #[serde(rename = "smtpPassword")]
    pub smtp_password: Option<String>,
    /// "none" | "starttls" | "ssl"
    #[serde(rename = "smtpEncryption", default = "default_encryption")]
    pub smtp_encryption: String,
    #[serde(rename = "fromEmail")]
    pub from_email: Option<String>,
    #[serde(rename = "toEmail")]
    pub to_email: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn default_encryption() -> String {
    "starttls".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertService {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    #[serde(rename = "checkInterval")]
    pub check_interval: i64,
    #[serde(rename = "timeoutSeconds")]
    pub timeout_seconds: i64,
    #[serde(rename = "maxRetries")]
    pub max_retries: i64,
    pub enabled: bool,
    #[serde(rename = "lastCheckAt")]
    pub last_check_at: Option<String>,
    #[serde(rename = "lastStatus")]
    pub last_status: Option<i64>,
    #[serde(rename = "consecutiveFailures")]
    pub consecutive_failures: i64,
    #[serde(rename = "alertSentAt")]
    pub alert_sent_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertResource {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    #[serde(rename = "expireAt")]
    pub expire_at: Option<String>,
    #[serde(rename = "alertAdvanceDays")]
    pub alert_advance_days: i64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "lastAlertSentAt")]
    pub last_alert_sent_at: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertHistory {
    pub id: String,
    #[serde(rename = "alertType")]
    pub alert_type: String,
    #[serde(rename = "refId")]
    pub ref_id: String,
    #[serde(rename = "refName")]
    pub ref_name: String,
    pub message: String,
    #[serde(rename = "sentAt")]
    pub sent_at: String,
}

// ==================== Email Config CRUD ====================

pub fn get_email_config(db: &mut Database) -> ApiResponse<Option<AlertEmailConfig>> {
    let mut stmt = match db
        .conn()
        .prepare("SELECT id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, from_email, to_email, updated_at FROM alert_email_config WHERE id = 1")
    {
        Ok(s) => s,
        Err(e) => return ApiResponse::err(format!("Prepare failed: {}", e)),
    };

    match stmt.query_row([], |row| {
        Ok(AlertEmailConfig {
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
        Ok(config) => ApiResponse::ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => ApiResponse::ok(None),
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}

pub fn upsert_email_config(db: &mut Database, config: AlertEmailConfig) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "INSERT OR REPLACE INTO alert_email_config (id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, from_email, to_email, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
        params![
            config.smtp_host,
            config.smtp_port,
            config.smtp_username,
            config.smtp_password,
            config.smtp_encryption,
            config.from_email,
            config.to_email,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Upsert failed: {}", e)),
    }
}

// ==================== Alert Service CRUD ====================

fn row_to_alert_service(row: &rusqlite::Row<'_>) -> rusqlite::Result<AlertService> {
    let enabled: i64 = row.get("enabled")?;
    Ok(AlertService {
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
}

pub fn get_alert_services(db: &mut Database) -> ApiResponse<Vec<AlertService>> {
    match db
        .conn()
        .prepare("SELECT * FROM alert_services ORDER BY name")
    {
        Ok(mut stmt) => match stmt.query_map([], row_to_alert_service) {
            Ok(rows) => {
                let services: Result<Vec<AlertService>, _> = rows.collect();
                match services {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_alert_service(db: &mut Database, service: AlertService) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "INSERT INTO alert_services (id, name, host, port, check_interval, timeout_seconds, max_retries, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
        params![
            service.id,
            service.name,
            service.host,
            service.port,
            service.check_interval,
            service.timeout_seconds,
            service.max_retries,
            if service.enabled { 1 } else { 0 },
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_alert_service(db: &mut Database, service: AlertService) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "UPDATE alert_services SET name=?2, host=?3, port=?4, check_interval=?5, timeout_seconds=?6, max_retries=?7, enabled=?8 WHERE id=?1",
        params![
            service.id,
            service.name,
            service.host,
            service.port,
            service.check_interval,
            service.timeout_seconds,
            service.max_retries,
            if service.enabled { 1 } else { 0 },
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_alert_service(db: &mut Database, id: String) -> ApiResponse<()> {
    match db.conn_mut().execute("DELETE FROM alert_services WHERE id = ?1", params![id]) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

// ==================== Alert Resource CRUD ====================

fn row_to_alert_resource(row: &rusqlite::Row<'_>) -> rusqlite::Result<AlertResource> {
    let enabled: i64 = row.get("enabled")?;
    Ok(AlertResource {
        id: row.get("id")?,
        name: row.get("name")?,
        category: row.get("category").ok(),
        expire_at: row.get("expire_at").ok(),
        alert_advance_days: row.get("alert_advance_days")?,
        enabled: enabled == 1,
        last_alert_sent_at: row.get("last_alert_sent_at").ok(),
        created_at: row.get("created_at")?,
    })
}

pub fn get_alert_resources(db: &mut Database) -> ApiResponse<Vec<AlertResource>> {
    match db
        .conn()
        .prepare("SELECT * FROM alert_resources ORDER BY name")
    {
        Ok(mut stmt) => match stmt.query_map([], row_to_alert_resource) {
            Ok(rows) => {
                let resources: Result<Vec<AlertResource>, _> = rows.collect();
                match resources {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn add_alert_resource(db: &mut Database, resource: AlertResource) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "INSERT INTO alert_resources (id, name, category, expire_at, alert_advance_days, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        params![
            resource.id,
            resource.name,
            resource.category,
            resource.expire_at,
            resource.alert_advance_days,
            if resource.enabled { 1 } else { 0 },
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}

pub fn update_alert_resource(db: &mut Database, resource: AlertResource) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "UPDATE alert_resources SET name=?2, category=?3, expire_at=?4, alert_advance_days=?5, enabled=?6 WHERE id=?1",
        params![
            resource.id,
            resource.name,
            resource.category,
            resource.expire_at,
            resource.alert_advance_days,
            if resource.enabled { 1 } else { 0 },
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Update failed: {}", e)),
    }
}

pub fn delete_alert_resource(db: &mut Database, id: String) -> ApiResponse<()> {
    match db.conn_mut().execute("DELETE FROM alert_resources WHERE id = ?1", params![id]) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Delete failed: {}", e)),
    }
}

// ==================== Alert History ====================

pub fn get_alert_history(db: &mut Database) -> ApiResponse<Vec<AlertHistory>> {
    match db
        .conn()
        .prepare("SELECT id, type, ref_id, ref_name, message, sent_at FROM alert_history ORDER BY sent_at DESC LIMIT 100")
    {
        Ok(mut stmt) => match stmt.query_map([], |row| {
            Ok(AlertHistory {
                id: row.get("id")?,
                alert_type: row.get("type")?,
                ref_id: row.get("ref_id")?,
                ref_name: row.get("ref_name")?,
                message: row.get("message")?,
                sent_at: row.get("sent_at")?,
            })
        }) {
            Ok(rows) => {
                let history: Result<Vec<AlertHistory>, _> = rows.collect();
                match history {
                    Ok(list) => ApiResponse::ok(list),
                    Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
                }
            }
            Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
        },
        Err(e) => ApiResponse::err(format!("Prepare failed: {}", e)),
    }
}

pub fn get_recent_alert_by_ref(
    db: &mut Database,
    alert_type: &str,
    ref_id: &str,
    within_hours: i64,
) -> ApiResponse<Option<AlertHistory>> {
    let query = format!(
        "SELECT id, type, ref_id, ref_name, message, sent_at FROM alert_history \
         WHERE type = ?1 AND ref_id = ?2 AND sent_at > datetime('now', '-{} hours') \
         ORDER BY sent_at DESC LIMIT 1",
        within_hours
    );
    let mut stmt = match db.conn().prepare(&query) {
        Ok(s) => s,
        Err(e) => return ApiResponse::err(format!("Prepare failed: {}", e)),
    };

    match stmt.query_row(params![alert_type, ref_id], |row| {
        Ok(AlertHistory {
            id: row.get("id")?,
            alert_type: row.get("type")?,
            ref_id: row.get("ref_id")?,
            ref_name: row.get("ref_name")?,
            message: row.get("message")?,
            sent_at: row.get("sent_at")?,
        })
    }) {
        Ok(history) => ApiResponse::ok(Some(history)),
        Err(rusqlite::Error::QueryReturnedNoRows) => ApiResponse::ok(None),
        Err(e) => ApiResponse::err(format!("Query failed: {}", e)),
    }
}

pub fn insert_alert_history(db: &mut Database, history: AlertHistory) -> ApiResponse<()> {
    let result = db.conn_mut().execute(
        "INSERT INTO alert_history (id, type, ref_id, ref_name, message, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![
            history.id,
            history.alert_type,
            history.ref_id,
            history.ref_name,
            history.message,
        ],
    );
    match result {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Insert failed: {}", e)),
    }
}
