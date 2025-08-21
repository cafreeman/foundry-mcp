//! End-to-end workflow tests for the Project Manager MCP

use chrono::Utc;
use foundry_mcp::filesystem::FileSystemManager;
use foundry_mcp::models::*;
use foundry_mcp::repository::{ProjectRepository, SpecificationRepository};

/// Generate a unique test project name to avoid conflicts
fn generate_test_project_name(base: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("e2e_test_{}_{}", base, timestamp)
}

/// Clean up test project directory
async fn cleanup_test_project(fs_manager: &FileSystemManager, project_name: &str) {
    let project_dir = fs_manager.project_dir(project_name);
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir).ok();
    }
}

#[tokio::test]
async fn test_complete_project_development_workflow() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("complete_workflow");

    // Step 1: Create a new project
    let project = Project {
        name: project_name.clone(),
        description: "Complete workflow test project".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tech_stack: TechStack {
            languages: vec!["Rust".to_string(), "JavaScript".to_string()],
            frameworks: vec!["Actix-Web".to_string(), "React".to_string()],
            databases: vec!["PostgreSQL".to_string(), "Redis".to_string()],
            tools: vec!["Cargo".to_string(), "npm".to_string(), "Docker".to_string()],
            deployment: vec!["AWS".to_string(), "Kubernetes".to_string()],
        },
        vision: Vision {
            overview: "A comprehensive web application with modern architecture".to_string(),
            goals: vec![
                "Build scalable backend API".to_string(),
                "Create responsive frontend".to_string(),
                "Implement real-time features".to_string(),
                "Ensure high performance".to_string(),
            ],
            target_users: vec![
                "End users seeking productivity tools".to_string(),
                "Administrators managing the system".to_string(),
                "Developers extending functionality".to_string(),
            ],
            success_criteria: vec![
                "Handle 10,000+ concurrent users".to_string(),
                "Sub-200ms API response times".to_string(),
                "99.9% uptime availability".to_string(),
                "Mobile-responsive design".to_string(),
            ],
        },
    };

    let create_result = project_repo.create_project(project).await;
    assert!(
        create_result.is_ok(),
        "Failed to create project: {:?}",
        create_result
    );

    // Step 2: Verify project files were created correctly
    let project_dir = fs_manager.project_dir(&project_name);
    let project_info_dir = fs_manager.project_info_dir(&project_name);
    let specs_dir = fs_manager.specs_dir(&project_name);

    assert!(project_dir.exists());
    assert!(project_info_dir.exists());
    assert!(specs_dir.exists());

    // Verify content files
    let tech_stack_file = project_info_dir.join("tech-stack.md");
    let vision_file = project_info_dir.join("vision.md");

    assert!(fs_manager.file_exists(&tech_stack_file));
    assert!(fs_manager.file_exists(&vision_file));

    let tech_stack_content = fs_manager.read_file(&tech_stack_file).unwrap();
    assert!(tech_stack_content.contains("# Tech Stack"));
    assert!(tech_stack_content.contains("Rust"));
    assert!(tech_stack_content.contains("React"));
    assert!(tech_stack_content.contains("PostgreSQL"));

    let vision_content = fs_manager.read_file(&vision_file).unwrap();
    assert!(vision_content.contains("# Project Vision"));
    assert!(vision_content.contains("comprehensive web application"));
    assert!(vision_content.contains("scalable backend"));

    // Step 3: Create multiple specifications for different features
    let feature_specs = vec![
        ("user_authentication", "User Authentication System"),
        ("api_endpoints", "RESTful API Endpoints"),
        ("frontend_components", "React Frontend Components"),
        ("database_schema", "Database Schema Design"),
        ("deployment_pipeline", "CI/CD Deployment Pipeline"),
    ];

    let mut created_specs = Vec::new();

    for (spec_name, description) in feature_specs {
        let spec = spec_repo
            .create_spec(
                &project_name,
                spec_name,
                description,
                &format!(
                    "# {}\n\nDetailed specification for {}.",
                    description, spec_name
                ),
            )
            .await
            .unwrap();

        created_specs.push(spec);
    }

    assert_eq!(created_specs.len(), 5);

    // Step 4: Add tasks to each specification
    for (i, spec) in created_specs.iter().enumerate() {
        let tasks = match i {
            0 => vec![
                // user_authentication
                (
                    "Design authentication flow",
                    "High",
                    "Design JWT-based authentication system",
                ),
                (
                    "Implement user registration",
                    "High",
                    "Create user signup endpoint with validation",
                ),
                (
                    "Implement user login",
                    "High",
                    "Create login endpoint with JWT token generation",
                ),
                (
                    "Add password reset",
                    "Medium",
                    "Implement forgot password functionality",
                ),
            ],
            1 => vec![
                // api_endpoints
                ("Design API schema", "High", "Create OpenAPI specification"),
                (
                    "Implement user endpoints",
                    "High",
                    "CRUD operations for user management",
                ),
                (
                    "Implement data endpoints",
                    "Medium",
                    "Business logic API endpoints",
                ),
                (
                    "Add API documentation",
                    "Low",
                    "Generate and host API documentation",
                ),
            ],
            2 => vec![
                // frontend_components
                (
                    "Setup React project",
                    "High",
                    "Initialize React app with TypeScript",
                ),
                (
                    "Create authentication components",
                    "High",
                    "Login, register, logout components",
                ),
                (
                    "Build dashboard components",
                    "Medium",
                    "Main application dashboard",
                ),
                (
                    "Implement responsive design",
                    "Medium",
                    "Mobile-friendly UI components",
                ),
            ],
            3 => vec![
                // database_schema
                (
                    "Design user tables",
                    "High",
                    "User accounts and profiles schema",
                ),
                (
                    "Design application tables",
                    "High",
                    "Core business logic tables",
                ),
                (
                    "Add database migrations",
                    "Medium",
                    "Version-controlled schema changes",
                ),
                (
                    "Optimize database queries",
                    "Low",
                    "Add indexes and query optimization",
                ),
            ],
            4 => vec![
                // deployment_pipeline
                (
                    "Setup CI/CD pipeline",
                    "High",
                    "GitHub Actions or GitLab CI",
                ),
                (
                    "Configure Docker containers",
                    "High",
                    "Containerize application components",
                ),
                (
                    "Setup monitoring",
                    "Medium",
                    "Application and infrastructure monitoring",
                ),
                (
                    "Add automated testing",
                    "Medium",
                    "Unit, integration, and e2e tests",
                ),
            ],
            _ => continue,
        };

        for (title, priority_str, description) in tasks {
            let priority = match priority_str {
                "High" => TaskPriority::High,
                "Medium" => TaskPriority::Medium,
                "Low" => TaskPriority::Low,
                _ => TaskPriority::Medium,
            };

            let task = Task {
                id: format!(
                    "task_{}_{}",
                    spec.id,
                    title.to_lowercase().replace(" ", "_")
                ),
                title: title.to_string(),
                description: description.to_string(),
                status: TaskStatus::Todo,
                priority,
                dependencies: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            spec_repo
                .add_task(&project_name, &spec.id, task)
                .await
                .unwrap();
        }
    }

    // Step 5: Add notes to specifications
    let notes_data = vec![
        (
            "user_authentication",
            vec![
                (
                    "Consider OAuth 2.0 for third-party login",
                    NoteCategory::Implementation,
                ),
                ("Use bcrypt for password hashing", NoteCategory::Decision),
                (
                    "Implement rate limiting for login attempts",
                    NoteCategory::Enhancement,
                ),
            ],
        ),
        (
            "api_endpoints",
            vec![
                ("Follow REST conventions strictly", NoteCategory::Decision),
                (
                    "Add comprehensive input validation",
                    NoteCategory::Implementation,
                ),
                (
                    "Consider GraphQL for complex queries",
                    NoteCategory::Question,
                ),
            ],
        ),
        (
            "frontend_components",
            vec![
                (
                    "Use Material-UI for consistent design",
                    NoteCategory::Decision,
                ),
                (
                    "Implement lazy loading for performance",
                    NoteCategory::Enhancement,
                ),
                (
                    "Add accessibility features (ARIA)",
                    NoteCategory::Implementation,
                ),
            ],
        ),
    ];

    for (spec_name, note_list) in notes_data {
        let spec = created_specs.iter().find(|s| s.name == spec_name).unwrap();

        for (content, category) in note_list {
            let note = Note {
                id: format!("note_{}_{}", spec.id, Utc::now().timestamp()),
                content: content.to_string(),
                category,
                created_at: Utc::now(),
            };

            spec_repo
                .add_note(&project_name, &spec.id, note)
                .await
                .unwrap();
        }
    }

    // Step 6: Simulate development progress by updating task statuses
    let auth_spec = created_specs
        .iter()
        .find(|s| s.name == "user_authentication")
        .unwrap();
    let task_list = spec_repo
        .load_task_list(&project_name, &auth_spec.id)
        .await
        .unwrap();

    // Mark first two tasks as completed
    for (i, task) in task_list.tasks.iter().take(2).enumerate() {
        let new_status = if i == 0 {
            TaskStatus::Completed
        } else {
            TaskStatus::InProgress
        };
        spec_repo
            .update_task_status(&project_name, &auth_spec.id, &task.id, new_status)
            .await
            .unwrap();
    }

    // Step 7: Load and verify the complete project context
    let loaded_project = project_repo.load_project(&project_name).await.unwrap();
    let all_specs = spec_repo.list_specs(&project_name).await.unwrap();

    assert_eq!(loaded_project.name, project_name);
    assert_eq!(all_specs.len(), 5);

    // Step 8: Verify specific specification content
    for spec in &all_specs {
        let loaded_spec = spec_repo.load_spec(&project_name, &spec.id).await.unwrap();
        let task_list = spec_repo
            .load_task_list(&project_name, &spec.id)
            .await
            .unwrap();
        let _notes = spec_repo.load_notes(&project_name, &spec.id).await.unwrap();

        assert!(!loaded_spec.content.is_empty());
        assert!(!task_list.tasks.is_empty());

        // Verify file structure
        let spec_dir = fs_manager.spec_dir(&project_name, &spec.id);
        assert!(spec_dir.exists());

        let spec_files = vec!["spec.json", "spec.md", "task-list.md", "notes.md"];
        for file in spec_files {
            let file_path = spec_dir.join(file);
            assert!(fs_manager.file_exists(&file_path), "Missing file: {}", file);
        }
    }

    // Step 9: Simulate project completion
    let api_spec = created_specs
        .iter()
        .find(|s| s.name == "api_endpoints")
        .unwrap();
    let mut updated_spec = spec_repo
        .load_spec(&project_name, &api_spec.id)
        .await
        .unwrap();
    updated_spec.status = SpecStatus::Completed;

    spec_repo
        .update_spec(&project_name, &updated_spec)
        .await
        .unwrap();

    let completed_spec = spec_repo
        .load_spec(&project_name, &api_spec.id)
        .await
        .unwrap();
    assert_eq!(completed_spec.status, SpecStatus::Completed);

    // Step 10: Generate project summary
    let project_summary = generate_project_summary(&project_repo, &spec_repo, &project_name).await;

    assert!(project_summary.contains(&project_name));
    assert!(project_summary.contains("5 total"));
    assert!(project_summary.contains("user_authentication"));
    assert!(project_summary.contains("Completed"));
    assert!(project_summary.contains("Draft")); // Most specs are in Draft status

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_multi_developer_collaboration_workflow() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("collaboration");

    // Simulate multiple developers working on the same project
    let project = Project {
        name: project_name.clone(),
        description: "Multi-developer collaboration test".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tech_stack: TechStack {
            languages: vec!["Python".to_string(), "JavaScript".to_string()],
            frameworks: vec!["Django".to_string(), "Vue.js".to_string()],
            databases: vec!["PostgreSQL".to_string()],
            tools: vec!["Git".to_string(), "Docker".to_string()],
            deployment: vec!["Heroku".to_string()],
        },
        vision: Vision {
            overview: "Collaborative development workflow testing".to_string(),
            goals: vec!["Test concurrent access".to_string()],
            target_users: vec!["Developers".to_string()],
            success_criteria: vec!["No data corruption".to_string()],
        },
    };

    project_repo.create_project(project).await.unwrap();

    // Create specifications for different developers
    let developer_specs = vec![
        ("backend_api", "Backend Developer"),
        ("frontend_ui", "Frontend Developer"),
        ("database_design", "Database Developer"),
    ];

    let mut specs = Vec::new();
    for (spec_name, developer) in developer_specs {
        let spec = spec_repo
            .create_spec(
                &project_name,
                spec_name,
                &format!("Specification assigned to {}", developer),
                &format!("# {}\n\nDeveloper: {}", spec_name, developer),
            )
            .await
            .unwrap();
        specs.push(spec);
    }

    // Simulate concurrent updates by different developers
    for (i, spec) in specs.iter().enumerate() {
        // Add tasks specific to each developer
        let task = Task {
            id: format!("dev_task_{}", i),
            title: format!("Development task {}", i),
            description: format!("Task assigned to developer {}", i),
            status: TaskStatus::InProgress,
            priority: TaskPriority::Medium,
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        spec_repo
            .add_task(&project_name, &spec.id, task)
            .await
            .unwrap();

        // Add notes from different developers
        let note = Note {
            id: format!("dev_note_{}", i),
            content: format!("Development note from developer {}", i),
            category: NoteCategory::Implementation,
            created_at: Utc::now(),
        };

        spec_repo
            .add_note(&project_name, &spec.id, note)
            .await
            .unwrap();
    }

    // Verify all changes were persisted correctly
    for spec in &specs {
        let task_list = spec_repo
            .load_task_list(&project_name, &spec.id)
            .await
            .unwrap();
        let notes = spec_repo.load_notes(&project_name, &spec.id).await.unwrap();

        assert!(!task_list.tasks.is_empty());
        assert!(!notes.is_empty());
    }

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_project_pause_resume_workflow() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("pause_resume");

    // Create a project and work on it
    let project = Project {
        name: project_name.clone(),
        description: "Pause/resume workflow test".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tech_stack: TechStack {
            languages: vec!["Go".to_string()],
            frameworks: vec!["Gin".to_string()],
            databases: vec!["MongoDB".to_string()],
            tools: vec!["Docker".to_string()],
            deployment: vec!["GCP".to_string()],
        },
        vision: Vision {
            overview: "Testing project pause and resume functionality".to_string(),
            goals: vec!["Maintain work state".to_string()],
            target_users: vec!["Developers".to_string()],
            success_criteria: vec!["No work loss".to_string()],
        },
    };

    project_repo.create_project(project).await.unwrap();

    // Create a specification and start working
    let spec = spec_repo
        .create_spec(
            &project_name,
            "main_feature",
            "Main feature development",
            "# Main Feature\n\nWork in progress...",
        )
        .await
        .unwrap();

    // Add several tasks
    let initial_tasks = vec![
        (
            "Setup project structure",
            TaskStatus::Completed,
            TaskPriority::High,
        ),
        (
            "Implement core logic",
            TaskStatus::InProgress,
            TaskPriority::High,
        ),
        ("Add error handling", TaskStatus::Todo, TaskPriority::Medium),
        ("Write unit tests", TaskStatus::Todo, TaskPriority::Medium),
        ("Add documentation", TaskStatus::Todo, TaskPriority::Low),
    ];

    for (i, (title, status, priority)) in initial_tasks.iter().enumerate() {
        let task = Task {
            id: format!("pause_task_{}", i),
            title: title.to_string(),
            description: format!("Description for {}", title),
            status: status.clone(),
            priority: priority.clone(),
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        spec_repo
            .add_task(&project_name, &spec.id, task)
            .await
            .unwrap();
    }

    // Add progress notes
    let progress_notes = vec![
        "Completed initial setup successfully",
        "Currently working on core business logic",
        "Need to research error handling patterns",
        "Planning comprehensive test coverage",
    ];

    for (i, content) in progress_notes.iter().enumerate() {
        let note = Note {
            id: format!("progress_note_{}", i),
            content: content.to_string(),
            category: NoteCategory::Implementation,
            created_at: Utc::now(),
        };

        spec_repo
            .add_note(&project_name, &spec.id, note)
            .await
            .unwrap();
    }

    // Simulate project pause - update spec status
    let mut paused_spec = spec_repo.load_spec(&project_name, &spec.id).await.unwrap();
    paused_spec.status = SpecStatus::OnHold;
    spec_repo
        .update_spec(&project_name, &paused_spec)
        .await
        .unwrap();

    // Add a note about pausing
    let pause_note = Note {
        id: "pause_note".to_string(),
        content:
            "Project paused due to priority changes. Current progress: core logic 70% complete."
                .to_string(),
        category: NoteCategory::Decision,
        created_at: Utc::now(),
    };
    spec_repo
        .add_note(&project_name, &spec.id, pause_note)
        .await
        .unwrap();

    // === PAUSE PERIOD ===
    // Simulate some time passing, other work happening...

    // === RESUME WORK ===

    // Resume project - load current state
    let resumed_project = project_repo.load_project(&project_name).await.unwrap();
    let resumed_spec = spec_repo.load_spec(&project_name, &spec.id).await.unwrap();
    let current_tasks = spec_repo
        .load_task_list(&project_name, &spec.id)
        .await
        .unwrap();
    let current_notes = spec_repo.load_notes(&project_name, &spec.id).await.unwrap();

    // Verify all state was preserved
    assert_eq!(resumed_project.name, project_name);
    assert_eq!(resumed_spec.status, SpecStatus::OnHold);
    assert_eq!(current_tasks.tasks.len(), 5);
    assert_eq!(current_notes.len(), 5); // 4 progress + 1 pause note

    // Resume active development
    let mut active_spec = resumed_spec.clone();
    active_spec.status = SpecStatus::InProgress;
    spec_repo
        .update_spec(&project_name, &active_spec)
        .await
        .unwrap();

    // Continue work - complete next task
    let in_progress_task = current_tasks
        .tasks
        .iter()
        .find(|t| t.status == TaskStatus::InProgress)
        .unwrap();

    spec_repo
        .update_task_status(
            &project_name,
            &spec.id,
            &in_progress_task.id,
            TaskStatus::Completed,
        )
        .await
        .unwrap();

    // Start next task
    let next_task = current_tasks
        .tasks
        .iter()
        .find(|t| t.status == TaskStatus::Todo && t.priority == TaskPriority::Medium)
        .unwrap();

    spec_repo
        .update_task_status(
            &project_name,
            &spec.id,
            &next_task.id,
            TaskStatus::InProgress,
        )
        .await
        .unwrap();

    // Add resume note
    let resume_note = Note {
        id: "resume_note".to_string(),
        content: "Project resumed. Completed core logic implementation. Starting error handling."
            .to_string(),
        category: NoteCategory::Implementation,
        created_at: Utc::now(),
    };
    spec_repo
        .add_note(&project_name, &spec.id, resume_note)
        .await
        .unwrap();

    // Verify final state
    let final_tasks = spec_repo
        .load_task_list(&project_name, &spec.id)
        .await
        .unwrap();
    let completed_tasks: Vec<_> = final_tasks
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .collect();
    let in_progress_tasks: Vec<_> = final_tasks
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .collect();

    assert_eq!(completed_tasks.len(), 2); // Setup + core logic
    assert_eq!(in_progress_tasks.len(), 1); // Error handling

    let final_notes = spec_repo.load_notes(&project_name, &spec.id).await.unwrap();
    assert_eq!(final_notes.len(), 6); // All previous + resume note

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

// Helper function to generate project summary
async fn generate_project_summary(
    project_repo: &ProjectRepository,
    spec_repo: &SpecificationRepository,
    project_name: &str,
) -> String {
    let project = project_repo.load_project(project_name).await.unwrap();
    let specs = spec_repo.list_specs(project_name).await.unwrap();

    let mut summary = format!("# Project Summary: {}\n\n", project.name);
    summary.push_str(&format!("**Description:** {}\n", project.description));
    summary.push_str(&format!(
        "**Created:** {}\n",
        project.created_at.format("%Y-%m-%d")
    ));
    summary.push_str(&format!("**Specifications:** {} total\n\n", specs.len()));

    for spec in specs {
        let task_list = spec_repo
            .load_task_list(project_name, &spec.id)
            .await
            .unwrap();
        let notes = spec_repo.load_notes(project_name, &spec.id).await.unwrap();

        let completed_tasks = task_list
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let total_tasks = task_list.tasks.len();

        summary.push_str(&format!(
            "## {} ({})\n- Tasks: {}/{} completed\n- Notes: {}\n- Status: {:?}\n\n",
            spec.name,
            spec.id,
            completed_tasks,
            total_tasks,
            notes.len(),
            spec.status
        ));
    }

    summary
}
