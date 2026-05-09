mod types;
mod output;
mod utils;
mod guide;
mod commands;
mod runtime;

use clap::Parser;
use types::Cli;
use output::print_error;
use commands::*;
use runtime::CliRuntime;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        types::Commands::Version => {
            println!("SuperTool CLI v{}", env!("CARGO_PKG_VERSION"));
        }
        types::Commands::Guide => {
            guide::print_guide();
        }
        types::Commands::Todo { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_todo(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Subtask { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_subtask(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Project { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_project(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Server { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_server(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Cicd { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_cicd(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Db { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_db(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Log { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_log(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Git { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_git(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Mfa { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_mfa(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Note { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_note(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Accounting { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_accounting(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Weekly { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_weekly(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Nginx { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_nginx(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
        types::Commands::Backup { action } => {
            let mut rt = match init_runtime() {
                Ok(r) => r,
                Err(e) => { print_error(&format!("初始化失败: {}", e)); process::exit(1); }
            };
            if let Err(e) = cmd_backup(&mut rt, action).await { print_error(&e.to_string()); process::exit(1); }
        }
    }
}

fn init_runtime() -> Result<CliRuntime, anyhow::Error> {
    CliRuntime::init()
}
