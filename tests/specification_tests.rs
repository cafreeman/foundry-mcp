//! Integration tests for Foundry specification management tools
//!
//! These tests verify the full specification creation, loading, updating, and
//! deletion workflows using isolated filesystem operations.

mod common;

use common::{TestEnvironment, UpdateSpecArgs};
use foundry_mcp::cli::args::LoadSpecArgs;
use foundry_mcp::core::ops::{create_project, create_spec, delete_spec, load_spec, update_spec};
use foundry_mcp::types::edit_commands::EditCommandTarget;
use foundry_mcp::types::responses::ValidationStatus;

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
            spec: spec_args.content.spec,
            notes: spec_args.content.notes,
            tasks: spec_args.content.tasks,
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
            commands_json: update_args.commands_json,
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
            commands_json: update_args.commands_json,
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
        // Create specific update args for this test
        let commands = vec![serde_json::json!({
            "target": "tasks",
            "command": "upsert_task",
            "selector": {
                "type": "task_text",
                "value": "- [ ] Test task"
            },
            "content": "- [ ] Test task"
        })];

        let update_args = UpdateSpecArgs {
            project_name: "task-test-project".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
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
            commands_json: update_args.commands_json,
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
            commands_json: update_args.commands_json,
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
            commands_json: update_args.commands_json,
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
            commands_json: append_args.commands_json,
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
            commands_json: task_args.commands_json,
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

/// Test remove_list_item operation for removing tasks
#[test]
fn test_remove_list_item() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("remove-list-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("remove-list-test", "removal_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: "- [ ] Task to remove\n- [ ] Task to keep\n- [ ] Another task to keep"
                .to_string(),
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test remove_list_item operation
        let commands = vec![serde_json::json!({
            "target": "tasks",
            "command": "remove_list_item",
            "selector": {"type": "task_text", "value": "Task to remove"}
        })];

        let update_args = UpdateSpecArgs {
            project_name: "remove-list-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(
            response.data.file_updates[0].target,
            EditCommandTarget::Tasks
        );

        // Verify task was removed from file
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("remove-list-test")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        let task_content = std::fs::read_to_string(task_file).unwrap();

        assert!(!task_content.contains("Task to remove"));
        assert!(task_content.contains("Task to keep"));
        assert!(task_content.contains("Another task to keep"));
    });
}

/// Test remove_from_section operation for removing content from sections
#[test]
fn test_remove_from_section() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec with specific content
        let project_args = env.create_project_args("remove-content-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_content = r#"# Feature Specification

## Requirements
- Modern authentication system
- Ensure backward compatibility with existing configurations
- Support for multiple providers
- Comprehensive error handling

## Implementation
This will be implemented using standard patterns.
"#;

        let spec_args = env.create_spec_args("remove-content-test", "auth_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_content.to_string(),
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test remove_from_section operation - remove backward compatibility requirement
        let commands = vec![serde_json::json!({
            "target": "spec",
            "command": "remove_from_section",
            "selector": {"type": "section", "value": "## Requirements"},
            "content": "- Ensure backward compatibility with existing configurations"
        })];

        let update_args = UpdateSpecArgs {
            project_name: "remove-content-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(
            response.data.file_updates[0].target,
            EditCommandTarget::Spec
        );

        // Verify content was removed from section
        let foundry_dir = env.foundry_dir();
        let spec_file = foundry_dir
            .join("remove-content-test")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");
        let updated_content = std::fs::read_to_string(spec_file).unwrap();

        assert!(!updated_content.contains("Ensure backward compatibility"));
        assert!(updated_content.contains("Modern authentication system"));
        assert!(updated_content.contains("Support for multiple providers"));
        assert!(updated_content.contains("Comprehensive error handling"));
    });
}

/// Test remove_section operation for removing entire sections
#[test]
fn test_remove_section() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("remove-section-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let notes_content = r#"# Implementation Notes

## Design Decisions
Important architectural choices made during planning.

## Migration Path
Since this fixes bugs rather than adding features, no migration is needed. However:
- Existing servers will continue to work
- The fix prevents future issues
- No data structure changes required

## Testing Strategy
Comprehensive test coverage will be implemented.
"#;

        let spec_args = env.create_spec_args("remove-section-test", "section_removal");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: notes_content.to_string(),
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test remove_section operation - remove Migration Path section
        let commands = vec![serde_json::json!({
            "target": "notes",
            "command": "remove_section",
            "selector": {"type": "section", "value": "## Migration Path"}
        })];

        let update_args = UpdateSpecArgs {
            project_name: "remove-section-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(
            response.data.file_updates[0].target,
            EditCommandTarget::Notes
        );

        // Verify entire section was removed
        let foundry_dir = env.foundry_dir();
        let notes_file = foundry_dir
            .join("remove-section-test")
            .join("specs")
            .join(&spec_name)
            .join("notes.md");
        let updated_content = std::fs::read_to_string(notes_file).unwrap();

        assert!(!updated_content.contains("Migration Path"));
        assert!(!updated_content.contains("no migration is needed"));
        assert!(!updated_content.contains("Existing servers will continue"));
        assert!(updated_content.contains("Design Decisions"));
        assert!(updated_content.contains("Testing Strategy"));
    });
}

/// Test backward compatibility cleanup scenario from the real-world use case
#[test]
fn test_backward_compatibility_cleanup_scenario() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec that mirrors the real scenario
        let project_args = env.create_project_args("backward-compat-cleanup");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_content = r#"# Agent Tool Configuration Feature

## Requirements
### Non-Functional Requirements
- Maintain TypeScript type safety throughout
- Preserve existing functionality while fixing inconsistencies
- Ensure backward compatibility with existing server configurations
- Follow existing code patterns and conventions
"#;

        let notes_content = r#"# Implementation Notes

## Implementation Constraints

1. **Backward Compatibility:** Must not break existing saved server configurations
2. **API Compatibility:** Must work with current backend expectations
3. **Feature Flag Respect:** Agent tools only visible when flag is enabled
4. **Performance:** Tool selection should remain responsive with 100+ tools

## Migration Path

Since this fixes bugs rather than adding features, no migration is needed. However:
- Existing servers will continue to work
- The fix prevents future issues
- No data structure changes required
"#;

        let spec_args = env.create_spec_args("backward-compat-cleanup", "agent_tools");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_content.to_string(),
            notes: notes_content.to_string(),
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Execute the cleanup commands in sequence
        let commands = vec![
            // Remove backward compatibility requirement from spec
            serde_json::json!({
                "target": "spec",
                "command": "remove_from_section",
                "selector": {"type": "section", "value": "### Non-Functional Requirements"},
                "content": "- Ensure backward compatibility with existing server configurations"
            }),
            // Remove backward compatibility constraints from notes
            serde_json::json!({
                "target": "notes",
                "command": "remove_list_item",
                "selector": {"type": "task_text", "value": "1. **Backward Compatibility:** Must not break existing saved server configurations"}
            }),
            // Remove API compatibility constraint
            serde_json::json!({
                "target": "notes",
                "command": "remove_list_item",
                "selector": {"type": "task_text", "value": "2. **API Compatibility:** Must work with current backend expectations"}
            }),
            // Remove entire Migration Path section
            serde_json::json!({
                "target": "notes",
                "command": "remove_section",
                "selector": {"type": "section", "value": "## Migration Path"}
            })
        ];

        let update_args = UpdateSpecArgs {
            project_name: "backward-compat-cleanup".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify all operations succeeded
        assert_eq!(response.data.applied_count, 4);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert!(response.data.errors.as_ref().is_none_or(|e| e.is_empty()));

        // Verify spec.md changes
        let foundry_dir = env.foundry_dir();
        let spec_file = foundry_dir
            .join("backward-compat-cleanup")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");
        let spec_updated = std::fs::read_to_string(spec_file).unwrap();

        assert!(!spec_updated.contains("Ensure backward compatibility"));
        assert!(spec_updated.contains("Maintain TypeScript type safety"));
        assert!(spec_updated.contains("Follow existing code patterns"));

        // Verify notes.md changes
        let notes_file = foundry_dir
            .join("backward-compat-cleanup")
            .join("specs")
            .join(&spec_name)
            .join("notes.md");
        let notes_updated = std::fs::read_to_string(notes_file).unwrap();

        assert!(!notes_updated.contains("Backward Compatibility:"));
        assert!(!notes_updated.contains("API Compatibility:"));
        assert!(!notes_updated.contains("Migration Path"));
        assert!(!notes_updated.contains("no migration is needed"));
        assert!(notes_updated.contains("Feature Flag Respect"));
        assert!(notes_updated.contains("Performance"));
    });
}

/// Test replace_list_item operation for replacing task content
#[test]
fn test_replace_list_item() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("replace-list-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("replace-list-test", "replacement_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks:
                "- [ ] Implement basic authentication\n- [ ] Add error handling\n- [ ] Write tests"
                    .to_string(),
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test replace_list_item operation
        let commands = vec![serde_json::json!({
            "target": "tasks",
            "command": "replace_list_item",
            "selector": {"type": "task_text", "value": "Implement basic authentication"},
            "content": "Implement OAuth 2.0 authentication"
        })];

        let update_args = UpdateSpecArgs {
            project_name: "replace-list-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(
            response.data.file_updates[0].target,
            EditCommandTarget::Tasks
        );

        // Verify task was replaced in file
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("replace-list-test")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        let task_content = std::fs::read_to_string(task_file).unwrap();

        assert!(!task_content.contains("Implement basic authentication"));
        assert!(task_content.contains("Implement OAuth 2.0 authentication"));
        assert!(task_content.contains("Add error handling"));
        assert!(task_content.contains("Write tests"));
        // Verify task format is preserved
        assert!(task_content.contains("- [ ] Implement OAuth 2.0 authentication"));
    });
}

/// Test replace_in_section operation for replacing text within sections
#[test]
fn test_replace_in_section() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec with specific content
        let project_args = env.create_project_args("replace-text-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_content = r#"# Feature Specification

## Requirements
- Use PostgreSQL database for persistence
- Implement RESTful API endpoints
- Ensure proper input validation
- Add comprehensive logging

## Implementation
This will be implemented using standard patterns with PostgreSQL.
"#;

        let spec_args = env.create_spec_args("replace-text-test", "database_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_content.to_string(),
            notes: spec_args.notes,
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test replace_in_section operation - replace PostgreSQL with MongoDB
        let commands = vec![serde_json::json!({
            "target": "spec",
            "command": "replace_in_section",
            "selector": {"type": "text_in_section", "section": "## Requirements", "text": "PostgreSQL database"},
            "content": "MongoDB database"
        })];

        let update_args = UpdateSpecArgs {
            project_name: "replace-text-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(response.data.file_updates[0].target, EditCommandTarget::Spec);

        // Verify text was replaced in section
        let foundry_dir = env.foundry_dir();
        let spec_file = foundry_dir
            .join("replace-text-test")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");
        let updated_content = std::fs::read_to_string(spec_file).unwrap();

        assert!(!updated_content.contains("PostgreSQL database"));
        assert!(updated_content.contains("MongoDB database"));
        assert!(updated_content.contains("RESTful API endpoints"));
        assert!(updated_content.contains("input validation"));
        // Verify the Implementation section still references PostgreSQL since we only replaced in Requirements
        assert!(updated_content.contains("using standard patterns with PostgreSQL"));
    });
}

/// Test replace_section_content operation for replacing entire section content
#[test]
fn test_replace_section_content() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("replace-section-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let notes_content = r#"# Implementation Notes

## Architecture Decisions
We will use a microservices architecture for scalability.

## Technology Stack
- Backend: Node.js with Express
- Database: PostgreSQL
- Frontend: React with TypeScript

## Testing Strategy
Basic unit tests will be sufficient for this project.
"#;

        let spec_args = env.create_spec_args("replace-section-test", "section_replacement");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: notes_content.to_string(),
            tasks: spec_args.tasks,
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Test replace_section_content operation - replace entire Testing Strategy section
        let new_testing_content = r#"Comprehensive testing approach will be implemented:
- Unit tests for all business logic
- Integration tests for API endpoints
- End-to-end tests for critical user flows
- Performance testing for high-load scenarios
- Security testing for authentication and authorization"#;

        let commands = vec![serde_json::json!({
            "target": "notes",
            "command": "replace_section_content",
            "selector": {"type": "section", "value": "## Testing Strategy"},
            "content": new_testing_content
        })];

        let update_args = UpdateSpecArgs {
            project_name: "replace-section-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify response
        assert_eq!(response.data.applied_count, 1);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert_eq!(response.data.file_updates.len(), 1);
        assert_eq!(
            response.data.file_updates[0].target,
            EditCommandTarget::Notes
        );

        // Verify entire section content was replaced
        let foundry_dir = env.foundry_dir();
        let notes_file = foundry_dir
            .join("replace-section-test")
            .join("specs")
            .join(&spec_name)
            .join("notes.md");
        let updated_content = std::fs::read_to_string(notes_file).unwrap();

        // Old content should be gone
        assert!(!updated_content.contains("Basic unit tests will be sufficient"));

        // New content should be present
        assert!(updated_content.contains("Comprehensive testing approach"));
        assert!(updated_content.contains("Unit tests for all business logic"));
        assert!(updated_content.contains("Integration tests for API endpoints"));
        assert!(updated_content.contains("Performance testing"));
        assert!(updated_content.contains("Security testing"));

        // Other sections should remain unchanged
        assert!(updated_content.contains("Architecture Decisions"));
        assert!(updated_content.contains("microservices architecture"));
        assert!(updated_content.contains("Technology Stack"));
        assert!(updated_content.contains("Node.js with Express"));
    });
}

/// Test replacement operations with idempotent behavior
#[test]
fn test_replacement_idempotent_behavior() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec
        let project_args = env.create_project_args("idempotent-test");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_args = env.create_spec_args("idempotent-test", "idempotent_feature");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_args.spec,
            notes: spec_args.notes,
            tasks: "- [ ] Initial task\n- [ ] Second task".to_string(),
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // First replacement
        let commands = vec![serde_json::json!({
            "target": "tasks",
            "command": "replace_list_item",
            "selector": {"type": "task_text", "value": "Initial task"},
            "content": "Updated task"
        })];

        let update_args = UpdateSpecArgs {
            project_name: "idempotent-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response1 = update_spec::run(update_spec::Input {
            project_name: update_args.project_name.clone(),
            spec_name: update_args.spec_name.clone(),
            commands_json: update_args.commands_json.clone(),
        })
        .await
        .unwrap();

        // Debug output removed

        // Should apply the change
        assert_eq!(response1.data.applied_count, 1);
        assert_eq!(response1.data.skipped_idempotent_count, 0);

        // Second replacement - idempotent (replace "Updated task" with "Updated task")
        let idempotent_commands = vec![serde_json::json!({
            "target": "tasks",
            "command": "replace_list_item",
            "selector": {"type": "task_text", "value": "Updated task"},
            "content": "Updated task"
        })];

        let idempotent_args = UpdateSpecArgs {
            project_name: "idempotent-test".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&idempotent_commands).unwrap(),
        };

        let response2 = update_spec::run(update_spec::Input {
            project_name: idempotent_args.project_name,
            spec_name: idempotent_args.spec_name,
            commands_json: idempotent_args.commands_json,
        })
        .await
        .unwrap();

        // Should skip because already matches (idempotent)
        assert_eq!(response2.data.applied_count, 0);
        assert_eq!(response2.data.skipped_idempotent_count, 1);

        // Verify file contains updated content only once
        let foundry_dir = env.foundry_dir();
        let task_file = foundry_dir
            .join("idempotent-test")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        let task_content = std::fs::read_to_string(task_file).unwrap();

        assert!(!task_content.contains("Initial task"));
        assert!(task_content.contains("Updated task"));
        assert!(task_content.contains("Second task"));
        // Ensure no duplication
        assert_eq!(task_content.matches("Updated task").count(), 1);
    });
}

/// Test real-world scenario: Upgrading technology stack across specification
#[test]
fn test_technology_stack_upgrade_scenario() {
    let env = TestEnvironment::new().unwrap();
    env.with_env_async(|| async {
        // Setup: Create project and spec that mirrors a real technology upgrade scenario
        let project_args = env.create_project_args("tech-upgrade");
        create_project::run(create_project::Input {
            project_name: project_args.project_name,
            vision: project_args.vision,
            tech_stack: project_args.tech_stack,
            summary: project_args.summary,
        })
        .await
        .unwrap();

        let spec_content = r#"# Database Migration Feature

## Requirements
### Functional Requirements
- Migrate from MySQL 5.7 to latest version
- Ensure data integrity during migration
- Support rollback mechanisms
- Minimize downtime during migration

### Non-Functional Requirements
- Migration should complete within 2-hour maintenance window
- Zero data loss during migration process
"#;

        let notes_content = r#"# Implementation Notes

## Technology Decisions
The current MySQL 5.7 database will be upgraded to MySQL 8.0 for better performance and security features.

## Migration Strategy
We will use MySQL 5.7 native tools for the migration process:
1. Create backup using mysqldump from MySQL 5.7
2. Set up new MySQL 5.7 instance
3. Restore data and validate integrity
4. Switch application to new MySQL 5.7 database

## Performance Considerations
MySQL 5.7 has known limitations that will be addressed in this migration.
"#;

        let tasks_content = r#"- [ ] Set up MySQL 5.7 development environment
- [ ] Create migration scripts for MySQL 5.7 compatibility
- [ ] Test data export from current MySQL 5.7 instance
- [ ] Validate MySQL 5.7 performance benchmarks
- [ ] Schedule maintenance window for MySQL 5.7 upgrade"#;

        let spec_args = env.create_spec_args("tech-upgrade", "database_migration");
        let spec_response = create_spec::run(create_spec::Input {
            project_name: spec_args.project_name,
            feature_name: spec_args.feature_name,
            spec: spec_content.to_string(),
            notes: notes_content.to_string(),
            tasks: tasks_content.to_string(),
        })
        .await
        .unwrap();
        let spec_name = spec_response.data.spec_name;

        // Execute comprehensive technology upgrade using replacement operations
        let commands = vec![
            // Update the target version in spec requirements
            serde_json::json!({
                "target": "spec",
                "command": "replace_in_section",
                "selector": {"type": "text_in_section", "section": "### Functional Requirements", "text": "MySQL 5.7 to latest version"},
                "content": "MySQL 5.7 to MySQL 8.0"
            }),
            // Replace the entire technology decision rationale
            serde_json::json!({
                "target": "notes",
                "command": "replace_section_content",
                "selector": {"type": "section", "value": "## Technology Decisions"},
                "content": "The database will be upgraded from MySQL 5.7 to MySQL 8.0 to leverage:\n- Improved JSON support and performance\n- Enhanced security features including caching_sha2_password\n- Better query optimization and execution plans\n- Support for common table expressions (CTEs)\n- Improved replication and high availability features"
            }),
            // Update migration strategy to reflect MySQL 8.0
            serde_json::json!({
                "target": "notes",
                "command": "replace_in_section",
                "selector": {"type": "text_in_section", "section": "## Migration Strategy", "text": "MySQL 5.7 native tools"},
                "content": "MySQL 8.0 utilities and best practices"
            }),
            serde_json::json!({
                "target": "notes",
                "command": "replace_in_section",
                "selector": {"type": "text_in_section", "section": "## Migration Strategy", "text": "Set up new MySQL 5.7 instance"},
                "content": "Set up new MySQL 8.0 instance with proper configuration"
            }),
            serde_json::json!({
                "target": "notes",
                "command": "replace_in_section",
                "selector": {"type": "text_in_section", "section": "## Migration Strategy", "text": "Switch application to new MySQL 5.7 database"},
                "content": "Update application connection strings and test MySQL 8.0 compatibility"
            }),
            // Replace performance section entirely
            serde_json::json!({
                "target": "notes",
                "command": "replace_section_content",
                "selector": {"type": "section", "value": "## Performance Considerations"},
                "content": "MySQL 8.0 provides significant performance improvements:\n- Up to 2x faster read/write performance compared to 5.7\n- Improved query optimizer with cost-based optimization\n- Better handling of JSON data types and indexing\n- Enhanced memory management for large datasets\n- Reduced replication lag through improved binlog handling"
            }),
            // Update all task items to reflect MySQL 8.0
            serde_json::json!({
                "target": "tasks",
                "command": "replace_list_item",
                "selector": {"type": "task_text", "value": "Set up MySQL 5.7 development environment"},
                "content": "Set up MySQL 8.0 development environment with proper authentication"
            }),
            serde_json::json!({
                "target": "tasks",
                "command": "replace_list_item",
                "selector": {"type": "task_text", "value": "Create migration scripts for MySQL 5.7 compatibility"},
                "content": "Create migration scripts for MySQL 8.0 compatibility and test new features"
            }),
            serde_json::json!({
                "target": "tasks",
                "command": "replace_list_item",
                "selector": {"type": "task_text", "value": "Test data export from current MySQL 5.7 instance"},
                "content": "Test data export and import between MySQL 5.7 and MySQL 8.0"
            }),
            serde_json::json!({
                "target": "tasks",
                "command": "replace_list_item",
                "selector": {"type": "task_text", "value": "Validate MySQL 5.7 performance benchmarks"},
                "content": "Validate MySQL 8.0 performance improvements and benchmark comparisons"
            }),
            serde_json::json!({
                "target": "tasks",
                "command": "replace_list_item",
                "selector": {"type": "task_text", "value": "Schedule maintenance window for MySQL 5.7 upgrade"},
                "content": "Schedule maintenance window for MySQL 5.7 to 8.0 upgrade"
            })
        ];

        let update_args = UpdateSpecArgs {
            project_name: "tech-upgrade".to_string(),
            spec_name: spec_name.clone(),
            commands_json: serde_json::to_string(&commands).unwrap(),
        };

        let response = update_spec::run(update_spec::Input {
            project_name: update_args.project_name,
            spec_name: update_args.spec_name,
            commands_json: update_args.commands_json,
        })
        .await
        .unwrap();

        // Verify all operations succeeded
        assert_eq!(response.data.applied_count, 11);
        assert_eq!(response.data.skipped_idempotent_count, 0);
        assert!(response.data.errors.as_ref().is_none_or(|e| e.is_empty()));

        // Verify spec.md changes
        let foundry_dir = env.foundry_dir();
        let spec_file = foundry_dir
            .join("tech-upgrade")
            .join("specs")
            .join(&spec_name)
            .join("spec.md");
        let spec_updated = std::fs::read_to_string(spec_file).unwrap();

        assert!(spec_updated.contains("MySQL 5.7 to MySQL 8.0"));
        assert!(!spec_updated.contains("MySQL 5.7 to latest version"));

        // Verify notes.md changes - comprehensive technology upgrade
        let notes_file = foundry_dir
            .join("tech-upgrade")
            .join("specs")
            .join(&spec_name)
            .join("notes.md");
        let notes_updated = std::fs::read_to_string(notes_file).unwrap();

        // Technology decisions completely replaced
        assert!(notes_updated.contains("Enhanced security features including caching_sha2_password"));
        assert!(notes_updated.contains("common table expressions (CTEs)"));
        assert!(!notes_updated.contains("The current MySQL 5.7 database will be upgraded"));

        // Migration strategy updated
        assert!(notes_updated.contains("MySQL 8.0 utilities and best practices"));
        assert!(notes_updated.contains("Set up new MySQL 8.0 instance with proper configuration"));
        assert!(!notes_updated.contains("Set up new MySQL 5.7 instance"));

        // Performance section completely replaced
        assert!(notes_updated.contains("Up to 2x faster read/write performance"));
        assert!(notes_updated.contains("Enhanced memory management for large datasets"));
        assert!(!notes_updated.contains("MySQL 5.7 has known limitations"));

        // Verify tasks.md changes - all tasks updated
        let tasks_file = foundry_dir
            .join("tech-upgrade")
            .join("specs")
            .join(&spec_name)
            .join("task-list.md");
        let tasks_updated = std::fs::read_to_string(tasks_file).unwrap();

        assert!(tasks_updated.contains("Set up MySQL 8.0 development environment"));
        assert!(tasks_updated.contains("MySQL 8.0 compatibility and test new features"));
        assert!(tasks_updated.contains("between MySQL 5.7 and MySQL 8.0"));
        assert!(tasks_updated.contains("MySQL 8.0 performance improvements"));
        assert!(tasks_updated.contains("MySQL 5.7 to 8.0 upgrade"));

        // Ensure no old MySQL 5.7 references remain in tasks
        assert!(!tasks_updated.contains("Set up MySQL 5.7 development environment"));
        assert!(!tasks_updated.contains("Create migration scripts for MySQL 5.7 compatibility"));
    });
}
