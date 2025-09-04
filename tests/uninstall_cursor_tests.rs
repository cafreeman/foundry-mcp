//! Integration tests for Foundry CLI cursor uninstallation commands
//!
//! These tests verify the full uninstallation workflow for Cursor environment,
//! including MCP server removal from config and template cleanup.

use anyhow::Result;
// Note: Using test helper functions instead of direct command imports
use foundry_mcp::types::responses::InstallationStatus;

// Import TestEnvironment from the main crate
use foundry_mcp::test_utils::TestEnvironment;

/// Test cursor uninstall end-to-end workflow
#[test]
fn test_uninstall_cursor_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // First install cursor
        let install_response = env.install_and_parse("cursor").await?;
        assert_eq!(
            install_response.installation_status,
            InstallationStatus::Success
        );

        // Verify installation
        let config_path = env.cursor_config_path();
        assert!(config_path.exists(), "Config should exist after install");

        // Uninstall cursor
        let uninstall_response = env.uninstall_and_parse("cursor", false).await?;

        // Verify uninstall response
        assert_eq!(uninstall_response.target, "cursor");
        assert!(!uninstall_response.actions_taken.is_empty());

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
        let install_response = env.install_and_parse("cursor").await?;
        assert_eq!(
            install_response.installation_status,
            InstallationStatus::Success
        );

        // Verify config exists
        let config_path = env.cursor_config_path();
        assert!(config_path.exists());

        // Uninstall with config removal
        let uninstall_response = env.uninstall_and_parse("cursor", true).await?;

        // Verify config file was completely removed
        assert!(
            !config_path.exists(),
            "Config file should be removed when empty and remove_config=true"
        );

        // Verify response mentions file removal
        assert!(
            uninstall_response
                .actions_taken
                .iter()
                .any(|action| action.contains("Removed configuration file")),
            "Actions should mention removing config file"
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
        let result = env.uninstall_with_args(uninstall_args).await;

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
        let result = env.uninstall_with_args(uninstall_args).await;

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

/// Test human-readable uninstall output format
#[test]
fn test_uninstall_human_readable_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // First install to have something to uninstall
        env.install_and_parse("cursor").await?;

        // Test human-readable uninstall output
        let output = env.uninstall_text_output("cursor", false).await?;

        // Verify output contains expected human-readable elements
        assert!(output.contains("✅"), "Output should contain success icon");
        assert!(
            output.contains("Successfully uninstalled"),
            "Output should contain success message"
        );
        assert!(
            output.contains("Foundry MCP from cursor"),
            "Output should mention target"
        );
        assert!(
            output.contains("📁 Config:"),
            "Output should show config path"
        );
        assert!(
            output.contains("📋 Actions taken:"),
            "Output should list actions"
        );
        assert!(
            output.contains("🎉 Uninstallation complete!"),
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

/// Test JSON output format for uninstall
#[test]
fn test_uninstall_json_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // First install to have something to uninstall
        env.install_and_parse("cursor").await?;

        // Test JSON uninstall output using the helper that forces JSON mode
        let response = env.uninstall_and_parse("cursor", false).await?;

        // Verify structured data is available and correct
        assert_eq!(response.target, "cursor");
        assert_eq!(response.uninstallation_status, InstallationStatus::Success);
        assert!(!response.config_path.is_empty());
        assert!(!response.actions_taken.is_empty());

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
