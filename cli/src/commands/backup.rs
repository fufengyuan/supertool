use crate::runtime::CliRuntime;
use crate::output::{print_error, print_success};
use anyhow::{Result, anyhow};
use std::fs;

pub async fn cmd_backup(runtime: &mut CliRuntime, action: &crate::types::BackupCommands) -> Result<()> {
    use crate::types::BackupCommands;
    match action {
        BackupCommands::Export { output } => {
            let result = runtime.core.export_all_data().await.map_err(|e| anyhow!(e))?;
            let data_json = serde_json::to_string_pretty(&result).map_err(|e| anyhow!("序列化失败: {}", e))?;

            let default_name = format!("supertool-backup-{}.json", chrono::Local::now().format("%Y-%m-%d"));
            let path = output.as_deref().unwrap_or(&default_name);

            fs::write(path, data_json).map_err(|e| anyhow!("写入文件失败: {}", e))?;

            let table_count = result.as_object().map(|o| o.len()).unwrap_or(0);
            let total_items: usize = result.as_object().map(|o| {
                o.values().map(|v| v.as_array().map(|a| a.len()).unwrap_or(0)).sum()
            }).unwrap_or(0);
            print_success(&format!("数据已导出到: {}", path));
            println!("  表数: {}, 总记录数: {}", table_count, total_items);
        }
        BackupCommands::Import { file, mode } => {
            let content = fs::read_to_string(file).map_err(|e| anyhow!("读取文件失败: {}", e))?;
            let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| anyhow!("JSON 解析失败: {}", e))?;
            let result = runtime.core.import_all_data(data, mode).await.map_err(|e| anyhow!(e))?;
            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                print_success("数据导入成功");
            } else {
                let imported = result.get("importedCount").and_then(|v| v.as_u64()).unwrap_or(0);
                let skipped = result.get("skippedCount").and_then(|v| v.as_u64()).unwrap_or(0);
                let errors = result.get("errors").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                println!("导入: {}, 跳过: {}, 错误: {}", imported, skipped, errors);
                if errors > 0 {
                    if let Some(errs) = result.get("errors").and_then(|v| v.as_array()) {
                        for e in errs.iter().take(5) {
                            print_error(&format!("  {}", e.as_str().unwrap_or("?")));
                        }
                    }
                }
            }
        }
        BackupCommands::ExportCsv => {
            let result = runtime.core.export_all_data().await.map_err(|e| anyhow!(e))?;
            let todos = result.get("todos").and_then(|t| t.as_array());
            let mut csv = String::from("id,text,completed,priority,createdAt,dueDate\n");
            if let Some(arr) = todos {
                for todo in arr {
                    let id = todo.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let text = todo.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let completed = todo.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
                    let priority = todo.get("priority").and_then(|v| v.as_str()).unwrap_or("");
                    let created = todo.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
                    let due = todo.get("dueDate").and_then(|v| v.as_str()).unwrap_or("");
                    let text_escaped = text.replace('"', "\"\"");
                    csv.push_str(&format!("{},\"{}\",{},{},{},{}\n", id, text_escaped, completed, priority, created, due));
                }
            }
            println!("{}", csv);
        }
    }
    Ok(())
}
