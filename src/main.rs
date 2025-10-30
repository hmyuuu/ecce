use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod utils;

use commands::api::{handle_api_command, ApiCommand};
use config::Config;

#[derive(Parser)]
#[command(name = "ecce")]
#[command(about = "Ecce Claude CodE - Behold Claude Code", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// API profile management
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Api { command } => {
            handle_api_command(command, &mut config).await?;
        }
    }

    Ok(())
}
