//! Integration tests for Foundry CLI commands
//!
//! These tests verify the full command flows using isolated filesystem operations
//! following CLI testing best practices for reliable and reproducible tests.

use anyhow::Result;
use foundry_mcp::cli::args::*;
use std::env;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

/// Global mutex to ensure tests don't interfere with each other when setting environment variables
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Test environment that sets up a temporary foundry directory with proper isolation
/// Uses thread-safe environment variable manipulation following CLI testing best practices
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub original_home: Option<String>,
    _lock: MutexGuard<'static, ()>, // Held for the lifetime of the test environment
}

impl TestEnvironment {
    /// Create a new test environment with isolated foundry directory
    /// Sets HOME environment variable in a thread-safe manner
    pub fn new() -> Result<Self> {
        // Acquire global lock to prevent parallel tests from interfering
        // Use expect instead of unwrap to handle poisoned mutex gracefully
        let lock = TEST_MUTEX.lock().unwrap_or_else(|poisoned| {
            // Clear the poisoned state and acquire the lock
            poisoned.into_inner()
        });

        let temp_dir = TempDir::new()?;
        let original_home = env::var("HOME").ok();

        // Set HOME to temp directory so foundry uses temp/.foundry
        unsafe {
            env::set_var("HOME", temp_dir.path());
        }

        Ok(TestEnvironment {
            temp_dir,
            original_home,
            _lock: lock,
        })
    }

    /// Get the foundry directory path within the test environment
    pub fn foundry_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".foundry")
    }

    /// Create valid test arguments for create_project
    pub fn create_project_args(&self, project_name: &str) -> CreateProjectArgs {
        CreateProjectArgs {
            project_name: project_name.to_string(),
            vision: "This is a comprehensive test vision that meets all validation requirements. It describes a revolutionary software project that aims to solve complex problems in the development workflow. The project targets developers and teams who need better tooling for managing project contexts and specifications. Our unique value proposition includes seamless AI integration and deterministic project management that enhances rather than replaces existing workflows.".to_string(),
            tech_stack: "This project leverages Rust as the primary programming language for its performance and safety guarantees. We use clap for CLI argument parsing, serde for JSON serialization, anyhow for error handling, and chrono for timestamp management. The architecture follows modular design principles with clear separation between CLI interfaces, core business logic, and utility functions. For deployment, we target cross-platform compatibility with distribution through cargo install.".to_string(),
            summary: "A comprehensive Rust-based project management CLI tool that creates structured contexts for AI-assisted software development with atomic file operations and rich JSON responses.".to_string(),
        }
    }

    /// Create valid test arguments for create_spec
    pub fn create_spec_args(&self, project_name: &str, feature_name: &str) -> CreateSpecArgs {
        CreateSpecArgs {
            project_name: project_name.to_string(),
            feature_name: feature_name.to_string(),
            spec: "This specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations. The feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage.".to_string(),
            notes: "Implementation notes include important considerations for security, performance, and maintainability. Special attention should be paid to error handling and edge cases. Consider using established libraries where appropriate and ensure compatibility with existing system components.".to_string(),
            tasks: "Create feature scaffolding and basic structure, Implement core functionality with proper error handling, Add comprehensive test coverage for all scenarios, Update documentation and user guides, Perform integration testing with existing features, Conduct code review and optimization".to_string(),
        }
    }

    /// Create test arguments for load_project
    pub fn load_project_args(&self, project_name: &str) -> LoadProjectArgs {
        LoadProjectArgs {
            project_name: project_name.to_string(),
        }
    }

    /// Create test arguments for update_spec with single file update
    pub fn update_spec_args_single(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: &str,
        operation: &str,
    ) -> UpdateSpecArgs {
        let content = "Updated content for testing that meets the minimum length requirements and provides comprehensive information for the specification update.".to_string();

        match file_type {
            "spec" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: Some(content),
                tasks: None,
                notes: None,
                operation: operation.to_string(),
            },
            "task-list" | "tasks" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: None,
                tasks: Some(content),
                notes: None,
                operation: operation.to_string(),
            },
            "notes" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: None,
                tasks: None,
                notes: Some(content),
                operation: operation.to_string(),
            },
            _ => panic!("Invalid file_type: {}", file_type),
        }
    }

    /// Create test arguments for update_spec with multiple file updates
    pub fn update_spec_args_multi(
        &self,
        project_name: &str,
        spec_name: &str,
        operation: &str,
        spec_content: Option<&str>,
        tasks_content: Option<&str>,
        notes_content: Option<&str>,
    ) -> UpdateSpecArgs {
        UpdateSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            spec: spec_content.map(|s| s.to_string()),
            tasks: tasks_content.map(|s| s.to_string()),
            notes: notes_content.map(|s| s.to_string()),
            operation: operation.to_string(),
        }
    }

    /// Create test arguments for delete_spec
    pub fn delete_spec_args(&self, project_name: &str, spec_name: &str) -> DeleteSpecArgs {
        DeleteSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            confirm: "true".to_string(),
        }
    }

    /// Create test arguments for install command
    pub fn install_args(&self, target: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(self.mock_binary_path()),
        }
    }

    /// Create test arguments for install command with explicit binary path
    pub fn install_args_with_binary(&self, target: &str, binary_path: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(binary_path.to_string()),
        }
    }

    /// Create test arguments for uninstall command
    pub fn uninstall_args(&self, target: &str, remove_config: bool) -> UninstallArgs {
        UninstallArgs {
            target: target.to_string(),
            remove_config,
        }
    }

    /// Create test arguments for status command
    pub fn status_args(&self, target: Option<&str>, detailed: bool) -> StatusArgs {
        StatusArgs {
            target: target.map(|s| s.to_string()),
            detailed,
        }
    }

    /// Return a realistic binary path without creating actual file
    /// Uses current executable for realistic testing
    fn mock_binary_path(&self) -> String {
        // Use current foundry binary for realistic testing
        std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/foundry"))
            .to_string_lossy()
            .to_string()
    }

    /// Get cursor config path within test environment
    /// Returns the path where ~/.cursor/mcp.json would be created in the isolated test environment
    pub fn cursor_config_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".cursor").join("mcp.json")
    }

    /// Get cursor config directory within test environment
    pub fn cursor_config_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".cursor")
    }

    /// Get claude code config path within test environment
    pub fn claude_code_config_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".claude.json")
    }

    /// Create an invalid binary path for error testing
    pub fn invalid_binary_path(&self) -> String {
        "/definitely/does/not/exist/foundry".to_string()
    }

    /// Create a binary path that exists but is not executable (for platforms that check)
    pub fn non_executable_binary_path(&self) -> String {
        let binary_path = self.temp_dir.path().join("non-executable");
        std::fs::write(&binary_path, b"not executable content").unwrap();
        // Note: We still create this file since some tests might check file existence
        // but execution permission validation happens at runtime, not install time
        binary_path.to_string_lossy().to_string()
    }

    /// Create an existing cursor config with custom content for testing conflict scenarios
    pub fn create_existing_cursor_config(&self, content: &str) -> Result<()> {
        let config_dir = self.cursor_config_dir();
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(self.cursor_config_path(), content)?;
        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore original HOME environment variable
        unsafe {
            if let Some(original_home) = &self.original_home {
                env::set_var("HOME", original_home);
            } else {
                env::remove_var("HOME");
            }
        }
        // Lock is automatically released when _lock is dropped
    }
}

use foundry_mcp::cli::commands::{create_project, create_spec, load_project, load_spec};
use foundry_mcp::types::responses::{InstallationStatus, ValidationStatus};

/// Test the complete project creation workflow
#[tokio::test]
async fn test_create_project_full_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let args = env.create_project_args("test-integration-project");

    // Execute create_project command
    let response = create_project::execute(args).await?;

    // Verify response structure
    assert_eq!(response.data.project_name, "test-integration-project");
    // Note: validation_status may be Incomplete due to content validation warnings, but project creation succeeded
    assert!(response.data.files_created.len() >= 3); // vision.md, tech-stack.md, summary.md, specs/

    // Verify actual files were created in filesystem
    let foundry_dir = env.foundry_dir();
    let project_dir = foundry_dir.join("test-integration-project");

    assert!(project_dir.exists(), "Project directory should exist");
    assert!(
        project_dir.join("vision.md").exists(),
        "Vision file should exist"
    );
    assert!(
        project_dir.join("tech-stack.md").exists(),
        "Tech stack file should exist"
    );
    assert!(
        project_dir.join("summary.md").exists(),
        "Summary file should exist"
    );
    assert!(
        project_dir.join("specs").exists(),
        "Specs directory should exist"
    );
    assert!(
        project_dir.join("specs").is_dir(),
        "Specs should be a directory"
    );

    // Verify file contents are not empty
    let vision_content = std::fs::read_to_string(project_dir.join("vision.md"))?;
    assert!(
        !vision_content.trim().is_empty(),
        "Vision file should have content"
    );

    Ok(())
}

/// Test creating a spec for an existing project
#[tokio::test]
async fn test_create_spec_full_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;

    // First create a project
    let project_args = env.create_project_args("test-spec-project");
    create_project::execute(project_args).await?;

    // Then create a spec
    let spec_args = env.create_spec_args("test-spec-project", "user_authentication");
    let response = create_spec::execute(spec_args).await?;

    // Verify response
    assert_eq!(response.data.project_name, "test-spec-project");
    assert!(response.data.spec_name.contains("user_authentication"));
    assert!(response.data.spec_name.len() > 20); // Should have timestamp prefix
    // Spec creation succeeded
    assert_eq!(response.data.files_created.len(), 3); // spec.md, notes.md, task-list.md

    // Verify actual spec files were created
    let foundry_dir = env.foundry_dir();
    let specs_dir = foundry_dir.join("test-spec-project").join("specs");
    let spec_dir = specs_dir.join(&response.data.spec_name);

    assert!(spec_dir.exists(), "Spec directory should exist");
    assert!(spec_dir.join("spec.md").exists(), "Spec file should exist");
    assert!(
        spec_dir.join("notes.md").exists(),
        "Notes file should exist"
    );
    assert!(
        spec_dir.join("task-list.md").exists(),
        "Task list file should exist"
    );

    // Verify file contents
    let spec_content = std::fs::read_to_string(spec_dir.join("spec.md"))?;
    assert!(spec_content.contains("comprehensive feature implementation"));

    Ok(())
}

/// Test loading an empty project (no specs)
#[tokio::test]
async fn test_load_project_empty() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create project without specs
    let project_args = env.create_project_args("empty-project");
    create_project::execute(project_args).await?;

    // Load the project
    let load_args = env.load_project_args("empty-project");
    let response = load_project::execute(load_args).await?;

    // Verify response for empty project
    assert_eq!(response.data.project.name, "empty-project");
    assert!(response.data.project.specs_available.is_empty());
    assert!(matches!(
        response.validation_status,
        ValidationStatus::Incomplete
    ));

    // Verify next steps mention creating specs
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("no specifications"))
    );
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("create-spec"))
    );

    // Verify project content was loaded
    assert!(!response.data.project.vision.is_empty());
    assert!(!response.data.project.tech_stack.is_empty());
    assert!(!response.data.project.summary.is_empty());

    Ok(())
}

/// Test loading a project with specs
#[tokio::test]
async fn test_load_project_with_specs() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Create project and add specs
    let project_args = env.create_project_args("project-with-specs");
    create_project::execute(project_args).await?;

    let spec1_args = env.create_spec_args("project-with-specs", "feature_one");
    let spec1_response = create_spec::execute(spec1_args).await?;

    let spec2_args = env.create_spec_args("project-with-specs", "feature_two");
    let spec2_response = create_spec::execute(spec2_args).await?;

    // Load the project
    let load_args = env.load_project_args("project-with-specs");
    let response = load_project::execute(load_args).await?;

    // Verify response for project with specs
    assert_eq!(response.data.project.name, "project-with-specs");
    assert_eq!(response.data.project.specs_available.len(), 2);
    assert!(matches!(
        response.validation_status,
        ValidationStatus::Complete
    ));

    // Verify specs are listed
    assert!(
        response
            .data
            .project
            .specs_available
            .contains(&spec1_response.data.spec_name)
    );
    assert!(
        response
            .data
            .project
            .specs_available
            .contains(&spec2_response.data.spec_name)
    );

    // Verify next steps mention loading specs
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("loaded with 2 specification"))
    );
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("load-spec"))
    );

    Ok(())
}

/// Test error handling for missing project
#[tokio::test]
async fn test_error_missing_project() -> Result<()> {
    let _env = TestEnvironment::new()?; // Isolated environment

    // Try to load non-existent project
    let load_args = foundry_mcp::cli::args::LoadProjectArgs {
        project_name: "non-existent-project".to_string(),
    };

    let result = load_project::execute(load_args).await;
    assert!(result.is_err(), "Should fail for missing project");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not found"),
        "Error should mention project not found"
    );
    assert!(
        error_msg.contains("list-projects"),
        "Error should suggest list-projects command"
    );

    Ok(())
}

/// Test error handling for creating spec in missing project
#[tokio::test]
async fn test_error_spec_missing_project() -> Result<()> {
    let env = TestEnvironment::new()?;

    // Try to create spec in non-existent project
    let spec_args = env.create_spec_args("missing-project", "some_feature");
    let result = create_spec::execute(spec_args).await;

    assert!(result.is_err(), "Should fail for missing project");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not found"),
        "Error should mention project not found"
    );

    Ok(())
}

/// Test full end-to-end workflow
#[tokio::test]
async fn test_end_to_end_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "e2e-test-project";

    // Step 1: Create project
    let project_args = env.create_project_args(project_name);
    let _project_response = create_project::execute(project_args).await?;
    // Project creation succeeded (validation_status may be Incomplete due to content warnings)

    // Step 2: Load empty project
    let load_args = env.load_project_args(project_name);
    let load_response = load_project::execute(load_args).await?;
    assert!(matches!(
        load_response.validation_status,
        ValidationStatus::Incomplete
    ));
    assert!(load_response.data.project.specs_available.is_empty());

    // Step 3: Create a spec
    let spec_args = env.create_spec_args(project_name, "auth_system");
    let _spec_response = create_spec::execute(spec_args).await?;
    // Spec creation succeeded

    // Step 4: Load project with spec
    let load_args2 = env.load_project_args(project_name);
    let load_response2 = load_project::execute(load_args2).await?;
    assert!(matches!(
        load_response2.validation_status,
        ValidationStatus::Complete
    ));
    assert_eq!(load_response2.data.project.specs_available.len(), 1);
    assert!(load_response2.data.project.specs_available[0].contains("auth_system"));

    // Step 5: Verify filesystem state
    let foundry_dir = env.foundry_dir();
    let project_dir = foundry_dir.join(project_name);
    assert!(project_dir.exists());

    let specs_dir = project_dir.join("specs");
    let spec_files: Vec<_> = std::fs::read_dir(specs_dir)?.collect();
    assert_eq!(
        spec_files.len(),
        1,
        "Should have exactly one spec directory"
    );

    Ok(())
}

/// Test filesystem cleanup and isolation
#[tokio::test]
async fn test_filesystem_isolation() -> Result<()> {
    let project_name = "isolation-test";

    // Create and drop first environment
    {
        let env1 = TestEnvironment::new()?;
        let args = env1.create_project_args(project_name);
        create_project::execute(args).await?;

        // Verify project exists in this environment
        let foundry_dir = env1.foundry_dir();
        assert!(foundry_dir.join(project_name).exists());
    } // env1 drops here

    // Create second environment - should not see first project
    {
        let env2 = TestEnvironment::new()?;
        let foundry_dir = env2.foundry_dir();
        assert!(
            !foundry_dir.join(project_name).exists(),
            "Projects should be isolated between test environments"
        );

        // Can create project with same name
        let args = env2.create_project_args(project_name);
        let response = create_project::execute(args).await?;
        // Project creation succeeded (validation_status may be Incomplete due to content warnings)
        assert_eq!(response.data.project_name, project_name);
    }

    Ok(())
}

/// Test load_spec listing functionality (no spec_name provided)
#[tokio::test]
async fn test_load_spec_list_empty_project() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "spec-list-test";

    // Create project first
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Load specs (should be empty)
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: None,
    };

    let response = load_spec::execute(load_args).await?;

    // Verify response structure
    assert_eq!(response.data.project_name, project_name);
    assert!(response.data.spec_name.is_none());
    assert!(response.data.created_at.is_none());
    assert!(response.data.spec_content.is_none());
    assert!(response.data.available_specs.is_empty());
    assert!(!response.data.project_summary.is_empty());

    // Should be incomplete due to no specs
    assert!(matches!(
        response.validation_status,
        ValidationStatus::Incomplete
    ));

    // Check next steps mention creating specs
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("No specifications found"))
    );
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("create-spec"))
    );

    Ok(())
}

/// Test load_spec listing with available specs
#[tokio::test]
async fn test_load_spec_list_with_specs() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "spec-list-populated";

    // Create project
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Create two specs
    let spec1_args = env.create_spec_args(project_name, "auth_system");
    create_spec::execute(spec1_args).await?;

    let spec2_args = env.create_spec_args(project_name, "user_profile");
    create_spec::execute(spec2_args).await?;

    // Load specs list
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: None,
    };

    let response = load_spec::execute(load_args).await?;

    // Verify response structure
    assert_eq!(response.data.project_name, project_name);
    assert!(response.data.spec_name.is_none());
    assert!(response.data.created_at.is_none());
    assert!(response.data.spec_content.is_none());
    assert_eq!(response.data.available_specs.len(), 2);
    assert!(!response.data.project_summary.is_empty());

    // Should be complete with specs available
    assert!(matches!(
        response.validation_status,
        ValidationStatus::Complete
    ));

    // Verify spec info structure
    let spec_names: Vec<String> = response
        .data
        .available_specs
        .iter()
        .map(|spec| spec.feature_name.clone())
        .collect();
    assert!(spec_names.contains(&"auth_system".to_string()));
    assert!(spec_names.contains(&"user_profile".to_string()));

    // Check next steps mention loading specific specs
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("Found 2 specification"))
    );
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("load-spec"))
    );

    Ok(())
}

/// Test load_spec with specific spec name
#[tokio::test]
async fn test_load_spec_specific_spec() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "spec-load-test";

    // Create project
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Create a spec
    let spec_args = env.create_spec_args(project_name, "payment_system");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Load the specific spec
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: Some(spec_name.clone()),
    };

    let response = load_spec::execute(load_args).await?;

    // Verify response structure
    assert_eq!(response.data.project_name, project_name);
    assert_eq!(response.data.spec_name, Some(spec_name.clone()));
    assert!(response.data.created_at.is_some());
    assert!(response.data.spec_content.is_some());
    assert!(response.data.available_specs.is_empty()); // Empty when loading specific spec
    assert!(!response.data.project_summary.is_empty());

    // Should be complete
    assert!(matches!(
        response.validation_status,
        ValidationStatus::Complete
    ));

    // Verify spec content structure
    let spec_content = response.data.spec_content.unwrap();
    assert_eq!(
        spec_content.spec,
        "This specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations. The feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage."
    );
    assert_eq!(
        spec_content.notes,
        "Implementation notes include important considerations for security, performance, and maintainability. Special attention should be paid to error handling and edge cases. Consider using established libraries where appropriate and ensure compatibility with existing system components."
    );
    assert_eq!(
        spec_content.task_list,
        "Create feature scaffolding and basic structure, Implement core functionality with proper error handling, Add comprehensive test coverage for all scenarios, Update documentation and user guides, Perform integration testing with existing features, Conduct code review and optimization"
    );

    // Check workflow hints match PRD requirements
    assert!(
        response
            .workflow_hints
            .iter()
            .any(|hint| hint.contains("Update task-list.md as work progresses"))
    );
    assert!(
        response
            .workflow_hints
            .iter()
            .any(|hint| hint.contains("Add notes for design decisions"))
    );

    // Check next steps
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("loaded successfully"))
    );

    Ok(())
}

/// Test load_spec error handling for missing project
#[tokio::test]
async fn test_load_spec_missing_project() -> Result<()> {
    let _env = TestEnvironment::new()?;

    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: "non-existent-project".to_string(),
        spec_name: None,
    };

    let result = load_spec::execute(load_args).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"));
    assert!(error_msg.contains("list-projects"));

    Ok(())
}

/// Test load_spec error handling for missing spec
#[tokio::test]
async fn test_load_spec_missing_spec() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "missing-spec-test";

    // Create project but no specs
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Try to load non-existent spec
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: Some("20240101_120000_nonexistent".to_string()),
    };

    let result = load_spec::execute(load_args).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to load spec"));

    Ok(())
}

/// Test load_spec with invalid spec name format
#[tokio::test]
async fn test_load_spec_invalid_spec_name() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "invalid-spec-test";

    // Create project
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Try to load spec with invalid name format
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: Some("invalid-spec-name".to_string()),
    };

    let result = load_spec::execute(load_args).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to load spec"));

    Ok(())
}

/// Test complete workflow: create project -> create spec -> load spec
#[tokio::test]
async fn test_load_spec_end_to_end_workflow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let project_name = "e2e-spec-workflow";

    // Step 1: Create project
    let project_args = env.create_project_args(project_name);
    create_project::execute(project_args).await?;

    // Step 2: List specs (should be empty)
    let list_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: None,
    };
    let list_response = load_spec::execute(list_args).await?;
    assert!(list_response.data.available_specs.is_empty());
    assert!(matches!(
        list_response.validation_status,
        ValidationStatus::Incomplete
    ));

    // Step 3: Create spec
    let spec_args = env.create_spec_args(project_name, "notification_system");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Step 4: List specs (should have one)
    let list_args2 = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: None,
    };
    let list_response2 = load_spec::execute(list_args2).await?;
    assert_eq!(list_response2.data.available_specs.len(), 1);
    assert!(matches!(
        list_response2.validation_status,
        ValidationStatus::Complete
    ));

    // Step 5: Load specific spec
    let load_args = foundry_mcp::cli::args::LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: Some(spec_name.clone()),
    };
    let load_response = load_spec::execute(load_args).await?;
    assert_eq!(load_response.data.spec_name, Some(spec_name));
    assert!(load_response.data.spec_content.is_some());
    assert!(matches!(
        load_response.validation_status,
        ValidationStatus::Complete
    ));

    // Step 6: Verify file system state
    let foundry_dir = env.foundry_dir();
    let spec_dir = foundry_dir
        .join(project_name)
        .join("specs")
        .join(load_response.data.spec_name.unwrap());

    assert!(spec_dir.exists());
    assert!(spec_dir.join("spec.md").exists());
    assert!(spec_dir.join("notes.md").exists());
    assert!(spec_dir.join("task-list.md").exists());

    Ok(())
}

/// Test the get_foundry_help command functionality
#[test]
fn test_get_foundry_help_command() -> Result<()> {
    use foundry_mcp::cli::commands::get_foundry_help;

    // Test overview (default topic)
    let result = tokio_test::block_on(get_foundry_help::execute(GetFoundryHelpArgs {
        topic: None,
    }))?;

    assert_eq!(result.data.topic, "overview");
    assert_eq!(
        result.data.content.title,
        "Foundry - Project Management for AI Coding Assistants"
    );
    assert!(!result.data.content.examples.is_empty());
    assert!(!result.data.content.workflow_guide.is_empty());

    // Test specific topics
    let topics = vec![
        "workflows",
        "content-examples",
        "project-structure",
        "parameter-guidance",
    ];

    for topic in topics {
        let result = tokio_test::block_on(get_foundry_help::execute(GetFoundryHelpArgs {
            topic: Some(topic.to_string()),
        }))?;

        assert_eq!(result.data.topic, topic);
        assert!(!result.data.content.title.is_empty());
        assert!(!result.data.content.description.is_empty());
        assert!(!result.data.content.examples.is_empty());
        assert!(!result.data.content.workflow_guide.is_empty());
    }

    Ok(())
}

/// Test updating spec content with replace operation
#[tokio::test]
async fn test_update_spec_replace() -> Result<()> {
    use foundry_mcp::cli::commands::update_spec;

    let env = TestEnvironment::new()?;

    // Setup: Create project and spec
    let project_args = env.create_project_args("update-test-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("update-test-project", "update_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Test replace operation on spec.md
    let update_args =
        env.update_spec_args_single("update-test-project", &spec_name, "spec", "replace");
    let response = update_spec::execute(update_args).await?;

    // Verify response
    assert_eq!(response.data.project_name, "update-test-project");
    assert_eq!(response.data.spec_name, spec_name);
    assert_eq!(response.data.total_files_updated, 1);
    assert_eq!(response.data.files_updated.len(), 1);

    let file_update = &response.data.files_updated[0];
    assert_eq!(file_update.file_type, "spec");
    assert_eq!(file_update.operation, "replace");
    assert!(file_update.content_length > 0);
    assert!(file_update.success);

    // Verify file was actually updated
    let foundry_dir = env.foundry_dir();
    let spec_file = foundry_dir
        .join("update-test-project")
        .join("specs")
        .join(&spec_name)
        .join("spec.md");

    let content = std::fs::read_to_string(spec_file)?;
    assert!(content.contains("Updated content for testing"));
    assert!(!content.contains("comprehensive feature implementation")); // Original content should be gone

    // Verify next steps and workflow hints
    assert!(
        response
            .next_steps
            .iter()
            .any(|s| s.contains("Successfully updated"))
    );
    assert!(
        response
            .workflow_hints
            .iter()
            .any(|h| h.contains("Updated files: spec.md"))
    );

    Ok(())
}

/// Test updating spec content with append operation
#[tokio::test]
async fn test_update_spec_append() -> Result<()> {
    use foundry_mcp::cli::commands::update_spec;

    let env = TestEnvironment::new()?;

    // Setup: Create project and spec
    let project_args = env.create_project_args("append-test-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("append-test-project", "append_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Test append operation on notes.md
    let update_args =
        env.update_spec_args_single("append-test-project", &spec_name, "notes", "append");
    let response = update_spec::execute(update_args).await?;

    // Verify response
    assert_eq!(response.data.total_files_updated, 1);
    assert_eq!(response.data.files_updated.len(), 1);

    let file_update = &response.data.files_updated[0];
    assert_eq!(file_update.operation, "append");
    assert_eq!(file_update.file_type, "notes");
    assert!(file_update.success);

    // Verify file contains both original and appended content
    let foundry_dir = env.foundry_dir();
    let notes_file = foundry_dir
        .join("append-test-project")
        .join("specs")
        .join(&spec_name)
        .join("notes.md");

    let content = std::fs::read_to_string(notes_file)?;
    assert!(content.contains("Implementation notes")); // Original content
    assert!(content.contains("Updated content for testing")); // Appended content

    Ok(())
}

/// Test updating task list with proper formatting
#[tokio::test]
async fn test_update_spec_task_list() -> Result<()> {
    use foundry_mcp::cli::commands::update_spec;

    let env = TestEnvironment::new()?;

    // Setup
    let project_args = env.create_project_args("task-test-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("task-test-project", "task_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Update task list with new tasks
    let mut update_args =
        env.update_spec_args_single("task-test-project", &spec_name, "task-list", "append");
    update_args.tasks = Some("## Phase 3: Additional Tasks\n- [ ] New task to complete\n- [x] Completed task from previous work".to_string());

    let response = update_spec::execute(update_args).await?;

    // Verify task-list file was updated
    let foundry_dir = env.foundry_dir();
    let task_file = foundry_dir
        .join("task-test-project")
        .join("specs")
        .join(&spec_name)
        .join("task-list.md");

    let content = std::fs::read_to_string(task_file)?;
    assert!(content.contains("- [ ] New task to complete"));
    assert!(content.contains("- [x] Completed task"));

    // Verify workflow hints mention file updates
    assert!(
        response
            .workflow_hints
            .iter()
            .any(|h| h.contains("Updated files:"))
    );

    Ok(())
}

/// Test update_spec error handling for invalid inputs
#[tokio::test]
async fn test_update_spec_error_handling() -> Result<()> {
    use foundry_mcp::cli::commands::update_spec;

    let env = TestEnvironment::new()?;

    // Test nonexistent project
    let update_args =
        env.update_spec_args_single("nonexistent-project", "fake-spec", "spec", "replace");
    let result = update_spec::execute(update_args).await;
    assert!(result.is_err());

    // Setup valid project and spec for further tests
    let project_args = env.create_project_args("error-test-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("error-test-project", "error_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Test nonexistent spec
    let update_args =
        env.update_spec_args_single("error-test-project", "nonexistent-spec", "spec", "replace");
    let result = update_spec::execute(update_args).await;
    assert!(result.is_err());

    // Test invalid operation (helper function validates file types)
    let update_args = UpdateSpecArgs {
        project_name: "error-test-project".to_string(),
        spec_name: spec_name.clone(),
        spec: Some("test content".to_string()),
        tasks: None,
        notes: None,
        operation: "invalid".to_string(),
    };
    let result = update_spec::execute(update_args).await;
    assert!(result.is_err());

    // Test empty content
    let mut update_args =
        env.update_spec_args_single("error-test-project", &spec_name, "spec", "replace");
    update_args.spec = Some("".to_string());
    let result = update_spec::execute(update_args).await;
    assert!(result.is_err());

    Ok(())
}

/// Test deleting a spec completely
#[tokio::test]
async fn test_delete_spec_success() -> Result<()> {
    use foundry_mcp::cli::commands::delete_spec;

    let env = TestEnvironment::new()?;

    // Setup: Create project and spec
    let project_args = env.create_project_args("delete-test-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("delete-test-project", "delete_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Verify spec exists before deletion
    let foundry_dir = env.foundry_dir();
    let spec_dir = foundry_dir
        .join("delete-test-project")
        .join("specs")
        .join(&spec_name);
    assert!(spec_dir.exists());
    assert!(spec_dir.join("spec.md").exists());
    assert!(spec_dir.join("notes.md").exists());
    assert!(spec_dir.join("task-list.md").exists());

    // Delete the spec
    let delete_args = env.delete_spec_args("delete-test-project", &spec_name);
    let response = delete_spec::execute(delete_args).await?;

    // Verify response
    assert_eq!(response.data.project_name, "delete-test-project");
    assert_eq!(response.data.spec_name, spec_name);
    assert!(response.data.files_deleted.len() >= 3); // At least spec.md, notes.md, task-list.md

    // Verify spec directory no longer exists
    assert!(!spec_dir.exists());

    // Verify workflow hints mention permanence
    assert!(
        response
            .workflow_hints
            .iter()
            .any(|h| h.contains("cannot be undone"))
    );
    assert!(
        response
            .next_steps
            .iter()
            .any(|s| s.contains("Successfully deleted"))
    );

    Ok(())
}

/// Test delete_spec error handling and confirmation
#[tokio::test]
async fn test_delete_spec_error_handling() -> Result<()> {
    use foundry_mcp::cli::commands::delete_spec;

    let env = TestEnvironment::new()?;

    // Test nonexistent project
    let delete_args = env.delete_spec_args("nonexistent-project", "fake-spec");
    let result = delete_spec::execute(delete_args).await;
    assert!(result.is_err());

    // Setup valid project for further tests
    let project_args = env.create_project_args("delete-error-project");
    create_project::execute(project_args).await?;

    // Test nonexistent spec
    let delete_args = env.delete_spec_args("delete-error-project", "nonexistent-spec");
    let result = delete_spec::execute(delete_args).await;
    assert!(result.is_err());

    // Test lack of confirmation
    let spec_args = env.create_spec_args("delete-error-project", "confirm_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    let mut delete_args = env.delete_spec_args("delete-error-project", &spec_name);
    delete_args.confirm = "false".to_string();
    let result = delete_spec::execute(delete_args).await;
    assert!(result.is_err());

    // Verify spec still exists after failed deletion attempt
    let foundry_dir = env.foundry_dir();
    let spec_dir = foundry_dir
        .join("delete-error-project")
        .join("specs")
        .join(&spec_name);
    assert!(spec_dir.exists());

    Ok(())
}

/// Test update_spec and delete_spec integration workflow
#[tokio::test]
async fn test_spec_lifecycle_workflow() -> Result<()> {
    use foundry_mcp::cli::commands::{delete_spec, update_spec};

    let env = TestEnvironment::new()?;

    // Setup: Create project and spec
    let project_args = env.create_project_args("lifecycle-project");
    create_project::execute(project_args).await?;

    let spec_args = env.create_spec_args("lifecycle-project", "lifecycle_feature");
    let spec_response = create_spec::execute(spec_args).await?;
    let spec_name = spec_response.data.spec_name;

    // Phase 1: Update spec with replace
    let update_args =
        env.update_spec_args_single("lifecycle-project", &spec_name, "spec", "replace");
    let update_response = update_spec::execute(update_args).await?;
    assert_eq!(
        update_response.validation_status,
        ValidationStatus::Complete
    );

    // Phase 2: Append to notes
    let append_args =
        env.update_spec_args_single("lifecycle-project", &spec_name, "notes", "append");
    let append_response = update_spec::execute(append_args).await?;
    assert_eq!(append_response.data.files_updated[0].operation, "append");

    // Phase 3: Update task list
    let mut task_args =
        env.update_spec_args_single("lifecycle-project", &spec_name, "tasks", "replace");
    task_args.tasks = Some("## Implementation Progress\n- [x] Initial setup complete\n- [ ] Core implementation pending\n- [ ] Testing and documentation needed".to_string());
    let task_response = update_spec::execute(task_args).await?;
    assert!(task_response.data.files_updated[0].content_length > 50);

    // Phase 4: Load spec to verify all updates
    let load_args = LoadSpecArgs {
        project_name: "lifecycle-project".to_string(),
        spec_name: Some(spec_name.clone()),
    };
    let load_response = load_spec::execute(load_args).await?;

    let spec_content = load_response.data.spec_content.unwrap();
    assert!(spec_content.spec.contains("Updated content for testing"));
    assert!(spec_content.notes.contains("Implementation notes")); // Original + appended
    assert!(
        spec_content
            .task_list
            .contains("- [x] Initial setup complete")
    );

    // Phase 5: Delete spec to complete lifecycle
    let delete_args = env.delete_spec_args("lifecycle-project", &spec_name);
    let delete_response = delete_spec::execute(delete_args).await?;
    assert_eq!(
        delete_response.validation_status,
        ValidationStatus::Complete
    );

    // Verify spec is completely removed
    let foundry_dir = env.foundry_dir();
    let spec_dir = foundry_dir
        .join("lifecycle-project")
        .join("specs")
        .join(&spec_name);
    assert!(!spec_dir.exists());

    Ok(())
}

// =============================================================================
// CURSOR INSTALLATION INTEGRATION TESTS
// =============================================================================

/// Test cursor installation end-to-end workflow
#[tokio::test]
async fn test_install_cursor_end_to_end() -> Result<()> {
    use foundry_mcp::cli::commands::install;
    use foundry_mcp::core::installation::json_config::{McpConfig, has_server_config};
    use foundry_mcp::types::responses::InstallationStatus;

    let env = TestEnvironment::new()?;

    // Verify config doesn't exist initially
    let config_path = env.cursor_config_path();
    assert!(!config_path.exists(), "Config should not exist initially");

    // Execute install command
    let install_args = env.install_args("cursor");
    let response = install::execute(install_args).await?;

    // Verify response structure
    assert_eq!(response.data.target, "cursor");
    assert_eq!(
        response.data.installation_status,
        InstallationStatus::Success
    );
    assert!(!response.data.binary_path.is_empty());
    assert!(!response.data.config_path.is_empty());
    assert!(!response.data.actions_taken.is_empty());

    // Verify config file was created
    assert!(
        config_path.exists(),
        "Config file should exist after installation"
    );

    // Verify config content
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: McpConfig = serde_json::from_str(&config_content)?;
    assert!(
        has_server_config(&config, "foundry"),
        "Foundry server should be configured"
    );

    // Verify server configuration details
    let server_config = foundry_mcp::core::installation::get_server_config(&config, "foundry")
        .expect("Foundry server config should exist");
    assert!(
        !server_config.command.is_empty(),
        "Should use foundry command"
    );
    assert_eq!(
        server_config.args,
        vec!["serve"],
        "Should have serve argument"
    );
    assert!(
        server_config.env.is_some(),
        "Should have environment variables"
    );

    // Verify next steps contain restart guidance
    assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("Restart")),
        "Should suggest restarting AI environment"
    );

    Ok(())
}

/// Test cursor installation config verification and validation
#[tokio::test]
async fn test_install_cursor_config_verification() -> Result<()> {
    use foundry_mcp::cli::commands::install;
    use foundry_mcp::core::installation::json_config::{McpConfig, validate_config};

    let env = TestEnvironment::new()?;

    // Install cursor
    let install_args = env.install_args("cursor");
    let response = install::execute(install_args).await?;
    assert_eq!(
        response.data.installation_status,
        InstallationStatus::Success
    );

    // Read and validate config structure
    let config_path = env.cursor_config_path();
    let config_content = std::fs::read_to_string(&config_path)?;

    // Verify JSON is valid and well-formed
    let config: McpConfig =
        serde_json::from_str(&config_content).expect("Config should be valid JSON");

    // Verify config passes validation
    let validation_result = validate_config(&config);
    assert!(
        validation_result.is_ok(),
        "Config should pass validation: {:?}",
        validation_result.err()
    );

    // Verify mcpServers structure
    assert!(
        !config.mcp_servers.is_empty(),
        "Should have at least one server"
    );
    assert_eq!(
        config.mcp_servers.len(),
        1,
        "Should have exactly one server"
    );

    // Verify JSON formatting (should be pretty-printed)
    assert!(
        config_content.contains("  "),
        "Config should be pretty-printed"
    );
    assert!(
        config_content.contains("\"mcpServers\""),
        "Should contain mcpServers key"
    );
    assert!(
        config_content.contains("\"foundry\""),
        "Should contain foundry server"
    );

    Ok(())
}

/// Test cursor installation always overwrites existing configuration
#[tokio::test]
async fn test_install_cursor_always_overwrites() -> Result<()> {
    use foundry_mcp::cli::commands::install;
    use foundry_mcp::types::responses::InstallationStatus;

    let env = TestEnvironment::new()?;

    // Create existing config with different content
    let other_binary = env.mock_binary_path(); // Use realistic binary path for other server
    let existing_config = format!(
        r#"{{
  "mcpServers": {{
    "foundry": {{
      "command": "/old/path/foundry",
      "args": ["serve"],
      "env": {{
        "FOUNDRY_LOG_LEVEL": "debug"
      }}
    }},
    "other-server": {{
      "command": "{}",
      "args": ["start"]
    }}
  }}
}}"#,
        other_binary
    );
    env.create_existing_cursor_config(&existing_config)?;

    // Install should succeed and overwrite existing configuration
    let install_args = env.install_args("cursor");
    let result = install::execute(install_args).await;
    assert!(
        result.is_ok(),
        "Install should succeed and overwrite existing configuration"
    );

    // Install again (should succeed and overwrite existing configuration)
    let install_args_force = env.install_args("cursor");
    let response = install::execute(install_args_force).await?;
    assert_eq!(
        response.data.installation_status,
        InstallationStatus::Success
    );

    // Verify foundry config was updated but other-server preserved
    let config_content = std::fs::read_to_string(env.cursor_config_path())?;
    let config: foundry_mcp::core::installation::json_config::McpConfig =
        serde_json::from_str(&config_content)?;

    assert!(foundry_mcp::core::installation::has_server_config(
        &config, "foundry"
    ));
    assert!(foundry_mcp::core::installation::has_server_config(
        &config,
        "other-server"
    ));

    // Verify foundry config was updated
    let foundry_config =
        foundry_mcp::core::installation::get_server_config(&config, "foundry").unwrap();
    assert!(
        !foundry_config.command.is_empty(),
        "Should use foundry command"
    );

    Ok(())
}

/// Test cursor uninstall end-to-end workflow
#[tokio::test]
async fn test_uninstall_cursor_end_to_end() -> Result<()> {
    use foundry_mcp::cli::commands::{install, uninstall};
    use foundry_mcp::types::responses::InstallationStatus;

    let env = TestEnvironment::new()?;

    // First install cursor
    let install_args = env.install_args("cursor");
    let install_response = install::execute(install_args).await?;
    assert_eq!(
        install_response.data.installation_status,
        InstallationStatus::Success
    );

    // Verify installation
    let config_path = env.cursor_config_path();
    assert!(config_path.exists(), "Config should exist after install");

    // Uninstall cursor
    let uninstall_args = env.uninstall_args("cursor", false);
    let uninstall_response = uninstall::execute(uninstall_args).await?;

    // Verify uninstall response
    assert_eq!(uninstall_response.data.target, "cursor");
    assert!(!uninstall_response.data.actions_taken.is_empty());

    // Config file should still exist but foundry server should be removed
    assert!(config_path.exists(), "Config file should still exist");

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: foundry_mcp::core::installation::json_config::McpConfig =
        serde_json::from_str(&config_content)?;
    assert!(
        !foundry_mcp::core::installation::has_server_config(&config, "foundry"),
        "Foundry server should be removed from config"
    );

    Ok(())
}

/// Test cursor uninstall with config removal
#[tokio::test]
async fn test_uninstall_cursor_remove_config() -> Result<()> {
    use foundry_mcp::cli::commands::{install, uninstall};
    use foundry_mcp::types::responses::InstallationStatus;

    let env = TestEnvironment::new()?;

    // Install cursor (creates only foundry server)
    let install_args = env.install_args("cursor");
    let install_response = install::execute(install_args).await?;
    assert_eq!(
        install_response.data.installation_status,
        InstallationStatus::Success
    );

    // Verify config exists
    let config_path = env.cursor_config_path();
    assert!(config_path.exists());

    // Uninstall with config removal
    let uninstall_args = env.uninstall_args("cursor", true);
    let uninstall_response = uninstall::execute(uninstall_args).await?;

    // Verify config file was completely removed
    assert!(
        !config_path.exists(),
        "Config file should be removed when empty and remove_config=true"
    );

    // Verify response mentions file removal
    assert!(
        uninstall_response
            .data
            .actions_taken
            .iter()
            .any(|action| action.contains("Removed configuration file")),
        "Actions should mention removing config file"
    );

    Ok(())
}

/// Test cursor install uses PATH-based command
#[tokio::test]
async fn test_install_cursor_path_command() -> Result<()> {
    use foundry_mcp::cli::commands::install;

    let env = TestEnvironment::new()?;

    // Test cursor installation without explicit binary path
    let install_args = env.install_args("cursor");
    let response = install::execute(install_args).await?;

    assert_eq!(
        response.data.installation_status,
        foundry_mcp::types::responses::InstallationStatus::Success
    );
    assert_eq!(response.data.binary_path, "foundry (from PATH)");

    // Verify config uses PATH-based 'foundry' command
    let config_content = std::fs::read_to_string(env.cursor_config_path())?;
    let config: foundry_mcp::core::installation::json_config::McpConfig =
        serde_json::from_str(&config_content)?;
    let server_config =
        foundry_mcp::core::installation::get_server_config(&config, "foundry").unwrap();
    assert_eq!(server_config.command, "foundry");

    Ok(())
}

// =============================================================================
// ERROR SCENARIO TESTS
// =============================================================================

/// Test installation with cursor (no binary path validation needed)
#[tokio::test]
async fn test_install_cursor_path_based() -> Result<()> {
    use foundry_mcp::cli::commands::install;

    let env = TestEnvironment::new()?;

    // Test cursor installation using PATH-based command (no binary path needed)
    let install_args = env.install_args("cursor");
    let result = install::execute(install_args).await;

    assert!(
        result.is_ok(),
        "Install should succeed using PATH-based foundry command"
    );
    let response = result.unwrap();
    assert_eq!(
        response.data.installation_status,
        foundry_mcp::types::responses::InstallationStatus::Success
    );
    assert_eq!(response.data.binary_path, "foundry (from PATH)");

    // Verify config uses 'foundry' command
    let config_content = std::fs::read_to_string(env.cursor_config_path())?;
    assert!(config_content.contains("\"command\": \"foundry\""));

    Ok(())
}

/// Test cursor installation succeeds without binary path concerns
#[tokio::test]
async fn test_install_cursor_runtime_validation() -> Result<()> {
    use foundry_mcp::cli::commands::install;

    let env = TestEnvironment::new()?;

    // Cursor installation should succeed as it uses PATH-based command
    // Execution validation happens at runtime when MCP server is started
    let install_args = env.install_args("cursor");
    let result = install::execute(install_args).await;

    assert!(
        result.is_ok(),
        "Install should succeed - runtime validation happens when MCP server starts"
    );

    Ok(())
}

/// Test installation with malformed existing config
#[tokio::test]
async fn test_install_cursor_malformed_config() -> Result<()> {
    use foundry_mcp::cli::commands::install;

    let env = TestEnvironment::new()?;

    // Create malformed JSON config
    let malformed_config = r#"{
  "mcpServers": {
    "foundry": {
      "command": "/path/to/foundry"
      // Missing comma and args field
    }
  }
  // Missing closing brace
"#;
    env.create_existing_cursor_config(malformed_config)?;

    // Install should handle malformed config gracefully
    let install_args = env.install_args("cursor");
    let result = install::execute(install_args).await;

    assert!(result.is_err(), "Install should fail with malformed config");
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("Failed to read") || error_msg.contains("parse"),
        "Error should mention config parsing failure"
    );

    Ok(())
}

/// Test uninstall of non-existent installation
#[tokio::test]
async fn test_uninstall_cursor_not_installed() -> Result<()> {
    use foundry_mcp::cli::commands::uninstall;

    let env = TestEnvironment::new()?;

    // Try to uninstall when nothing is installed
    let uninstall_args = env.uninstall_args("cursor", false);
    let result = uninstall::execute(uninstall_args).await;

    assert!(result.is_err(), "Uninstall should fail when not installed");
    let error_msg = format!("{:#}", result.unwrap_err());
    assert!(
        error_msg.contains("not configured"),
        "Error should mention not configured"
    );

    Ok(())
}

/// Test uninstall when not installed (should fail)
#[tokio::test]
async fn test_uninstall_cursor_not_installed_fails() -> Result<()> {
    use foundry_mcp::cli::commands::uninstall;

    let env = TestEnvironment::new()?;

    // Try to uninstall when nothing is installed (should fail)
    let uninstall_args = env.uninstall_args("cursor", false);
    let result = uninstall::execute(uninstall_args).await;

    assert!(result.is_err(), "Uninstall should fail when not installed");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    // The error should contain either the original message or the wrapped message
    assert!(
        error_msg.contains("not configured")
            || error_msg.contains("Failed to uninstall from Cursor"),
        "Error should mention that foundry was not configured. Actual error: {}",
        error_msg
    );

    Ok(())
}

/// Test installation with empty config file
#[tokio::test]
async fn test_install_cursor_empty_config() -> Result<()> {
    use foundry_mcp::cli::commands::install;

    let env = TestEnvironment::new()?;

    // Create empty config file
    env.create_existing_cursor_config("")?;

    // Install should handle empty config gracefully
    let install_args = env.install_args("cursor");
    let result = install::execute(install_args).await;

    assert!(
        result.is_ok(),
        "Install should succeed with empty config file"
    );

    // Verify config was created properly
    let config_path = env.cursor_config_path();
    let config_content = std::fs::read_to_string(&config_path)?;
    let config: foundry_mcp::core::installation::json_config::McpConfig =
        serde_json::from_str(&config_content)?;
    assert!(foundry_mcp::core::installation::has_server_config(
        &config, "foundry"
    ));

    Ok(())
}

// =============================================================================
// STATUS INTEGRATION TESTS
// =============================================================================

/// Test status command before and after installation
#[tokio::test]
async fn test_cursor_status_before_after_install() -> Result<()> {
    use foundry_mcp::cli::commands::{install, status};

    let env = TestEnvironment::new()?;

    // Test status before installation
    let status_args = env.status_args(Some("cursor"), false);
    let status_response = status::execute(status_args).await?;

    assert_eq!(status_response.data.environments.len(), 1);
    let cursor_status = &status_response.data.environments[0];
    assert_eq!(cursor_status.name, "cursor");
    assert!(
        !cursor_status.installed,
        "Should not be installed initially"
    );
    assert!(
        !cursor_status.config_exists,
        "Config should not exist initially"
    );

    // Install cursor
    let install_args = env.install_args("cursor");
    let install_response = install::execute(install_args).await?;
    assert_eq!(
        install_response.data.installation_status,
        InstallationStatus::Success
    );

    // Test status after installation
    let status_args_after = env.status_args(Some("cursor"), false);
    let status_response_after = status::execute(status_args_after).await?;

    let cursor_status_after = &status_response_after.data.environments[0];
    assert_eq!(cursor_status_after.name, "cursor");
    assert!(
        cursor_status_after.installed,
        "Should be installed after install"
    );
    assert!(
        cursor_status_after.config_exists,
        "Config should exist after install"
    );
    assert!(
        cursor_status_after.binary_accessible,
        "Binary should be accessible"
    );

    Ok(())
}

/// Test status command with detailed flag
#[tokio::test]
async fn test_cursor_status_detailed_mode() -> Result<()> {
    use foundry_mcp::cli::commands::{install, status};

    let env = TestEnvironment::new()?;

    // Install cursor first
    let install_args = env.install_args("cursor");
    install::execute(install_args).await?;

    // Test detailed status
    let status_args = env.status_args(Some("cursor"), true);
    let status_response = status::execute(status_args).await?;

    let cursor_status = &status_response.data.environments[0];
    assert!(
        cursor_status.config_content.is_some(),
        "Detailed status should include config content"
    );

    let config_content = cursor_status.config_content.as_ref().unwrap();
    assert!(
        config_content.contains("foundry"),
        "Config content should contain foundry server"
    );
    assert!(
        config_content.contains("mcpServers"),
        "Config content should contain mcpServers"
    );

    Ok(())
}

/// Test status command for all environments
#[tokio::test]
async fn test_status_all_environments() -> Result<()> {
    use foundry_mcp::cli::commands::status;

    let env = TestEnvironment::new()?;

    // Test status for all environments (no target specified)
    let status_args = env.status_args(None, false);
    let status_response = status::execute(status_args).await?;

    // Should return status for both claude-code and cursor
    assert_eq!(
        status_response.data.environments.len(),
        2,
        "Should return status for both environments"
    );

    let env_names: Vec<&String> = status_response
        .data
        .environments
        .iter()
        .map(|env| &env.name)
        .collect();
    assert!(
        env_names.contains(&&"claude-code".to_string()),
        "Should include claude-code"
    );
    assert!(
        env_names.contains(&&"cursor".to_string()),
        "Should include cursor"
    );

    // Neither should be installed initially
    for env_status in &status_response.data.environments {
        assert!(
            !env_status.installed,
            "No environments should be installed initially"
        );
    }

    Ok(())
}

/// Test status command with issues detection
#[tokio::test]
async fn test_cursor_status_with_issues() -> Result<()> {
    use foundry_mcp::cli::commands::{install, status};

    let env = TestEnvironment::new()?;

    // Install with valid binary
    let install_args = env.install_args("cursor");
    install::execute(install_args).await?;

    // Manually corrupt the config to create an issue
    let corrupt_config = r#"{
  "mcpServers": {
    "foundry": {
      "command": "/nonexistent/binary",
      "args": ["serve"]
    }
  }
}"#;
    env.create_existing_cursor_config(corrupt_config)?;

    // Test status - should detect issues
    let status_args = env.status_args(Some("cursor"), false);
    let status_response = status::execute(status_args).await?;

    let cursor_status = &status_response.data.environments[0];
    assert!(
        cursor_status.installed,
        "Should still be considered installed"
    );
    assert!(
        !cursor_status.binary_accessible,
        "Binary should not be accessible"
    );
    assert!(!cursor_status.issues.is_empty(), "Should have issues");
    assert!(
        cursor_status
            .issues
            .iter()
            .any(|issue| issue.contains("does not exist")),
        "Should report binary does not exist issue"
    );

    Ok(())
}

// =============================================================================
// CLAP CLI ARGUMENT PARSING TESTS
// =============================================================================

/// Test CLI argument parsing for install command
#[test]
fn test_install_args_parsing() {
    use clap::Parser;

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
    use clap::Parser;

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
    use clap::Parser;

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
    use clap::Parser;

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

// =============================================================================
// FOCUSED BINARY VALIDATION TESTS
// =============================================================================

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

    let env = TestEnvironment::new().unwrap();
    let mock_path = env.mock_binary_path();

    // Mock path should be a real executable (current binary)
    assert!(
        validate_binary_path(&mock_path).is_ok(),
        "Mock binary path should be valid: {}",
        mock_path
    );

    // Invalid path should fail validation
    let invalid_path = env.invalid_binary_path();
    assert!(
        validate_binary_path(&invalid_path).is_err(),
        "Invalid binary path should fail validation: {}",
        invalid_path
    );
}
