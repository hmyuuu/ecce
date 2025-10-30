use anyhow::{Context, Result};
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::config::{Config, Profile};

pub fn interactive_pickup(config: &mut Config) -> Result<Option<String>> {
    if config.profiles.is_empty() {
        println!("{}", "No profiles configured".yellow());
        return Ok(None);
    }

    let mut selected_idx = 0;

    // Enable raw mode for reading key events
    terminal::enable_raw_mode()?;

    let result = (|| -> Result<Option<String>> {
        loop {
            // Clear screen and move cursor to top
            execute!(
                io::stdout(),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
            io::stdout().flush()?;

            // Get terminal size to prevent wrapping with right-side content
            let (terminal_width, _) = terminal::size().unwrap_or((80, 24));
            // Reserve space for prefix (2 chars) + right-side content (20 chars) + padding
            let max_display_width = (terminal_width as usize).saturating_sub(25);

            println!("\r{}", "Available profiles:".bold());
            println!(
                "\r{}",
                "(↑/↓: navigate, Enter: select, Esc/q: cancel)".dimmed()
            );
            println!();

            for (idx, profile) in config.profiles.iter().enumerate() {
                let mut markers = Vec::new();

                if config.active_profile.as_deref() == Some(&profile.name) {
                    markers.push("→".green().to_string());
                }

                if config.default_profile.as_deref() == Some(&profile.name) {
                    markers.push("★".yellow().to_string());
                }

                let marker_text = if markers.is_empty() {
                    None
                } else {
                    Some(format!("[{}]", markers.join(" ")))
                };

                let prefix = if idx == selected_idx {
                    "→".green().bold()
                } else {
                    " ".normal()
                };

                // Show compact single-line format with URL
                // Calculate available space for name and URL
                let marker_len = marker_text.as_ref().map_or(0, |m| m.len() + 1);
                let available_for_content = max_display_width.saturating_sub(marker_len + 5); // 5 for " - "

                let name_and_url = format!("{} - {}", profile.name, profile.url);
                let display_text = if name_and_url.len() > available_for_content {
                    format!("{}...", &name_and_url[..available_for_content.saturating_sub(3)])
                } else {
                    name_and_url
                };

                match &marker_text {
                    Some(marker) => println!("\r{} {} {}", prefix, display_text.cyan(), marker),
                    None => println!("\r{} {}", prefix, display_text.cyan()),
                }
            }

            io::stdout().flush()?;

            // Read key event with timeout to handle edge cases
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => {
                    if let Event::Key(KeyEvent {
                        code, modifiers, ..
                    }) = event::read()?
                    {
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                if selected_idx > 0 {
                                    selected_idx -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if selected_idx < config.profiles.len() - 1 {
                                    selected_idx += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if !config.profiles.is_empty() {
                                    let selected_profile = &config.profiles[selected_idx];
                                    return Ok(Some(selected_profile.name.clone()));
                                } else {
                                    return Ok(None);
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                return Ok(None);
                            }
                            KeyCode::Char('c') => {
                                // Handle Ctrl+C
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    return Ok(None);
                                } else {
                                    // Just 'c' key - cancel
                                    return Ok(None);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(false) => {
                    // No event, continue loop
                }
                Err(_) => {
                    // Error polling, exit gracefully
                    return Ok(None);
                }
            }
        }
    })();

    // Disable raw mode before returning
    terminal::disable_raw_mode()?;

    // Clear screen one more time and move cursor to top
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    result
}

pub fn check_mise_installation() -> (bool, bool) {
    // Check if mise command exists
    let mise_installed = Command::new("mise")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    // Check if mise is activated (has MISE_SHELL env var or mise is in a typical location)
    let mise_activated = std::env::var("MISE_SHELL").is_ok()
        || std::env::var("__MISE_WATCH").is_ok()
        || std::env::var("MISE_DATA_DIR").is_ok();

    (mise_installed, mise_activated)
}

pub fn show_mise_warning(mise_installed: bool, mise_activated: bool) {
    if !mise_installed {
        println!();
        println!(
            "{}",
            "⚠ Warning: mise is not installed or not in PATH"
                .yellow()
                .bold()
        );
        println!();
        println!("{}", "The .mise.toml file has been created, but mise is required to load the environment variables.".yellow());
        println!();
        println!("{}", "To install mise, run one of:".bold());
        println!("  curl https://mise.run | sh");
        println!("  brew install mise");
        println!("  cargo install mise");
        println!();
        println!(
            "{}",
            "After installation, activate mise in your shell:".bold()
        );
        println!("  eval \"$(mise activate bash)\"  # for bash");
        println!("  eval \"$(mise activate zsh)\"   # for zsh");
        println!("  mise activate fish | source    # for fish");
        println!();
        println!(
            "{}",
            "See https://mise.jdx.dev/ for more information.".dimmed()
        );
        println!();
    } else if !mise_activated {
        println!();
        println!(
            "{}",
            "⚠ Warning: mise is installed but may not be activated in your shell"
                .yellow()
                .bold()
        );
        println!();
        println!("{}", "The .mise.toml file has been created, but mise needs to be activated to load environment variables.".yellow());
        println!();
        println!(
            "{}",
            "To activate mise, add this to your shell configuration (~/.bashrc, ~/.zshrc, etc.):"
                .bold()
        );
        println!("  eval \"$(mise activate bash)\"  # for bash");
        println!("  eval \"$(mise activate zsh)\"   # for zsh");
        println!("  mise activate fish | source    # for fish");
        println!();
        println!(
            "{}",
            "Then restart your shell or run: source ~/.bashrc (or equivalent)".dimmed()
        );
        println!();
    }
}

pub fn apply_profile(profile: &Profile) -> Result<()> {
    match profile.service.as_str() {
        "claude-code" => {
            // Check mise installation status
            let (mise_installed, mise_activated) = check_mise_installation();

            // Update .mise.toml with environment variables
            let mise_path = PathBuf::from(".mise.toml");

            let mise_content = format!(
                r#"# mise configuration for ecce project
# Environment variables set by ecce tool

[env]
ANTHROPIC_BASE_URL = "{}"
ANTHROPIC_API_KEY = "{}"
"#,
                profile.url, profile.key
            );

            fs::write(&mise_path, mise_content).context("Failed to write .mise.toml file")?;

            println!(
                "{}",
                "✓ Environment variables updated in .mise.toml".green()
            );
            println!();
            println!("{}", "Profile applied:".bold());
            println!("  ANTHROPIC_BASE_URL = {}", profile.url.cyan());
            println!(
                "  ANTHROPIC_API_KEY = {}***",
                profile.key[..profile.key.len().min(8)].cyan()
            );

            // Show warning if mise is not properly set up
            if !mise_installed || !mise_activated {
                show_mise_warning(mise_installed, mise_activated);
            } else {
                println!();
                println!("{}", "✓ mise is installed and activated".green());
                println!(
                    "{}",
                    "  Environment variables will be loaded automatically in this directory."
                        .dimmed()
                );
                println!();
            }
        }
        "codex" => {
            // Placeholder for Codex configuration
            eprintln!(
                "{}",
                "✓ Codex configuration (placeholder - implement based on Codex config location)"
                    .yellow()
            );
        }
        _ => {
            eprintln!(
                "{}",
                format!("⚠ Unknown service type: {}", profile.service).yellow()
            );
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum ConnectionStatus {
    Success(Duration),
    Failed(String),
    Timeout,
}

pub async fn check_url_status(url: &str, api_key: &str) -> ConnectionStatus {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return ConnectionStatus::Failed(format!("Client error: {}", e)),
    };

    let start = std::time::Instant::now();

    // Try a simple HEAD or GET request to check connectivity
    let result = client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("anthropic-version", "2023-06-01")
        .send()
        .await;

    let duration = start.elapsed();

    match result {
        Ok(response) => {
            if response.status().is_success() || response.status().as_u16() == 401 {
                // 401 means the endpoint is reachable but auth might be wrong
                // We consider this a success for connectivity check
                ConnectionStatus::Success(duration)
            } else {
                ConnectionStatus::Failed(format!("HTTP {}", response.status()))
            }
        }
        Err(e) => {
            if e.is_timeout() {
                ConnectionStatus::Timeout
            } else {
                ConnectionStatus::Failed(format!("{}", e))
            }
        }
    }
}
