//! Integration tests for Foundry CLI Claude Code installation commands
//!
//! These tests verify the full installation workflow for Claude Code environment,
//! including MCP server registration with CLI and subagent template creation.

use anyhow::Result;
// Note: Using test helper functions instead of direct command imports
use foundry_mcp::types::responses::InstallationStatus;

// Import TestEnvironment from the main crate
use foundry_mcp::test_utils::TestEnvironment;

/// Test Claude Code installation end-to-end workflow with template creation
#[test]
fn test_install_claude_code_end_to_end() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create a mock claude command for testing
    let _mock_claude_path = env.create_mock_claude_binary()?;

    // Use TestEnvironment pattern with proper PATH isolation
    let _ = env.with_env_and_path_async(|| async {
        // Verify subagent doesn't exist initially
        let subagent_path = env.claude_subagent_path();
        assert!(
            !subagent_path.exists(),
            "Subagent should not exist initially"
        );

        // Execute install command
        let install_args = env.install_args("claude-code");
        let response = env.install_with_args(install_args).await?;

        // Verify response structure
        assert_eq!(response.target, "claude-code");
        assert_eq!(response.installation_status, InstallationStatus::Success);

        // Verify subagent template was created with expected content
        env.verify_claude_subagent_template()?;

        // Verify response mentions subagent creation
        assert!(
            response
                .actions_taken
                .iter()
                .any(|action| action.contains("subagent")),
            "Response should mention subagent creation"
        );

        // Verify we get a successful response structure
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
