use anyhow::{Context, Result};
use clap::Subcommand;
use colored::*;

use crate::config::{Config, Task};

#[derive(Subcommand)]
pub enum TaskCommand {
    /// Add a new task
    Add {
        /// Task name
        name: String,
        /// Task prompt (additional instructions for the agent)
        #[arg(short, long, conflicts_with = "prompt_file")]
        prompt: Option<String>,
        /// File containing the task prompt
        #[arg(short = 'f', long, conflicts_with = "prompt")]
        prompt_file: Option<String>,
    },
    /// List all tasks
    #[command(alias = "ls")]
    List,
    /// Delete a task
    Delete {
        /// Task name to delete
        name: String,
    },
}

pub fn handle_task_command(command: TaskCommand, config: &mut Config) -> Result<()> {
    match command {
        TaskCommand::Add {
            name,
            prompt,
            prompt_file,
        } => {
            // Get prompt from either direct input or file
            let task_prompt = match (prompt, prompt_file) {
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

            let task = Task {
                name: name.clone(),
                template: task_prompt,
            };

            config.add_task(task)?;
            println!(
                "{}",
                format!("✓ Task '{}' added successfully", name).green()
            );
        }
        TaskCommand::List => {
            if config.tasks.is_empty() {
                println!("{}", "No tasks configured".yellow());
            } else {
                println!("{}", "Available tasks:".bold());
                for (name, task) in &config.tasks {
                    println!("  {}", name.cyan());

                    // Show truncated prompt
                    let prompt_preview = task
                        .template
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(100)
                        .collect::<String>();
                    let prompt_display = if task.template.len() > 100 {
                        format!("{}...", prompt_preview)
                    } else {
                        prompt_preview
                    };
                    println!("    Prompt: {}", prompt_display.dimmed());
                }
            }
        }
        TaskCommand::Delete { name } => {
            if config.delete_task(&name)? {
                println!("{}", format!("✓ Task '{}' deleted", name).green());
            } else {
                println!("{}", format!("✗ Task '{}' not found", name).red());
            }
        }
    }

    Ok(())
}
