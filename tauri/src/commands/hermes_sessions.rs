//! Hermes Sessions management — direct state.db operations.

use hermes_config::paths;

fn state_db_path() -> std::path::PathBuf {
    paths::hermes_home().join("state.db")
}

fn open_db() -> Result<rusqlite::Connection, String> {
    let db_path = state_db_path();
    if !db_path.exists() {
        return Err("No state.db found".to_string());
    }
    rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open state.db: {e}"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn sessions_export(
    output: String,
    source: Option<String>,
    session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = open_db()?;

    let mut where_clauses = Vec::new();
    if let Some(ref src) = source {
        if !src.is_empty() {
            where_clauses.push(format!("s.source = '{}'", src.replace('\'', "''")));
        }
    }
    if let Some(ref sid) = session_id {
        if !sid.is_empty() {
            where_clauses.push(format!("s.id = '{}'", sid.replace('\'', "''")));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Query sessions
    let query = format!(
        "SELECT s.id, s.source, s.title, s.model, \
                s.started_at, s.ended_at, s.message_count, \
                s.input_tokens, s.output_tokens, s.estimated_cost_usd, \
                s.system_prompt \
         FROM sessions s {} \
         ORDER BY s.started_at DESC",
        where_sql
    );

    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("Query failed: {e}"))?;

    let sessions: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "source": row.get::<_, String>(1)?,
                "title": row.get::<_, Option<String>>(2)?,
                "model": row.get::<_, Option<String>>(3)?,
                "startedAt": row.get::<_, f64>(4)?,
                "endedAt": row.get::<_, Option<f64>>(5)?,
                "messageCount": row.get::<_, i64>(6)?,
                "inputTokens": row.get::<_, i64>(7)?,
                "outputTokens": row.get::<_, i64>(8)?,
                "estimatedCost": row.get::<_, Option<f64>>(9)?,
                "systemPrompt": row.get::<_, Option<String>>(10)?,
            }))
        })
        .map_err(|e| format!("Row mapping failed: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    let jsonl: String = sessions
        .iter()
        .map(|s| serde_json::to_string(s).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    if output == "-" {
        // Return to stdout
        Ok(serde_json::json!({
            "success": true,
            "output": jsonl,
            "session_count": sessions.len(),
        }))
    } else {
        // Write to file
        std::fs::write(&output, &jsonl)
            .map_err(|e| format!("Failed to write output file: {e}"))?;
        Ok(serde_json::json!({
            "success": true,
            "output": format!("Exported {} sessions to {}", sessions.len(), output),
            "session_count": sessions.len(),
        }))
    }
}

#[tauri::command(rename_all = "camelCase")]
pub fn sessions_prune(
    older_than: Option<i32>,
    source: Option<String>,
    yes: Option<bool>,
) -> Result<serde_json::Value, String> {
    let conn = open_db()?;
    let days = older_than.unwrap_or(30);

    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_ts = cutoff.timestamp() as f64;

    // Count sessions to be pruned
    let mut count_sql = format!(
        "SELECT COUNT(*) FROM sessions WHERE ended_at IS NOT NULL AND ended_at < {}",
        cutoff_ts
    );
    if let Some(ref src) = source {
        if !src.is_empty() {
            count_sql.push_str(&format!(" AND source = '{}'", src.replace('\'', "''")));
        }
    }

    let count: i64 = conn
        .query_row(&count_sql, [], |row| row.get(0))
        .unwrap_or(0);

    if count == 0 {
        return Ok(serde_json::json!({
            "success": true,
            "output": "No sessions to prune.",
            "pruned": 0,
        }));
    }

    if !yes.unwrap_or(false) {
        return Ok(serde_json::json!({
            "success": true,
            "output": format!("Would prune {} sessions. Use --yes to confirm.", count),
            "dry_run": true,
            "pruned": count,
        }));
    }

    // Delete messages first, then sessions
    let mut delete_msgs = format!(
        "DELETE FROM messages WHERE session_id IN \
         (SELECT id FROM sessions WHERE ended_at IS NOT NULL AND ended_at < {})",
        cutoff_ts
    );
    if let Some(ref src) = source {
        if !src.is_empty() {
            delete_msgs.push_str(&format!(" AND source = '{}'", src.replace('\'', "''")));
        }
    }

    conn.execute(&delete_msgs, [])
        .map_err(|e| format!("Failed to delete messages: {e}"))?;

    let mut delete_sessions = format!(
        "DELETE FROM sessions WHERE ended_at IS NOT NULL AND ended_at < {}",
        cutoff_ts
    );
    if let Some(ref src) = source {
        if !src.is_empty() {
            delete_sessions.push_str(&format!(" AND source = '{}'", src.replace('\'', "''")));
        }
    }

    conn.execute(&delete_sessions, [])
        .map_err(|e| format!("Failed to delete sessions: {e}"))?;

    Ok(serde_json::json!({
        "success": true,
        "output": format!("Pruned {} sessions older than {} days.", count, days),
        "pruned": count,
    }))
}
