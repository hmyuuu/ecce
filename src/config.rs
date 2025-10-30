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
}
