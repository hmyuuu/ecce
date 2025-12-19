use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod utils;
mod agent;
mod watcher;
mod pattern;

use commands::api::{handle_api_command, ApiCommand};
use commands::agent::{handle_agent_command, AgentCommand};
use commands::homo::{handle_homo_command, HomoArgs};
use commands::mcp::{handle_mcp_command, McpCommand};
use commands::task::{handle_task_command, TaskCommand};
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
    /// Agent management for Claude Code integration
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Task template management
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
    /// Watch file and trigger agents on pattern detection
    Homo(HomoArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Api { command } => {
            handle_api_command(command, &mut config).await?;
        }
        Commands::Agent { command } => {
            handle_agent_command(command, &mut config)?;
        }
        Commands::Task { command } => {
            handle_task_command(command, &mut config)?;
        }
        Commands::Mcp { command } => {
            handle_mcp_command(command, &mut config)?;
        }
        Commands::Homo(args) => {
            handle_homo_command(args, &config).await?;
        }
    }

    Ok(())
}
