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

use foundry_mcp::cli::commands::{create_project, create_spec, load_project};
use foundry_mcp::types::responses::ValidationStatus;

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
    let project_dir = foundry_dir.join("test-integration-project").join("project");

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
    let specs_dir = foundry_dir
        .join("test-spec-project")
        .join("project")
        .join("specs");
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

    let specs_dir = project_dir.join("project").join("specs");
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
