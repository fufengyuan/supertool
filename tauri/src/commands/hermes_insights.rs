//! Hermes Insights / usage analytics — direct state.db queries.

use hermes_config::paths;

fn state_db_path() -> std::path::PathBuf {
    paths::hermes_home().join("state.db")
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_insights(
    days: Option<i32>,
    source: Option<String>,
) -> Result<serde_json::Value, String> {
    let db_path = state_db_path();
    if !db_path.exists() {
        return Ok(serde_json::json!({
            "success": true,
            "output": "No state.db found — no session data available.",
            "error": "",
        }));
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open state.db: {e}"))?;

    let days = days.unwrap_or(7);
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_ts = cutoff.timestamp() as f64;

    let mut where_clauses = vec![format!("s.started_at >= {}", cutoff_ts)];
    let mut source_filter = String::new();
    if let Some(ref src) = source {
        if !src.is_empty() {
            where_clauses.push(format!("'{}'", src)); // placeholder, resolved below
            source_filter = src.clone();
        }
    }

    let where_sql = if source_filter.is_empty() {
        where_clauses[0].clone()
    } else {
        format!("s.started_at >= {} AND s.source = '{}'", cutoff_ts, source_filter.replace('\'', "''"))
    };

    // Total sessions
    let total_sessions: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM sessions s WHERE {}", where_sql),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total messages
    let total_messages: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM messages m \
                 JOIN sessions s ON m.session_id = s.id WHERE {}",
                where_sql
            ),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total tokens
    let total_input: i64 = conn
        .query_row(
            &format!("SELECT COALESCE(SUM(input_tokens), 0) FROM sessions s WHERE {}", where_sql),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_output: i64 = conn
        .query_row(
            &format!("SELECT COALESCE(SUM(output_tokens), 0) FROM sessions s WHERE {}", where_sql),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total cost
    let total_cost: f64 = conn
        .query_row(
            &format!("SELECT COALESCE(SUM(estimated_cost_usd), 0.0) FROM sessions s WHERE {}", where_sql),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    // Top models
    let mut models = Vec::new();
    let model_query = format!(
        "SELECT COALESCE(s.model, 'unknown') as model, COUNT(*) as cnt \
         FROM sessions s WHERE {} \
         GROUP BY model ORDER BY cnt DESC LIMIT 10",
        where_sql
    );
    if let Ok(mut stmt) = conn.prepare(&model_query) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let model: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((model, count))
        }) {
            for row in rows.flatten() {
                models.push(serde_json::json!({
                    "model": row.0,
                    "count": row.1,
                }));
            }
        }
    }

    // Daily breakdown
    let mut daily = Vec::new();
    let daily_query = format!(
        "SELECT DATE(s.started_at, 'unixepoch') as day, \
         COUNT(DISTINCT s.id) as sessions, \
         COUNT(m.id) as messages \
         FROM sessions s \
         LEFT JOIN messages m ON m.session_id = s.id \
         WHERE {} \
         GROUP BY day ORDER BY day",
        where_sql
    );
    if let Ok(mut stmt) = conn.prepare(&daily_query) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let day: String = row.get(0)?;
            let sessions: i64 = row.get(1)?;
            let messages: i64 = row.get(2)?;
            Ok((day, sessions, messages))
        }) {
            for row in rows.flatten() {
                daily.push(serde_json::json!({
                    "date": row.0,
                    "sessions": row.1,
                    "messages": row.2,
                }));
            }
        }
    }

    let output = format!(
        r#"📊 Hermes Insights (last {} days)
━━━━━━━━━━━━━━━━━━━━━━━━━━
📈 Sessions: {}
💬 Messages: {}
🔤 Input tokens: {}
🔤 Output tokens: {}
💰 Estimated cost: ${:.4}
📋 Top models: {}"#,
        days,
        total_sessions,
        total_messages,
        total_input,
        total_output,
        total_cost,
        if models.is_empty() {
            "none".to_string()
        } else {
            models
                .iter()
                .map(|m| format!("{} ({})", m["model"], m["count"]))
                .collect::<Vec<_>>()
                .join(", ")
        },
    );

    Ok(serde_json::json!({
        "success": true,
        "output": output,
        "error": "",
        "stats": {
            "totalSessions": total_sessions,
            "totalMessages": total_messages,
            "totalInputTokens": total_input,
            "totalOutputTokens": total_output,
            "totalCost": total_cost,
            "models": models,
            "daily": daily,
        },
    }))
}
