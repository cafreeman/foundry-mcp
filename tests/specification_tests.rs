//! Integration tests for Foundry specification management tools
//!
//! These tests verify the full specification creation, loading, updating, and
//! deletion workflows using isolated filesystem operations.

use foundry_mcp::cli::args::LoadSpecArgs;
use foundry_mcp::core::ops::{create_project, create_spec, delete_spec, load_spec, update_spec};
use foundry_mcp::types::edit_commands::EditCommandTarget;
use foundry_mcp::types::responses::ValidationStatus;

// Import TestEnvironment from the main crate
use foundry_mcp::test_utils::TestEnvironment;

/// Test creating a spec for an existing project
#[test]
fn test_create_spec_full_workflow() {
    let env = TestEnvironment::new().unwrap();

    env.with_env_async(|| async {
        // First create a project
        let project_args = env.create_project_args("test-spec-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Then create a spec
        let spec_args = env.create_spec_args("test-spec-project", "user_authentication");
        let response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();

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
        let spec_content = std::fs::read_to_string(spec_dir.join("spec.md")).unwrap();
        assert!(spec_content.contains("comprehensive feature implementation"));
    });
}

/// Test error handling for creating spec in missing project
#[test]
fn test_error_spec_missing_project() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Try to create spec in non-existent project
        let spec_args = env.create_spec_args("missing-project", "some_feature");
        let result = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await;

        assert!(result.is_err(), "Should fail for missing project");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("not found"),
            "Error should mention project not found"
        );
    });
}

/// Test load_spec listing functionality (no spec_name provided)
#[test]
fn test_load_spec_list_empty_project() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let project_name = "spec-list-test";

        // Create project first
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Load specs (should be empty)
        let load_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: None,
        };

        let response = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await
        .unwrap();

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
                .any(|step| step.contains("mcp_foundry_create_spec"))
        );
    });
}

/// Test load_spec listing with available specs
#[test]
fn test_load_spec_list_with_specs() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let project_name = "spec-list-populated";

        // Create project
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Create two specs
        let spec1_args = env.create_spec_args(project_name, "auth_system");
        create_spec::run(create_spec::Input {
            project_name: spec1_args.project_name,
            feature_name: spec1_args.feature_name,
            spec: spec1_args.spec,
            notes: spec1_args.notes,
            tasks: spec1_args.tasks,
        })
        .await
        .unwrap();

        let spec2_args = env.create_spec_args(project_name, "user_profile");
        create_spec::run(create_spec::Input {
            project_name: spec2_args.project_name,
            feature_name: spec2_args.feature_name,
            spec: spec2_args.spec,
            notes: spec2_args.notes,
            tasks: spec2_args.tasks,
        })
        .await
        .unwrap();

        // Load specs list
        let load_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: None,
        };

        let response = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await
        .unwrap();

        // Verify response structure
        assert_eq!(response.data.project_name, project_name);
        assert_eq!(response.data.spec_name, None);
        assert_eq!(response.data.created_at, None);
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
                .any(|step| step.contains("mcp_foundry_load_spec"))
        );
    });
}

/// Test load_spec with specific spec name
#[test]
fn test_load_spec_specific_spec() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {

        let project_name = "spec-load-test";

        // Create project
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input { project_name: project_args.project_name, vision: project_args.vision, tech_stack: project_args.tech_stack, summary: project_args.summary }).await.unwrap();

        // Create a spec
        let spec_args = env.create_spec_args(project_name, "payment_system");
        let spec_response = create_spec::run(create_spec::Input { project_name: spec_args.project_name, feature_name: spec_args.feature_name, spec: spec_args.spec, notes: spec_args.notes, tasks: spec_args.tasks }).await.unwrap();
        let spec_name = spec_response.data.spec_name;

        // Load the specific spec
        let load_args = LoadSpecArgs {
        project_name: project_name.to_string(),
        spec_name: Some(spec_name.clone()),
        };

        let response = load_spec::run(load_spec::Input { project_name: load_args.project_name, spec_name: load_args.spec_name }).await.unwrap();

        // Verify response structure
        assert_eq!(response.data.project_name, project_name);
        assert_eq!(response.data.spec_name, Some(spec_name.clone()));
        assert!(response.data.created_at.is_some());
        assert!(response.data.spec_content.is_some());
        assert_eq!(response.data.available_specs.len(), 0); // Empty when loading specific spec
        assert!(!response.data.project_summary.is_empty());

        // Should be complete
        assert!(matches!(
        response.validation_status,
        ValidationStatus::Complete
        ));

        // Verify spec content structure
        let spec_content = response.data.spec_content.unwrap();
        assert_eq!(
        spec_content.content.spec,
        "# Feature Name\n\n## Overview\nThis specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations.\n\n## Requirements\nThe feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage."
        );
        assert_eq!(
        spec_content.content.notes,
        "# Implementation Notes\n\n## Security Considerations\nImplementation notes include important considerations for security, performance, and maintainability.\n\n## Error Handling\nSpecial attention should be paid to error handling and edge cases.\n\n## Dependencies\nConsider using established libraries where appropriate and ensure compatibility with existing system components."
        );
        assert_eq!(
        spec_content.content.tasks,
        "Create feature scaffolding and basic structure, Implement core functionality with proper error handling, Add comprehensive test coverage for all scenarios, Update documentation and user guides, Perform integration testing with existing features, Conduct code review and optimization"
        );

        // Check workflow hints match PRD requirements
        assert!(
        response
            .workflow_hints
            .iter()
            .any(|hint| hint.contains("You must update task-list.md as work progresses"))
        );
        assert!(
        response
            .workflow_hints
            .iter()
            .any(|hint| hint.contains("You can add notes for design decisions"))
        );

        // Check next steps
        assert!(
        response
            .next_steps
            .iter()
            .any(|step| step.contains("loaded successfully"))
        );

        });
}

/// Test load_spec error handling for missing project
#[test]
fn test_load_spec_missing_project() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let load_args = LoadSpecArgs {
            project_name: "non-existent-project".to_string(),
            spec_name: None,
        };

        let result = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));
        assert!(error_msg.contains("mcp_foundry_list_projects"));
    });
}

/// Test load_spec error handling for missing spec
#[test]
fn test_load_spec_missing_spec() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let project_name = "missing-spec-test";

        // Create project but no specs
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Try to load non-existent spec
        let load_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: Some("20240101_120000_nonexistent".to_string()),
        };

        let result = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No spec found matching"));
    });
}

/// Test load_spec with invalid spec name format
#[test]
fn test_load_spec_invalid_spec_name() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let project_name = "invalid-spec-test";

        // Create project
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Try to load spec with invalid name format
        let load_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: Some("invalid-spec-name".to_string()),
        };

        let result = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No spec found matching"));
    });
}

/// Test complete workflow: create project -> create spec -> load spec
#[test]
fn test_load_spec_end_to_end_workflow() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        let project_name = "e2e-spec-workflow";

        // Step 1: Create project
        let project_args = env.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Step 2: List specs (should be empty)
        let list_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: None,
        };
        let list_response = load_spec::run(load_spec::Input {
            project_name: list_args.project_name,
            spec_name: list_args.spec_name,
        })
        .await
        .unwrap();
        assert!(list_response.data.available_specs.is_empty());
        assert!(matches!(
            list_response.validation_status,
            ValidationStatus::Incomplete
        ));

        // Step 3: Create spec
        let spec_args = env.create_spec_args(project_name, "notification_system");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Step 4: List specs (should have one)
        let list_args2 = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: None,
        };
        let list_response2 = load_spec::run(load_spec::Input {
            project_name: list_args2.project_name,
            spec_name: list_args2.spec_name,
        })
        .await
        .unwrap();
        assert_eq!(list_response2.data.available_specs.len(), 1);
        assert!(matches!(
            list_response2.validation_status,
            ValidationStatus::Complete
        ));

        // Step 5: Load specific spec
        let load_args = LoadSpecArgs {
            project_name: project_name.to_string(),
            spec_name: Some(spec_name.clone()),
        };
        let load_response = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await
        .unwrap();
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
    });
}

/// Test updating spec content with replace operation
#[test]
fn test_update_spec_replace() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("update-test-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("update-test-project", "update_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test replace operation on spec.md
        let update_args = env.update_spec_args_single("update-test-project", &spec_name, "spec");
        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);

        let file_update = &response.data.file_updates[0];
        assert_eq!(file_update.target, EditCommandTarget::Spec);
        assert_eq!(file_update.applied, 1);
        assert_eq!(file_update.skipped_idempotent, 0);

        // Verify file was actually updated
        let foundry_dir = env.foundry_dir();
        let spec_file = foundry_dir
            .join("update-test-project")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");

        let content = std::fs::read_to_string(spec_file).unwrap();
        assert!(content.contains("Updated content for testing"));
        assert!(content.contains("comprehensive feature implementation")); // Original content should still be there (append operation)

        // Verify next steps and workflow hints
        assert!(
            response
                .next_steps
                .iter()
                .any(|s| s.contains("Load updated spec"))
        );
        assert!(
            response
                .workflow_hints
                .iter()
                .any(|h| h.contains("copy exact task text"))
        );
    });
}

/// Test updating spec content with append operation
#[test]
fn test_update_spec_append() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("append-test-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("append-test-project", "append_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test append operation on notes.md
        let update_args = env.update_spec_args_single("append-test-project", &spec_name, "notes");
        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);

        let file_update = &response.data.file_updates[0];
        assert_eq!(file_update.target, EditCommandTarget::Notes);
        assert_eq!(file_update.applied, 1);
        assert_eq!(file_update.skipped_idempotent, 0);

        // Verify file contains both original and appended content
        let foundry_dir = env.foundry_dir();
        let notes_file = foundry_dir
            .join("append-test-project")
            .join("specs")
            .join(&spec_name)
            .join("notes.md");

        let content = std::fs::read_to_string(notes_file).unwrap();
        assert!(content.contains("Implementation notes")); // Original content
        assert!(content.contains("Updated content for testing")); // Appended content
    });
}

/// Test updating task list with proper formatting
#[test]
fn test_update_spec_task_list() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup
        let project_args = env.create_project_args("task-test-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("task-test-project", "task_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Update task list with new tasks using edit_commands
        let update_args = env.update_spec_args_single("task-test-project", &spec_name, "tasks");

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await
        .unwrap();

        // Verify task-list file was updated
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("task-test-project")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");

        let content = std::fs::read_to_string(task_file).unwrap();
        assert!(content.contains("- [ ] Test task"));

        // Verify workflow hints mention editing guidance
        assert!(
            response
                .workflow_hints
                .iter()
                .any(|h| h.contains("copy exact task text"))
        );
    });
}

/// Test update_spec error handling for invalid inputs
#[test]
fn test_update_spec_error_handling() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Test nonexistent project
        let update_args = env.update_spec_args_single("nonexistent-project", "fake-spec", "spec");
        let result = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await;
        assert!(result.is_err());

        // Setup valid project and spec for further tests
        let project_args = env.create_project_args("error-test-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("error-test-project", "error_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let _spec_name = spec_response.data.spec_name;

        // Test nonexistent spec
        let update_args =
            env.update_spec_args_single("error-test-project", "nonexistent-spec", "spec");
        let result = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await;
        assert!(result.is_err());
    });
}

/// Test deleting a spec completely
#[test]
fn test_delete_spec_success() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("delete-test-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("delete-test-project", "delete_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
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
        let response = delete_spec::run(delete_spec::Input {
            project_name: delete_args.project_name,
            spec_name: delete_args.spec_name,
            confirm: delete_args.confirm,
        })
        .await
        .unwrap();

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
    });
}

/// Test delete_spec error handling and confirmation
#[test]
fn test_delete_spec_error_handling() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Test nonexistent project
        let delete_args = env.delete_spec_args("nonexistent-project", "fake-spec");
        let result = delete_spec::run(delete_spec::Input {
            project_name: delete_args.project_name,
            spec_name: delete_args.spec_name,
            confirm: delete_args.confirm,
        })
        .await;
        assert!(result.is_err());

        // Setup valid project for further tests
        let project_args = env.create_project_args("delete-error-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        // Test nonexistent spec
        let delete_args = env.delete_spec_args("delete-error-project", "nonexistent-spec");
        let result = delete_spec::run(delete_spec::Input {
            project_name: delete_args.project_name,
            spec_name: delete_args.spec_name,
            confirm: delete_args.confirm,
        })
        .await;
        assert!(result.is_err());

        // Test lack of confirmation
        let spec_args = env.create_spec_args("delete-error-project", "confirm_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        let mut delete_args = env.delete_spec_args("delete-error-project", &spec_name);
        delete_args.confirm = "false".to_string();
        let result = delete_spec::run(delete_spec::Input {
            project_name: delete_args.project_name,
            spec_name: delete_args.spec_name,
            confirm: delete_args.confirm,
        })
        .await;
        assert!(result.is_err());

        // Verify spec still exists after failed deletion attempt
        let foundry_dir = env.foundry_dir();
        let spec_dir = foundry_dir
            .join("delete-error-project")
            .join("specs")
            .join(&spec_name);
        assert!(spec_dir.exists());
    });
}

/// Test update_spec and delete_spec integration workflow
#[test]
fn test_spec_lifecycle_workflow() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("lifecycle-project");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("lifecycle-project", "lifecycle_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Phase 1: Update spec with replace
        let update_args = env.update_spec_args_single("lifecycle-project", &spec_name, "spec");
        let update_response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands,
        })
        .await
        .unwrap();
        assert_eq!(
            update_response.validation_status,
            ValidationStatus::Complete
        );

        // Phase 2: Append to notes
        let append_args = env.update_spec_args_single("lifecycle-project", &spec_name, "notes");
        let append_response = update_spec::run(update_spec::Input {
            project_name: append_args.project_name,
            spec_name: append_args.spec_name,
            commands_json: append_args.commands,
        })
        .await
        .unwrap();
        assert_eq!(append_response.data.applied_count, 1);
        assert_eq!(
            append_response.data.file_updates[0].target,
            EditCommandTarget::Notes
        );

        // Phase 3: Update task list
        let task_args = env.update_spec_args_single("lifecycle-project", &spec_name, "tasks");
        let task_response = update_spec::run(update_spec::Input {
            project_name: task_args.project_name,
            spec_name: task_args.spec_name,
            commands_json: task_args.commands,
        })
        .await
        .unwrap();
        assert_eq!(task_response.data.applied_count, 1);
        assert_eq!(task_response.data.file_updates.len(), 1);

        // Phase 4: Load spec to verify all updates
        let load_args = LoadSpecArgs {
            project_name: "lifecycle-project".to_string(),
            spec_name: Some(spec_name.clone()),
        };
        let load_response = load_spec::run(load_spec::Input {
            project_name: load_args.project_name,
            spec_name: load_args.spec_name,
        })
        .await
        .unwrap();

        let spec_content = load_response.data.spec_content.unwrap();
        assert!(
            spec_content
                .content
                .spec
                .contains("Updated content for testing")
        );
        assert!(spec_content.content.notes.contains("Implementation notes")); // Original + appended
        assert!(
            spec_content
                .content
                .tasks
                .contains("- [x] Initial setup complete")
        );

        // Phase 5: Delete spec to complete lifecycle
        let delete_args = env.delete_spec_args("lifecycle-project", &spec_name);
        let delete_response = delete_spec::run(delete_spec::Input {
            project_name: delete_args.project_name,
            spec_name: delete_args.spec_name,
            confirm: delete_args.confirm,
        })
        .await
        .unwrap();
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
    });
}
