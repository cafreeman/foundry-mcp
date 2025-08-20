use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub log_level: String,
    pub log_format: String,
    pub backup_retention_days: u32,
    pub default_output_format: String,
    pub custom_settings: HashMap<String, String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            backup_retention_days: 7,
            default_output_format: "table".to_string(),
            custom_settings: HashMap::new(),
        }
    }
}

impl CliConfig {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("project-manager-mcp").join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: CliConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "log_level" => Some(self.log_level.clone()),
            "log_format" => Some(self.log_format.clone()),
            "backup_retention_days" => Some(self.backup_retention_days.to_string()),
            "default_output_format" => Some(self.default_output_format.clone()),
            _ => self.custom_settings.get(key).cloned(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "log_level" => {
                if ["trace", "debug", "info", "warn", "error"].contains(&value) {
                    self.log_level = value.to_string();
                } else {
                    return Err(anyhow::anyhow!("Invalid log level: {}", value));
                }
            }
            "log_format" => {
                if ["json", "pretty", "compact"].contains(&value) {
                    self.log_format = value.to_string();
                } else {
                    return Err(anyhow::anyhow!("Invalid log format: {}", value));
                }
            }
            "backup_retention_days" => {
                let days: u32 = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid number for backup_retention_days: {}", value))?;
                self.backup_retention_days = days;
            }
            "default_output_format" => {
                if ["table", "json", "yaml"].contains(&value) {
                    self.default_output_format = value.to_string();
                } else {
                    return Err(anyhow::anyhow!("Invalid output format: {}", value));
                }
            }
            _ => {
                self.custom_settings.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    pub fn list_all(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        settings.insert("log_level".to_string(), self.log_level.clone());
        settings.insert("log_format".to_string(), self.log_format.clone());
        settings.insert("backup_retention_days".to_string(), self.backup_retention_days.to_string());
        settings.insert("default_output_format".to_string(), self.default_output_format.clone());
        
        for (key, value) in &self.custom_settings {
            settings.insert(key.clone(), value.clone());
        }
        
        settings
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}