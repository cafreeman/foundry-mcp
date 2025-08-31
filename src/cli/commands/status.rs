//! Implementation of the status command

use crate::cli::args::StatusArgs;
use crate::core::installation;
use crate::types::responses::{FoundryResponse, StatusResponse};
use crate::utils::response::build_success_response;
use anyhow::{Context, Result};

pub async fn execute(args: StatusArgs) -> Result<FoundryResponse<StatusResponse>> {
    // Detect binary path
    let binary_path =
        installation::detect_binary_path().context("Failed to detect current binary path")?;

    // Check if binary exists and is accessible
    let binary_found = installation::check_binary_accessible(&binary_path);

    // Get status for all environments or specific target
    let environments = if let Some(target) = &args.target {
        validate_target(target)?;
        vec![
            installation::get_environment_status(target, args.detailed)
                .await
                .context(format!("Failed to get status for {}", target))?,
        ]
    } else {
        installation::get_all_environment_statuses(args.detailed)
            .await
            .context("Failed to get environment statuses")?
    };

    // Build response
    let response_data = StatusResponse {
        binary_path,
        binary_found,
        environments,
    };

    let next_steps = vec![
        "MCP server status checked successfully".to_string(),
        "Review the installation status for each environment".to_string(),
        "Use --detailed flag for more configuration information".to_string(),
    ];

    let workflow_hints = vec![
        "Green checkmarks indicate properly installed environments".to_string(),
        "Red X marks indicate missing or misconfigured installations".to_string(),
        "Use 'foundry mcp install <target>' to install for missing environments".to_string(),
        "Use 'foundry mcp uninstall <target>' to remove unwanted installations".to_string(),
    ];

    Ok(build_success_response(
        response_data,
        next_steps,
        workflow_hints,
    ))
}

/// Validate the status target
fn validate_target(target: &str) -> Result<()> {
    match target {
        "claude-code" | "cursor" => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Unsupported status target: {}. Supported targets: claude-code, cursor",
            target
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::responses::EnvironmentStatus;

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
        let args = StatusArgs {
            detailed: false,
            target: Some("invalid-target".to_string()),
        };

        // Test the validation logic without calling execute()
        if let Some(target) = &args.target {
            let validation_result = validate_target(target);
            assert!(validation_result.is_err());
            assert!(
                validation_result
                    .unwrap_err()
                    .to_string()
                    .contains("Unsupported status target")
            );
        } else {
            panic!("Expected target to be Some");
        }
    }

    #[test]
    fn test_status_args_creation() {
        let args = StatusArgs {
            detailed: true,
            target: Some("claude-code".to_string()),
        };

        assert!(args.detailed);
        assert_eq!(args.target, Some("claude-code".to_string()));
    }

    #[test]
    fn test_status_args_default_values() {
        let args = StatusArgs {
            detailed: false,
            target: None,
        };

        assert!(!args.detailed);
        assert!(args.target.is_none());
    }

    #[test]
    fn test_status_args_with_target() {
        let args = StatusArgs {
            detailed: true,
            target: Some("cursor".to_string()),
        };

        assert!(args.detailed);
        assert_eq!(args.target, Some("cursor".to_string()));
    }

    #[test]
    fn test_environment_status_creation() {
        let status = EnvironmentStatus {
            name: "test-env".to_string(),
            installed: true,
            config_path: "/path/to/config.json".to_string(),
            config_exists: true,
            binary_path: "/usr/bin/test".to_string(),
            binary_accessible: true,
            config_content: Some("test config content".to_string()),
            issues: vec![],
        };

        assert_eq!(status.name, "test-env");
        assert!(status.installed);
        assert!(status.config_exists);
        assert!(status.binary_accessible);
        assert_eq!(status.config_path, "/path/to/config.json");
        assert_eq!(status.binary_path, "/usr/bin/test");
        assert_eq!(
            status.config_content,
            Some("test config content".to_string())
        );
        assert!(status.issues.is_empty());
    }

    #[test]
    fn test_environment_status_with_issues() {
        let status = EnvironmentStatus {
            name: "problematic-env".to_string(),
            installed: false,
            config_path: "/missing/config.json".to_string(),
            config_exists: false,
            binary_path: "missing-binary".to_string(),
            binary_accessible: false,
            config_content: None,
            issues: vec![
                "Binary not found".to_string(),
                "Config file missing".to_string(),
            ],
        };

        assert_eq!(status.name, "problematic-env");
        assert!(!status.installed);
        assert!(!status.config_exists);
        assert!(!status.binary_accessible);
        assert_eq!(status.issues.len(), 2);
        assert!(status.issues.contains(&"Binary not found".to_string()));
        assert!(status.issues.contains(&"Config file missing".to_string()));
    }
}
