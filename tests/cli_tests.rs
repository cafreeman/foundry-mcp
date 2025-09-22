//! Integration tests for Foundry lifecycle command argument parsing and validation
//!
//! These tests verify that CLI arguments are properly parsed and validated
//! using Clap's built-in testing utilities.

use clap::Parser;

/// Test CLI argument parsing for install command
#[test]
fn test_install_args_parsing() {
    // Test structure specifically for install command
    #[derive(Parser)]
    #[command(name = "foundry")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(clap::Subcommand)]
    enum TestCommands {
        Mcp {
            #[command(subcommand)]
            command: TestMcpCommands,
        },
    }

    #[derive(clap::Subcommand)]
    enum TestMcpCommands {
        Install(foundry_mcp::cli::args::InstallArgs),
    }

    // Test valid install arguments
    let args = vec!["foundry", "mcp", "install", "cursor"];
    let parsed = TestCli::try_parse_from(args).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Install(install_args),
    } = parsed.command;
    assert_eq!(install_args.target, "cursor");
    assert!(install_args.binary_path.is_none()); // Default is None

    // Test install with binary path
    let args_with_binary = vec![
        "foundry",
        "mcp",
        "install",
        "claude-code",
        "--binary-path",
        "/custom/path/foundry",
    ];
    let parsed_with_binary = TestCli::try_parse_from(args_with_binary).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Install(install_args),
    } = parsed_with_binary.command;
    assert_eq!(install_args.target, "claude-code");
    assert_eq!(
        install_args.binary_path,
        Some("/custom/path/foundry".to_string())
    );
}

/// Test CLI argument parsing for uninstall command
#[test]
fn test_uninstall_args_parsing() {
    #[derive(Parser)]
    #[command(name = "foundry")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(clap::Subcommand)]
    enum TestCommands {
        Mcp {
            #[command(subcommand)]
            command: TestMcpCommands,
        },
    }

    #[derive(clap::Subcommand)]
    enum TestMcpCommands {
        Uninstall(foundry_mcp::cli::args::UninstallArgs),
    }

    // Test uninstall with all flags
    let args = vec!["foundry", "mcp", "uninstall", "cursor", "--remove-config"];
    let parsed = TestCli::try_parse_from(args).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Uninstall(uninstall_args),
    } = parsed.command;
    assert_eq!(uninstall_args.target, "cursor");
    assert!(uninstall_args.remove_config);

    // Test uninstall with minimal arguments
    let minimal_args = vec!["foundry", "mcp", "uninstall", "claude-code"];
    let parsed_minimal = TestCli::try_parse_from(minimal_args).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Uninstall(uninstall_args),
    } = parsed_minimal.command;
    assert_eq!(uninstall_args.target, "claude-code");
    assert!(!uninstall_args.remove_config); // Default is false
}

/// Test CLI argument parsing for status command
#[test]
fn test_status_args_parsing() {
    #[derive(Parser)]
    #[command(name = "foundry")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(clap::Subcommand)]
    enum TestCommands {
        Mcp {
            #[command(subcommand)]
            command: TestMcpCommands,
        },
    }

    #[derive(clap::Subcommand)]
    enum TestMcpCommands {
        Status(foundry_mcp::cli::args::StatusArgs),
    }

    // Test status for specific target with detailed flag
    let args = vec![
        "foundry",
        "mcp",
        "status",
        "--target",
        "cursor",
        "--detailed",
    ];
    let parsed = TestCli::try_parse_from(args).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Status(status_args),
    } = parsed.command;
    assert_eq!(status_args.target, Some("cursor".to_string()));
    assert!(status_args.detailed);

    // Test status for all environments (no target)
    let all_args = vec!["foundry", "mcp", "status"];
    let parsed_all = TestCli::try_parse_from(all_args).unwrap();

    let TestCommands::Mcp {
        command: TestMcpCommands::Status(status_args),
    } = parsed_all.command;
    assert_eq!(status_args.target, None);
    assert!(!status_args.detailed); // Default is false
}

/// Test CLI argument validation and error cases
#[test]
fn test_cli_error_handling() {
    #[derive(Parser)]
    #[command(name = "foundry")]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommands,
    }

    #[derive(clap::Subcommand)]
    enum TestCommands {
        Mcp {
            #[command(subcommand)]
            command: TestMcpCommands,
        },
    }

    #[derive(clap::Subcommand)]
    enum TestMcpCommands {
        Install(foundry_mcp::cli::args::InstallArgs),
    }

    // Test missing required target argument
    let missing_target = vec!["foundry", "mcp", "install"];
    let result = TestCli::try_parse_from(missing_target);
    assert!(result.is_err(), "Should fail when target is missing");

    // Test invalid subcommand
    let invalid_command = vec!["foundry", "mcp", "invalid-command"];
    let result = TestCli::try_parse_from(invalid_command);
    assert!(result.is_err(), "Should fail for invalid command");

    // Note: Target validation (cursor vs claude-code) happens at the application level,
    // not Clap level, so we test that in the integration tests
}

/// Test the get_foundry_help command functionality
#[test]
fn test_get_foundry_help_command() {
    use foundry_mcp::cli::commands::get_foundry_help;

    // Test overview (default topic)
    let result = tokio_test::block_on(get_foundry_help::execute(
        foundry_mcp::cli::args::GetFoundryHelpArgs { topic: None },
    ));

    assert!(result.is_ok(), "Help command should succeed");
    let response = result.unwrap();
    assert_eq!(response.data.topic, "overview");
    assert_eq!(
        response.data.content.title,
        "Foundry - Project Management for AI Coding Assistants"
    );
    assert!(!response.data.content.examples.is_empty());
    assert!(!response.data.content.workflow_guide.is_empty());

    // Test specific topics
    let topics = vec![
        "workflows",
        "content-examples",
        "project-structure",
        "parameter-guidance",
    ];

    for topic in topics {
        let result = tokio_test::block_on(get_foundry_help::execute(
            foundry_mcp::cli::args::GetFoundryHelpArgs {
                topic: Some(topic.to_string()),
            },
        ));

        assert!(
            result.is_ok(),
            "Help command should succeed for topic: {}",
            topic
        );
        let response = result.unwrap();
        assert_eq!(response.data.topic, topic);
        assert!(!response.data.content.title.is_empty());
        assert!(!response.data.content.description.is_empty());
        assert!(!response.data.content.examples.is_empty());
        assert!(!response.data.content.workflow_guide.is_empty());
    }
}

/// Test binary validation logic without creating files
#[test]
fn test_binary_validation_logic() {
    use foundry_mcp::core::installation::utils::validate_binary_path;

    // Test with current executable (should exist)
    let current_exe = std::env::current_exe().unwrap();
    let current_exe_str = current_exe.to_string_lossy();
    assert!(
        validate_binary_path(&current_exe_str).is_ok(),
        "Current executable should be valid: {}",
        current_exe_str
    );

    // Test with non-existent path
    assert!(
        validate_binary_path("/definitely/does/not/exist").is_err(),
        "Non-existent path should be invalid"
    );

    // Test with empty path
    assert!(
        validate_binary_path("").is_err(),
        "Empty path should be invalid"
    );

    // Test with whitespace-only path
    assert!(
        validate_binary_path("   ").is_err(),
        "Whitespace-only path should be invalid"
    );
}

/// Test that mock binary paths are realistic and work with validation
#[test]
fn test_mock_binary_path_validation() {
    use foundry_mcp::core::installation::utils::validate_binary_path;

    // Test with a realistic path (current executable)
    let realistic_path = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/foundry"))
        .to_string_lossy()
        .to_string();

    assert!(
        validate_binary_path(&realistic_path).is_ok(),
        "Realistic binary path should be valid: {}",
        realistic_path
    );

    // Test with a definitely invalid path
    let invalid_path = "/definitely/does/not/exist/foundry";
    assert!(
        validate_binary_path(invalid_path).is_err(),
        "Invalid binary path should fail validation: {}",
        invalid_path
    );
}
