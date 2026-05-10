/// Accounting module — extracted from mod.rs
use super::CoreService;
use crate::db::Database;
use serde_json::{json, Value};
use std::path::PathBuf;
use rusqlite::params;

/// Accounting module — extracted from mod.rs
///

impl super::CoreService {
    pub async fn get_accounting_categories(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM accounting_categories ORDER BY sortOrder, name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "type": row.get::<_, String>("type")?,
                        "icon": row.get::<_, Option<String>>("icon")?,
                        "sortOrder": row.get::<_, i64>("sortOrder")?,
                        "createdAt": row.get::<_, String>("createdAt")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let cats: Result<Vec<Value>, _> = rows.collect();
            cats.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_accounting_category(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let cat_type = params["type"].as_str().unwrap_or("expense").to_string();
        let icon = params.get("icon").and_then(|v| v.as_str());
        let sort_order: i64 = params["sortOrder"].as_i64().unwrap_or(0);
        let now = chrono::Utc::now().to_rfc3339();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO accounting_categories (id, name, type, icon, sortOrder, createdAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, name, cat_type, icon, sort_order, now],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_accounting_category(
        &self,
        id: &str,
        params: Value,
    ) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let cat_type = params["type"].as_str().unwrap_or("expense").to_string();
        let icon = params.get("icon").and_then(|v| v.as_str());
        let sort_order: i64 = params["sortOrder"].as_i64().unwrap_or(0);
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE accounting_categories SET name=?2, type=?3, icon=?4, sortOrder=?5 WHERE id=?1",
                    params![id, name, cat_type, icon, sort_order],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_accounting_category(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "DELETE FROM accounting_categories WHERE id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_accounting_records(&self, params: Value) -> Result<Value, String> {
        let start_date = params["startDate"].as_str().unwrap_or("");
        let end_date = params["endDate"].as_str().unwrap_or("");
        let r#type = params["type"].as_str().unwrap_or("");
        let category = params["category"].as_str().unwrap_or("");
        let status = params["status"].as_str().unwrap_or("");
        let entity = params["entity"].as_str().unwrap_or("");
        let project = params["project"].as_str().unwrap_or("");
        let payment_method = params["payment_method"].as_str().unwrap_or("");
        let search = params["search"].as_str().unwrap_or("");
        let page = params["page"].as_u64().unwrap_or(1).max(1);
        let page_size = params["pageSize"].as_u64().unwrap_or(50).max(1);
        let offset = (page - 1) * page_size;

        let result = self.with_db(|db| {
            // Build WHERE clauses with positional params
            let mut conditions: Vec<String> = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            let mut idx = 1;

            if !start_date.is_empty() {
                conditions.push(format!("date >= ?{}", idx));
                param_values.push(Box::new(start_date.to_string()));
                idx += 1;
            }
            if !end_date.is_empty() {
                conditions.push(format!("date <= ?{}", idx));
                param_values.push(Box::new(end_date.to_string()));
                idx += 1;
            }
            if !r#type.is_empty() {
                conditions.push(format!("type = ?{}", idx));
                param_values.push(Box::new(r#type.to_string()));
                idx += 1;
            }
            if !category.is_empty() {
                conditions.push(format!("category = ?{}", idx));
                param_values.push(Box::new(category.to_string()));
                idx += 1;
            }
            if !status.is_empty() {
                conditions.push(format!("status = ?{}", idx));
                param_values.push(Box::new(status.to_string()));
                idx += 1;
            }
            if !entity.is_empty() {
                conditions.push(format!("entity = ?{}", idx));
                param_values.push(Box::new(entity.to_string()));
                idx += 1;
            }
            if !project.is_empty() {
                conditions.push(format!("project = ?{}", idx));
                param_values.push(Box::new(project.to_string()));
                idx += 1;
            }
            if !payment_method.is_empty() {
                conditions.push(format!("payment_method = ?{}", idx));
                param_values.push(Box::new(payment_method.to_string()));
                idx += 1;
            }
            if !search.is_empty() {
                conditions.push(format!("(description LIKE ?{} OR supplier LIKE ?{})", idx, idx + 1));
                let sp = format!("%{}%", search);
                param_values.push(Box::new(sp.clone()));
                param_values.push(Box::new(sp));
                idx += 2;
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", conditions.join(" AND "))
            };

            // Count total
            let count_sql = format!("SELECT COUNT(*) FROM accounting_records {}", where_clause);
            let total: i64 = {
                let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
                db.conn().query_row(&count_sql, params_ref.as_slice(), |row| row.get(0)).unwrap_or(0)
            };

            // Query records with pagination
            let limit_idx = idx;
            let offset_idx = idx + 1;
            let query_sql = format!(
                "SELECT * FROM accounting_records {} ORDER BY date DESC, createdAt DESC LIMIT ?{} OFFSET ?{}",
                where_clause, limit_idx, offset_idx
            );
            param_values.push(Box::new(page_size as i64));
            param_values.push(Box::new(offset as i64));

            let mut stmt = db.conn().prepare(&query_sql).map_err(|e| e.to_string())?;
            let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(params_ref.as_slice(), |row| {
                let attachments_json: Option<String> = row.get("attachments_json").unwrap_or(None);
                let attachments: serde_json::Value = match attachments_json {
                    Some(ref s) if !s.is_empty() && s != "[]" => {
                        serde_json::from_str(s).unwrap_or(serde_json::json!([]))
                    }
                    _ => serde_json::json!([])
                };
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "date": row.get::<_, String>("date")?,
                    "type": row.get::<_, String>("type")?,
                    "category": row.get::<_, String>("category")?,
                    "amount": row.get::<_, f64>("amount")?,
                    "description": row.get::<_, Option<String>>("description")?,
                    "status": row.get::<_, Option<String>>("status")?,
                    "attachmentPath": row.get::<_, Option<String>>("attachmentPath")?,
                    "createdBy": row.get::<_, Option<String>>("createdBy")?,
                    "createdAt": row.get::<_, String>("createdAt")?,
                    "updatedAt": row.get::<_, Option<String>>("updatedAt")?,
                    "voucher_number": row.get::<_, Option<String>>("voucher_number")?,
                    "receipt_type": row.get::<_, Option<String>>("receipt_type")?,
                    "receipt_path": row.get::<_, Option<String>>("receipt_path")?,
                    "entity": row.get::<_, Option<String>>("entity")?,
                    "project": row.get::<_, Option<String>>("project")?,
                    "supplier": row.get::<_, Option<String>>("supplier")?,
                    "invoice_number": row.get::<_, Option<String>>("invoice_number")?,
                    "tax_amount": row.get::<_, Option<f64>>("tax_amount")?,
                    "payment_method": row.get::<_, Option<String>>("payment_method")?,
                    "approver": row.get::<_, Option<String>>("approver")?,
                    "attachments_json": attachments,
                }))
            }).map_err(|e| e.to_string())?;
            let records: Result<Vec<Value>, _> = rows.collect();
            let records = records.map_err(|e| e.to_string())?;
            Ok::<(i64, Vec<Value>), String>((total, records))
        });
        let (total, records) = result?;
        Ok(json!({
            "records": records,
            "total": total,
        }))
    }

    pub async fn add_accounting_record(&self, params: Value) -> Result<Value, String> {
        let id = params["id"].as_str().unwrap_or(&uuid::Uuid::new_v4().to_string()).to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let date = params["date"].as_str().unwrap_or("").to_string();
        let r#type = params["type"].as_str().unwrap_or("expense").to_string();
        let category = params["category"].as_str().unwrap_or("").to_string();
        let amount = params["amount"].as_f64().unwrap_or(0.0);
        let description = params["description"].as_str().or_else(|| params["note"].as_str());
        let status = params["status"].as_str().unwrap_or("completed");
        let attachment_path = params["attachmentPath"].as_str();
        let created_by = params["createdBy"].as_str().unwrap_or("");
        let voucher_number = params["voucher_number"].as_str().unwrap_or("");
        let receipt_type = params["receipt_type"].as_str().unwrap_or("");
        let receipt_path = params["receipt_path"].as_str().unwrap_or("");
        let entity = params["entity"].as_str().unwrap_or("");
        let project = params["project"].as_str().unwrap_or("");
        let supplier = params["supplier"].as_str().unwrap_or("");
        let invoice_number = params["invoice_number"].as_str().unwrap_or("");
        let tax_amount = params["tax_amount"].as_f64();
        let payment_method = params["payment_method"].as_str().unwrap_or("");
        let approver = params["approver"].as_str().unwrap_or("");
        let attachments_json = params["attachments_json"].as_str().unwrap_or("[]");
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO accounting_records (id, date, type, category, amount, description, status, attachmentPath, createdBy, createdAt, updatedAt, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
                    params![id, date, r#type, category, amount, description, status, attachment_path, created_by, now, now, voucher_number, receipt_type, receipt_path, entity, project, supplier, invoice_number, tax_amount, payment_method, approver, attachments_json],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn update_accounting_record(&self, id: &str, params: Value) -> Result<Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        // Build dynamic SET clauses
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        // ?1 will be id at the end

        let fields = [
            ("date", "date"), ("type", "type"), ("category", "category"),
            ("amount", "amount"), ("description", "description"), ("status", "status"),
            ("attachmentPath", "attachmentPath"), ("voucher_number", "voucher_number"),
            ("receipt_type", "receipt_type"), ("receipt_path", "receipt_path"),
            ("entity", "entity"), ("project", "project"), ("supplier", "supplier"),
            ("invoice_number", "invoice_number"), ("payment_method", "payment_method"),
            ("approver", "approver"), ("attachments_json", "attachments_json"),
        ];

        for (json_key, db_col) in &fields {
            if let Some(val) = params.get(*json_key) {
                if val.is_string() {
                    let idx = values.len() + 1;
                    sets.push(format!("{}=?{}", db_col, idx));
                    values.push(Box::new(val.as_str().unwrap_or("").to_string()));
                } else if val.is_number() {
                    let idx = values.len() + 1;
                    sets.push(format!("{}=?{}", db_col, idx));
                    values.push(Box::new(val.as_f64().unwrap_or(0.0)));
                }
            }
        }

        // Also accept "note" as alias for "description"
        if !sets.iter().any(|s| s.starts_with("description")) {
            if let Some(note) = params.get("note").and_then(|v| v.as_str()) {
                let idx = values.len() + 1;
                sets.push(format!("description=?{}", idx));
                values.push(Box::new(note.to_string()));
            }
        }

        // Always update updatedAt
        let idx = values.len() + 1;
        sets.push(format!("updatedAt=?{}", idx));
        values.push(Box::new(now));

        if sets.is_empty() {
            return Ok(json!({"id": id}));
        }

        // id is the last parameter
        let idx = values.len() + 1;
        let sql = format!("UPDATE accounting_records SET {} WHERE id=?{}", sets.join(", "), idx);
        values.push(Box::new(id.to_string()));

        self.with_db(|db| {
            let params_ref: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|p| p.as_ref()).collect();
            db.conn_mut()
                .execute(&sql, params_ref.as_slice())
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_accounting_record(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM accounting_records WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_accounting_stats(&self, params: Value) -> Result<Value, String> {
        let start_date = params["startDate"].as_str().unwrap_or("");
        let end_date = params["endDate"].as_str().unwrap_or("");

        let result = self.with_db(|db| {
            // Build date conditions
            let mut date_conds = Vec::new();
            let mut date_vals: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            if !start_date.is_empty() {
                date_conds.push("date >= ?");
                date_vals.push(Box::new(start_date.to_string()));
            }
            if !end_date.is_empty() {
                date_conds.push("date <= ?");
                date_vals.push(Box::new(end_date.to_string()));
            }
            let date_where = if date_conds.is_empty() { String::new() } else { format!(" WHERE {}", date_conds.join(" AND ")) };

            let query_sum = |db: &mut crate::db::Database, extra: &str| -> f64 {
                let where_clause = if date_conds.is_empty() {
                    format!(" WHERE {}", extra)
                } else {
                    format!("{} AND {}", date_where, extra)
                };
                let sql = format!("SELECT COALESCE(SUM(amount), 0) FROM accounting_records{}", where_clause);
                let vals: Vec<&dyn rusqlite::types::ToSql> = date_vals.iter().map(|v| v.as_ref()).collect();
                db.conn().query_row(&sql, vals.as_slice(), |row| row.get(0)).unwrap_or(0.0)
            };

            let total_income = query_sum(db, "type = 'income'");
            let total_expense = query_sum(db, "type = 'expense'");
            let pending_amount = query_sum(db, "status = 'pending'");
            let reimbursed_amount = query_sum(db, "status = 'reimbursed'");

            // Category breakdown (expense only)
            let cat_where = if date_conds.is_empty() {
                " WHERE type = 'expense' AND category != ''".to_string()
            } else {
                format!("{} AND type = 'expense' AND category != ''", date_where)
            };
            let cat_sql = format!("SELECT category, SUM(amount) as amount FROM accounting_records{} GROUP BY category ORDER BY amount DESC", cat_where);
            let vals: Vec<&dyn rusqlite::types::ToSql> = date_vals.iter().map(|v| v.as_ref()).collect();
            let mut cat_stmt = db.conn().prepare(&cat_sql).map_err(|e| e.to_string())?;
            let by_category: Vec<Value> = cat_stmt.query_map(vals.as_slice(), |row| {
                Ok(json!({
                    "category": row.get::<_, String>(0)?,
                    "amount": row.get::<_, f64>(1)?,
                }))
            }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

            Ok::<_, String>(json!({
                "totalIncome": total_income,
                "totalExpense": total_expense,
                "balance": total_income - total_expense,
                "pendingAmount": pending_amount,
                "reimbursedAmount": reimbursed_amount,
                "byCategory": by_category,
            }))
        });
        Ok(result?)
    }

    pub async fn get_budgets(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM budgets ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "limit": row.get::<_, f64>("limit")?,
                        "period": row.get::<_, String>("period")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let budgets: Result<Vec<Value>, _> = rows.collect();
            budgets.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_budget(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let limit = params["limit"].as_f64().unwrap_or(0.0);
        let period = params["period"].as_str().unwrap_or("monthly").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO budgets (id, name, limit, period) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, limit, period],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_budget(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let limit = params["limit"].as_f64().unwrap_or(0.0);
        let period = params["period"].as_str().unwrap_or("monthly").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE budgets SET name=?2, limit=?3, period=?4 WHERE id=?1",
                    params![id, name, limit, period],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_budget(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM budgets WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn get_templates(&self) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM templates ORDER BY name")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>("id")?,
                        "name": row.get::<_, String>("name")?,
                        "content": row.get::<_, String>("content")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let templates: Result<Vec<Value>, _> = rows.collect();
            templates.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    pub async fn add_template(&self, params: Value) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let name = params["name"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "INSERT INTO templates (id, name, content) VALUES (?1, ?2, ?3)",
                    params![id, name, content],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "name": name}))
    }

    pub async fn update_template(&self, id: &str, params: Value) -> Result<Value, String> {
        let name = params["name"].as_str().unwrap_or("").to_string();
        let content = params["content"].as_str().unwrap_or("").to_string();
        self.with_db(|db| {
            db.conn_mut()
                .execute(
                    "UPDATE templates SET name=?2, content=?3 WHERE id=?1",
                    params![id, name, content],
                )
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn delete_template(&self, id: &str) -> Result<Value, String> {
        self.with_db(|db| {
            db.conn_mut()
                .execute("DELETE FROM templates WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?;
        Ok(json!({"id": id}))
    }

    pub async fn use_template(&self, id: &str) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db
                .conn()
                .prepare("SELECT * FROM templates WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            stmt.query_row(params![id], |row| {
                Ok(json!({
                    "id": row.get::<_, String>("id")?,
                    "name": row.get::<_, String>("name")?,
                    "content": row.get::<_, String>("content")?,
                }))
            })
            .map_err(|e| e.to_string())
        });
        Ok(result?)
    }

    pub async fn get_accounting_trend(&self, months: usize) -> Result<Value, String> {
        let result = self.with_db(|db| {
            let mut stmt = db.conn().prepare(
                "SELECT strftime('%Y-%m', date) as month, type, SUM(amount) as total FROM accounting_records GROUP BY month, type ORDER BY month DESC LIMIT ?1"
            ).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![months as i64], |row| {
                    Ok(json!({
                        "month": row.get::<_, String>("month")?,
                        "type": row.get::<_, String>("type")?,
                        "total": row.get::<_, f64>("total")?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            let trends: Result<Vec<Value>, _> = rows.collect();
            trends.map_err(|e| e.to_string())
        });
        Ok(json!(result?))
    }

    // ============ LAN ============


}
