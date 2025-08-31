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
    use crate::test_utils::TestEnvironment;

    #[test]
    fn test_install_for_cursor_fresh_environment() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();

            let result = install_for_cursor(&binary_path.to_string_lossy(), false).await;

            assert!(
                result.is_ok(),
                "Install should succeed on fresh environment"
            );
            let install_result = result.unwrap();
            assert!(install_result.success);
            assert!(install_result.actions_taken.len() >= 3); // Add server, write config, validate

            // Use assert_fs for rich assertions
            assert!(
                env.cursor_config_path().exists(),
                "Config file should exist"
            );
            assert!(
                env.cursor_config_path().is_file(),
                "Config should be a file"
            );

            // Verify config content
            let config_content = std::fs::read_to_string(env.cursor_config_path()).unwrap();
            assert!(config_content.contains("foundry"));
            assert!(config_content.contains("mcpServers"));
        });
    }

    #[test]
    fn test_install_for_cursor_already_configured() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();

            // Pre-configure with existing foundry server
            env.create_cursor_config(&[("foundry", "/old/foundry/path")])
                .unwrap();

            let result = install_for_cursor(&binary_path.to_string_lossy(), false).await;

            assert!(
                result.is_err(),
                "Install should fail when already configured without force"
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("already configured")
            );
        });
    }

    #[test]
    fn test_install_for_cursor_force_overwrite() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();

            // Pre-configure with existing foundry server
            env.create_cursor_config(&[("foundry", "/old/foundry/path")])
                .unwrap();

            let result = install_for_cursor(&binary_path.to_string_lossy(), true).await;

            assert!(
                result.is_ok(),
                "Install should succeed with force overwrite"
            );
            let install_result = result.unwrap();
            assert!(install_result.success);
            assert!(
                install_result
                    .actions_taken
                    .iter()
                    .any(|action| action.contains("Added Foundry MCP server"))
            );

            // Verify config was updated
            let config_content = std::fs::read_to_string(env.cursor_config_path()).unwrap();
            assert!(config_content.contains(&binary_path.to_string_lossy().to_string()));
        });
    }

    #[test]
    fn test_install_for_cursor_invalid_binary_path() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let result = install_for_cursor("/nonexistent/path/foundry", false).await;

            assert!(
                result.is_err(),
                "Install should fail with invalid binary path"
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Binary path does not exist")
            );
        });
    }

    #[test]
    fn test_uninstall_from_cursor_configured() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Pre-configure with foundry server and another server
            env.create_cursor_config(&[
                ("foundry", "/usr/local/bin/foundry"),
                ("other-server", "/other/binary"),
            ])
            .unwrap();

            let result = uninstall_from_cursor(false, false).await;

            assert!(
                result.is_ok(),
                "Uninstall should succeed when foundry is configured"
            );
            let uninstall_result = result.unwrap();
            assert!(uninstall_result.success);
            assert!(
                uninstall_result
                    .actions_taken
                    .iter()
                    .any(|action| action.contains("Removed Foundry MCP server"))
            );
            assert!(uninstall_result.files_removed.is_empty()); // Config file should remain

            // Verify foundry was removed but other server remains
            let config_content = std::fs::read_to_string(env.cursor_config_path()).unwrap();
            assert!(!config_content.contains("foundry"));
            assert!(config_content.contains("other-server"));
        });
    }

    #[test]
    fn test_uninstall_from_cursor_not_configured() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Create empty config
            env.create_cursor_config(&[]).unwrap();

            let result = uninstall_from_cursor(false, false).await;

            assert!(
                result.is_err(),
                "Uninstall should fail when foundry is not configured"
            );
            assert!(result.unwrap_err().to_string().contains("not configured"));
        });
    }

    #[test]
    fn test_uninstall_from_cursor_force_not_configured() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Create empty config
            env.create_cursor_config(&[]).unwrap();

            let result = uninstall_from_cursor(false, true).await;

            assert!(
                result.is_ok(),
                "Uninstall should succeed with force even when not configured"
            );
            let uninstall_result = result.unwrap();
            assert!(uninstall_result.success);
            assert!(
                uninstall_result
                    .actions_taken
                    .iter()
                    .any(|action| action.contains("not configured"))
            );
        });
    }

    #[test]
    fn test_uninstall_from_cursor_remove_config_when_empty() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Pre-configure with only foundry server
            env.create_cursor_config(&[("foundry", "/usr/local/bin/foundry")])
                .unwrap();

            let result = uninstall_from_cursor(true, false).await;

            assert!(
                result.is_ok(),
                "Uninstall should succeed and remove config when empty"
            );
            let uninstall_result = result.unwrap();
            assert!(uninstall_result.success);
            assert!(
                uninstall_result
                    .actions_taken
                    .iter()
                    .any(|action| action.contains("Removed configuration file"))
            );
            assert!(
                uninstall_result
                    .files_removed
                    .iter()
                    .any(|file| file.contains("mcp.json"))
            );

            // Verify config file was removed
            assert!(!env.cursor_config_path().exists());
        });
    }

    #[test]
    fn test_get_cursor_status_not_installed() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let result = get_cursor_status(false).await;

            assert!(result.is_ok(), "Should be able to get Cursor status");
            let status = result.unwrap();
            assert_eq!(status.name, "cursor");
            assert!(!status.installed);
            assert!(!status.config_exists);
            assert!(!status.binary_accessible);
            assert!(status.issues.len() >= 1);
            assert!(
                status
                    .issues
                    .iter()
                    .any(|issue| issue.contains("does not exist"))
            );
        });
    }

    #[test]
    fn test_get_cursor_status_installed() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();
            env.create_cursor_config(&[("foundry", &binary_path.to_string_lossy())])
                .unwrap();

            let result = get_cursor_status(false).await;

            assert!(result.is_ok(), "Should be able to get Cursor status");
            let status = result.unwrap();
            assert_eq!(status.name, "cursor");
            assert!(status.installed);
            assert!(status.config_exists);
            assert!(status.binary_accessible);
            assert!(status.issues.is_empty());
        });
    }

    #[test]
    fn test_get_cursor_status_detailed() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();
            env.create_cursor_config(&[("foundry", &binary_path.to_string_lossy())])
                .unwrap();

            let result = get_cursor_status(true).await;

            assert!(
                result.is_ok(),
                "Should be able to get detailed Cursor status"
            );
            let status = result.unwrap();
            assert!(status.config_content.is_some());
            let config_content = status.config_content.unwrap();
            assert!(config_content.contains("foundry"));
            assert!(config_content.contains("mcpServers"));
        });
    }

    #[test]
    fn test_get_cursor_status_invalid_config() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Create invalid config file
            std::fs::create_dir_all(&env.cursor_config_dir).unwrap();
            std::fs::write(env.cursor_config_path(), "invalid json content").unwrap();

            let result = get_cursor_status(false).await;

            assert!(result.is_ok(), "Should handle invalid config gracefully");
            let status = result.unwrap();
            assert!(!status.installed);
            assert!(status.config_exists);
            assert!(!status.binary_accessible);
            assert!(
                status
                    .issues
                    .iter()
                    .any(|issue| issue.contains("Failed to read configuration"))
            );
        });
    }

    #[test]
    fn test_get_cursor_status_missing_binary() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Configure with non-existent binary path
            env.create_cursor_config(&[("foundry", "/nonexistent/foundry")])
                .unwrap();

            let result = get_cursor_status(false).await;

            assert!(result.is_ok(), "Should handle missing binary gracefully");
            let status = result.unwrap();
            assert!(status.installed);
            assert!(status.config_exists);
            assert!(!status.binary_accessible);
            assert!(
                status
                    .issues
                    .iter()
                    .any(|issue| issue.contains("does not exist"))
            );
        });
    }

    #[test]
    fn test_is_cursor_configured() {
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // Initially not configured
            assert!(!is_cursor_configured());

            // Configure with foundry server
            let binary_path = env.create_mock_binary("foundry").unwrap();
            env.create_cursor_config(&[("foundry", &binary_path.to_string_lossy())])
                .unwrap();

            // Now should be configured
            assert!(is_cursor_configured());
        });
    }

    #[test]
    fn test_binary_path_validation() {
        let env = TestEnvironment::new().unwrap();
        let binary_path = env.create_mock_binary("foundry").unwrap();

        // Test binary path validation (this should succeed)
        let binary_path_str = binary_path.to_string_lossy().to_string();
        let validation_result = crate::core::installation::validate_binary_path(&binary_path_str);
        assert!(
            validation_result.is_ok(),
            "Binary path validation should succeed for valid path"
        );

        // Test with invalid binary path (this should fail)
        let invalid_result = crate::core::installation::validate_binary_path("/nonexistent/path");
        assert!(
            invalid_result.is_err(),
            "Binary path validation should fail for invalid path"
        );
    }

    #[test]
    fn test_config_validation() {
        let env = TestEnvironment::new().unwrap();
        let binary_path = env.create_mock_binary("foundry").unwrap();

        // Create a config with valid foundry server
        let mut config = crate::core::installation::json_config::McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        };
        let server_config =
            crate::core::installation::create_server_config(&binary_path.to_string_lossy());
        config = crate::core::installation::add_server_to_config(config, "foundry", server_config);

        // This should pass validation since the binary exists
        let result = crate::core::installation::validate_config(&config);
        assert!(result.is_ok());

        // Test with invalid binary path
        let mut bad_config = crate::core::installation::json_config::McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        };
        let bad_server_config =
            crate::core::installation::create_server_config("/nonexistent/path");
        bad_config = crate::core::installation::add_server_to_config(
            bad_config,
            "foundry",
            bad_server_config,
        );

        let result = crate::core::installation::validate_config(&bad_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("command does not exist")
        );
    }
}
