use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::config::{Config, McpServer};

#[derive(Subcommand)]
pub enum McpCommand {
    /// Add an MCP server to ecce config
    Add {
        /// Server name
        name: String,
        /// Server configuration as JSON (e.g., '{"command": "bun", "args": ["run", "server.ts"]}')
        json: String,
    },
    /// Remove an MCP server from ecce config
    Remove {
        /// Server name to remove
        name: String,
    },
    /// List all MCP servers in ecce config
    List,
    /// Install an MCP server to ~/.claude.json (local project or --global)
    Install {
        /// Server name to install
        name: String,
        /// Install globally to ~/.claude.json mcpServers instead of project-specific
        #[arg(long, short)]
        global: bool,
    },
    /// Uninstall an MCP server from ~/.claude.json (local project or --global)
    Uninstall {
        /// Server name to uninstall
        name: String,
        /// Uninstall from global ~/.claude.json mcpServers instead of project-specific
        #[arg(long, short)]
        global: bool,
    },
    /// Show MCP servers status
    Status,
    /// Build ecce's MCP server
    Build,
}

fn get_mcp_server_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let workspace_mcp = cwd.join("mcp-server");
    if workspace_mcp.exists() {
        return Ok(workspace_mcp);
    }

    let exe_path = std::env::current_exe().context("Could not get executable path")?;
    let mut possible_paths = vec![];

    let mut current = exe_path.parent();
    while let Some(dir) = current {
        let mcp_path = dir.join("mcp-server");
        if mcp_path.exists() {
            possible_paths.push(mcp_path);
        }
        current = dir.parent();
    }

    if let Some(home) = dirs::home_dir() {
        possible_paths.push(home.join(".ecce").join("mcp-server"));
    }

    for path in &possible_paths {
        if path.join("dist").join("index.js").exists() {
            return Ok(path.clone());
        }
    }

    for path in &possible_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Ok(workspace_mcp)
}

pub fn handle_mcp_command(command: McpCommand, config: &mut Config) -> Result<()> {
    match command {
        McpCommand::Add { name, json } => add_mcp(config, name, json),
        McpCommand::Remove { name } => remove_mcp(config, name),
        McpCommand::List => list_mcp(config),
        McpCommand::Install { name, global } => install_mcp(config, name, global),
        McpCommand::Uninstall { name, global } => uninstall_mcp(name, global),
        McpCommand::Status => show_status(config),
        McpCommand::Build => build_mcp(),
    }
}

fn add_mcp(config: &mut Config, name: String, json_str: String) -> Result<()> {
    let server_config: Value = serde_json::from_str(&json_str)
        .context("Invalid JSON. Example: '{\"command\": \"bun\", \"args\": [\"run\", \"server.ts\"]}'")?;

    let server = McpServer {
        name: name.clone(),
        config: server_config,
    };

    config.add_mcp_server(server)?;
    println!("{} Added MCP server '{}'", "✓".green(), name);
    println!("  Run 'ecce mcp install {}' to install it to Claude Code", name);

    Ok(())
}

fn remove_mcp(config: &mut Config, name: String) -> Result<()> {
    if config.delete_mcp_server(&name)? {
        println!("{} Removed MCP server '{}'", "✓".green(), name);
    } else {
        println!("{} MCP server '{}' not found", "!".yellow(), name);
    }
    Ok(())
}

fn list_mcp(config: &Config) -> Result<()> {
    if config.mcp_servers.is_empty() {
        println!("{}", "No MCP servers configured.".yellow());
        println!("Use 'ecce mcp add <name> <json>' to add one.");
        return Ok(());
    }

    println!("{}", "MCP Servers in ecce config:".bold());
    for (name, server) in &config.mcp_servers {
        println!("\n  {}", name.cyan());
        println!("    {}", serde_json::to_string_pretty(&server.config)?
            .lines()
            .collect::<Vec<_>>()
            .join("\n    "));
    }

    Ok(())
}

fn get_claude_json_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".claude.json"))
}

fn load_claude_json() -> Result<Value> {
    let path = get_claude_json_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path).context("Failed to read ~/.claude.json")?;
        serde_json::from_str(&content).context("Failed to parse ~/.claude.json")
    } else {
        Ok(json!({ "projects": {} }))
    }
}

fn save_claude_json(config: &Value) -> Result<()> {
    let path = get_claude_json_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

fn get_current_project_path() -> Result<String> {
    let cwd = std::env::current_dir().context("Could not get current directory")?;
    Ok(cwd.to_string_lossy().to_string())
}

fn install_mcp(config: &Config, name: String, global: bool) -> Result<()> {
    let server = config.get_mcp_server(&name)
        .context(format!("MCP server '{}' not found in ecce config", name))?;

    let mut claude_json = load_claude_json()?;

    if global {
        // Install to root-level mcpServers
        if claude_json.get("mcpServers").is_none() {
            claude_json["mcpServers"] = json!({});
        }
        claude_json["mcpServers"][&name] = server.config.clone();
        save_claude_json(&claude_json)?;
        println!("{} Installed '{}' globally to ~/.claude.json", "✓".green(), name);
    } else {
        let project_path = get_current_project_path()?;

        // Ensure projects object exists
        if claude_json.get("projects").is_none() {
            claude_json["projects"] = json!({});
        }

        // Ensure project entry exists
        if claude_json["projects"].get(&project_path).is_none() {
            claude_json["projects"][&project_path] = json!({});
        }

        // Ensure mcpServers object exists for this project
        if claude_json["projects"][&project_path].get("mcpServers").is_none() {
            claude_json["projects"][&project_path]["mcpServers"] = json!({});
        }

        claude_json["projects"][&project_path]["mcpServers"][&name] = server.config.clone();
        save_claude_json(&claude_json)?;
        println!("{} Installed '{}' to ~/.claude.json for project:", "✓".green(), name);
        println!("  {}", project_path);
    }

    println!("\n{}", "Restart Claude Code to load the MCP server.".cyan());
    Ok(())
}

fn uninstall_mcp(name: String, global: bool) -> Result<()> {
    let mut claude_json = load_claude_json()?;

    if global {
        // Uninstall from root-level mcpServers
        if let Some(servers) = claude_json.get_mut("mcpServers") {
            if let Some(obj) = servers.as_object_mut() {
                if obj.remove(&name).is_some() {
                    save_claude_json(&claude_json)?;
                    println!("{} Uninstalled '{}' globally from ~/.claude.json", "✓".green(), name);
                    println!("\n{}", "Restart Claude Code to apply changes.".cyan());
                    return Ok(());
                }
            }
        }
        println!("{} '{}' not found in global ~/.claude.json mcpServers", "!".yellow(), name);
    } else {
        let project_path = get_current_project_path()?;

        if let Some(projects) = claude_json.get_mut("projects") {
            if let Some(project) = projects.get_mut(&project_path) {
                if let Some(servers) = project.get_mut("mcpServers") {
                    if let Some(obj) = servers.as_object_mut() {
                        if obj.remove(&name).is_some() {
                            save_claude_json(&claude_json)?;
                            println!("{} Uninstalled '{}' from ~/.claude.json for project:", "✓".green(), name);
                            println!("  {}", project_path);
                            println!("\n{}", "Restart Claude Code to apply changes.".cyan());
                            return Ok(());
                        }
                    }
                }
            }
        }
        println!("{} '{}' not found in ~/.claude.json for project:", "!".yellow(), name);
        println!("  {}", project_path);
    }
    Ok(())
}

fn show_status(_config: &Config) -> Result<()> {
    let mcp_server_path = get_mcp_server_path()?;
    let dist_path = mcp_server_path.join("dist").join("index.js");

    println!("{}", "Ecce MCP Status".bold());
    println!();

    // Ecce's own MCP server
    print!("Ecce MCP Server Built: ");
    if dist_path.exists() {
        println!("{}", "Yes".green());
        println!("  Path: {}", dist_path.display());
    } else {
        println!("{}", "No".red());
        println!("  Run 'ecce mcp build' to build");
    }

    let claude_json = load_claude_json()?;

    println!();

    // Global MCP servers from ~/.claude.json root mcpServers
    println!("{}", "Global MCP Servers in ~/.claude.json:".bold());
    let mut global_found = false;
    if let Some(servers) = claude_json.get("mcpServers") {
        if let Some(obj) = servers.as_object() {
            if !obj.is_empty() {
                global_found = true;
                for name in obj.keys() {
                    println!("  - {}", name);
                }
            }
        }
    }
    if !global_found {
        println!("  {}", "None".yellow());
    }

    println!();

    // Local project MCP servers from ~/.claude.json
    let project_path = get_current_project_path()?;
    println!("{}", "Project MCP Servers in ~/.claude.json:".bold());
    println!("  {}", project_path);

    let mut found = false;
    if let Some(projects) = claude_json.get("projects") {
        if let Some(project) = projects.get(&project_path) {
            if let Some(servers) = project.get("mcpServers") {
                if let Some(obj) = servers.as_object() {
                    if !obj.is_empty() {
                        found = true;
                        for name in obj.keys() {
                            println!("  - {}", name);
                        }
                    }
                }
            }
        }
    }
    if !found {
        println!("  {}", "None".yellow());
    }

    Ok(())
}

fn build_mcp() -> Result<()> {
    let mcp_path = get_mcp_server_path()?;

    if !mcp_path.exists() {
        anyhow::bail!("MCP server directory not found at: {}", mcp_path.display());
    }

    println!("{}", "Building MCP server...".cyan());

    let install_status = std::process::Command::new("bun")
        .arg("install")
        .current_dir(&mcp_path)
        .status()
        .context("Failed to run 'bun install'")?;

    if !install_status.success() {
        anyhow::bail!("'bun install' failed");
    }

    let build_status = std::process::Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir(&mcp_path)
        .status()
        .context("Failed to run 'bun run build'")?;

    if !build_status.success() {
        anyhow::bail!("'bun run build' failed");
    }

    println!("{} MCP server built successfully!", "✓".green());

    Ok(())
}
