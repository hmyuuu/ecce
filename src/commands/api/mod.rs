use anyhow::Result;
use clap::Subcommand;
use colored::*;

use crate::config::{Config, Profile};
use crate::utils::{apply_profile, check_url_status, interactive_pickup, ConnectionStatus};

#[derive(Subcommand)]
pub enum ApiCommand {
    /// Add a new profile
    Add {
        /// Profile name
        name: String,
        /// API URL
        #[arg(short, long)]
        url: String,
        /// API Key
        #[arg(short, long)]
        key: String,
        /// Service type (claude-code or codex)
        #[arg(short, long, default_value = "claude-code")]
        service: String,
    },
    /// List all profiles
    List,
    /// Switch to a profile (or default if no name provided)
    Switch {
        /// Profile name to switch to (optional, uses default if not specified)
        name: Option<String>,
    },
    /// Delete a profile
    Delete {
        /// Profile name to delete
        name: String,
    },
    /// Show current active profile
    Current,
    /// Check connection status of all profiles
    Status,
    /// Set default profile
    SetDefault {
        /// Profile name to set as default
        name: String,
    },
    /// Clear default profile
    ClearDefault,
    /// Interactively pick a profile to switch to
    #[command(hide = true)]
    Pickup,
}

pub async fn handle_api_command(command: ApiCommand, config: &mut Config) -> Result<()> {
    match command {
        ApiCommand::Add {
            name,
            url,
            key,
            service,
        } => {
            let profile = Profile {
                name: name.clone(),
                url,
                key,
                service,
            };
            config.add_profile(profile)?;
            println!(
                "{}",
                format!("✓ Profile '{}' added successfully", name).green()
            );
        }
        ApiCommand::List => {
            if config.profiles.is_empty() {
                println!("{}", "No profiles configured".yellow());
            } else {
                println!("{}", "Available profiles:".bold());
                for profile in &config.profiles {
                    let mut markers = Vec::new();

                    if config.active_profile.as_deref() == Some(&profile.name) {
                        markers.push("active".green().to_string());
                    }

                    if config.default_profile.as_deref() == Some(&profile.name) {
                        markers.push("default".yellow().to_string());
                    }

                    let marker_text = if markers.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", markers.join(", "))
                    };

                    println!(
                        "  {} - {} [{}]{}",
                        profile.name.cyan(),
                        profile.url,
                        profile.service,
                        marker_text
                    );
                }
            }
        }
        ApiCommand::Switch { name } => {
            let target_name = match name {
                Some(n) => n,
                None => {
                    // Use default profile if available, otherwise trigger pickup
                    match &config.default_profile {
                        Some(default) => default.clone(),
                        None => {
                            // No default set, trigger interactive pickup
                            match interactive_pickup(config)? {
                                Some(selected) => selected,
                                None => return Ok(()), // User cancelled or invalid selection
                            }
                        }
                    }
                }
            };

            match config.switch_profile(&target_name)? {
                Some(profile) => {
                    apply_profile(&profile)?;
                }
                None => {
                    eprintln!("{}", format!("✗ Profile '{}' not found", target_name).red());
                }
            }
        }
        ApiCommand::Delete { name } => {
            if config.delete_profile(&name)? {
                println!("{}", format!("✓ Profile '{}' deleted", name).green());
            } else {
                println!("{}", format!("✗ Profile '{}' not found", name).red());
            }
        }
        ApiCommand::Current => match config.get_active_profile() {
            Some(profile) => {
                println!("{}", "Current active profile:".bold());
                println!("  Name:    {}", profile.name.cyan());
                println!("  URL:     {}", profile.url);
                println!("  Service: {}", profile.service);
                println!("  Key:     {}***", &profile.key[..profile.key.len().min(8)]);
            }
            None => {
                println!("{}", "No active profile".yellow());
            }
        },
        ApiCommand::Status => {
            if config.profiles.is_empty() {
                println!("{}", "No profiles configured".yellow());
            } else {
                println!(
                    "{}",
                    "Checking connection status for all profiles...".bold()
                );
                println!();

                for profile in &config.profiles {
                    let active = if config.active_profile.as_deref() == Some(&profile.name) {
                        " (active)".green()
                    } else {
                        "".normal()
                    };

                    print!(
                        "  {}{} [{}] - ",
                        profile.name.cyan(),
                        active,
                        profile.service
                    );

                    let status = check_url_status(&profile.url, &profile.key).await;

                    match status {
                        ConnectionStatus::Success(duration) => {
                            println!("{} ({}ms)", "✓ Connected".green(), duration.as_millis());
                        }
                        ConnectionStatus::Failed(reason) => {
                            println!("{}: {}", "✗ Failed".red(), reason);
                        }
                        ConnectionStatus::Timeout => {
                            println!("{}", "✗ Timeout".red());
                        }
                    }
                }
            }
        }
        ApiCommand::SetDefault { name } => {
            if config.set_default_profile(&name)? {
                println!("{}", format!("✓ Default profile set to '{}'", name).green());
            } else {
                println!("{}", format!("✗ Profile '{}' not found", name).red());
            }
        }
        ApiCommand::ClearDefault => {
            config.clear_default_profile()?;
            println!("{}", "✓ Default profile cleared".green());
        }
        ApiCommand::Pickup => {
            match interactive_pickup(config)? {
                Some(profile_name) => match config.switch_profile(&profile_name)? {
                    Some(profile) => {
                        apply_profile(&profile)?;
                    }
                    None => {
                        eprintln!("{}", "✗ Failed to switch profile".red());
                    }
                },
                None => {
                    // User cancelled or invalid selection, already handled in interactive_pickup
                }
            }
        }
    }

    Ok(())
}
