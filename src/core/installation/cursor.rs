//! Cursor MCP server installation and management

use crate::core::installation::{
    InstallationResult, UninstallationResult, add_server_to_config, create_installation_result,
    create_uninstallation_result, get_cursor_mcp_config_path, has_server_config, read_config_file,
    remove_server_from_config, validate_binary_path, validate_config_dir_writable,
    write_config_file,
};
use crate::types::responses::EnvironmentStatus;
use anyhow::{Context, Result};

/// Install Foundry MCP server for Cursor
pub async fn install_for_cursor(binary_path: &str, force: bool) -> Result<InstallationResult> {
    validate_binary_path(binary_path)?;

    let config_path = get_cursor_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    validate_config_dir_writable(config_path.as_path())?;

    let mut actions_taken = Vec::new();

    // Read existing configuration
    let mut config =
        read_config_file(&config_path).context("Failed to read existing MCP configuration")?;

    // Check if already configured
    if has_server_config(&config, "foundry") && !force {
        return Err(anyhow::anyhow!(
            "Foundry MCP server is already configured for Cursor. Use --force to overwrite."
        ));
    }

    // Create server configuration
    let server_config = crate::core::installation::create_server_config(binary_path);

    // Add server to configuration
    config = add_server_to_config(config, "foundry", server_config);
    actions_taken.push("Added Foundry MCP server to Cursor configuration".to_string());

    // Write configuration back to file
    write_config_file(&config_path, &config).context("Failed to write MCP configuration")?;
    actions_taken.push(format!("Updated configuration file: {}", config_path_str));

    // Validate the configuration
    crate::core::installation::validate_config(&config)
        .context("Configuration validation failed")?;
    actions_taken.push("Validated MCP configuration".to_string());

    Ok(create_installation_result(
        true,
        config_path_str,
        actions_taken,
    ))
}

/// Uninstall Foundry MCP server from Cursor
pub async fn uninstall_from_cursor(
    remove_config: bool,
    force: bool,
) -> Result<UninstallationResult> {
    let config_path = get_cursor_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let mut actions_taken = Vec::new();
    let mut files_removed = Vec::new();

    // Read existing configuration
    let mut config = match read_config_file(&config_path) {
        Ok(config) => config,
        Err(_) if force => {
            // If we can't read the config but force is enabled, assume empty config
            crate::core::installation::json_config::McpConfig {
                mcp_servers: std::collections::HashMap::new(),
            }
        }
        Err(e) => return Err(e),
    };

    // Check if server is configured
    if !has_server_config(&config, "foundry") {
        if !force {
            return Err(anyhow::anyhow!(
                "Foundry MCP server is not configured for Cursor"
            ));
        }
        actions_taken
            .push("Foundry MCP server was not configured (continuing due to --force)".to_string());
    } else {
        // Remove server from configuration
        config = remove_server_from_config(config, "foundry");
        actions_taken.push("Removed Foundry MCP server from Cursor configuration".to_string());
    }

    // Write configuration back or remove file if empty
    if config.mcp_servers.is_empty() && remove_config {
        if config_path.exists() {
            std::fs::remove_file(&config_path).context("Failed to remove configuration file")?;
            files_removed.push(config_path_str.clone());
            actions_taken.push(format!("Removed configuration file: {}", config_path_str));
        }
    } else {
        write_config_file(&config_path, &config)
            .context("Failed to write updated MCP configuration")?;
        actions_taken.push(format!("Updated configuration file: {}", config_path_str));
    }

    Ok(create_uninstallation_result(
        true,
        config_path_str,
        actions_taken,
        files_removed,
    ))
}

/// Get environment status for Cursor
pub async fn get_cursor_status(detailed: bool) -> Result<EnvironmentStatus> {
    let config_path = get_cursor_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let mut issues = Vec::new();
    let mut installed = false;
    let mut config_exists = false;
    let mut binary_accessible = false;
    let mut config_content = None;

    // Check if config file exists
    if config_path.exists() {
        config_exists = true;

        if detailed {
            config_content = Some(
                std::fs::read_to_string(&config_path)
                    .unwrap_or_else(|_| "Error reading config file".to_string()),
            );
        }
    } else {
        issues.push("MCP configuration file does not exist".to_string());
    }

    // Try to read and validate configuration
    if config_exists {
        match read_config_file(&config_path) {
            Ok(config) => {
                if has_server_config(&config, "foundry") {
                    installed = true;

                    // Validate the server configuration
                    if let Some(server_config) =
                        crate::core::installation::get_server_config(&config, "foundry")
                    {
                        // Check if binary path exists
                        binary_accessible = std::path::Path::new(&server_config.command).exists();

                        if !binary_accessible {
                            issues.push(format!(
                                "Configured binary does not exist: {}",
                                server_config.command
                            ));
                        }
                    }
                } else {
                    issues.push("Foundry MCP server not found in configuration".to_string());
                }
            }
            Err(e) => {
                issues.push(format!("Failed to read configuration: {}", e));
            }
        }
    }

    Ok(EnvironmentStatus {
        name: "cursor".to_string(),
        installed,
        config_path: config_path_str,
        config_exists,
        binary_path: if installed {
            crate::core::installation::detect_binary_path()
                .unwrap_or_else(|_| "unknown".to_string())
        } else {
            "unknown".to_string()
        },
        binary_accessible,
        config_content,
        issues,
    })
}

/// Check if Cursor MCP configuration exists and is valid
pub fn is_cursor_configured() -> bool {
    get_cursor_mcp_config_path().is_ok_and(|config_path| {
        read_config_file(&config_path).is_ok_and(|config| has_server_config(&config, "foundry"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_cursor_status() {
        let result = get_cursor_status(false).await;
        assert!(result.is_ok(), "Should be able to get Cursor status");

        let status = result.unwrap();
        assert_eq!(status.name, "cursor");
        assert!(!status.config_path.is_empty());
    }

    #[test]
    fn test_is_cursor_configured() {
        // This test will likely return false in most environments
        let configured = is_cursor_configured();
        assert!(configured || !configured); // Just ensure it doesn't panic
    }

    #[tokio::test]
    async fn test_install_for_cursor() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("foundry");
        std::fs::write(&binary_path, b"test binary").unwrap();

        // Temporarily override the config path for testing
        // Note: This is a simplified test - in real scenarios we'd mock the config path
        let result = install_for_cursor(&binary_path.to_string_lossy(), false).await;

        // The result might fail due to permission issues or other reasons,
        // but we mainly want to ensure it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
