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

            let _audit_started = std::time::Instant::now();

            let result = cmd_todo(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Subtask { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_subtask(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Project { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_project(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Server { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_server(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Cicd { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_cicd(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Db { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_db(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Log { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_log(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Git { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_git(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Mfa { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_mfa(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Note { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_note(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Accounting { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_accounting(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Weekly { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_weekly(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Nginx { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_nginx(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Backup { action } => {
            let mut rt = init_rt(cli)?;

            let _audit_started = std::time::Instant::now();

            let result = cmd_backup(&mut rt, action).await;

            audit_record(&mut rt, &cli.command, &result, _audit_started.elapsed());

            result
        }
        types::Commands::Audit { action } => {
            let mut rt = init_rt(cli)?;
            cmd_audit(&mut rt, action).await
        }
        types::Commands::Mcp { action } => {
            let mut rt = init_rt(cli)?;
            cmd_mcp(&mut rt, action).await
        }
        types::Commands::WgTunnel { conf, status } => {
            supertool_core::logic::wireguard_tunnel::run_tunnel(conf, status)
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

/// 敏感/变更操作名单：审计所有可能影响系统状态（exec/deploy/写操作）或读取敏感内容
/// （server read/db query/nginx fetch 等）的命令；纯只读列表类命令不记，避免审计噪音
fn is_write_operation(cmd: &types::Commands) -> bool {
    use types::{
        AccountingCommands as A, BackupCommands as B, CicdCommands as C, Commands as Cmd,
        DbCommands as D, GitCommands as G, LogCommands as L, MfaCommands as M, NginxCommands as N,
        NoteCommands as NT, ProjectCommands as P, ServerCommands as S, SubtaskCommands as ST,
        TodoCommands as T, WeeklyCommands as W,
    };
    match cmd {
        Cmd::Todo { action } => matches!(
            action,
            T::Add { .. } | T::Complete { .. } | T::Uncomplete { .. } | T::Delete { .. } | T::Edit { .. } | T::Clear
        ),
        Cmd::Subtask { action } => matches!(action, ST::Add { .. } | ST::Complete { .. } | ST::Delete { .. }),
        Cmd::Project { action } => matches!(action, P::Add { .. } | P::Update { .. } | P::Delete { .. }),
        Cmd::Server { action } => matches!(
            action,
            S::Add { .. }
                | S::Delete { .. }
                | S::Exec { .. }
                | S::ExecBatch { .. }
                | S::Read { .. }
                | S::Download { .. }
                | S::Mkdir { .. }
                | S::Rm { .. }
                | S::JavaRestart { .. }
        ),
        Cmd::Cicd { action } => matches!(action, C::Deploy { .. } | C::Rollback { .. } | C::Cancel { .. }),
        Cmd::Db { action } => matches!(action, D::Query { .. } | D::Redis { .. } | D::Disconnect { .. }),
        Cmd::Log { action } => matches!(action, L::Add { .. } | L::Delete { .. }),
        Cmd::Git { action } => matches!(action, G::Pull { .. } | G::Push { .. } | G::Commit { .. } | G::Checkout { .. }),
        Cmd::Mfa { action } => matches!(action, M::Add { .. } | M::Delete { .. }),
        Cmd::Note { action } => matches!(
            action,
            NT::Add { .. }
                | NT::Update { .. }
                | NT::Delete { .. }
                | NT::AddGroup { .. }
                | NT::UpdateGroup { .. }
                | NT::DeleteGroup { .. }
        ),
        Cmd::Accounting { action } => matches!(
            action,
            A::Add { .. }
                | A::Update { .. }
                | A::Delete { .. }
                | A::AddCategory { .. }
                | A::DeleteCategory { .. }
                | A::AddBudget { .. }
                | A::DeleteBudget { .. }
        ),
        Cmd::Weekly { action } => matches!(action, W::Save { .. }),
        Cmd::Nginx { action } => matches!(
            action,
            N::Add { .. }
                | N::Update { .. }
                | N::Delete { .. }
                | N::Fetch { .. }
                | N::Test { .. }
                | N::Deploy { .. }
        ),
        Cmd::Backup { action } => matches!(action, B::Export { .. } | B::Import { .. } | B::ExportCsv),
        _ => false,
    }
}

/// 审计记录（仅写操作）：命令描述（含参数）经 log_sanitizer 脱敏后入库，
/// 审计失败静默忽略，不阻塞主流程
fn audit_record(
    rt: &mut CliRuntime,
    cmd: &types::Commands,
    result: &Result<(), anyhow::Error>,
    elapsed: std::time::Duration,
) {
    if !is_write_operation(cmd) {
        return;
    }
    let desc =
        supertool_core::logic::log_sanitizer::sanitize_string(&format!("{:?}", cmd));
    let desc: String = desc.chars().take(500).collect();
    let status = if result.is_ok() { "success" } else { "failed" };
    let _ = rt.core.record_audit(
        "cli",
        "",
        &desc,
        "",
        "",
        status,
        elapsed.as_millis() as i64,
    );
}
