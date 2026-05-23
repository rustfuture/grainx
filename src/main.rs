mod agent;
mod analytics;
mod cli;
mod config;
mod export;
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

use clap::Parser;
use std::io;
use std::process;

async fn run() -> io::Result<()> {
    match cli::Cli::parse().resolved_command() {
        cli::Commands::Monitor(args) => tui::run(args).await,
        cli::Commands::Agent { bind, port } => agent::run(&bind, port).await,
        cli::Commands::Version => {
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
            1
        }
    };
    process::exit(exit_code);
}
