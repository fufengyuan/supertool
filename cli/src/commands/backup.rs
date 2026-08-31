use crate::output::{print_json, print_success};
use crate::runtime::CliRuntime;
use anyhow::{Result, anyhow};
use std::io::{Cursor, Read, Write};
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

            // 打包为 ZIP（内含 all-data.json + receipts/），与 GUI 格式一致
            let mut zip_buf = Cursor::new(Vec::new());
            {
                let mut zip = zip::ZipWriter::new(&mut zip_buf);
                let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                zip.start_file("all-data.json", opts)
                    .map_err(|e| anyhow!("ZIP创建失败: {}", e))?;
                zip.write_all(data_json.as_bytes())
                    .map_err(|e| anyhow!("写入ZIP失败: {}", e))?;

                // 打包 accounting-receipts 目录到 receipts/ 路径
                let data_dir = resolve_data_dir();
                let receipt_dir = data_dir.join("accounting-receipts");
                if receipt_dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&receipt_dir) {
                        for entry in entries.flatten() {
                            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                                let filename = entry.file_name();
                                if let Ok(content) = std::fs::read(entry.path()) {
                                    let zip_path = format!("receipts/{}", filename.to_string_lossy());
                                    let _ = zip.start_file(&zip_path, opts);
                                    let _ = zip.write_all(&content);
                                }
                            }
                        }
                    }
                }
                zip.finish().map_err(|e| anyhow!("ZIP完成失败: {}", e))?;
            }

            std::fs::write(path, zip_buf.into_inner())
                .map_err(|e| anyhow!("写入文件失败: {}", e))?;

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
            let mut archive = zip::ZipArchive::new(Cursor::new(zip_data))
                .map_err(|e| anyhow!("ZIP解析失败: {}", e))?;

            // 读取 all-data.json
            let all_data_json = {
                let mut zf = archive
                    .by_name("all-data.json")
                    .map_err(|_| anyhow!("备份文件格式错误：缺少 all-data.json"))?;
                let mut content = Vec::new();
                zf.read_to_end(&mut content)
                    .map_err(|e| anyhow!("读取all-data.json失败: {}", e))?;
                String::from_utf8(content).map_err(|e| anyhow!("解码失败: {}", e))?
            };
            let data: serde_json::Value = serde_json::from_str(&all_data_json)
                .map_err(|e| anyhow!("JSON 解析失败: {}", e))?;

            // 解压 receipts/ 路径下的文件到 accounting-receipts 目录
            let data_dir = resolve_data_dir();
            let receipt_dir = data_dir.join("accounting-receipts");
            for i in 0..archive.len() {
                let mut zf = archive
                    .by_index(i)
                    .map_err(|e| anyhow!("ZIP读取失败: {}", e))?;
                let name = zf.name().to_string();
                if name.starts_with("receipts/") && !name.ends_with("/") {
                    let filename = std::path::Path::new(&name)
                        .file_name()
                        .ok_or_else(|| anyhow!("无效的收据文件路径"))?;
                    if !receipt_dir.exists() {
                        std::fs::create_dir_all(&receipt_dir)
                            .map_err(|e| anyhow!("创建收据目录失败: {}", e))?;
                    }
                    let mut content = Vec::new();
                    zf.read_to_end(&mut content)
                        .map_err(|e| anyhow!("读取收据文件失败: {}", e))?;
                    std::fs::write(receipt_dir.join(filename), content)
                        .map_err(|e| anyhow!("写入收据文件失败: {}", e))?;
                }
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
            let result = runtime
                .core
                .export_all_tables()
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
