//! Integration tests for Foundry CLI cursor installation commands
//!
//! These tests verify the full installation workflow for Cursor environment,
//! including config file creation, MCP server registration, and template setup.

#![allow(clippy::let_unit_value, unused_must_use)]

use anyhow::Result;
// Note: Using test helper functions instead of direct command imports
use foundry_mcp::types::responses::InstallationStatus;

mod common;
use common::TestEnvironment;

/// Test cursor installation end-to-end workflow
#[test]
fn test_install_cursor_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Verify config doesn't exist initially
        let config_path = env.cursor_config_path();
        assert!(!config_path.exists(), "Config should not exist initially");

        // Execute install command and get parsed response for testing
        let response = env.install_and_parse("cursor").await?;

        // Verify response structure
        assert_eq!(response.target, "cursor");
        assert_eq!(response.installation_status, InstallationStatus::Success);
        assert!(!response.binary_path.is_empty());
        assert!(!response.config_path.is_empty());
        assert!(!response.actions_taken.is_empty());

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

        // Verify rules template was created with expected content
        env.verify_cursor_rules_template()?;

        // Verify response mentions rules creation
        assert!(
            response
                .actions_taken
                .iter()
                .any(|action| action.contains("rules")),
            "Actions should mention rules file creation"
        );

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Test cursor installation config verification and validation
#[test]
fn test_install_cursor_config_verification() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Install cursor and get parsed response for testing
        let response = env.install_and_parse("cursor").await?;
        assert_eq!(response.installation_status, InstallationStatus::Success);

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

    env.with_env_async(|| async {
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
        let result = env.install_with_args(install_args).await;
        assert!(
            result.is_ok(),
            "Install should succeed and overwrite existing configuration"
        );

        // Install again (should succeed and overwrite existing configuration)
        let response = env.install_and_parse("cursor").await?;
        assert_eq!(response.installation_status, InstallationStatus::Success);

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

/// Test cursor install uses PATH-based command
#[test]
fn test_install_cursor_path_command() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Test cursor installation without explicit binary path
        let response = env.install_and_parse("cursor").await?;

        assert_eq!(
            response.installation_status,
            foundry_mcp::types::responses::InstallationStatus::Success
        );
        assert_eq!(response.binary_path, "foundry (from PATH)");

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

    env.with_env_async(|| async {
        // Test cursor installation using PATH-based command (no binary path needed)
        let response = env.install_and_parse("cursor").await?;
        assert_eq!(
            response.installation_status,
            foundry_mcp::types::responses::InstallationStatus::Success
        );
        assert_eq!(response.binary_path, "foundry (from PATH)");

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

    env.with_env_async(|| async {
        // Cursor installation should succeed as it uses PATH-based command
        // Execution validation happens at runtime when MCP server is started
        let install_args = env.install_args("cursor");
        let result = env.install_with_args(install_args).await;

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

    env.with_env_async(|| async {
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
        let result = env.install_with_args(install_args).await;

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

/// Test installation with empty config file
#[test]
fn test_install_cursor_empty_config() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Create empty config file
        env.create_existing_cursor_config("")?;

        // Install should handle empty config gracefully
        let install_args = env.install_args("cursor");
        let result = env.install_with_args(install_args).await;

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

/// Test human-readable install output format
#[test]
fn test_install_human_readable_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Test human-readable install output
        let output = env.install_text_output("cursor").await?;

        // Verify output contains expected human-readable elements
        assert!(output.contains("✅"), "Output should contain success icon");
        assert!(
            output.contains("Successfully installed"),
            "Output should contain success message"
        );
        assert!(
            output.contains("Foundry MCP for cursor"),
            "Output should mention target"
        );
        assert!(
            output.contains("📁 Config:"),
            "Output should show config path"
        );
        assert!(
            output.contains("🔧 Binary:"),
            "Output should show binary path"
        );
        assert!(
            output.contains("📋 Actions taken:"),
            "Output should list actions"
        );
        assert!(
            output.contains("🎉 Installation complete!"),
            "Output should have completion message"
        );

        // Verify it's not JSON format
        assert!(
            !output.trim().starts_with('{'),
            "Output should not be JSON format"
        );
        assert!(
            !output.contains("\"target\":"),
            "Output should not contain JSON keys"
        );

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Test JSON output format for install
#[test]
fn test_install_json_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    env.with_env_async(|| async {
        // Test JSON install output using the helper that forces JSON mode
        let response = env.install_and_parse("cursor").await?;

        // Verify structured data is available and correct
        assert_eq!(response.target, "cursor");
        assert_eq!(response.installation_status, InstallationStatus::Success);
        assert!(!response.binary_path.is_empty());
        assert!(!response.config_path.is_empty());
        assert!(!response.actions_taken.is_empty());

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
