use crate::output::{print_error, print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};

pub async fn cmd_accounting(
    runtime: &mut CliRuntime,
    action: &crate::types::AccountingCommands,
) -> Result<()> {
    use crate::types::AccountingCommands;
    match action {
        AccountingCommands::List {
            category,
            r#type,
            year,
            month,
            json,
        } => {
            let params = serde_json::json!({
                "category": category, "type": r#type, "year": year, "month": month
            });
            let result = runtime
                .core
                .get_accounting_records(params)
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_accounting_list(&result);
            }
        }
        AccountingCommands::Add {
            amount,
            category,
            r#type,
            note,
            date,
        } => {
            let data = serde_json::json!({
                "amount": amount, "category": category, "type": r#type,
                "note": note.as_deref().unwrap_or(""),
                "date": date.as_deref().unwrap_or(&chrono::Local::now().format("%Y-%m-%d").to_string()),
                "createdAt": chrono::Utc::now().to_rfc3339(),
            });
            let result = runtime
                .core
                .add_accounting_record(data)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success(&format!("已添加记录: {} {}", r#type, amount));
            } else {
                print_error(&format!("添加失败: {}", result));
            }
        }
        AccountingCommands::Update {
            id,
            amount,
            category,
            r#type,
            note,
        } => {
            let mut updates = serde_json::Map::new();
            if let Some(a) = amount {
                updates.insert(
                    "amount".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(*a).unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }
            if let Some(c) = category {
                updates.insert("category".to_string(), serde_json::Value::String(c.clone()));
            }
            if let Some(t) = r#type {
                updates.insert("type".to_string(), serde_json::Value::String(t.clone()));
            }
            if let Some(n) = note {
                updates.insert("note".to_string(), serde_json::Value::String(n.clone()));
            }
            let result = runtime
                .core
                .update_accounting_record(id, serde_json::Value::Object(updates))
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success("记录已更新");
            } else {
                print_error(&format!("更新失败: {}", result));
            }
        }
        AccountingCommands::Delete { id } => {
            let result = runtime
                .core
                .delete_accounting_record(id)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success("记录已删除");
            } else {
                print_error(&format!("删除失败: {}", result));
            }
        }
        AccountingCommands::Categories { json } => {
            let result = runtime
                .core
                .get_accounting_categories()
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_categories(&result);
            }
        }
        AccountingCommands::AddCategory { name, icon, color } => {
            let data = serde_json::json!({ "name": name, "icon": icon.as_deref().unwrap_or(""), "color": color.as_deref().unwrap_or("") });
            let result = runtime
                .core
                .add_accounting_category(data)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id, name} on success
            if result.get("id").is_some() {
                print_success(&format!("分类已添加: {}", name));
            } else {
                print_error(&format!("添加失败: {}", result));
            }
        }
        AccountingCommands::DeleteCategory { id } => {
            let result = runtime
                .core
                .delete_accounting_category(id)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success("分类已删除");
            } else {
                print_error(&format!("删除失败: {}", result));
            }
        }
        AccountingCommands::Budgets { json } => {
            let result = runtime.core.get_budgets().await.map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_budgets(&result);
            }
        }
        AccountingCommands::AddBudget {
            category,
            amount,
            month,
        } => {
            let data = serde_json::json!({ "category": category, "amount": amount, "month": month.as_deref().unwrap_or("") });
            let result = runtime
                .core
                .add_budget(data)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id, name} on success
            if result.get("id").is_some() {
                print_success(&format!("预算已添加: {} {}", category, amount));
            } else {
                print_error(&format!("添加失败: {}", result));
            }
        }
        AccountingCommands::DeleteBudget { id } => {
            let result = runtime
                .core
                .delete_budget(id)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns {id} on success
            if result.get("id").is_some() {
                print_success("预算已删除");
            } else {
                print_error(&format!("删除失败: {}", result));
            }
        }
        AccountingCommands::Stats { year, json } => {
            let params = serde_json::json!({ "year": year });
            let result = runtime
                .core
                .get_accounting_stats(params)
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_stats(&result);
            }
        }
        AccountingCommands::Trend { months, json } => {
            let result = runtime
                .core
                .get_accounting_trend(*months)
                .await
                .map_err(|e| anyhow!(e))?;
            if *json {
                print_json(&result);
            } else {
                print_trend(&result);
            }
        }
    }
    Ok(())
}

fn print_accounting_list(result: &serde_json::Value) {
    let records = result.get("records").and_then(|v| v.as_array());
    if let Some(arr) = records {
        if arr.is_empty() {
            println!("暂无账单记录");
            return;
        }
        for (i, r) in arr.iter().enumerate() {
            let amount = r.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let r#type = r.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let category = r
                .get("categoryName")
                .and_then(|v| v.as_str())
                .or_else(|| r.get("category").and_then(|v| v.as_str()))
                .unwrap_or("");
            let date = r.get("date").and_then(|v| v.as_str()).unwrap_or("");
            let note = r.get("note").and_then(|v| v.as_str()).unwrap_or("");
            let color = if r#type == "收入" {
                "\x1b[32m"
            } else {
                "\x1b[31m"
            };
            println!(
                "{}{:<4} {:<12.2} {}{:<10}\x1b[0m {:<12} {:<10} {}",
                color,
                i + 1,
                amount,
                color,
                r#type,
                category,
                date,
                note
            );
        }
    }
}

fn print_categories(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无分类");
            return;
        }
        for c in arr {
            let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let icon = c.get("icon").and_then(|v| v.as_str()).unwrap_or("");
            println!("{}  {}", icon, name);
        }
    }
}

fn print_budgets(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        if arr.is_empty() {
            println!("暂无预算");
            return;
        }
        for b in arr {
            let category = b.get("category").and_then(|v| v.as_str()).unwrap_or("");
            let amount = b.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let month = b.get("month").and_then(|v| v.as_str()).unwrap_or("");
            println!("{}  {}  {:.2}", month, category, amount);
        }
    }
}

fn print_stats(result: &serde_json::Value) {
    let income = result.get("income").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let expense = result
        .get("expense")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let balance = income - expense;
    println!("收入: \x1b[32m{:.2}\x1b[0m", income);
    println!("支出: \x1b[31m{:.2}\x1b[0m", expense);
    println!("结余: \x1b[36m{:.2}\x1b[0m", balance);
}

fn print_trend(result: &serde_json::Value) {
    if let Some(arr) = result.as_array() {
        for t in arr {
            let month = t.get("month").and_then(|v| v.as_str()).unwrap_or("");
            let income = t.get("income").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let expense = t.get("expense").and_then(|v| v.as_f64()).unwrap_or(0.0);
            println!("{}  收入:{:.2}  支出:{:.2}", month, income, expense);
        }
    }
}
