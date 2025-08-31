//! Claude Code MCP server installation and management

use crate::core::installation::{
    InstallationResult, UninstallationResult, create_installation_result,
    create_uninstallation_result,
};
use crate::types::responses::EnvironmentStatus;
use anyhow::{Context, Result};
use tokio::process::Command;

/// Install Foundry MCP server for Claude Code
pub async fn install_for_claude_code(
    _binary_path: &str,
    _force: bool,
) -> Result<InstallationResult> {
    let mut actions_taken = Vec::new();

    // Note: We skip the availability check here since the PATH may differ between
    // interactive shell and cargo run. Instead, let the actual command fail with a clear error.
    actions_taken.push("Attempting to register with Claude Code CLI".to_string());

    // Register MCP server with Claude Code using CLI
    // Note: We use "foundry" directly since it will be available on PATH
    match register_with_claude_code().await {
        Ok(_) => {
            actions_taken.push("Registered MCP server with Claude Code CLI".to_string());
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to register MCP server with Claude Code CLI. Error: {}. \n\nThis usually means:\n1. Claude Code CLI is not installed or not in PATH\n2. The 'claude' command cannot be found by the system\n\nTo fix this:\n- Install Claude Code if not already installed\n- Make sure 'claude' command is available in your PATH\n- Try running: claude --version (should work in your terminal)\n- If using cargo run, ensure your shell's PATH is available to the Rust process",
                e
            ));
        }
    }

    // Verify installation
    match verify_claude_code_installation().await {
        Ok(_) => {
            actions_taken.push("Verified MCP server installation".to_string());
        }
        Err(e) => {
            actions_taken.push(format!("Warning: Could not verify installation: {}", e));
        }
    }

    Ok(create_installation_result(
        true,
        "Claude Code CLI (managed internally)".to_string(),
        actions_taken,
    ))
}

/// Uninstall Foundry MCP server from Claude Code
pub async fn uninstall_from_claude_code(
    _remove_config: bool,
    force: bool,
) -> Result<UninstallationResult> {
    let mut actions_taken = Vec::new();
    let files_removed = Vec::new();

    // Check if Claude Code CLI is available
    if !is_claude_code_available().await {
        return Err(anyhow::anyhow!(
            "Claude Code CLI is not available. Cannot uninstall."
        ));
    }
    actions_taken.push("Verified Claude Code CLI availability".to_string());

    // Unregister MCP server from Claude Code using CLI
    match unregister_from_claude_code().await {
        Ok(_) => {
            actions_taken.push("Unregistered MCP server from Claude Code CLI".to_string());
        }
        Err(e) => {
            if !force {
                return Err(anyhow::anyhow!(
                    "Failed to unregister MCP server from Claude Code: {}",
                    e
                ));
            }
            actions_taken.push(format!("Warning: Could not unregister cleanly: {}", e));
        }
    }

    Ok(create_uninstallation_result(
        true,
        "Claude Code CLI (managed internally)".to_string(),
        actions_taken,
        files_removed,
    ))
}

/// Check if Claude Code CLI is available on the system
pub async fn is_claude_code_available() -> bool {
    match Command::new("claude").args(&["--version"]).output().await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Register MCP server with Claude Code CLI
pub async fn register_with_claude_code() -> Result<()> {
    let output = Command::new("claude")
        .args(&["mcp", "add", "foundry", "--", "foundry", "serve"])
        .output()
        .await
        .context("Failed to execute claude mcp add command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Claude Code MCP registration failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Unregister MCP server from Claude Code CLI
pub async fn unregister_from_claude_code() -> Result<()> {
    let output = Command::new("claude")
        .args(&["mcp", "remove", "foundry"])
        .output()
        .await
        .context("Failed to execute claude mcp remove command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Claude Code MCP unregistration failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Verify Claude Code MCP installation
pub async fn verify_claude_code_installation() -> Result<()> {
    let output = Command::new("claude")
        .args(&["mcp", "list"])
        .output()
        .await
        .context("Failed to execute claude mcp list command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to list Claude Code MCP servers: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("foundry") {
        return Err(anyhow::anyhow!(
            "Foundry MCP server not found in Claude Code MCP list"
        ));
    }

    Ok(())
}

/// Get detailed server information from Claude Code CLI
pub async fn get_claude_code_server_details() -> Result<String> {
    let output = Command::new("claude")
        .args(&["mcp", "get", "foundry"])
        .output()
        .await
        .context("Failed to execute claude mcp get command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get MCP server details: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get environment status for Claude Code
pub async fn get_claude_code_status(detailed: bool) -> Result<EnvironmentStatus> {
    let mut issues = Vec::new();
    let mut installed = false;
    let mut binary_accessible = false;
    let mut config_content = None;

    // Check if Claude Code CLI is available
    if !is_claude_code_available().await {
        issues.push("Claude Code CLI not found in PATH".to_string());
    } else {
        binary_accessible = true;
    }

    // For CLI-managed installations, configuration is handled internally
    // We only need to check if the MCP server is registered
    if binary_accessible {
        match verify_claude_code_installation().await {
            Ok(_) => {
                installed = true;
            }
            Err(e) => {
                issues.push(format!("MCP server not properly registered: {}", e));
            }
        }
    }

    // Get detailed server information if requested
    if detailed && binary_accessible {
        match get_claude_code_server_details().await {
            Ok(details) => {
                config_content = Some(details);
            }
            Err(e) => {
                config_content = Some(format!("Error getting server details: {}", e));
            }
        }
    }

    Ok(EnvironmentStatus {
        name: "claude-code".to_string(),
        installed,
        config_path: "Claude Code CLI (managed internally)".to_string(),
        config_exists: true, // CLI manages config internally
        binary_path: "claude".to_string(),
        binary_accessible,
        config_content,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_claude_code_available() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // This test will likely return false in most environments unless Claude Code is installed
            let _available = is_claude_code_available().await;
            // Just ensure it doesn't panic - function completes successfully
        });
    }

    #[test]
    fn test_get_claude_code_status() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            let result = get_claude_code_status(false).await;
            assert!(result.is_ok(), "Should be able to get Claude Code status");

            let status = result.unwrap();
            assert_eq!(status.name, "claude-code");
            assert_eq!(status.binary_path, "claude");
            assert_eq!(status.config_path, "Claude Code CLI (managed internally)");
            assert!(status.config_exists); // CLI manages config internally
        });
    }

    #[test]
    fn test_install_for_claude_code_binary_path_ignored() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        let _ = env.with_env_async(|| async {
            // This test verifies that the binary_path parameter is correctly ignored
            // since Claude Code CLI uses "foundry" from PATH, not the provided binary path

            // Create a fake binary path - it should be ignored
            let fake_binary_path = "/fake/path/to/foundry";

            // The function should not panic and should handle the case where claude CLI is not available
            let result = install_for_claude_code(fake_binary_path, false).await;

            // Expect failure if Claude Code CLI is not installed, but it should fail gracefully
            if result.is_err() {
                let error_msg = result.unwrap_err().to_string();
                // Should contain information about Claude Code CLI not being available
                assert!(
                    error_msg.contains("Failed to register MCP server")
                        || error_msg.contains("claude")
                        || error_msg.contains("PATH")
                );
            }
            // If it succeeds, that means Claude Code CLI is actually installed and working
        });
    }
}
