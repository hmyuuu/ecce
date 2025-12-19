use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub url: String,
    pub key: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub context_files: Vec<String>,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub name: String,
    pub template: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServer {
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub agents: HashMap<String, Agent>,
    #[serde(default)]
    pub tasks: HashMap<String, Task>,
    #[serde(default)]
    pub default_agent: Option<String>,
    #[serde(default)]
    pub claude_executable: Option<String>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServer>,
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".config").join("ecce");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_profile(&mut self, profile: Profile) -> Result<()> {
        // Remove existing profile with same name if exists
        self.profiles.retain(|p| p.name != profile.name);
        self.profiles.push(profile);
        self.save()
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<bool> {
        let initial_len = self.profiles.len();
        self.profiles.retain(|p| p.name != name);

        if self.profiles.len() < initial_len {
            // If deleted profile was active, clear active profile
            if self.active_profile.as_deref() == Some(name) {
                self.active_profile = None;
            }
            // If deleted profile was default, clear default profile
            if self.default_profile.as_deref() == Some(name) {
                self.default_profile = None;
            }
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn switch_profile(&mut self, name: &str) -> Result<Option<Profile>> {
        if let Some(profile) = self.profiles.iter().find(|p| p.name == name) {
            self.active_profile = Some(name.to_string());
            self.save()?;
            Ok(Some(profile.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_active_profile(&self) -> Option<&Profile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.iter().find(|p| p.name == *name))
    }

    pub fn set_default_profile(&mut self, name: &str) -> Result<bool> {
        if self.profiles.iter().any(|p| p.name == name) {
            self.default_profile = Some(name.to_string());
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_default_profile(&mut self) -> Result<()> {
        self.default_profile = None;
        self.save()
    }

    pub fn add_agent(&mut self, agent: Agent) -> Result<()> {
        self.agents.insert(agent.name.clone(), agent);
        self.save()
    }

    pub fn delete_agent(&mut self, name: &str) -> Result<bool> {
        if self.agents.remove(name).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_agent(&self, name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    pub fn add_task(&mut self, task: Task) -> Result<()> {
        self.tasks.insert(task.name.clone(), task);
        self.save()
    }

    pub fn delete_task(&mut self, name: &str) -> Result<bool> {
        if self.tasks.remove(name).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_task(&self, name: &str) -> Option<&Task> {
        self.tasks.get(name)
    }

    pub fn set_default_agent(&mut self, name: &str) -> Result<bool> {
        if self.agents.contains_key(name) {
            self.default_agent = Some(name.to_string());
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_default_agent(&mut self) -> Result<()> {
        self.default_agent = None;
        self.save()
    }

    pub fn get_default_agent(&self) -> Option<&Agent> {
        self.default_agent
            .as_ref()
            .and_then(|name| self.agents.get(name))
    }

    pub fn get_claude_executable(&self) -> String {
        self.claude_executable
            .clone()
            .unwrap_or_else(|| "claude".to_string())
    }

    /// Get the .claude/agents directory path (project-level)
    pub fn claude_agents_dir() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;
        let agents_dir = current_dir.join(".claude").join("agents");
        Ok(agents_dir)
    }

    /// Get the user-level agents directory path
    pub fn user_agents_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let agents_dir = home.join(".claude").join("agents");
        Ok(agents_dir)
    }

    /// Export an agent to a markdown file in .claude/agents/
    pub fn export_agent_to_file(&self, agent_name: &str, user_level: bool) -> Result<()> {
        let agent = self.agents.get(agent_name)
            .context(format!("Agent '{}' not found", agent_name))?;

        let agents_dir = if user_level {
            Self::user_agents_dir()?
        } else {
            Self::claude_agents_dir()?
        };

        fs::create_dir_all(&agents_dir)?;

        let file_path = agents_dir.join(format!("{}.md", agent.name));
        let mut file = fs::File::create(&file_path)?;

        // Write YAML frontmatter
        writeln!(file, "---")?;
        writeln!(file, "name: {}", agent.name)?;

        if let Some(ref description) = agent.description {
            writeln!(file, "description: {}", description)?;
        }

        if let Some(ref tools) = agent.tools {
            writeln!(file, "tools: {}", tools.join(", "))?;
        }

        if let Some(ref model) = agent.model {
            writeln!(file, "model: {}", model)?;
        }

        writeln!(file, "---")?;
        writeln!(file)?;

        // Write system prompt
        writeln!(file, "{}", agent.system_prompt)?;

        Ok(())
    }

    /// Import an agent from a markdown file
    pub fn import_agent_from_file(file_path: &PathBuf) -> Result<Agent> {
        let content = fs::read_to_string(file_path)?;

        // Parse YAML frontmatter and content
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid agent file format: missing frontmatter"));
        }

        let frontmatter = parts[1].trim();
        let system_prompt = parts[2].trim().to_string();

        // Parse frontmatter manually (simple key-value parsing)
        let mut name = String::new();
        let mut description = None;
        let mut tools = None;
        let mut model = None;

        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "name" => name = value.to_string(),
                    "description" => description = Some(value.to_string()),
                    "tools" => {
                        tools = Some(value.split(',').map(|s| s.trim().to_string()).collect());
                    }
                    "model" => model = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        if name.is_empty() {
            return Err(anyhow::anyhow!("Agent name is required in frontmatter"));
        }

        Ok(Agent {
            name,
            description,
            system_prompt,
            context_files: Vec::new(),
            tools,
            model,
        })
    }

    /// Sync agents from .claude/agents/ directory to config
    pub fn sync_agents_from_files(&mut self, user_level: bool) -> Result<Vec<String>> {
        let agents_dir = if user_level {
            Self::user_agents_dir()?
        } else {
            Self::claude_agents_dir()?
        };

        if !agents_dir.exists() {
            return Ok(Vec::new());
        }

        let mut imported = Vec::new();

        for entry in fs::read_dir(agents_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match Self::import_agent_from_file(&path) {
                    Ok(agent) => {
                        let name = agent.name.clone();
                        self.agents.insert(name.clone(), agent);
                        imported.push(name);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to import {:?}: {}", path, e);
                    }
                }
            }
        }

        if !imported.is_empty() {
            self.save()?;
        }

        Ok(imported)
    }

    /// Export all agents to .claude/agents/ directory
    pub fn export_all_agents(&self, user_level: bool) -> Result<Vec<String>> {
        let mut exported = Vec::new();

        for agent_name in self.agents.keys() {
            self.export_agent_to_file(agent_name, user_level)?;
            exported.push(agent_name.clone());
        }

        Ok(exported)
    }

    // MCP Server methods
    pub fn add_mcp_server(&mut self, server: McpServer) -> Result<()> {
        self.mcp_servers.insert(server.name.clone(), server);
        self.save()
    }

    pub fn delete_mcp_server(&mut self, name: &str) -> Result<bool> {
        if self.mcp_servers.remove(name).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_mcp_server(&self, name: &str) -> Option<&McpServer> {
        self.mcp_servers.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        (config, temp_dir)
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.profiles.is_empty());
        assert!(config.active_profile.is_none());
        assert!(config.default_profile.is_none());
        assert!(config.agents.is_empty());
        assert!(config.tasks.is_empty());
    }

    #[test]
    fn test_add_profile() {
        let (mut config, _temp) = setup_test_config();

        let profile = Profile {
            name: "test".to_string(),
            url: "https://api.test.com".to_string(),
            key: "test-key".to_string(),
            service: "claude-code".to_string(),
        };

        config.profiles.push(profile.clone());

        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "test");
        assert_eq!(config.profiles[0].url, "https://api.test.com");
    }

    #[test]
    fn test_delete_profile() {
        let (mut config, _temp) = setup_test_config();

        let profile = Profile {
            name: "test".to_string(),
            url: "https://api.test.com".to_string(),
            key: "test-key".to_string(),
            service: "claude-code".to_string(),
        };

        config.profiles.push(profile);
        assert_eq!(config.profiles.len(), 1);

        config.profiles.retain(|p| p.name != "test");
        assert_eq!(config.profiles.len(), 0);
    }

    #[test]
    fn test_delete_active_profile_clears_active() {
        let (mut config, _temp) = setup_test_config();

        let profile = Profile {
            name: "test".to_string(),
            url: "https://api.test.com".to_string(),
            key: "test-key".to_string(),
            service: "claude-code".to_string(),
        };

        config.profiles.push(profile);
        config.active_profile = Some("test".to_string());

        let initial_len = config.profiles.len();
        config.profiles.retain(|p| p.name != "test");

        if config.profiles.len() < initial_len {
            if config.active_profile.as_deref() == Some("test") {
                config.active_profile = None;
            }
        }

        assert!(config.active_profile.is_none());
    }

    #[test]
    fn test_get_active_profile() {
        let (mut config, _temp) = setup_test_config();

        let profile = Profile {
            name: "test".to_string(),
            url: "https://api.test.com".to_string(),
            key: "test-key".to_string(),
            service: "claude-code".to_string(),
        };

        config.profiles.push(profile);
        config.active_profile = Some("test".to_string());

        let active = config.get_active_profile();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "test");
    }

    #[test]
    fn test_add_agent() {
        let (mut config, _temp) = setup_test_config();

        let agent = Agent {
            name: "test-agent".to_string(),
            description: Some("Test agent".to_string()),
            system_prompt: "You are a test agent".to_string(),
            context_files: vec![],
            tools: Some(vec!["tool1".to_string()]),
            model: Some("sonnet".to_string()),
        };

        config.agents.insert(agent.name.clone(), agent);

        assert_eq!(config.agents.len(), 1);
        assert!(config.agents.contains_key("test-agent"));
    }

    #[test]
    fn test_delete_agent() {
        let (mut config, _temp) = setup_test_config();

        let agent = Agent {
            name: "test-agent".to_string(),
            description: None,
            system_prompt: "Test".to_string(),
            context_files: vec![],
            tools: None,
            model: None,
        };

        config.agents.insert(agent.name.clone(), agent);
        assert_eq!(config.agents.len(), 1);

        let removed = config.agents.remove("test-agent");
        assert!(removed.is_some());
        assert_eq!(config.agents.len(), 0);
    }

    #[test]
    fn test_get_agent() {
        let (mut config, _temp) = setup_test_config();

        let agent = Agent {
            name: "test-agent".to_string(),
            description: None,
            system_prompt: "Test".to_string(),
            context_files: vec![],
            tools: None,
            model: None,
        };

        config.agents.insert(agent.name.clone(), agent);

        let retrieved = config.get_agent("test-agent");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-agent");
    }

    #[test]
    fn test_add_task() {
        let (mut config, _temp) = setup_test_config();

        let task = Task {
            name: "test-task".to_string(),
            template: "Test template".to_string(),
        };

        config.tasks.insert(task.name.clone(), task);

        assert_eq!(config.tasks.len(), 1);
        assert!(config.tasks.contains_key("test-task"));
    }

    #[test]
    fn test_delete_task() {
        let (mut config, _temp) = setup_test_config();

        let task = Task {
            name: "test-task".to_string(),
            template: "Test template".to_string(),
        };

        config.tasks.insert(task.name.clone(), task);
        assert_eq!(config.tasks.len(), 1);

        let removed = config.tasks.remove("test-task");
        assert!(removed.is_some());
        assert_eq!(config.tasks.len(), 0);
    }

    #[test]
    fn test_get_task() {
        let (mut config, _temp) = setup_test_config();

        let task = Task {
            name: "test-task".to_string(),
            template: "Test template".to_string(),
        };

        config.tasks.insert(task.name.clone(), task);

        let retrieved = config.get_task("test-task");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-task");
    }

    #[test]
    fn test_set_default_agent() {
        let (mut config, _temp) = setup_test_config();

        let agent = Agent {
            name: "test-agent".to_string(),
            description: None,
            system_prompt: "Test".to_string(),
            context_files: vec![],
            tools: None,
            model: None,
        };

        config.agents.insert(agent.name.clone(), agent);

        if config.agents.contains_key("test-agent") {
            config.default_agent = Some("test-agent".to_string());
        }

        assert_eq!(config.default_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_get_default_agent() {
        let (mut config, _temp) = setup_test_config();

        let agent = Agent {
            name: "test-agent".to_string(),
            description: None,
            system_prompt: "Test".to_string(),
            context_files: vec![],
            tools: None,
            model: None,
        };

        config.agents.insert(agent.name.clone(), agent);
        config.default_agent = Some("test-agent".to_string());

        let default = config.get_default_agent();
        assert!(default.is_some());
        assert_eq!(default.unwrap().name, "test-agent");
    }

    #[test]
    fn test_get_claude_executable_default() {
        let (config, _temp) = setup_test_config();
        assert_eq!(config.get_claude_executable(), "claude");
    }

    #[test]
    fn test_get_claude_executable_custom() {
        let (mut config, _temp) = setup_test_config();
        config.claude_executable = Some("/custom/path/claude".to_string());
        assert_eq!(config.get_claude_executable(), "/custom/path/claude");
    }

    #[test]
    fn test_import_agent_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file = temp_dir.path().join("test-agent.md");

        let content = r#"---
name: test-agent
description: A test agent
tools: Read, Write, Grep
model: sonnet
---

You are a helpful test agent."#;

        fs::write(&agent_file, content).unwrap();

        let agent = Config::import_agent_from_file(&agent_file).unwrap();

        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.description, Some("A test agent".to_string()));
        assert_eq!(agent.system_prompt, "You are a helpful test agent.");
        assert_eq!(agent.tools, Some(vec!["Read".to_string(), "Write".to_string(), "Grep".to_string()]));
        assert_eq!(agent.model, Some("sonnet".to_string()));
    }

    #[test]
    fn test_import_agent_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file = temp_dir.path().join("invalid-agent.md");

        let content = "This is not a valid agent file";
        fs::write(&agent_file, content).unwrap();

        let result = Config::import_agent_from_file(&agent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_agent_missing_name() {
        let temp_dir = TempDir::new().unwrap();
        let agent_file = temp_dir.path().join("no-name-agent.md");

        let content = r#"---
description: An agent without a name
---

System prompt here"#;

        fs::write(&agent_file, content).unwrap();

        let result = Config::import_agent_from_file(&agent_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_mcp_server() {
        let (mut config, _temp) = setup_test_config();

        let server = McpServer {
            name: "test-server".to_string(),
            config: serde_json::json!({"command": "node", "args": ["server.js"]}),
        };

        config.mcp_servers.insert(server.name.clone(), server);

        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("test-server"));
    }

    #[test]
    fn test_delete_mcp_server() {
        let (mut config, _temp) = setup_test_config();

        let server = McpServer {
            name: "test-server".to_string(),
            config: serde_json::json!({"command": "node"}),
        };

        config.mcp_servers.insert(server.name.clone(), server);
        assert_eq!(config.mcp_servers.len(), 1);

        let removed = config.mcp_servers.remove("test-server");
        assert!(removed.is_some());
        assert_eq!(config.mcp_servers.len(), 0);
    }

    #[test]
    fn test_get_mcp_server() {
        let (mut config, _temp) = setup_test_config();

        let server = McpServer {
            name: "test-server".to_string(),
            config: serde_json::json!({"command": "node"}),
        };

        config.mcp_servers.insert(server.name.clone(), server);

        let retrieved = config.get_mcp_server("test-server");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-server");
    }
}
