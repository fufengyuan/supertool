mod types;
mod transport;
mod output;
mod utils;
mod guide;
mod commands;

use clap::Parser;
use types::Cli;
use transport::ApiClient;
use output::print_error;
use commands::*;
use std::process;

fn main() {
    let cli = Cli::parse();
    let client = ApiClient::new();

    let result = match &cli.command {
        types::Commands::Version => { println!("SuperTool CLI v{}", env!("CARGO_PKG_VERSION")); Ok(()) }
        types::Commands::Guide => { guide::print_guide(); Ok(()) }
        types::Commands::Todo { action } => cmd_todo(&client, action),
        types::Commands::Subtask { action } => cmd_subtask(&client, action),
        types::Commands::Project { action } => cmd_project(&client, action),
        types::Commands::Server { action } => cmd_server(&client, action),
        types::Commands::Cicd { action } => cmd_cicd(&client, action),
        types::Commands::Db { action } => cmd_db(&client, action),
        types::Commands::Log { action } => cmd_log(&client, action),
        types::Commands::Git { action } => cmd_git(&client, action),
    };

    if let Err(e) = result { print_error(&e.to_string()); process::exit(1); }
}
