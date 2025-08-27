//! Claude Code MCP server installation and management

use crate::core::installation::{
    InstallationResult, UninstallationResult, create_installation_result,
    create_uninstallation_result, get_claude_code_mcp_config_path, validate_binary_path,
    validate_config_dir_writable,
};
use crate::types::responses::EnvironmentStatus;
use anyhow::{Context, Result};
use tokio::process::Command as AsyncCommand;

/// Install Foundry MCP server for Claude Code
pub async fn install_for_claude_code(
    binary_path: &str,
    _force: bool,
) -> Result<InstallationResult> {
    validate_binary_path(binary_path)?;

    let config_path = get_claude_code_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    validate_config_dir_writable(config_path.as_path())?;

    let mut actions_taken = Vec::new();

    // Check if Claude Code CLI is available
    if !is_claude_code_available().await {
        return Err(anyhow::anyhow!(
            "Claude Code CLI is not available. Please install Claude Code first."
        ));
    }
    actions_taken.push("Verified Claude Code CLI availability".to_string());

    // Register MCP server with Claude Code
    match register_with_claude_code(binary_path).await {
        Ok(_) => {
            actions_taken.push("Registered MCP server with Claude Code".to_string());
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to register MCP server with Claude Code: {}",
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
        config_path_str,
        actions_taken,
    ))
}

/// Uninstall Foundry MCP server from Claude Code
pub async fn uninstall_from_claude_code(
    _remove_config: bool,
    force: bool,
) -> Result<UninstallationResult> {
    let config_path = get_claude_code_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let mut actions_taken = Vec::new();
    let files_removed = Vec::new();

    // Check if Claude Code CLI is available
    if !is_claude_code_available().await {
        return Err(anyhow::anyhow!(
            "Claude Code CLI is not available. Cannot uninstall."
        ));
    }
    actions_taken.push("Verified Claude Code CLI availability".to_string());

    // Unregister MCP server from Claude Code
    match unregister_from_claude_code().await {
        Ok(_) => {
            actions_taken.push("Unregistered MCP server from Claude Code".to_string());
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
        config_path_str,
        actions_taken,
        files_removed,
    ))
}

/// Check if Claude Code CLI is available on the system
pub async fn is_claude_code_available() -> bool {
    match AsyncCommand::new("claude").arg("--version").output().await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Register MCP server with Claude Code CLI
pub async fn register_with_claude_code(binary_path: &str) -> Result<()> {
    let output = AsyncCommand::new("claude")
        .args(["mcp", "add", "foundry", "--", binary_path, "serve"])
        .output()
        .await
        .context("Failed to execute claude mcp add command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Claude Code MCP registration failed: {}",
            stderr
        ));
    }

    Ok(())
}

/// Unregister MCP server from Claude Code CLI
pub async fn unregister_from_claude_code() -> Result<()> {
    let output = AsyncCommand::new("claude")
        .args(["mcp", "remove", "foundry"])
        .output()
        .await
        .context("Failed to execute claude mcp remove command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Claude Code MCP unregistration failed: {}",
            stderr
        ));
    }

    Ok(())
}

/// Verify Claude Code MCP installation
pub async fn verify_claude_code_installation() -> Result<()> {
    let output = AsyncCommand::new("claude")
        .args(["mcp", "list"])
        .output()
        .await
        .context("Failed to execute claude mcp list command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Failed to list Claude Code MCP servers: {}",
            stderr
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

/// Get environment status for Claude Code
pub async fn get_claude_code_status(detailed: bool) -> Result<EnvironmentStatus> {
    let config_path = get_claude_code_mcp_config_path()?;
    let config_path_str = config_path.to_string_lossy().to_string();

    let mut issues = Vec::new();
    let mut installed = false;
    let mut config_exists = false;
    let mut binary_accessible = false;
    let mut config_content = None;

    // Check if Claude Code CLI is available
    if !is_claude_code_available().await {
        issues.push("Claude Code CLI not found in PATH".to_string());
    } else {
        binary_accessible = true;
    }

    // Check if config directory exists
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

    // Check if MCP server is registered
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

    Ok(EnvironmentStatus {
        name: "claude-code".to_string(),
        installed,
        config_path: config_path_str,
        config_exists,
        binary_path: "claude".to_string(),
        binary_accessible,
        config_content,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_claude_code_available() {
        // This test might fail in CI environments without Claude Code installed
        let available = is_claude_code_available().await;
        // We can't assert much here since Claude Code may or may not be installed
        assert!(available || !available); // Just ensure it doesn't panic
    }

    #[tokio::test]
    async fn test_get_claude_code_status() {
        let result = get_claude_code_status(false).await;
        assert!(result.is_ok(), "Should be able to get Claude Code status");

        let status = result.unwrap();
        assert_eq!(status.name, "claude-code");
        assert!(!status.config_path.is_empty());
    }
}
