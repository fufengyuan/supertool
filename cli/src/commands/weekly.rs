use crate::output::{print_error, print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};

pub async fn cmd_weekly(
    runtime: &mut CliRuntime,
    action: &crate::types::WeeklyCommands,
) -> Result<()> {
    use crate::types::WeeklyCommands;
    match action {
        WeeklyCommands::List { limit, json } => {
            let result = runtime
                .core
                .get_weekly_reports(*limit)
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_weekly_list(&result);
            }
        }
        WeeklyCommands::Show { id, json } => {
            let result = runtime
                .core
                .get_weekly_report(*id)
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_weekly_detail(&result);
            }
        }
        WeeklyCommands::Save {
            title,
            content,
            start_date,
            end_date,
        } => {
            let data = serde_json::json!({
                "title": title, "content": content,
                "startDate": start_date.as_deref().unwrap_or(""),
                "endDate": end_date.as_deref().unwrap_or(""),
                "createdAt": chrono::Utc::now().to_rfc3339(),
            });
            let result = runtime
                .core
                .save_weekly_report(data)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success(&format!("周报已保存: {}", title));
            } else {
                print_error(&format!("保存失败: {}", result));
            }
        }
    }
    Ok(())
}

fn print_weekly_list(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无周报");
            return;
        }
        for (i, r) in arr.iter().enumerate() {
            let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let date = r.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
            println!("\x1b[1;36m[{}]\x1b[0m {}  {}", i + 1, title, date);
        }
    }
}

fn print_weekly_detail(result: &serde_json::Value) {
    let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let content = result.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let start = result
        .get("startDate")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let end = result.get("endDate").and_then(|v| v.as_str()).unwrap_or("");
    println!("\x1b[1;36m===== {}\x1b[0m", title);
    if !start.is_empty() {
        println!("周期: {} ~ {}", start, end);
    }
    println!("\n{}", content);
}
