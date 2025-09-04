//! Claude Code MCP server installation and management

use crate::core::filesystem::write_file_atomic;
use crate::core::installation::{
    InstallationResult, UninstallationResult, create_installation_result,
    create_uninstallation_result, get_claude_code_config_dir,
};
use crate::core::templates::ClientTemplate;
use crate::core::templates::claude_subagent::ClaudeSubagentTemplate;
use crate::types::responses::EnvironmentStatus;
use anyhow::{Context, Result};
use std::fs;
use tokio::process::Command;
use which::which;

/// Execute a claude command through the user's shell
/// This properly handles aliases, shell functions, and PATH resolution
async fn execute_claude_command(args: &[&str]) -> Result<std::process::Output> {
    // First, try to find the claude executable using the which crate
    // This handles PATH resolution properly across different platforms
    if let Ok(claude_path) = which("claude") {
        // Found claude in PATH, execute it directly
        let mut command = Command::new(&claude_path);
        command.args(args);

        if let Ok(output) = command.output().await {
            if output.status.success() {
                return Ok(output);
            }
        }
    }

    // If direct execution fails or claude not found in PATH,
    // try to execute through shell to handle aliases and shell functions
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let current_path = std::env::var("PATH").unwrap_or_default();

    // Build the command string
    let mut cmd_parts = vec!["claude"];
    cmd_parts.extend(args);
    let cmd_string = cmd_parts.join(" ");

    // Try interactive shell to load user configuration and aliases
    if let Ok(output) = Command::new(&shell)
        .args(["-i", "-c", &cmd_string])
        .env("PATH", &current_path)
        .output()
        .await
    {
        if output.status.success() {
            return Ok(output);
        }
    }

    // If all approaches fail, return a descriptive error
    Err(anyhow::anyhow!(
        "Failed to execute claude command. Claude Code CLI may not be installed or not accessible in PATH. \
         Please ensure Claude Code is installed and the 'claude' command is available."
    ))
}

/// Install Foundry MCP server for Claude Code
pub async fn install_for_claude_code() -> Result<InstallationResult> {
    let mut actions_taken = Vec::new();

    // Note: We skip the availability check here since the PATH may differ between
    // interactive shell and cargo run. Instead, let the actual command fail with a clear error.
    actions_taken.push("Attempting to register with Claude Code CLI".to_string());

    // Always proceed with installation (overwrite existing if present)

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

    // Install Claude subagent template
    match install_claude_subagent_template().await {
        Ok(template_message) => {
            actions_taken.push(template_message);
        }
        Err(e) => {
            // Template installation failure is non-fatal - just log a warning
            actions_taken.push(format!(
                "Warning: Failed to install Claude subagent template: {}",
                e
            ));
        }
    }

    Ok(create_installation_result(
        true,
        "Claude Code CLI (managed internally)".to_string(),
        actions_taken,
    ))
}

/// Uninstall Foundry MCP server from Claude Code
pub async fn uninstall_from_claude_code() -> Result<UninstallationResult> {
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
            return Err(anyhow::anyhow!(
                "Failed to unregister MCP server from Claude Code: {}",
                e
            ));
        }
    }

    // Remove Claude subagent template
    let mut files_removed = files_removed;
    match remove_claude_subagent_template().await {
        Ok(Some(template_message)) => {
            actions_taken.push(template_message);
            files_removed.push("Claude subagent template".to_string());
        }
        Ok(None) => {
            // Template didn't exist - that's fine
        }
        Err(e) => {
            // Template removal failure is non-fatal - just log a warning
            actions_taken.push(format!(
                "Warning: Failed to remove Claude subagent template: {}",
                e
            ));
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
    // Execute through shell to handle aliases and PATH properly
    match execute_claude_command(&["--version"]).await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Register MCP server with Claude Code CLI
pub async fn register_with_claude_code() -> Result<()> {
    let output = execute_claude_command(&["mcp", "add", "foundry", "--", "foundry", "serve"])
        .await
        .context("Failed to execute claude mcp add command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if the error is because the server already exists
        if stderr.contains("already exists") || stdout.contains("already exists") {
            // This is actually a success case - the server is already registered
            return Ok(());
        }

        return Err(anyhow::anyhow!(
            "Claude Code MCP registration failed: {}",
            stderr
        ));
    }

    Ok(())
}

/// Unregister MCP server from Claude Code CLI
pub async fn unregister_from_claude_code() -> Result<()> {
    let output = execute_claude_command(&["mcp", "remove", "foundry"])
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
    let output = execute_claude_command(&["mcp", "list"])
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
    let output = execute_claude_command(&["mcp", "get", "foundry"])
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

/// Install Claude subagent template
async fn install_claude_subagent_template() -> Result<String> {
    // Get the Claude Code config directory
    let config_dir =
        get_claude_code_config_dir().context("Failed to get Claude Code config directory")?;

    // Get the template file path
    let template_path = ClaudeSubagentTemplate::file_path(&config_dir)
        .context("Failed to resolve Claude subagent template path")?;

    // Create parent directory if needed
    if let Some(parent) = template_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create template directory: {:?}", parent))?;
    }

    // Get the embedded template content
    let content = ClaudeSubagentTemplate::content();

    // Write template content atomically
    write_file_atomic(&template_path, content).with_context(|| {
        format!(
            "Failed to write Claude subagent template: {:?}",
            template_path
        )
    })?;

    // Return success message
    Ok(format!(
        "Created Claude subagent template: {}",
        template_path.to_string_lossy()
    ))
}

/// Remove Claude subagent template
async fn remove_claude_subagent_template() -> Result<Option<String>> {
    // Get the Claude Code config directory
    let config_dir =
        get_claude_code_config_dir().context("Failed to get Claude Code config directory")?;

    // Get the template file path
    let template_path = ClaudeSubagentTemplate::file_path(&config_dir)
        .context("Failed to resolve Claude subagent template path")?;

    // Check if template file exists
    if !template_path.exists() {
        return Ok(None);
    }

    // Remove the template file
    fs::remove_file(&template_path).with_context(|| {
        format!(
            "Failed to remove Claude subagent template: {:?}",
            template_path
        )
    })?;

    // Clean up empty parent directories
    if let Some(parent) = template_path.parent() {
        // Only remove if directory is empty and not the config root
        if parent.read_dir()?.next().is_none() && parent != config_dir {
            fs::remove_dir(parent)
                .with_context(|| format!("Failed to remove empty directory: {:?}", parent))?;
        }
    }

    // Return success message
    Ok(Some(format!(
        "Removed Claude subagent template: {}",
        template_path.to_string_lossy()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_claude_code_available() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            // This test will likely return false in most environments unless Claude Code is installed
            let _available = is_claude_code_available().await;
            // Just ensure it doesn't panic - function completes successfully
        });
    }

    #[test]
    fn test_get_claude_code_status() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
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
    fn test_install_for_claude_code() {
        use crate::test_utils::TestEnvironment;
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            // The function should not panic and should handle the case where claude CLI is not available
            let result = install_for_claude_code().await;

            // Expect failure if Claude Code CLI is not installed, but it should fail gracefully
            if let Err(error) = result {
                let error_msg = error.to_string();
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
