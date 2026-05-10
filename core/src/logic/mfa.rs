use serde_json::{json, Value};
use rusqlite::params;
use hmac::{KeyInit, Mac};

/// Mfa module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_all_mfa_secrets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM mfa_secrets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "secret": row.get::<_, String>("secret")?,
                        "issuer": row.get::<_, Option<String>>("issuer")?,
                        "digits": row.get::<_, i64>("digits")?,
                        "period": row.get::<_, i64>("period")?,
                        "algorithm": row.get::<_, String>("algorithm")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let secrets: Result<Vec<Value>, _> = rows.collect();
            secrets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_mfa_secret(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let secret = params["secret"].as_str().unwrap_or("").to_string();
        let issuer = params.get("issuer").and_then(|v| v.as_str()).unwrap_or("");
        let digits = params["digits"].as_u64().unwrap_or(6);
        let period = params["period"].as_u64().unwrap_or(30);
        let algorithm = params["algorithm"].as_str().unwrap_or("SHA1");
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO mfa_secrets (id, name, secret, issuer, digits, period, algorithm, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![id, name, secret, issuer, digits as i64, period as i64, algorithm, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_mfa_secret(&self, id: &str, params: Value) -> Result<Value, String> {
        let mut updates = Vec::new();
        if let Some(name) = params["name"].as_str() {
            updates.push(format!("name='{}'", name.replace('\'', "''")));
        }
        if let Some(secret) = params["secret"].as_str() {
            updates.push(format!("secret='{}'", secret.replace('\'', "''")));
        }
        if updates.is_empty() {
            return Ok(json!({"id": id}));
        }
        let sql = format!(
            "UPDATE mfa_secrets SET {} WHERE id='{}'",
            updates.join(", "),
            id
        );
        self.with_db(|db| db.conn_mut().execute(&sql, []).map_err(|e| e.to_string()))
            .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_mfa_secret(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM mfa_secrets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn generate_totp(
        &self,
        secret: &str,
        digits: u32,
        period: u32,
        algorithm: &str,
    ) -> Result<Value, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
use sha1::Sha1;

        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let time_step = epoch.as_secs() / period as u64;
        let remaining = period - (epoch.as_secs() % period as u64) as u32;

        type HmacSha1 = hmac::Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).map_err(|e| e.to_string())?;
        mac.update(&time_step.to_be_bytes());
        let result = mac.finalize().into_bytes();

        let offset = (result[19] & 0xf) as usize;
        let code = ((result[offset] as u32 & 0x7f) << 24)
            | ((result[offset + 1] as u32) << 16)
            | ((result[offset + 2] as u32) << 8)
            | (result[offset + 3] as u32);
        let modulo = 10u32.pow(digits);
        let otp = code % modulo;

        Ok(json!({
            "code": format!("{:0>width$}", otp, width = digits as usize),
            "remaining": remaining,
            "algorithm": algorithm,
        }))
    }

    // ============ Notes ============


}
