mod cli;
mod repl;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    
    match cli.command {
        Some(cmd) => {
            // Single-shot command mode
            cli::execute_command(cmd).await
        }
        None => {
            // Interactive REPL mode
            repl::start().await
        }
    }
}