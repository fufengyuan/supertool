use crate::output::{print_error, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};
use std::fs;

pub async fn cmd_backup(
    runtime: &mut CliRuntime,
    action: &crate::types::BackupCommands,
) -> Result<()> {
    use crate::types::BackupCommands;
    match action {
        BackupCommands::Export { output } => {
            let result = runtime
                .core
                .export_all_data()
                .await
                .map_err(|e| anyhow!(e))?;
            let data_json =
                serde_json::to_string_pretty(&result).map_err(|e| anyhow!("序列化失败: {}", e))?;

            let default_name = format!(
                "supertool-backup-{}.json",
                chrono::Local::now().format("%Y-%m-%d")
            );
            let path = output.as_deref().unwrap_or(&default_name);

            fs::write(path, data_json).map_err(|e| anyhow!("写入文件失败: {}", e))?;

            let table_count = result.as_object().map(|o| o.len()).unwrap_or(0);
            let total_items: usize = result
                .as_object()
                .map(|o| {
                    o.values()
                        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
                        .sum()
                })
                .unwrap_or(0);
            print_success(&format!("数据已导出到: {}", path));
            println!("  表数: {}, 总记录数: {}", table_count, total_items);
        }
        BackupCommands::Import { file, mode } => {
            let content = fs::read_to_string(file).map_err(|e| anyhow!("读取文件失败: {}", e))?;
            let data: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| anyhow!("JSON 解析失败: {}", e))?;
            let result = runtime
                .core
                .import_all_data(data, mode)
                .await
                .map_err(|e| anyhow!(e))?;
            // Core returns counts hashmap like {"todos": 10, "servers": 5}
            // Check if any items were imported
            let total_imported: u64 = result
                .as_object()
                .map(|o| o.values().filter_map(|v| v.as_u64()).sum())
                .unwrap_or(0);
            if total_imported > 0 {
                print_success(&format!("数据导入成功: {} 条记录", total_imported));
                for (table, count) in result.as_object().unwrap_or(&serde_json::Map::new()) {
                    if let Some(c) = count.as_u64() {
                        if c > 0 {
                            println!("  {}: {}", table, c);
                        }
                    }
                }
            } else {
                print_error("导入失败: 无数据导入");
            }
        }
        BackupCommands::ExportCsv => {
            let result = runtime
                .core
                .export_all_data()
                .await
                .map_err(|e| anyhow!(e))?;
            let todos = result.get("todos").and_then(|t| t.as_array());
            let mut csv = String::from("id,text,completed,priority,createdAt,dueDate\n");
            if let Some(arr) = todos {
                for todo in arr {
                    let id = todo.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let text = todo.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let completed = todo
                        .get("completed")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let priority = todo.get("priority").and_then(|v| v.as_str()).unwrap_or("");
                    let created = todo.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
                    let due = todo.get("dueDate").and_then(|v| v.as_str()).unwrap_or("");
                    let text_escaped = text.replace('"', "\"\"");
                    csv.push_str(&format!(
                        "{},\"{}\",{},{},{},{}\n",
                        id, text_escaped, completed, priority, created, due
                    ));
                }
            }
            println!("{}", csv);
        }
    }
    Ok(())
}
