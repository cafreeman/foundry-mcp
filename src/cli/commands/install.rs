//! Implementation of the install command

use crate::cli::args::InstallArgs;
use crate::core::installation;
use crate::types::responses::{FoundryResponse, InstallResponse, InstallationStatus};
use crate::utils::response::{build_incomplete_response, build_success_response};
use anyhow::{Context, Result};

pub async fn execute(args: InstallArgs) -> Result<FoundryResponse<InstallResponse>> {
    // Validate installation target
    validate_target(&args.target)?;

    // Detect or use provided binary path
    let binary_path = match args.binary_path {
        Some(path) => path,
        None => {
            installation::detect_binary_path().context("Failed to detect current binary path")?
        }
    };

    // Perform installation based on target
    let result = match args.target.as_str() {
        "claude-code" => installation::install_for_claude_code(&binary_path, args.force)
            .await
            .context("Failed to install for Claude Code")?,
        "cursor" => installation::install_for_cursor(&binary_path, args.force)
            .await
            .context("Failed to install for Cursor")?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported installation target: {}. Supported targets: claude-code, cursor",
                args.target
            ));
        }
    };

    // Build response
    let response_data = InstallResponse {
        target: args.target.clone(),
        binary_path: binary_path.clone(),
        config_path: result.config_path,
        installation_status: if result.success {
            InstallationStatus::Success
        } else {
            InstallationStatus::Partial
        },
        actions_taken: result.actions_taken,
    };

    let next_steps = vec![
        format!("Foundry MCP server installed for {}", args.target),
        "Restart your AI development environment to pick up the changes".to_string(),
        "Test the installation with: foundry mcp status".to_string(),
    ];

    let workflow_hints = vec![
        "Installation complete - MCP server is now available in your environment".to_string(),
        "If you encounter issues, use 'foundry mcp status --detailed' to diagnose".to_string(),
        "Consider testing with a simple command like 'foundry mcp list-projects'".to_string(),
    ];

    if result.success {
        Ok(build_success_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    } else {
        Ok(build_incomplete_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    }
}

/// Validate the installation target
fn validate_target(target: &str) -> Result<()> {
    match target {
        "claude-code" | "cursor" => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Unsupported installation target: {}. Supported targets: claude-code, cursor",
            target
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::installation::create_installation_result;

    #[test]
    fn test_validate_target_valid() {
        let valid_targets = vec!["claude-code", "cursor"];

        for target in valid_targets {
            assert!(
                validate_target(target).is_ok(),
                "Target '{}' should be valid",
                target
            );
        }
    }

    #[test]
    fn test_validate_target_invalid() {
        let invalid_targets = vec!["", "vscode", "claude-desktop"];

        for target in invalid_targets {
            assert!(
                validate_target(target).is_err(),
                "Target '{}' should be invalid",
                target
            );
        }
    }

    #[test]
    fn test_execute_invalid_target() {
        let args = InstallArgs {
            target: "invalid-target".to_string(),
            binary_path: None,
            force: false,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(execute(args));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported installation target")
        );
    }

    #[test]
    fn test_install_args_creation() {
        let args = InstallArgs {
            target: "claude-code".to_string(),
            binary_path: Some("/custom/path/foundry".to_string()),
            force: true,
        };

        assert_eq!(args.target, "claude-code");
        assert_eq!(args.binary_path, Some("/custom/path/foundry".to_string()));
        assert!(args.force);
    }

    #[test]
    fn test_install_args_default_values() {
        let args = InstallArgs {
            target: "cursor".to_string(),
            binary_path: None,
            force: false,
        };

        assert_eq!(args.target, "cursor");
        assert!(args.binary_path.is_none());
        assert!(!args.force);
    }

    #[test]
    fn test_create_installation_result() {
        let result = create_installation_result(
            true,
            "/path/to/config.json".to_string(),
            vec!["Action 1".to_string(), "Action 2".to_string()],
        );

        assert!(result.success);
        assert_eq!(result.config_path, "/path/to/config.json");
        assert_eq!(result.actions_taken.len(), 2);
        assert_eq!(result.actions_taken[0], "Action 1");
        assert_eq!(result.actions_taken[1], "Action 2");
    }
}
