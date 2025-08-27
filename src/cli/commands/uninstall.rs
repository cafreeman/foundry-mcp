//! Implementation of the uninstall command

use crate::cli::args::UninstallArgs;
use crate::core::installation;
use crate::types::responses::{FoundryResponse, InstallationStatus, UninstallResponse};
use crate::utils::response::{build_incomplete_response, build_success_response};
use anyhow::{Context, Result};

pub async fn execute(args: UninstallArgs) -> Result<FoundryResponse<UninstallResponse>> {
    // Validate uninstallation target
    validate_target(&args.target)?;

    // Perform uninstallation based on target
    let result = match args.target.as_str() {
        "claude-code" => installation::uninstall_from_claude_code(args.remove_config, args.force)
            .await
            .context("Failed to uninstall from Claude Code")?,
        "cursor" => installation::uninstall_from_cursor(args.remove_config, args.force)
            .await
            .context("Failed to uninstall from Cursor")?,

        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported uninstallation target: {}. Supported targets: claude-code, cursor",
                args.target
            ));
        }
    };

    // Build response
    let response_data = UninstallResponse {
        target: args.target.clone(),
        config_path: result.config_path,
        uninstallation_status: if result.success {
            InstallationStatus::Success
        } else {
            InstallationStatus::Partial
        },
        actions_taken: result.actions_taken,
        files_removed: result.files_removed,
    };

    let next_steps = vec![
        format!("Foundry MCP server uninstalled from {}", args.target),
        "Restart your AI development environment to complete the uninstallation".to_string(),
        "Check status with: foundry mcp status".to_string(),
    ];

    let workflow_hints = vec![
        "Uninstallation complete - MCP server has been removed from your environment".to_string(),
        "If you removed config files, they will need to be recreated for future installations"
            .to_string(),
        "Use 'foundry mcp install' to reinstall if needed".to_string(),
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

/// Validate the uninstallation target
fn validate_target(target: &str) -> Result<()> {
    match target {
        "claude-code" | "cursor" => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Unsupported uninstallation target: {}. Supported targets: claude-code, cursor",
            target
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::installation::create_uninstallation_result;

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
        let args = UninstallArgs {
            target: "invalid-target".to_string(),
            remove_config: false,
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
                .contains("Unsupported uninstallation target")
        );
    }

    #[test]
    fn test_uninstall_args_creation() {
        let args = UninstallArgs {
            target: "claude-code".to_string(),
            remove_config: true,
            force: true,
        };

        assert_eq!(args.target, "claude-code");
        assert!(args.remove_config);
        assert!(args.force);
    }

    #[test]
    fn test_uninstall_args_default_values() {
        let args = UninstallArgs {
            target: "cursor".to_string(),
            remove_config: false,
            force: false,
        };

        assert_eq!(args.target, "cursor");
        assert!(!args.remove_config);
        assert!(!args.force);
    }

    #[test]
    fn test_create_uninstallation_result() {
        let result = create_uninstallation_result(
            true,
            "/path/to/config.json".to_string(),
            vec!["Action 1".to_string(), "Action 2".to_string()],
            vec!["file1.json".to_string(), "file2.json".to_string()],
        );

        assert!(result.success);
        assert_eq!(result.config_path, "/path/to/config.json");
        assert_eq!(result.actions_taken.len(), 2);
        assert_eq!(result.files_removed.len(), 2);
        assert_eq!(result.actions_taken[0], "Action 1");
        assert_eq!(result.files_removed[0], "file1.json");
    }

    #[test]
    fn test_uninstall_args_with_config_removal() {
        let args = UninstallArgs {
            target: "cursor".to_string(),
            remove_config: true,
            force: false,
        };

        assert_eq!(args.target, "cursor");
        assert!(args.remove_config);
        assert!(!args.force);
    }
}
