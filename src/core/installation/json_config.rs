//! JSON configuration file management utilities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// MCP server configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

/// MCP configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// Create a new MCP server configuration entry
pub fn create_server_config(binary_path: &str) -> McpServerConfig {
    McpServerConfig {
        command: binary_path.to_string(),
        args: vec!["serve".to_string()],
        env: Some(HashMap::from([(
            "FOUNDRY_LOG_LEVEL".to_string(),
            "info".to_string(),
        )])),
    }
}

/// Read MCP configuration from a JSON file
pub fn read_config_file(config_path: &Path) -> Result<McpConfig> {
    if !config_path.exists() {
        // Return empty config if file doesn't exist
        return Ok(McpConfig {
            mcp_servers: HashMap::new(),
        });
    }

    let content = std::fs::read_to_string(config_path).context(format!(
        "Failed to read config file: {}",
        config_path.display()
    ))?;

    if content.trim().is_empty() {
        return Ok(McpConfig {
            mcp_servers: HashMap::new(),
        });
    }

    let config: McpConfig = serde_json::from_str(&content).context(format!(
        "Failed to parse config file: {}",
        config_path.display()
    ))?;

    Ok(config)
}

/// Write MCP configuration to a JSON file
pub fn write_config_file(config_path: &Path, config: &McpConfig) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).context(format!(
            "Failed to create config directory: {}",
            parent.display()
        ))?;
    }

    let content =
        serde_json::to_string_pretty(config).context("Failed to serialize config to JSON")?;

    std::fs::write(config_path, content).context(format!(
        "Failed to write config file: {}",
        config_path.display()
    ))?;

    Ok(())
}

/// Add or update a server in the MCP configuration
pub fn add_server_to_config(
    mut config: McpConfig,
    server_name: &str,
    server_config: McpServerConfig,
) -> McpConfig {
    config
        .mcp_servers
        .insert(server_name.to_string(), server_config);
    config
}

/// Remove a server from the MCP configuration
pub fn remove_server_from_config(mut config: McpConfig, server_name: &str) -> McpConfig {
    config.mcp_servers.remove(server_name);
    config
}

/// Check if a server is already configured
pub fn has_server_config(config: &McpConfig, server_name: &str) -> bool {
    config.mcp_servers.contains_key(server_name)
}

/// Get server configuration if it exists
pub fn get_server_config<'a>(
    config: &'a McpConfig,
    server_name: &str,
) -> Option<&'a McpServerConfig> {
    config.mcp_servers.get(server_name)
}

/// Validate MCP configuration
pub fn validate_config(config: &McpConfig) -> Result<()> {
    for (server_name, server_config) in &config.mcp_servers {
        if server_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Server name cannot be empty"));
        }

        if server_config.command.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Server '{}' has empty command",
                server_name
            ));
        }

        // Check if command path exists
        let command_path = Path::new(&server_config.command);
        if !command_path.exists() {
            return Err(anyhow::anyhow!(
                "Server '{}' command does not exist: {}",
                server_name,
                server_config.command
            ));
        }
    }

    Ok(())
}

/// Format configuration for display
pub fn format_config_for_display(config: &McpConfig) -> String {
    if config.mcp_servers.is_empty() {
        return "No MCP servers configured".to_string();
    }

    let mut output = format!("Configured MCP servers ({}):\n", config.mcp_servers.len());

    for (name, server_config) in &config.mcp_servers {
        output.push_str(&format!(
            "• {}: {} {:?}\n",
            name, server_config.command, server_config.args
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_server_config() {
        let config = create_server_config("/usr/bin/foundry");
        assert_eq!(config.command, "/usr/bin/foundry");
        assert_eq!(config.args, vec!["serve"]);
        assert!(config.env.is_some());
    }

    #[test]
    fn test_add_server_to_config() {
        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };

        let server_config = create_server_config("/usr/bin/foundry");
        config = add_server_to_config(config, "foundry", server_config.clone());

        assert!(has_server_config(&config, "foundry"));
        assert_eq!(config.mcp_servers.len(), 1);

        let retrieved = get_server_config(&config, "foundry").unwrap();
        assert_eq!(retrieved.command, server_config.command);
    }

    #[test]
    fn test_remove_server_from_config() {
        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };

        let server_config = create_server_config("/usr/bin/foundry");
        config = add_server_to_config(config, "foundry", server_config);
        assert!(has_server_config(&config, "foundry"));

        config = remove_server_from_config(config, "foundry");
        assert!(!has_server_config(&config, "foundry"));
        assert_eq!(config.mcp_servers.len(), 0);
    }

    #[test]
    fn test_read_write_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        // Create test config
        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };
        let server_config = create_server_config("/usr/bin/foundry");
        config = add_server_to_config(config, "foundry", server_config);

        // Write config
        write_config_file(&config_path, &config).unwrap();
        assert!(config_path.exists());

        // Read config back
        let read_config = read_config_file(&config_path).unwrap();
        assert!(has_server_config(&read_config, "foundry"));
        assert_eq!(read_config.mcp_servers.len(), 1);
    }

    #[test]
    fn test_validate_config_valid() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("foundry");
        std::fs::write(&binary_path, b"test").unwrap();

        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };
        let server_config = create_server_config(&binary_path.to_string_lossy());
        config = add_server_to_config(config, "foundry", server_config);

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_invalid_command() {
        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };
        let server_config = create_server_config("/nonexistent/command");
        config = add_server_to_config(config, "foundry", server_config);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_format_config_for_display() {
        let mut config = McpConfig {
            mcp_servers: HashMap::new(),
        };

        // Empty config
        let display = format_config_for_display(&config);
        assert!(display.contains("No MCP servers configured"));

        // Config with server
        let server_config = create_server_config("/usr/bin/foundry");
        config = add_server_to_config(config, "foundry", server_config);

        let display = format_config_for_display(&config);
        assert!(display.contains("Configured MCP servers (1)"));
        assert!(display.contains("foundry"));
    }
}
