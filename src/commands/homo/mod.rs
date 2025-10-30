use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::signal;

use crate::agent::ClaudeAgent;
use crate::config::{Agent, Config, Task};
use crate::pattern::EccePattern;
use crate::watcher::FileWatcher;

#[derive(Args)]
pub struct HomoArgs {
    /// File or folder to watch (if folder, looks for slides.md)
    pub file_path: PathBuf,

    /// Agent to use (optional, uses default or prompts)
    #[arg(short, long)]
    pub agent: Option<String>,

    /// Task template to use (optional)
    #[arg(short, long)]
    pub task: Option<String>,

    /// Watch interval in milliseconds
    #[arg(long, default_value = "100")]
    pub watch_interval: u64,
}

pub async fn handle_homo_command(args: HomoArgs, config: &Config) -> Result<()> {
    // Resolve file path (handle both files and folders)
    let file_path = resolve_file_path(&args.file_path)?;

    // Select agent
    let agent_config = select_agent(config, args.agent.clone())?;

    // Select task (interactive if not specified)
    let task_config = select_task(config, args.task.clone())?;

    // Get Claude Code executable path from config
    let claude_executable = config.get_claude_executable();

    // Display task name before moving task_config
    let task_display = if let Some(ref task) = task_config {
        task.name.clone()
    } else {
        "(none)".to_string()
    };

    // Create agent
    let claude_agent = ClaudeAgent::new(claude_executable, agent_config.clone(), task_config);

    println!("{}", "\n🎭 Ecce Homo - File Watcher Started".bold().green());
    println!("{}", "═".repeat(60).dimmed());
    println!("  📄 File:     {}", file_path.display().to_string().cyan());
    println!("  🤖 Agent:    {}", agent_config.name.cyan());
    println!("  📋 Task:     {}", task_display.cyan());
    println!("{}", "═".repeat(60).dimmed());
    println!("{}", "\n👀 Watching for patterns...".yellow());
    println!("   Pattern 1: {}", "ecce <prompt> ecce".cyan());
    println!("   Pattern 2: {}", "```ecce\\n<prompt>\\n```".cyan());
    println!("   Interval:  {}ms", args.watch_interval.to_string().cyan());
    println!("\n   Press {} to stop\n", "Ctrl+C".bold());

    // Start watching with signal handling
    watch_and_process_with_signals(&file_path, claude_agent, args.watch_interval).await
}

/// Resolve file path - if it's a directory, look for slides.md
fn resolve_file_path(path: &PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Path not found: {}",
            path.display()
        ));
    }

    if path.is_dir() {
        // Look for slides.md in the directory
        let slides_path = path.join("slides.md");
        if slides_path.exists() {
            println!(
                "{}",
                format!("📁 Found slides.md in directory: {}", path.display())
                    .green()
            );
            Ok(slides_path)
        } else {
            Err(anyhow::anyhow!(
                "Directory provided but slides.md not found in: {}",
                path.display()
            ))
        }
    } else if path.is_file() {
        Ok(path.clone())
    } else {
        Err(anyhow::anyhow!(
            "Invalid path (not a file or directory): {}",
            path.display()
        ))
    }
}

/// Select agent from config, with fallback to interactive selection
fn select_agent(config: &Config, agent_name: Option<String>) -> Result<Agent> {
    match agent_name {
        Some(name) => config
            .get_agent(&name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", name)),
        None => {
            // Try default agent first
            if let Some(agent) = config.get_default_agent() {
                return Ok(agent.clone());
            }

            // Fall back to interactive selection
            interactive_agent_selection(config)
        }
    }
}

/// Select task from config, with fallback to interactive selection
fn select_task(config: &Config, task_name: Option<String>) -> Result<Option<Task>> {
    match task_name {
        Some(name) => {
            config
                .get_task(&name)
                .cloned()
                .map(Some)
                .ok_or_else(|| anyhow::anyhow!("Task '{}' not found", name))
        }
        None => {
            // If no tasks configured, return None (no task)
            if config.tasks.is_empty() {
                return Ok(None);
            }

            // Interactive task selection
            interactive_task_selection(config)
        }
    }
}

/// Interactive agent selection
fn interactive_agent_selection(config: &Config) -> Result<Agent> {
    if config.agents.is_empty() {
        return Err(anyhow::anyhow!(
            "No agents configured. Use 'ecce agent add' to create an agent first."
        ));
    }

    println!("{}", "\n🤖 Available agents:".cyan().bold());
    let agent_names: Vec<_> = config.agents.keys().cloned().collect();

    for (i, name) in agent_names.iter().enumerate() {
        if let Some(agent) = config.get_agent(name) {
            println!(
                "  {}. {} - {}",
                (i + 1).to_string().yellow(),
                name.cyan(),
                agent
                    .system_prompt
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(50)
                    .collect::<String>()
                    .dimmed()
            );
        }
    }

    print!(
        "\n{} ",
        format!("Select agent (1-{}):", agent_names.len()).yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input
        .trim()
        .parse()
        .context("Invalid number. Please enter a valid choice.")?;

    if choice < 1 || choice > agent_names.len() {
        return Err(anyhow::anyhow!(
            "Invalid choice. Please select a number between 1 and {}",
            agent_names.len()
        ));
    }

    let agent_name = &agent_names[choice - 1];
    config
        .get_agent(agent_name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Agent not found"))
}

/// Interactive task selection
fn interactive_task_selection(config: &Config) -> Result<Option<Task>> {
    println!("{}", "\n📋 Available tasks:".cyan().bold());
    let task_names: Vec<_> = config.tasks.keys().cloned().collect();

    // Option 0: No task
    println!("  {}. {}", "0".yellow(), "(No task - use default)".dimmed());

    for (i, name) in task_names.iter().enumerate() {
        if let Some(task) = config.get_task(name) {
            let template_preview = task
                .template
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(50)
                .collect::<String>();
            println!(
                "  {}. {} - {}",
                (i + 1).to_string().yellow(),
                name.cyan(),
                template_preview.dimmed()
            );
        }
    }

    print!(
        "\n{} ",
        format!("Select task (0-{}):", task_names.len()).yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input
        .trim()
        .parse()
        .context("Invalid number. Please enter a valid choice.")?;

    if choice == 0 {
        return Ok(None);
    }

    if choice < 1 || choice > task_names.len() {
        return Err(anyhow::anyhow!(
            "Invalid choice. Please select a number between 0 and {}",
            task_names.len()
        ));
    }

    let task_name = &task_names[choice - 1];
    config
        .get_task(task_name)
        .cloned()
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))
}

/// Watch file with signal handling for graceful shutdown
async fn watch_and_process_with_signals(
    file_path: &PathBuf,
    claude_agent: ClaudeAgent,
    watch_interval: u64,
) -> Result<()> {
    tokio::select! {
        result = watch_and_process(file_path, claude_agent, watch_interval) => result,
        _ = signal::ctrl_c() => {
            println!("\n\n{}", "👋 Stopped watching file. Goodbye!".yellow().bold());
            Ok(())
        }
    }
}

/// Main file watching loop
async fn watch_and_process(file_path: &PathBuf, mut claude_agent: ClaudeAgent, watch_interval: u64) -> Result<()> {
    let mut watcher = FileWatcher::with_interval(file_path, watch_interval)?;
    watcher.watch(file_path)?;

    loop {
        // Wait for new patterns
        let patterns = watcher.wait_for_changes(file_path)?;

        if !patterns.is_empty() {
            println!(
                "\n{}",
                format!("🔍 Found {} new pattern(s)", patterns.len())
                    .green()
                    .bold()
            );
            println!("{}", "─".repeat(60).dimmed());
        }

        // Process each pattern
        for (idx, pattern) in patterns.iter().enumerate() {
            println!(
                "\n{} Pattern {}/{}",
                "▶".cyan(),
                idx + 1,
                patterns.len()
            );
            println!("  Type:    {:?}", pattern.pattern_type);
            println!(
                "  Content: {}",
                pattern
                    .content
                    .lines()
                    .next()
                    .unwrap_or(&pattern.content)
                    .chars()
                    .take(60)
                    .collect::<String>()
                    .cyan()
            );

            // Process the pattern
            match process_pattern(pattern, &mut claude_agent, file_path, &mut watcher).await {
                Ok(_) => {
                    println!("  {}", "✅ Success".green().bold());
                }
                Err(e) => {
                    println!("  {} {}", "❌ Error:".red().bold(), e);
                    eprintln!("Failed to process pattern: {}", e);
                }
            }
        }

        if !patterns.is_empty() {
            println!("\n{}", "─".repeat(60).dimmed());
            println!("{}", "👀 Continuing to watch...".yellow());
        }
    }
}

/// Process a single pattern: generate response and replace in file
async fn process_pattern(
    pattern: &EccePattern,
    agent: &mut ClaudeAgent,
    file_path: &PathBuf,
    watcher: &mut FileWatcher,
) -> Result<()> {
    println!("  {}", "🤖 Generating response...".yellow());

    // Immediately replace pattern with "generating" message
    replace_pattern_in_file(file_path, &pattern.content, "🤖 Generating response...")?;

    // Update watcher's content to avoid detecting our own change
    watcher.update_content(file_path)?;

    // Call agent to generate response
    let response = agent
        .generate_response(&pattern.content)
        .await
        .context("Failed to generate response from Claude API")?;

    println!("  {}", "📝 Replacing with response...".yellow());

    // Replace "generating" message with actual response
    replace_pattern_in_file(file_path, "🤖 Generating response...", &response)?;

    // Update watcher's content again
    watcher.update_content(file_path)?;

    // Mark pattern as processed to avoid reprocessing
    watcher.mark_processed(&pattern.content);

    Ok(())
}

/// Replace a pattern in the file with new content
fn replace_pattern_in_file(
    file_path: &PathBuf,
    old_text: &str,
    new_text: &str,
) -> Result<()> {
    // Read the entire file
    let content = std::fs::read_to_string(file_path)
        .context("Failed to read file for pattern replacement")?;

    let mut new_content = content.clone();
    let mut replaced = false;

    // Try to find and replace inline pattern: ecce <prompt> ecce
    let patterns_to_try = vec![
        format!("ecce {} ecce", old_text),
        format!("ecce  {}  ecce", old_text),
        format!("ecce\n{}\necce", old_text),
        format!("ecce {} ecce", old_text.trim()),
        format!("ecce  {}  ecce", old_text.trim()),
        // Also try direct replacement (for replacing "generating" message)
        old_text.to_string(),
    ];

    for pattern in &patterns_to_try {
        if content.contains(pattern) {
            new_content = content.replace(pattern, new_text);
            replaced = true;
            break;
        }
    }

    // If inline pattern not found, try code block pattern
    if !replaced {
        let block_patterns = vec![
            format!("```ecce\n{}\n```", old_text),
            format!("```ecce\n{}\n```", old_text.trim()),
            format!("```ecce\n  {}\n```", old_text.trim()),
        ];

        for pattern in &block_patterns {
            if content.contains(pattern) {
                new_content = content.replace(pattern, new_text);
                replaced = true;
                break;
            }
        }
    }

    if !replaced {
        return Err(anyhow::anyhow!(
            "Pattern not found in file: '{}'",
            old_text
        ));
    }

    // Write the modified content back
    std::fs::write(file_path, new_content)
        .context("Failed to write file after pattern replacement")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_replace_pattern_in_file() {
        let mut temp = NamedTempFile::new().unwrap();
        let path = PathBuf::from(temp.path());

        fs::write(&path, "ecce test prompt ecce").unwrap();

        replace_pattern_in_file(&path, "test prompt", "Generated response").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Generated response");
    }
}
