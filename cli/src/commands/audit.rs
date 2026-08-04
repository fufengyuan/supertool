use crate::output::print_json;
use crate::runtime::CliRuntime;
use crate::types::AuditCommands;
use anyhow::Result;

pub async fn cmd_audit(runtime: &mut CliRuntime, action: &AuditCommands) -> Result<()> {
    match action {
        AuditCommands::List {
            actor,
            result,
            limit,
            json,
        } => {
            runtime.set_json(*json);
            let entries = runtime
                .core
                .list_audit(actor.as_deref(), result.as_deref(), *limit)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if runtime.json_mode {
                print_json(&entries);
                return Ok(());
            }
            let arr = entries.as_array().cloned().unwrap_or_default();
            if arr.is_empty() {
                println!("  暂无审计记录");
                return Ok(());
            }
            println!("\n  操作审计 (最近 {} 条):", arr.len());
            println!("  {}", "─".repeat(80));
            for e in &arr {
                let ts = e.get("createdAt").and_then(|v| v.as_str()).unwrap_or("");
                let act = e.get("actorType").and_then(|v| v.as_str()).unwrap_or("");
                let st = e.get("result").and_then(|v| v.as_str()).unwrap_or("");
                let cmd = e.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let icon = match st {
                    "success" => "✅",
                    "failed" => "❌",
                    "blocked" => "🚫",
                    _ => "•",
                };
                // 命令描述可能很长，截断展示
                let cmd_short: String = cmd.chars().take(90).collect();
                println!("  {} {} | {} | {}ms | {}", icon, ts, act, e.get("durationMs").and_then(|v| v.as_i64()).unwrap_or(0), cmd_short);
            }
        }
    }
    Ok(())
}
