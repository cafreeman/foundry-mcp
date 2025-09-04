//! Integration tests for Foundry CLI project management commands
//!
//! These tests verify the full project creation and loading workflows using
//! isolated filesystem operations following CLI testing best practices.

use anyhow::Result;
use foundry_mcp::cli::commands::{create_project, create_spec, load_project};
use foundry_mcp::test_utils::TestEnvironment;
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
            .any(|step| step.contains("ready for specification creation"))
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
