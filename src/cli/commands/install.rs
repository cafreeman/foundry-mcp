//! Implementation of the install command

use crate::{
    cli::args::InstallArgs,
    core::installation,
    types::responses::{InstallResponse, InstallationStatus},
    utils::formatting::format_install_output,
};
use anyhow::Result;

pub async fn execute(args: InstallArgs) -> Result<String> {
    // Validate installation target
    validate_target(&args.target)?;

    // Handle installation and response building in a single match statement
    let (result, binary_path) = match args.target.as_str() {
        "claude-code" => {
            // Claude Code uses "foundry" from PATH, no need for binary path detection
            let result = installation::install_for_claude_code()
                .await
                .map_err(|e| enhance_installation_error("claude-code", &e))?;
            (result, "foundry (from PATH)".to_string())
        }
        "cursor" => {
            // Cursor uses "foundry" from PATH, same as Claude Code
            let result = installation::install_for_cursor()
                .await
                .map_err(|e| enhance_installation_error("cursor", &e))?;
            (result, "foundry (from PATH)".to_string())
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported installation target: {}. Supported targets: claude-code, cursor",
                args.target
            ));
        }
    };

    // Build response data for both JSON and human-readable output
    let response_data = InstallResponse {
        target: args.target.clone(),
        binary_path: binary_path.clone(),
        config_path: result.config_path.clone(),
        installation_status: if result.success {
            InstallationStatus::Success
        } else {
            InstallationStatus::Partial
        },
        actions_taken: result.actions_taken.clone(),
    };

    // Return formatted output based on --json flag
    if args.json {
        // Return JSON format - only include the data for clean CLI output
        Ok(serde_json::to_string_pretty(&response_data)?)
    } else {
        // Return human-readable format
        Ok(format_install_output(
            &args.target,
            &binary_path,
            &result.config_path,
            result.success,
            &result.actions_taken,
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

/// Enhance installation errors with specific context and actionable guidance
fn enhance_installation_error(target: &str, original_error: &anyhow::Error) -> anyhow::Error {
    let error_msg = original_error.to_string();

    // Handle specific error cases with detailed guidance
    if error_msg.contains("already configured") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 Foundry MCP is already installed for {} but will be overwritten.\n\
            💡 Check current status: foundry status --detailed",
            target,
            error_msg,
            target
        )
    } else if error_msg.contains("Binary path does not exist") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 The Foundry binary could not be found. This can happen when:\n\
            💡   • Running via 'cargo run' (use explicit path: --binary-path /path/to/foundry)\n\
            💡   • Binary was moved or deleted after compilation\n\
            💡   • Permissions prevent access to the binary\n\
            💡 Try: foundry install {} --binary-path $(which foundry)",
            target,
            error_msg,
            target
        )
    } else if error_msg.contains("command does not exist") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 The configured binary path is invalid. This usually means:\n\
            💡   • The Foundry binary was moved or deleted\n\
            💡   • The configuration points to an old development build\n\
            💡 Try: foundry install {} --binary-path $(which foundry)",
            target,
            error_msg,
            target
        )
    } else if error_msg.contains("not found in PATH") || error_msg.contains("CLI not found") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 {} is not installed or not available in PATH.\n\
            💡 Please install {} first, then retry the installation.\n\
            💡 Check installation status: foundry status --detailed",
            target,
            error_msg,
            target,
            target
        )
    } else if error_msg.contains("Permission denied") || error_msg.contains("not writable") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 Permission denied accessing configuration directory.\n\
            💡 This can happen when:\n\
            💡   • Configuration directory is owned by another user\n\
            💡   • Disk is full or read-only\n\
            💡   • System security policies prevent file creation\n\
            💡 Try running with appropriate permissions or check disk space",
            target,
            error_msg
        )
    } else if error_msg.contains("Failed to read") || error_msg.contains("Failed to write") {
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 File system error during configuration. This can indicate:\n\
            💡   • Insufficient disk space\n\
            💡   • Corrupted configuration file\n\
            💡   • File system permissions issue\n\
            💡 Try: foundry status --detailed to diagnose the problem",
            target,
            error_msg
        )
    } else {
        // Generic enhancement for unknown errors
        anyhow::anyhow!(
            "Installation failed for {}: {}\n\n\
            💡 For detailed diagnosis: foundry status --detailed\n\
            💡 For help: foundry install --help\n\
            💡 To report this issue: include the error above and output of 'foundry status --detailed'",
            target,
            error_msg
        )
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
            json: false,
        };

        // Test the validation logic without calling execute()
        let validation_result = validate_target(&args.target);
        assert!(validation_result.is_err());
        assert!(
            validation_result
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
            json: false,
        };

        assert_eq!(args.target, "claude-code");
        assert_eq!(args.binary_path, Some("/custom/path/foundry".to_string()));
    }

    #[test]
    fn test_install_args_default_values() {
        let args = InstallArgs {
            target: "cursor".to_string(),
            binary_path: None,
            json: false,
        };

        assert_eq!(args.target, "cursor");
        assert!(args.binary_path.is_none());
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

    #[test]
    fn test_execute_with_explicit_binary_path() {
        use crate::test_utils::TestEnvironment;

        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();

            let args = InstallArgs {
                target: "cursor".to_string(),
                binary_path: Some(binary_path.to_string_lossy().to_string()),
                json: false,
            };

            // This test validates the CLI argument processing
            assert_eq!(args.target, "cursor");
            assert!(args.binary_path.is_some());

            // Test target validation
            assert!(validate_target(&args.target).is_ok());
        });
    }

    #[test]
    fn test_execute_response_structure() {
        use crate::test_utils::TestEnvironment;

        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            let binary_path = env.create_mock_binary("foundry").unwrap();

            let _args = InstallArgs {
                target: "cursor".to_string(),
                binary_path: Some(binary_path.to_string_lossy().to_string()),
                json: false,
            };

            // We can't easily test the full execute function due to path dependencies,
            // but we can test the response structure building
            let mock_result = create_installation_result(
                true,
                "/test/config/path".to_string(),
                vec!["Mock action".to_string()],
            );

            assert!(mock_result.success);
            assert_eq!(mock_result.config_path, "/test/config/path");
            assert_eq!(mock_result.actions_taken.len(), 1);
        });
    }

    #[test]
    fn test_validate_target_comprehensive() {
        // Test all supported targets
        let supported_targets = vec!["claude-code", "cursor"];
        for target in supported_targets {
            assert!(
                validate_target(target).is_ok(),
                "Target '{}' should be supported",
                target
            );
        }

        // Test unsupported targets
        let unsupported_targets = vec![
            "",
            "vscode",
            "claude-desktop",
            "intellij",
            "vim",
            "invalid-target",
            "CURSOR",      // Case sensitive
            "Claude-Code", // Case sensitive
        ];

        for target in unsupported_targets {
            let result = validate_target(target);
            assert!(
                result.is_err(),
                "Target '{}' should not be supported",
                target
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Unsupported installation target"),
                "Error message should mention unsupported target for '{}'",
                target
            );
        }
    }

    #[test]
    fn test_install_args_builder_pattern() {
        // Test different combinations of arguments
        let args1 = InstallArgs {
            target: "claude-code".to_string(),
            binary_path: None,
            json: false,
        };
        assert_eq!(args1.target, "claude-code");
        assert!(args1.binary_path.is_none());

        let args2 = InstallArgs {
            target: "cursor".to_string(),
            binary_path: Some("/custom/path".to_string()),
            json: true,
        };
        assert_eq!(args2.target, "cursor");
        assert_eq!(args2.binary_path, Some("/custom/path".to_string()));
    }

    #[test]
    fn test_enhance_installation_error_already_configured() {
        let original = anyhow::anyhow!("Foundry MCP server is already configured for Cursor.");

        // Test error enhancement
        let enhanced = enhance_installation_error("cursor", &original);
        let error_msg = enhanced.to_string();
        assert!(error_msg.contains("Installation failed for cursor"));
        assert!(error_msg.contains("already installed for cursor"));
        assert!(error_msg.contains("will be overwritten"));
    }

    #[test]
    fn test_enhance_installation_error_binary_path() {
        let original = anyhow::anyhow!("Binary path does not exist: /nonexistent/path");
        let enhanced = enhance_installation_error("cursor", &original);
        let error_msg = enhanced.to_string();

        assert!(error_msg.contains("Installation failed for cursor"));
        assert!(error_msg.contains("binary could not be found"));
        assert!(error_msg.contains("cargo run"));
        assert!(error_msg.contains("--binary-path"));
        assert!(error_msg.contains("$(which foundry)"));
    }

    #[test]
    fn test_enhance_installation_error_command_not_exist() {
        let original = anyhow::anyhow!("Server 'foundry' command does not exist: /old/path");
        let enhanced = enhance_installation_error("cursor", &original);
        let error_msg = enhanced.to_string();

        assert!(error_msg.contains("Installation failed for cursor"));
        assert!(error_msg.contains("configured binary path is invalid"));
        assert!(error_msg.contains("moved or deleted"));
    }

    #[test]
    fn test_enhance_installation_error_cli_not_found() {
        let original = anyhow::anyhow!("Claude Code CLI not found in PATH");
        let enhanced = enhance_installation_error("claude-code", &original);
        let error_msg = enhanced.to_string();

        assert!(error_msg.contains("Installation failed for claude-code"));
        assert!(error_msg.contains("not installed or not available"));
        assert!(error_msg.contains("Please install claude-code first"));
    }

    #[test]
    fn test_enhance_installation_error_permission_denied() {
        let original = anyhow::anyhow!("Permission denied accessing configuration directory");
        let enhanced = enhance_installation_error("cursor", &original);
        let error_msg = enhanced.to_string();

        assert!(error_msg.contains("Installation failed for cursor"));
        assert!(error_msg.contains("Permission denied"));
        assert!(error_msg.contains("owned by another user"));
        assert!(error_msg.contains("appropriate permissions"));
    }

    #[test]
    fn test_enhance_installation_error_generic() {
        let original = anyhow::anyhow!("Some unexpected error occurred");
        let enhanced = enhance_installation_error("cursor", &original);
        let error_msg = enhanced.to_string();

        assert!(error_msg.contains("Installation failed for cursor"));
        assert!(error_msg.contains("Some unexpected error occurred"));
        assert!(error_msg.contains("detailed diagnosis"));
        assert!(error_msg.contains("foundry status --detailed"));
        assert!(error_msg.contains("report this issue"));
    }
}
