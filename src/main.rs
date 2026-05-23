mod agent;
mod analytics;
mod cli;
mod config;
mod error;
mod export;
mod export_cmd;
mod help;
mod input;
mod logging;
mod metrics;
mod monitor;
mod network;
mod performance;
mod rendering;
mod theme;
mod tui;
mod ui;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use cli::{Cli, Commands};
use error::Result;
use std::io;
use std::process;

async fn run() -> Result<()> {
    match Cli::parse().resolved_command() {
        Commands::Monitor(args) => tui::run(args).await,
        Commands::Agent { bind, port } => agent::run(&bind, port).await,
        Commands::Export(args) => export_cmd::run(&args.json, &args.csv, args.remote.as_deref()),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let shell: Shell = shell.into();
            generate(shell, &mut cmd, "grainx", &mut io::stdout());
            Ok(())
        }
        Commands::Version => {
            println!("grainx {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let exit_code = match run().await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("Error: {err}");
            err.exit_code()
        }
    };
    process::exit(exit_code);
}
