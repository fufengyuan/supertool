use crate::output::{print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};
use supertool_core::logic::data_dir::resolve_data_dir;

pub async fn cmd_backup(
    runtime: &mut CliRuntime,
    action: &crate::types::BackupCommands,
) -> Result<()> {
    use crate::types::BackupCommands;
    match action {
        BackupCommands::Export { output, json } => {
            runtime.set_json(*json);
            let result = runtime
                .core
                .export_all_tables()
                .await
                .map_err(|e| anyhow!(e))?;
            let data_json =
                serde_json::to_string_pretty(&result).map_err(|e| anyhow!("序列化失败: {}", e))?;

            let default_name = format!(
                "supertool-backup-{}.stbackup",
                chrono::Local::now().format("%Y-%m-%d")
            );
            let path = output.as_deref().unwrap_or(&default_name);

            // 打包为 ZIP（内含 all-data.json + receipts/），统一走 core 的 write_backup_zip
            supertool_core::logic::backup::write_backup_zip(&data_json, std::path::Path::new(path))
                .map_err(|e| anyhow!(e))?;

            let table_count = result.as_object().map(|o| o.len()).unwrap_or(0);
            let total_items: usize = result
                .as_object()
                .map(|o| {
                    o.values()
                        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
                        .sum()
                })
                .unwrap_or(0);
            if runtime.json_mode {
                print_json(&serde_json::json!({
                    "path": path,
                    "tableCount": table_count,
                    "totalItems": total_items,
                }));
            } else {
                print_success(&format!("数据已导出到: {}", path));
                println!("  表数: {}, 总记录数: {}", table_count, total_items);
            }
        }
        BackupCommands::Import { file, mode, json } => {
            runtime.set_json(*json);
            let zip_data = std::fs::read(file).map_err(|e| anyhow!("读取文件失败: {}", e))?;

            // 解包（all-data.json + receipts/），统一走 core 的 read_backup_zip
            let (all_data_json, receipts) = supertool_core::logic::backup::read_backup_zip(&zip_data)
                .map_err(|e| anyhow!(e))?;
            let data: serde_json::Value = serde_json::from_str(&all_data_json)
                .map_err(|e| anyhow!("JSON 解析失败: {}", e))?;

            // 解压 receipts/ 路径下的文件到 accounting-receipts 目录
            let data_dir = resolve_data_dir();
            let receipt_dir = data_dir.join("accounting-receipts");
            for (name, content) in receipts {
                if !receipt_dir.exists() {
                    std::fs::create_dir_all(&receipt_dir)
                        .map_err(|e| anyhow!("创建收据目录失败: {}", e))?;
                }
                let filename = std::path::Path::new(&name)
                    .file_name()
                    .ok_or_else(|| anyhow!("无效的收据文件路径"))?;
                std::fs::write(receipt_dir.join(filename), content)
                    .map_err(|e| anyhow!("写入收据文件失败: {}", e))?;
            }

            let (imported, skipped, import_errors, path_rewritten) = runtime
                .core
                .import_all_tables(data, mode)
                .await
                .map_err(|e| anyhow!(e))?;

            if import_errors.is_empty() {
                if runtime.json_mode {
                    print_json(&serde_json::json!({"imported": imported, "skipped": skipped, "pathRewritten": path_rewritten, "errors": []}));
                } else {
                    print_success(&format!(
                        "数据导入成功: 导入 {} 条, 跳过 {} 条, 路径改写 {} 处",
                        imported, skipped, path_rewritten
                    ));
                }
            } else {
                if runtime.json_mode {
                    print_json(&serde_json::json!({
                        "imported": imported,
                        "skipped": skipped,
                        "pathRewritten": path_rewritten,
                        "errors": import_errors.iter().take(10).collect::<Vec<_>>(),
                    }));
                } else {
                    print_success(&format!(
                        "数据导入完成（含 {} 个错误）: 导入 {} 条, 跳过 {} 条, 路径改写 {} 处",
                        import_errors.len(),
                        imported,
                        skipped,
                        path_rewritten
                    ));
                    for e in import_errors.iter().take(10) {
                        println!("  - {}", e);
                    }
                }
            }
        }
        BackupCommands::ExportCsv => {
            let csv = runtime
                .core
                .export_todos_csv()
                .await
                .map_err(|e| anyhow!(e))?;
            println!("{}", csv);
        }
    }
    Ok(())
}
