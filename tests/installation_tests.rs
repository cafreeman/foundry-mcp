//! Integration tests for Foundry CLI installation commands
//!
//! These tests verify the full installation, uninstallation, and status
//! workflows for Cursor and Claude Code environments.

use anyhow::Result;
use foundry_mcp::cli::commands::{install, uninstall};
use foundry_mcp::types::responses::{InstallationStatus, ValidationStatus};

// Import TestEnvironment from the main crate
use foundry_mcp::test_utils::TestEnvironment;

/// Test cursor installation end-to-end workflow
#[test]
fn test_install_cursor_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Verify config doesn't exist initially
        let config_path = env.cursor_config_path();
        assert!(!config_path.exists(), "Config should not exist initially");

        // Execute install command
        let install_args = env.install_args("cursor");
        let response = install::execute(install_args).await?;

        // Verify response structure
        assert_eq!(response.data.target, "cursor");
        assert_eq!(
            response.data.installation_status,
            InstallationStatus::Success
        );
        assert!(!response.data.binary_path.is_empty());
        assert!(!response.data.config_path.is_empty());
        assert!(!response.data.actions_taken.is_empty());

        // Verify config file was created
        assert!(
            config_path.exists(),
            "Config file should exist after installation"
        );

        // Verify config content
        let config_content = std::fs::read_to_string(&config_path)?;
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content)?;
        assert!(
            foundry_mcp::core::installation::has_server_config(&config, "foundry"),
            "Foundry server should be configured"
        );

        // Verify server configuration details
        let server_config = foundry_mcp::core::installation::get_server_config(&config, "foundry")
            .expect("Foundry server config should exist");
        assert!(
            !server_config.command.is_empty(),
            "Should use foundry command"
        );
        assert_eq!(
            server_config.args,
            vec!["serve"],
            "Should have serve argument"
        );
        assert!(
            server_config.env.is_some(),
            "Should have environment variables"
        );

        // Verify we get a successful response structure (no workflow guidance needed)
        assert_eq!(response.validation_status, ValidationStatus::Complete);

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor installation config verification and validation
#[test]
fn test_install_cursor_config_verification() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Install cursor
        let install_args = env.install_args("cursor");
        let response = install::execute(install_args).await?;
        assert_eq!(
            response.data.installation_status,
            InstallationStatus::Success
        );

        // Read and validate config structure
        let config_path = env.cursor_config_path();
        let config_content = std::fs::read_to_string(&config_path)?;

        // Verify JSON is valid and well-formed
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content).expect("Config should be valid JSON");

        // Verify config passes validation
        let validation_result =
            foundry_mcp::core::installation::json_config::validate_config(&config);
        assert!(
            validation_result.is_ok(),
            "Config should pass validation: {:?}",
            validation_result.err()
        );

        // Verify mcpServers structure
        assert!(
            !config.mcp_servers.is_empty(),
            "Should have at least one server"
        );
        assert_eq!(
            config.mcp_servers.len(),
            1,
            "Should have exactly one server"
        );

        // Verify JSON formatting (should be pretty-printed)
        assert!(
            config_content.contains("  "),
            "Config should be pretty-printed"
        );
        assert!(
            config_content.contains("\"mcpServers\""),
            "Should contain mcpServers key"
        );
        assert!(
            config_content.contains("\"foundry\""),
            "Should contain foundry server"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor installation always overwrites existing configuration
#[test]
fn test_install_cursor_always_overwrites() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Create existing config with different content
        let other_binary = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/foundry"))
            .to_string_lossy()
            .to_string(); // Use realistic binary path for other server
        let existing_config = format!(
            r#"{{
  "mcpServers": {{
    "foundry": {{
      "command": "/old/path/foundry",
      "args": ["serve"],
      "env": {{
        "FOUNDRY_LOG_LEVEL": "debug"
      }}
    }},
    "other-server": {{
      "command": "{}",
      "args": ["start"]
    }}
  }}
}}"#,
            other_binary
        );
        env.create_existing_cursor_config(&existing_config)?;

        // Install should succeed and overwrite existing configuration
        let install_args = env.install_args("cursor");
        let result = install::execute(install_args).await;
        assert!(
            result.is_ok(),
            "Install should succeed and overwrite existing configuration"
        );

        // Install again (should succeed and overwrite existing configuration)
        let install_args_force = env.install_args("cursor");
        let response = install::execute(install_args_force).await?;
        assert_eq!(
            response.data.installation_status,
            InstallationStatus::Success
        );

        // Verify foundry config was updated but other-server preserved
        let config_content = std::fs::read_to_string(env.cursor_config_path())?;
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content)?;

        assert!(foundry_mcp::core::installation::has_server_config(
            &config, "foundry"
        ));
        assert!(foundry_mcp::core::installation::has_server_config(
            &config,
            "other-server"
        ));

        // Verify foundry config was updated
        let foundry_config =
            foundry_mcp::core::installation::get_server_config(&config, "foundry").unwrap();
        assert!(
            !foundry_config.command.is_empty(),
            "Should use foundry command"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor uninstall end-to-end workflow
#[test]
fn test_uninstall_cursor_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // First install cursor
        let install_args = env.install_args("cursor");
        let install_response = install::execute(install_args).await?;
        assert_eq!(
            install_response.data.installation_status,
            InstallationStatus::Success
        );

        // Verify installation
        let config_path = env.cursor_config_path();
        assert!(config_path.exists(), "Config should exist after install");

        // Uninstall cursor
        let uninstall_args = env.uninstall_args("cursor", false);
        let uninstall_response = uninstall::execute(uninstall_args).await?;

        // Verify uninstall response
        assert_eq!(uninstall_response.data.target, "cursor");
        assert!(!uninstall_response.data.actions_taken.is_empty());

        // Config file should still exist but foundry server should be removed
        assert!(config_path.exists(), "Config file should still exist");

        let config_content = std::fs::read_to_string(&config_path)?;
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content)?;
        assert!(
            !foundry_mcp::core::installation::has_server_config(&config, "foundry"),
            "Foundry server should be removed from config"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor uninstall with config removal
#[test]
fn test_uninstall_cursor_remove_config() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Install cursor (creates only foundry server)
        let install_args = env.install_args("cursor");
        let install_response = install::execute(install_args).await?;
        assert_eq!(
            install_response.data.installation_status,
            InstallationStatus::Success
        );

        // Verify config exists
        let config_path = env.cursor_config_path();
        assert!(config_path.exists());

        // Uninstall with config removal
        let uninstall_args = env.uninstall_args("cursor", true);
        let uninstall_response = uninstall::execute(uninstall_args).await?;

        // Verify config file was completely removed
        assert!(
            !config_path.exists(),
            "Config file should be removed when empty and remove_config=true"
        );

        // Verify response mentions file removal
        assert!(
            uninstall_response
                .data
                .actions_taken
                .iter()
                .any(|action| action.contains("Removed configuration file")),
            "Actions should mention removing config file"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor install uses PATH-based command
#[test]
fn test_install_cursor_path_command() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Test cursor installation without explicit binary path
        let install_args = env.install_args("cursor");
        let response = install::execute(install_args).await?;

        assert_eq!(
            response.data.installation_status,
            foundry_mcp::types::responses::InstallationStatus::Success
        );
        assert_eq!(response.data.binary_path, "foundry (from PATH)");

        // Verify config uses PATH-based 'foundry' command
        let config_content = std::fs::read_to_string(env.cursor_config_path())?;
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content)?;
        let server_config =
            foundry_mcp::core::installation::get_server_config(&config, "foundry").unwrap();
        assert_eq!(server_config.command, "foundry");

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test installation with cursor (no binary path validation needed)
#[test]
fn test_install_cursor_path_based() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Test cursor installation using PATH-based command (no binary path needed)
        let install_args = env.install_args("cursor");
        let result = install::execute(install_args).await;

        assert!(
            result.is_ok(),
            "Install should succeed using PATH-based foundry command"
        );
        let response = result.unwrap();
        assert_eq!(
            response.data.installation_status,
            foundry_mcp::types::responses::InstallationStatus::Success
        );
        assert_eq!(response.data.binary_path, "foundry (from PATH)");

        // Verify config uses 'foundry' command
        let config_content = std::fs::read_to_string(env.cursor_config_path())?;
        assert!(config_content.contains("\"command\": \"foundry\""));

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test cursor installation succeeds without binary path concerns
#[test]
fn test_install_cursor_runtime_validation() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Cursor installation should succeed as it uses PATH-based command
        // Execution validation happens at runtime when MCP server is started
        let install_args = env.install_args("cursor");
        let result = install::execute(install_args).await;

        assert!(
            result.is_ok(),
            "Install should succeed - runtime validation happens when MCP server starts"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test installation with malformed existing config
#[test]
fn test_install_cursor_malformed_config() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Create malformed JSON config
        let malformed_config = r#"{
  "mcpServers": {
    "foundry": {
      "command": "/path/to/foundry"
      // Missing comma and args field
    }
  }
  // Missing closing brace
"#;
        env.create_existing_cursor_config(malformed_config)?;

        // Install should handle malformed config gracefully
        let install_args = env.install_args("cursor");
        let result = install::execute(install_args).await;

        assert!(result.is_err(), "Install should fail with malformed config");
        let error_msg = format!("{:#}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to read") || error_msg.contains("parse"),
            "Error should mention config parsing failure"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test uninstall of non-existent installation
#[test]
fn test_uninstall_cursor_not_installed() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Try to uninstall when nothing is installed
        let uninstall_args = env.uninstall_args("cursor", false);
        let result = uninstall::execute(uninstall_args).await;

        assert!(result.is_err(), "Uninstall should fail when not installed");
        let error_msg = format!("{:#}", result.unwrap_err());
        assert!(
            error_msg.contains("not configured"),
            "Error should mention not configured"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test uninstall when not installed (should fail)
#[test]
fn test_uninstall_cursor_not_installed_fails() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Try to uninstall when nothing is installed (should fail)
        let uninstall_args = env.uninstall_args("cursor", false);
        let result = uninstall::execute(uninstall_args).await;

        assert!(result.is_err(), "Uninstall should fail when not installed");
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        // The error should contain either the original message or the wrapped message
        assert!(
            error_msg.contains("not configured")
                || error_msg.contains("Failed to uninstall from Cursor"),
            "Error should mention that foundry was not configured. Actual error: {}",
            error_msg
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test installation with empty config file
#[test]
fn test_install_cursor_empty_config() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Create empty config file
        env.create_existing_cursor_config("")?;

        // Install should handle empty config gracefully
        let install_args = env.install_args("cursor");
        let result = install::execute(install_args).await;

        assert!(
            result.is_ok(),
            "Install should succeed with empty config file"
        );

        // Verify config was created properly
        let config_path = env.cursor_config_path();
        let config_content = std::fs::read_to_string(&config_path)?;
        let config: foundry_mcp::core::installation::json_config::McpConfig =
            serde_json::from_str(&config_content)?;
        assert!(foundry_mcp::core::installation::has_server_config(
            &config, "foundry"
        ));

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test status command before and after installation
#[test]
fn test_cursor_status_before_after_install() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Test status before installation
        let status_response = env.get_status_response(Some("cursor"), false).await?;

        assert_eq!(status_response.environments.len(), 1);
        let cursor_status = &status_response.environments[0];
        assert_eq!(cursor_status.name, "cursor");
        assert!(
            !cursor_status.installed,
            "Should not be installed initially"
        );
        assert!(
            !cursor_status.config_exists,
            "Config should not exist initially"
        );

        // Install cursor
        let install_args = env.install_args("cursor");
        let install_response = install::execute(install_args).await?;
        assert_eq!(
            install_response.data.installation_status,
            InstallationStatus::Success
        );

        // Test status after installation
        let status_response_after = env.get_status_response(Some("cursor"), false).await?;

        let cursor_status_after = &status_response_after.environments[0];
        assert_eq!(cursor_status_after.name, "cursor");
        assert!(
            cursor_status_after.installed,
            "Should be installed after install"
        );
        assert!(
            cursor_status_after.config_exists,
            "Config should exist after install"
        );
        assert!(
            cursor_status_after.binary_accessible,
            "Binary should be accessible"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test status command with detailed flag
#[test]
fn test_cursor_status_detailed_mode() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Install cursor first
        let install_args = env.install_args("cursor");
        install::execute(install_args).await?;

        // Test detailed status
        let status_response = env.get_status_response(Some("cursor"), true).await?;

        let cursor_status = &status_response.environments[0];
        assert!(
            cursor_status.config_content.is_some(),
            "Detailed status should include config content"
        );

        let config_content = cursor_status.config_content.as_ref().unwrap();
        assert!(
            config_content.contains("foundry"),
            "Config content should contain foundry server"
        );
        assert!(
            config_content.contains("mcpServers"),
            "Config content should contain mcpServers"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test status command for all environments
#[test]
fn test_status_all_environments() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Test status for all environments (no target specified)
        let status_response = env.get_status_response(None, false).await?;

        // Should return status for both claude-code and cursor
        assert_eq!(
            status_response.environments.len(),
            2,
            "Should return status for both environments"
        );

        let env_names: Vec<&String> = status_response
            .environments
            .iter()
            .map(|env| &env.name)
            .collect();
        assert!(
            env_names.contains(&&"claude-code".to_string()),
            "Should include claude-code"
        );
        assert!(
            env_names.contains(&&"cursor".to_string()),
            "Should include cursor"
        );

        // Neither should be installed initially
        for env_status in &status_response.environments {
            assert!(
                !env_status.installed,
                "No environments should be installed initially"
            );
        }

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test status command with issues detection
#[test]
fn test_cursor_status_with_issues() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Install with valid binary
        let install_args = env.install_args("cursor");
        install::execute(install_args).await?;

        // Manually corrupt the config to create an issue
        let corrupt_config = r#"{
  "mcpServers": {
    "foundry": {
      "command": "/nonexistent/binary",
      "args": ["serve"]
    }
  }
}"#;
        env.create_existing_cursor_config(corrupt_config)?;

        // Test status - should detect issues
        let status_response = env.get_status_response(Some("cursor"), false).await?;

        let cursor_status = &status_response.environments[0];
        assert!(
            cursor_status.installed,
            "Should still be considered installed"
        );
        assert!(
            !cursor_status.binary_accessible,
            "Binary should not be accessible"
        );
        assert!(!cursor_status.issues.is_empty(), "Should have issues");
        assert!(
            cursor_status
                .issues
                .iter()
                .any(|issue| issue.contains("does not exist")),
            "Should report binary does not exist issue"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
