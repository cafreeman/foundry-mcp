//! TestEnvironment for integration tests
//!
//! This module provides the same TestEnvironment implementation that's used
//! by unit tests, ensuring consistency across the test suite.

#![allow(dead_code)]

use anyhow::Result;
use assert_fs::TempDir;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

// Integration test version - imports from external crate
use foundry_mcp::core::ops::create_project;
use foundry_mcp::core::ops::create_spec;

// CLI testing imports
use foundry_mcp::cli::args::{InstallArgs, StatusArgs, UninstallArgs};
use foundry_mcp::types::responses::{InstallResponse, StatusResponse, UninstallResponse};

// Include shared base implementation
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test_support/test_environment_base.rs"
));

// CLI Testing Extensions for Integration Tests Only
impl TestEnvironment {
    // CLI Testing Helper Methods

    /// Create install args for testing
    pub fn install_args(&self, target: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: None,
            json: true, // Always use JSON mode for testing
        }
    }

    /// Execute install command and parse response
    pub async fn install_and_parse(&self, target: &str) -> Result<InstallResponse> {
        let args = self.install_args(target);
        self.install_with_args(args).await
    }

    /// Execute install command with specific args and parse response
    pub async fn install_with_args(&self, args: InstallArgs) -> Result<InstallResponse> {
        use foundry_mcp::cli::commands::install;

        let response_text = install::execute(args).await?;

        // Parse JSON response - install command returns JSON when used in tests
        let response: InstallResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse install response: {}", e))?;

        Ok(response)
    }

    /// Execute install command and return text output
    pub async fn install_text_output(&self, target: &str) -> Result<String> {
        use foundry_mcp::cli::commands::install;

        let args = InstallArgs {
            target: target.to_string(),
            binary_path: None,
            json: false, // Text mode for this helper
        };
        install::execute(args).await
    }

    /// Execute uninstall command and parse response
    pub async fn uninstall_and_parse(
        &self,
        target: &str,
        remove_config: bool,
    ) -> Result<UninstallResponse> {
        use foundry_mcp::cli::commands::uninstall;

        let args = UninstallArgs {
            target: target.to_string(),
            remove_config,
            json: true, // Always use JSON mode for testing
        };

        let response_text = uninstall::execute(args).await?;

        // Parse JSON response - uninstall command returns JSON when used in tests
        let response: UninstallResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse uninstall response: {}", e))?;

        Ok(response)
    }

    /// Execute status command and parse response
    pub async fn status_and_parse(&self, target: &str) -> Result<StatusResponse> {
        use foundry_mcp::cli::commands::status;

        let args = StatusArgs {
            target: Some(target.to_string()),
            detailed: false,
            json: true, // Always use JSON mode for testing
        };

        let response_text = status::execute(args).await?;

        // Parse JSON response - status command returns JSON when used in tests
        let response: StatusResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse status response: {}", e))?;

        Ok(response)
    }

    /// Create uninstall args for testing
    pub fn uninstall_args(&self, target: &str, remove_config: bool) -> UninstallArgs {
        UninstallArgs {
            target: target.to_string(),
            remove_config,
            json: true, // Always use JSON mode for testing
        }
    }

    /// Execute uninstall command with specific args and parse response
    pub async fn uninstall_with_args(&self, args: UninstallArgs) -> Result<UninstallResponse> {
        use foundry_mcp::cli::commands::uninstall;

        let response_text = uninstall::execute(args).await?;

        // Parse JSON response - uninstall command returns JSON when used in tests
        let response: UninstallResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse uninstall response: {}", e))?;

        Ok(response)
    }

    /// Execute uninstall command and return text output
    pub async fn uninstall_text_output(&self, target: &str, remove_config: bool) -> Result<String> {
        use foundry_mcp::cli::commands::uninstall;

        let args = UninstallArgs {
            target: target.to_string(),
            remove_config,
            json: false, // Text mode for this helper
        };

        uninstall::execute(args).await
    }

    /// Get status response with specific target and detailed flag
    pub async fn get_status_response(
        &self,
        target: Option<&str>,
        detailed: bool,
    ) -> Result<StatusResponse> {
        use foundry_mcp::cli::commands::status;

        let args = StatusArgs {
            target: target.map(|t| t.to_string()),
            detailed,
            json: true, // Always use JSON mode for testing
        };

        let response_text = status::execute(args).await?;

        // Parse JSON response - status command returns JSON when used in tests
        let response: StatusResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse status response: {}", e))?;

        Ok(response)
    }
}

impl TestEnvironment {
    /// Create a test project with minimal valid content (integration test version)
    pub async fn create_test_project(&self, project_name: &str) -> Result<()> {
        let input = create_project::Input {
            project_name: project_name.to_string(),
            vision: format!(
                "## Problem Statement\n\n{} solves testing isolation and environment management for foundry-mcp development.\n\n## Target Users\n\nDevelopers working on foundry-mcp who need isolated test environments that don't interfere with each other or with production data.\n\n## Value Proposition\n\nProvides reliable, reproducible test execution with complete environment isolation using modern Rust testing patterns and assert_fs temporary directory management.",
                project_name
            ),
            tech_stack: "## Core Technologies\n\n- **Language**: Rust for performance and safety\n- **Testing**: Integration tests with assert_fs isolation\n- **Architecture**: Component-based design for maintainability".to_string(),
            summary: format!(
                "Test project {} created for foundry-mcp integration testing with complete environment isolation, modern testing patterns using assert_fs, and reliable reproducible test execution that prevents interference between test runs.",
                project_name
            ),
        };

        create_project::run(input).await.map(|_| ())
    }

    /// Create a test spec with minimal valid content (integration test version)
    pub async fn create_test_spec(
        &self,
        project_name: &str,
        feature_name: &str,
        spec_content: &str,
    ) -> Result<()> {
        let input = create_spec::Input {
            project_name: project_name.to_string(),
            feature_name: feature_name.to_string(),
            spec: format!(
                "# {}\n\n## Overview\n\n{}\n\n## Requirements\n\n- Requirement 1: Basic functionality\n- Requirement 2: Error handling\n\n## Implementation\n\nImplementation details here.\n\n## Testing\n\n- Unit tests for core functionality\n- Integration tests for API",
                feature_name, spec_content
            ),
            tasks: format!(
                "## Setup Phase\n\n- [ ] Create base structure for {}\n- [ ] Initialize configuration\n\n## Development Phase\n\n- [ ] Implement core functionality\n- [ ] Add error handling\n\n## Testing Phase\n\n- [ ] Write unit tests\n- [ ] Run integration tests\n- [ ] Validate implementation",
                feature_name
            ),
            notes: format!(
                "## Design Decisions\n\n- **Architecture**: Component-based design for {}\n- **Testing**: Comprehensive test coverage with isolation\n\n## Implementation Context\n\nThis feature provides {} functionality with proper error handling and validation.",
                feature_name,
                spec_content.to_lowercase()
            ),
        };

        create_spec::run(input).await.map(|_| ())
    }

    // Legacy helper methods for integration test compatibility

    /// Create project args (legacy integration test helper)
    pub fn create_project_args(&self, project_name: &str) -> CreateProjectArgs {
        CreateProjectArgs {
            project_name: project_name.to_string(),
            vision: format!(
                "## Problem Statement\n\n{} solves testing isolation and environment management for foundry-mcp development.\n\n## Target Users\n\nDevelopers working on foundry-mcp who need isolated test environments that don't interfere with each other or with production data.\n\n## Value Proposition\n\nProvides reliable, reproducible test execution with complete environment isolation using modern Rust testing patterns and assert_fs temporary directory management.",
                project_name
            ),
            tech_stack: "## Core Technologies\n\n- **Language**: Rust for performance and safety\n- **Testing**: Integration tests with assert_fs isolation\n- **Architecture**: Component-based design for maintainability".to_string(),
            summary: format!(
                "Test project {} created for foundry-mcp integration testing with complete environment isolation, modern testing patterns using assert_fs, and reliable reproducible test execution that prevents interference between test runs.",
                project_name
            ),
        }
    }

    /// Create spec args (legacy integration test helper)
    pub fn create_spec_args(&self, project_name: &str, feature_name: &str) -> CreateSpecArgs {
        // Use template-like content that matches test expectations
        let spec_content = "# Feature Name\n\n## Overview\nThis specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations.\n\n## Requirements\nThe feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage.";

        let notes_content = "# Implementation Notes\n\n## Security Considerations\nImplementation notes include important considerations for security, performance, and maintainability.\n\n## Error Handling\nSpecial attention should be paid to error handling and edge cases.\n\n## Dependencies\nConsider using established libraries where appropriate and ensure compatibility with existing system components.";

        let tasks_content = "Create feature scaffolding and basic structure, Implement core functionality with proper error handling, Add comprehensive test coverage for all scenarios, Update documentation and user guides, Perform integration testing with existing features, Conduct code review and optimization";

        CreateSpecArgs {
            project_name: project_name.to_string(),
            feature_name: feature_name.to_string(),
            spec: spec_content.to_string(),
            tasks: tasks_content.to_string(),
            notes: notes_content.to_string(),
            content: CreateSpecContent {
                spec: spec_content.to_string(),
                tasks: tasks_content.to_string(),
                notes: notes_content.to_string(),
            },
        }
    }

    /// Load project args (legacy integration test helper)
    pub fn load_project_args(&self, project_name: &str) -> LoadProjectArgs {
        LoadProjectArgs {
            project_name: project_name.to_string(),
        }
    }

    /// Update spec args single (legacy integration test helper)
    pub fn update_spec_args_single(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: &str,
    ) -> UpdateSpecArgs {
        let commands = if file_type == "tasks" {
            // For tasks, use upsert_task instead of append_to_section
            vec![UpdateCommand {
                target: file_type.to_string(),
                command: "upsert_task".to_string(),
                selector: UpdateSelector {
                    r#type: "task_text".to_string(),
                    value: "- [x] Initial setup complete".to_string(),
                },
                content: Some("- [x] Initial setup complete".to_string()),
                status: None,
            }]
        } else {
            // For spec and notes, use append_to_section with correct headers
            vec![UpdateCommand {
                target: file_type.to_string(),
                command: "append_to_section".to_string(),
                selector: UpdateSelector {
                    r#type: "section".to_string(),
                    value: match file_type {
                        "spec" => "## Requirements".to_string(),
                        "notes" => "## Security Considerations".to_string(), // Match actual section from template
                        _ => "## Overview".to_string(),
                    },
                },
                content: Some("\n\nUpdated content for testing".to_string()),
                status: None,
            }]
        };

        UpdateSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        }
    }

    /// Delete spec args (legacy integration test helper)
    pub fn delete_spec_args(&self, project_name: &str, spec_name: &str) -> DeleteSpecArgs {
        DeleteSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            confirm: "true".to_string(),
        }
    }
}

// Legacy argument structs for integration test compatibility

#[derive(Debug, Clone)]
pub struct CreateProjectArgs {
    pub project_name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct CreateSpecArgs {
    pub project_name: String,
    pub feature_name: String,
    pub spec: String,
    pub tasks: String,
    pub notes: String,
    pub content: CreateSpecContent,
}

#[derive(Debug, Clone)]
pub struct CreateSpecContent {
    pub spec: String,
    pub tasks: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct LoadProjectArgs {
    pub project_name: String,
}

#[derive(Debug, Clone)]
pub struct UpdateSpecArgs {
    pub project_name: String,
    pub spec_name: String,
    pub commands_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommand {
    pub target: String,
    pub command: String,
    pub selector: UpdateSelector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSelector {
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct DeleteSpecArgs {
    pub project_name: String,
    pub spec_name: String,
    pub confirm: String,
}
