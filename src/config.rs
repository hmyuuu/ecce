use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub url: String,
    pub key: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
    #[serde(default)]
    pub default_profile: Option<String>,
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
}
