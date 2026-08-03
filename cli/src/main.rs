mod commands;
mod guide;
mod output;
mod runtime;
mod types;
mod utils;

use clap::Parser;
use commands::*;
use output::print_error;
use runtime::CliRuntime;
use std::process;
use types::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code = run(&cli).await;
    process::exit(code);
}

/// 统一执行入口：所有命令的错误处理与 exit code 规范化都在这里收口。
/// - JSON 模式（全局 --json 或命令级 -j）下错误输出 `{"ok": false, "error": {...}}` 到 stderr
/// - exit code 规范：0=成功 1=业务错误 2=参数错误(clap) 3=需审批 4=连接失败 5=高危拦截
async fn run(cli: &Cli) -> i32 {
    // 全局 --json 开启时置位 JSON 模式（命令级 -j 由各命令内部处理）
    if cli.json {
        output::set_json_mode(true);
    }

    match dispatch(cli).await {
        Ok(()) => output::EXIT_OK,
        Err(e) => {
            let code = output::exit_code_for(&e);
            let raw = e.to_string();
            let msg = output::strip_error_code_prefix(&raw);
            if output::is_json_mode() {
                output::print_error_json(code, msg);
            } else {
                print_error(msg);
            }
            code
        }
    }
}

/// 命令分发（独立函数以便各分支使用 `?` 传播 init 错误）
async fn dispatch(cli: &Cli) -> Result<(), anyhow::Error> {
    match &cli.command {
        types::Commands::Version => {
            if output::is_json_mode() {
                output::print_json(&serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }));
            } else {
                println!("SuperTool CLI v{}", env!("CARGO_PKG_VERSION"));
            }
            Ok(())
        }
        types::Commands::Guide => {
            guide::print_guide();
            Ok(())
        }
        types::Commands::Todo { action } => {
            let mut rt = init_rt(cli)?;
            cmd_todo(&mut rt, action).await
        }
        types::Commands::Subtask { action } => {
            let mut rt = init_rt(cli)?;
            cmd_subtask(&mut rt, action).await
        }
        types::Commands::Project { action } => {
            let mut rt = init_rt(cli)?;
            cmd_project(&mut rt, action).await
        }
        types::Commands::Server { action } => {
            let mut rt = init_rt(cli)?;
            cmd_server(&mut rt, action).await
        }
        types::Commands::Cicd { action } => {
            let mut rt = init_rt(cli)?;
            cmd_cicd(&mut rt, action).await
        }
        types::Commands::Db { action } => {
            let mut rt = init_rt(cli)?;
            cmd_db(&mut rt, action).await
        }
        types::Commands::Log { action } => {
            let mut rt = init_rt(cli)?;
            cmd_log(&mut rt, action).await
        }
        types::Commands::Git { action } => {
            let mut rt = init_rt(cli)?;
            cmd_git(&mut rt, action).await
        }
        types::Commands::Mfa { action } => {
            let mut rt = init_rt(cli)?;
            cmd_mfa(&mut rt, action).await
        }
        types::Commands::Note { action } => {
            let mut rt = init_rt(cli)?;
            cmd_note(&mut rt, action).await
        }
        types::Commands::Accounting { action } => {
            let mut rt = init_rt(cli)?;
            cmd_accounting(&mut rt, action).await
        }
        types::Commands::Weekly { action } => {
            let mut rt = init_rt(cli)?;
            cmd_weekly(&mut rt, action).await
        }
        types::Commands::Nginx { action } => {
            let mut rt = init_rt(cli)?;
            cmd_nginx(&mut rt, action).await
        }
        types::Commands::Backup { action } => {
            let mut rt = init_rt(cli)?;
            cmd_backup(&mut rt, action).await
        }
        types::Commands::WgTunnel { conf, uds } => {
            supertool_core::logic::wireguard_tunnel::run_tunnel(conf, uds)
                .await
                .map_err(|e| output::fail(output::EXIT_BUSINESS, format!("[wg-tunnel] {}", e)))
        }
    }
}

/// 初始化 runtime 并注入全局 --json 模式
fn init_rt(cli: &Cli) -> Result<CliRuntime, anyhow::Error> {
    let mut rt = CliRuntime::init()
        .map_err(|e| output::fail(output::EXIT_BUSINESS, format!("初始化失败: {}", e)))?;
    rt.json_mode = cli.json;
    Ok(rt)
}
