//! Integration tests for Foundry CLI cursor status commands
//!
//! These tests verify the status reporting functionality for Cursor environment,
//! including installation state, configuration validation, and issue detection.

use anyhow::Result;
// Note: Using test helper functions instead of direct command imports
use foundry_mcp::types::responses::InstallationStatus;

mod common;
use common::TestEnvironment;

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
        let install_response = env.install_with_args(install_args).await?;
        assert_eq!(
            install_response.installation_status,
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
        env.install_with_args(install_args).await?;

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

/// Test status command with issues detection
#[test]
fn test_cursor_status_with_issues() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_async(|| async {
        // Install with valid binary
        let install_args = env.install_args("cursor");
        env.install_with_args(install_args).await?;

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
