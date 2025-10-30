use anyhow::{Context, Result};
use clap::Subcommand;
use colored::*;

use crate::config::{Agent, Config};

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Add a new agent
    Add {
        /// Agent name
        name: String,
        /// System prompt for the agent
        #[arg(short, long, conflicts_with = "prompt_file")]
        prompt: Option<String>,
        /// File containing the system prompt
        #[arg(short = 'f', long, conflicts_with = "prompt")]
        prompt_file: Option<String>,
        /// Description of when to use this agent
        #[arg(short, long)]
        description: Option<String>,
        /// Context files (comma-separated)
        #[arg(short, long)]
        context: Option<String>,
        /// Tools available to the agent (comma-separated)
        #[arg(short, long)]
        tools: Option<String>,
        /// Model to use (sonnet, opus, haiku, or inherit)
        #[arg(short, long)]
        model: Option<String>,
    },
    /// List all agents
    #[command(alias = "ls")]
    List,
    /// Delete an agent
    Delete {
        /// Agent name to delete
        name: String,
    },
    /// Export agent(s) to .claude/agents/ directory
    Export {
        /// Agent name to export (exports all if not specified)
        name: Option<String>,
        /// Export to user-level directory (~/.claude/agents/)
        #[arg(short, long)]
        user: bool,
    },
    /// Import agent(s) from .claude/agents/ directory
    Import {
        /// Import from user-level directory (~/.claude/agents/)
        #[arg(short, long)]
        user: bool,
    },
    /// Sync agents between config and .claude/agents/
    Sync {
        /// Sync with user-level directory (~/.claude/agents/)
        #[arg(short, long)]
        user: bool,
        /// Direction: 'import' or 'export'
        #[arg(short, long, default_value = "import")]
        direction: String,
    },
}

pub fn handle_agent_command(command: AgentCommand, config: &mut Config) -> Result<()> {
    match command {
        AgentCommand::Add {
            name,
            prompt,
            prompt_file,
            description,
            context,
            tools,
            model,
        } => {
            // Get prompt from either direct input or file
            let system_prompt = match (prompt, prompt_file) {
                (Some(p), None) => p,
                (None, Some(f)) => {
                    std::fs::read_to_string(&f)
                        .with_context(|| format!("Failed to read prompt file: {}", f))?
                }
                (None, None) => {
                    return Err(anyhow::anyhow!(
                        "Either --prompt or --prompt-file must be provided"
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(anyhow::anyhow!(
                        "Cannot use both --prompt and --prompt-file"
                    ));
                }
            };

            let context_files = context
                .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let tools_list = tools
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

            let agent = Agent {
                name: name.clone(),
                description,
                system_prompt,
                context_files,
                tools: tools_list,
                model,
            };

            config.add_agent(agent)?;
            println!(
                "{}",
                format!("✓ Agent '{}' added successfully", name).green()
            );
        }
        AgentCommand::List => {
            if config.agents.is_empty() {
                println!("{}", "No agents configured".yellow());
            } else {
                println!("{}", "Available agents:".bold());
                for (name, agent) in &config.agents {
                    println!("  {}", name.cyan());

                    // Show truncated description
                    if let Some(ref desc) = agent.description {
                        let desc_preview = desc
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(100)
                            .collect::<String>();
                        let desc_display = if desc.len() > 100 {
                            format!("{}...", desc_preview)
                        } else {
                            desc_preview
                        };
                        println!("    Description: {}", desc_display.dimmed());
                    }

                    // Show only first line of prompt (truncated)
                    let prompt_preview = agent
                        .system_prompt
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(80)
                        .collect::<String>();
                    let prompt_display = if agent.system_prompt.len() > 80 {
                        format!("{}...", prompt_preview)
                    } else {
                        prompt_preview
                    };
                    println!("    Prompt: {}", prompt_display.dimmed());

                    if !agent.context_files.is_empty() {
                        println!("    Context: {}", agent.context_files.join(", "));
                    }
                    if let Some(ref tools) = agent.tools {
                        println!("    Tools: {}", tools.join(", "));
                    }
                    if let Some(ref model) = agent.model {
                        println!("    Model: {}", model);
                    }
                }
            }
        }
        AgentCommand::Delete { name } => {
            if config.delete_agent(&name)? {
                println!("{}", format!("✓ Agent '{}' deleted", name).green());
            } else {
                println!("{}", format!("✗ Agent '{}' not found", name).red());
            }
        }
        AgentCommand::Export { name, user } => {
            if let Some(agent_name) = name {
                config.export_agent_to_file(&agent_name, user)?;
                let location = if user { "~/.claude/agents/" } else { ".claude/agents/" };
                println!(
                    "{}",
                    format!("✓ Agent '{}' exported to {}", agent_name, location).green()
                );
            } else {
                let exported = config.export_all_agents(user)?;
                let location = if user { "~/.claude/agents/" } else { ".claude/agents/" };
                println!(
                    "{}",
                    format!("✓ Exported {} agent(s) to {}", exported.len(), location).green()
                );
                for name in exported {
                    println!("  - {}", name.cyan());
                }
            }
        }
        AgentCommand::Import { user } => {
            let imported = config.sync_agents_from_files(user)?;
            if imported.is_empty() {
                let location = if user { "~/.claude/agents/" } else { ".claude/agents/" };
                println!("{}", format!("No agents found in {}", location).yellow());
            } else {
                println!(
                    "{}",
                    format!("✓ Imported {} agent(s)", imported.len()).green()
                );
                for name in imported {
                    println!("  - {}", name.cyan());
                }
            }
        }
        AgentCommand::Sync { user, direction } => {
            match direction.as_str() {
                "import" => {
                    let imported = config.sync_agents_from_files(user)?;
                    if imported.is_empty() {
                        let location = if user { "~/.claude/agents/" } else { ".claude/agents/" };
                        println!("{}", format!("No agents found in {}", location).yellow());
                    } else {
                        println!(
                            "{}",
                            format!("✓ Synced {} agent(s) from files", imported.len()).green()
                        );
                        for name in imported {
                            println!("  - {}", name.cyan());
                        }
                    }
                }
                "export" => {
                    let exported = config.export_all_agents(user)?;
                    let location = if user { "~/.claude/agents/" } else { ".claude/agents/" };
                    println!(
                        "{}",
                        format!("✓ Synced {} agent(s) to {}", exported.len(), location).green()
                    );
                    for name in exported {
                        println!("  - {}", name.cyan());
                    }
                }
                _ => {
                    println!(
                        "{}",
                        format!("✗ Invalid direction '{}'. Use 'import' or 'export'", direction).red()
                    );
                }
            }
        }
    }

    Ok(())
}
