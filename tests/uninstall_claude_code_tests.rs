//! Integration tests for Foundry CLI Claude Code uninstallation commands
//!
//! These tests verify the full uninstallation workflow for Claude Code environment,
//! including MCP server deregistration from CLI and subagent template cleanup.

use anyhow::Result;
// Note: Using test helper functions instead of direct command imports
use foundry_mcp::types::responses::InstallationStatus;

// Import TestEnvironment from the main crate
use foundry_mcp::test_utils::TestEnvironment;

/// Test Claude Code uninstall end-to-end workflow
#[test]
fn test_uninstall_claude_code_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // First install Claude Code
        let install_response = env.install_and_parse("claude-code").await?;
        assert_eq!(
            install_response.installation_status,
            InstallationStatus::Success
        );

        // Verify subagent template was created
        let subagent_path = env.claude_subagent_path();
        assert!(
            subagent_path.exists(),
            "Subagent should exist after install"
        );

        // Uninstall Claude Code
        let uninstall_response = env.uninstall_and_parse("claude-code", false).await?;

        // Verify uninstall response
        assert_eq!(uninstall_response.target, "claude-code");
        assert!(!uninstall_response.actions_taken.is_empty());

        // Verify subagent template was removed
        assert!(
            !subagent_path.exists(),
            "Subagent should be removed after uninstall"
        );

        // Verify response mentions template removal
        assert!(
            uninstall_response
                .actions_taken
                .iter()
                .any(|action| action.contains("subagent")),
            "Actions should mention subagent removal"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test Claude Code uninstall with template removal verification
#[test]
fn test_uninstall_claude_code_template_removal() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // Install Claude Code
        let install_response = env.install_and_parse("claude-code").await?;
        assert_eq!(
            install_response.installation_status,
            InstallationStatus::Success
        );

        // Verify subagent template exists
        let subagent_path = env.claude_subagent_path();
        assert!(
            subagent_path.exists(),
            "Subagent should exist after install"
        );

        // Uninstall Claude Code
        let uninstall_response = env.uninstall_and_parse("claude-code", false).await?;

        // Verify subagent template was completely removed
        assert!(
            !subagent_path.exists(),
            "Subagent template should be removed after uninstall"
        );

        // Verify response mentions template removal
        assert!(
            uninstall_response
                .actions_taken
                .iter()
                .any(|action| action.contains("Removed Claude subagent template")),
            "Actions should mention removing subagent template"
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test uninstall of Claude Code when not installed
#[test]
fn test_uninstall_claude_code_not_installed() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing (but don't install)
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // Try to uninstall when nothing is installed
        let uninstall_args = env.uninstall_args("claude-code", false);
        let result = env.uninstall_with_args(uninstall_args).await;

        assert!(result.is_err(), "Uninstall should fail when not installed");
        let error_msg = format!("{:#}", result.unwrap_err());
        assert!(
            error_msg.contains("Failed to unregister MCP server from Claude Code"),
            "Error should mention unregistration failure. Actual: {}",
            error_msg
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test uninstall when Claude Code CLI is not available
#[test]
fn test_uninstall_claude_code_cli_not_available() -> Result<()> {
    let env = TestEnvironment::new()?;

    let _ = env.with_env_and_path_async(|| async {
        // Don't create mock claude binary - test without claude CLI available
        let uninstall_args = env.uninstall_args("claude-code", false);
        let result = env.uninstall_with_args(uninstall_args).await;

        assert!(
            result.is_err(),
            "Uninstall should fail when claude CLI is not available"
        );
        let error_msg = format!("{:#}", result.unwrap_err());
        // Since no mock binary is created, claude CLI should not be available
        assert!(
            error_msg.contains("Claude Code CLI is not available"),
            "Error should mention that claude CLI is not available. Actual: {}",
            error_msg
        );

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

/// Test human-readable uninstall output format for Claude Code
#[test]
fn test_uninstall_claude_code_human_readable_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // First install to have something to uninstall
        env.install_and_parse("claude-code").await?;

        // Test human-readable uninstall output
        let output = env.uninstall_text_output("claude-code", false).await?;

        // Verify output contains expected human-readable elements
        assert!(output.contains("✅"), "Output should contain success icon");
        assert!(
            output.contains("Successfully uninstalled"),
            "Output should contain success message"
        );
        assert!(
            output.contains("Foundry MCP from claude-code"),
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
    });

    Ok(())
}

/// Test JSON output format for Claude Code uninstall
#[test]
fn test_uninstall_claude_code_json_output() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // First install to have something to uninstall
        env.install_and_parse("claude-code").await?;

        // Test JSON uninstall output using the helper that forces JSON mode
        let response = env.uninstall_and_parse("claude-code", false).await?;

        // Verify structured data is available and correct
        assert_eq!(response.target, "claude-code");
        assert_eq!(response.uninstallation_status, InstallationStatus::Success);
        assert!(!response.config_path.is_empty());
        assert!(!response.actions_taken.is_empty());

        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
