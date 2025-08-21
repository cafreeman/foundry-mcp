//! Integration tests for the repository layer

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
    format!("test_{}_{}", base, timestamp)
}

/// Clean up test project directory
async fn cleanup_test_project(fs_manager: &FileSystemManager, project_name: &str) {
    let project_dir = fs_manager.project_dir(project_name);
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir).ok();
    }
}

/// Helper function to create a sample project for testing
fn create_sample_project(name: &str) -> Project {
    Project {
        name: name.to_string(),
        description: format!("Test project: {}", name),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tech_stack: TechStack {
            languages: vec!["Rust".to_string(), "TypeScript".to_string()],
            frameworks: vec!["Actix-Web".to_string(), "React".to_string()],
            databases: vec!["PostgreSQL".to_string(), "Redis".to_string()],
            tools: vec!["Cargo".to_string(), "npm".to_string()],
            deployment: vec!["Docker".to_string(), "AWS".to_string()],
        },
        vision: Vision {
            overview: "A comprehensive project management tool".to_string(),
            goals: vec![
                "Provide deterministic context management".to_string(),
                "Enable seamless project collaboration".to_string(),
            ],
            target_users: vec![
                "AI coding assistants".to_string(),
                "Development teams".to_string(),
            ],
            success_criteria: vec![
                "Reduces context switching time by 50%".to_string(),
                "Increases development velocity".to_string(),
            ],
        },
    }
}

#[tokio::test]
async fn test_project_creation_and_loading() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("basic");
    let project = create_sample_project(&project_name);

    // Test project creation
    let create_result = project_repo.create_project(project.clone()).await;
    assert!(
        create_result.is_ok(),
        "Failed to create project: {:?}",
        create_result
    );

    // Test project loading
    let loaded_project = project_repo.load_project(&project_name).await;
    assert!(
        loaded_project.is_ok(),
        "Failed to load project: {:?}",
        loaded_project
    );

    let loaded = loaded_project.unwrap();
    assert_eq!(loaded.name, project.name);
    assert_eq!(loaded.description, project.description);
    assert_eq!(loaded.tech_stack.languages, project.tech_stack.languages);
    assert_eq!(loaded.vision.overview, project.vision.overview);

    // Test project existence check
    assert!(project_repo.project_exists(&project_name).await);

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_project_files_structure() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("files");
    let project = create_sample_project(&project_name);

    project_repo.create_project(project).await.unwrap();

    // Verify directory structure
    let project_dir = fs_manager.project_dir(&project_name);
    let project_info_dir = fs_manager.project_info_dir(&project_name);
    let specs_dir = fs_manager.specs_dir(&project_name);

    assert!(project_dir.exists());
    assert!(project_info_dir.exists());
    assert!(specs_dir.exists());

    // Verify specific files
    let metadata_file = project_dir.join("project.json");
    let tech_stack_file = project_info_dir.join("tech-stack.md");
    let vision_file = project_info_dir.join("vision.md");

    assert!(fs_manager.file_exists(&metadata_file));
    assert!(fs_manager.file_exists(&tech_stack_file));
    assert!(fs_manager.file_exists(&vision_file));

    // Verify file contents contain expected data
    let tech_stack_content = fs_manager.read_file(&tech_stack_file).unwrap();
    assert!(tech_stack_content.contains("# Tech Stack"));
    assert!(tech_stack_content.contains("Rust"));

    let vision_content = fs_manager.read_file(&vision_file).unwrap();
    assert!(vision_content.contains("# Project Vision"));
    assert!(vision_content.contains("comprehensive project management tool"));

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_duplicate_project_creation() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("duplicate");
    let project = create_sample_project(&project_name);

    // First creation should succeed
    let result1 = project_repo.create_project(project.clone()).await;
    assert!(result1.is_ok());

    // Second creation should fail
    let result2 = project_repo.create_project(project).await;
    assert!(result2.is_err());

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_specification_creation() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("spec");
    let project = create_sample_project(&project_name);

    // Create project first
    project_repo.create_project(project).await.unwrap();

    // Create specification
    let spec_name = "test_feature";
    let spec_description = "A test feature specification";
    let spec_content = "# Test Feature\n\nThis is a test feature specification.";

    let spec_result = spec_repo
        .create_spec(&project_name, spec_name, spec_description, spec_content)
        .await;

    assert!(
        spec_result.is_ok(),
        "Failed to create specification: {:?}",
        spec_result
    );
    let spec = spec_result.unwrap();

    // Verify specification details
    assert_eq!(spec.name, spec_name);
    assert_eq!(spec.description, spec_description);
    assert_eq!(spec.content, spec_content);
    assert_eq!(spec.status, SpecStatus::Draft);
    assert!(spec.id.contains(spec_name));

    // Verify spec exists
    assert!(spec_repo.spec_exists(&project_name, &spec.id).await);

    // Load specification
    let loaded_spec = spec_repo.load_spec(&project_name, &spec.id).await;
    assert!(loaded_spec.is_ok());

    let loaded = loaded_spec.unwrap();
    assert_eq!(loaded.id, spec.id);
    assert_eq!(loaded.name, spec.name);
    assert_eq!(loaded.description, spec.description);
    assert_eq!(loaded.content, spec.content);

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_specification_files_structure() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("spec_files");
    let project = create_sample_project(&project_name);

    // Create project and specification
    project_repo.create_project(project).await.unwrap();

    let spec = spec_repo
        .create_spec(
            &project_name,
            "test_feature",
            "Test feature description",
            "# Test Feature Content",
        )
        .await
        .unwrap();

    // Verify specification directory structure
    let spec_dir = fs_manager.spec_dir(&project_name, &spec.id);
    assert!(spec_dir.exists());

    // Verify specification files exist
    let spec_metadata_file = spec_dir.join("spec.json");
    let spec_content_file = spec_dir.join("spec.md");
    let task_list_file = spec_dir.join("task-list.md");
    let notes_file = spec_dir.join("notes.md");

    assert!(fs_manager.file_exists(&spec_metadata_file));
    assert!(fs_manager.file_exists(&spec_content_file));
    assert!(fs_manager.file_exists(&task_list_file));
    assert!(fs_manager.file_exists(&notes_file));

    // Verify file contents
    let spec_content = fs_manager.read_file(&spec_content_file).unwrap();
    assert!(spec_content.contains("# Test Feature Content"));

    let task_list_content = fs_manager.read_file(&task_list_file).unwrap();
    assert!(task_list_content.contains("# Task List"));

    let notes_content = fs_manager.read_file(&notes_file).unwrap();
    assert!(notes_content.contains("# Notes"));

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_task_management() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("tasks");
    let project = create_sample_project(&project_name);

    // Create project and specification
    project_repo.create_project(project).await.unwrap();
    let spec = spec_repo
        .create_spec(
            &project_name,
            "task_feature",
            "Feature with tasks",
            "# Task Feature",
        )
        .await
        .unwrap();

    // Create a task
    let task = Task {
        id: "task_001".to_string(),
        title: "Implement authentication".to_string(),
        description: "Add JWT-based authentication".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::High,
        dependencies: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Add task to specification
    let add_result = spec_repo
        .add_task(&project_name, &spec.id, task.clone())
        .await;
    assert!(add_result.is_ok(), "Failed to add task: {:?}", add_result);

    // Load task list
    let task_list_result = spec_repo.load_task_list(&project_name, &spec.id).await;
    assert!(task_list_result.is_ok());

    let task_list = task_list_result.unwrap();
    assert_eq!(task_list.tasks.len(), 1);
    assert_eq!(task_list.tasks[0].title, task.title);
    assert_eq!(task_list.tasks[0].status, TaskStatus::Todo);

    // Update task status
    let update_result = spec_repo
        .update_task_status(&project_name, &spec.id, &task.id, TaskStatus::InProgress)
        .await;
    assert!(update_result.is_ok());

    // Verify status update
    let updated_task_list = spec_repo
        .load_task_list(&project_name, &spec.id)
        .await
        .unwrap();
    assert_eq!(updated_task_list.tasks[0].status, TaskStatus::InProgress);

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_notes_management() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name = generate_test_project_name("notes");
    let project = create_sample_project(&project_name);

    // Create project and specification
    project_repo.create_project(project).await.unwrap();
    let spec = spec_repo
        .create_spec(
            &project_name,
            "notes_feature",
            "Feature with notes",
            "# Notes Feature",
        )
        .await
        .unwrap();

    // Create a note
    let note = Note {
        id: "note_001".to_string(),
        content: "Consider using OAuth 2.0 for authentication".to_string(),
        category: NoteCategory::Implementation,
        created_at: Utc::now(),
    };

    // Add note to specification
    let add_result = spec_repo
        .add_note(&project_name, &spec.id, note.clone())
        .await;
    assert!(add_result.is_ok(), "Failed to add note: {:?}", add_result);

    // Load notes
    let notes_result = spec_repo.load_notes(&project_name, &spec.id).await;
    assert!(notes_result.is_ok());

    let notes = notes_result.unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, note.content);
    assert_eq!(notes[0].category, NoteCategory::Implementation);

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name).await;
}

#[tokio::test]
async fn test_list_operations() {
    let fs_manager = FileSystemManager::new().unwrap();
    let project_repo = ProjectRepository::new(fs_manager.clone());
    let spec_repo = SpecificationRepository::new(fs_manager.clone());

    let project_name1 = generate_test_project_name("list1");
    let project_name2 = generate_test_project_name("list2");

    // Create multiple projects
    let project1 = create_sample_project(&project_name1);
    let project2 = create_sample_project(&project_name2);

    project_repo.create_project(project1).await.unwrap();
    project_repo.create_project(project2).await.unwrap();

    // List projects - should include our test projects
    let projects = project_repo.list_projects().await.unwrap();
    let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
    assert!(project_names.contains(&project_name1));
    assert!(project_names.contains(&project_name2));

    // Create specifications for one project
    spec_repo
        .create_spec(&project_name1, "feature_a", "Feature A", "# Feature A")
        .await
        .unwrap();
    spec_repo
        .create_spec(&project_name1, "feature_b", "Feature B", "# Feature B")
        .await
        .unwrap();

    // List specifications
    let specs = spec_repo.list_specs(&project_name1).await.unwrap();
    assert_eq!(specs.len(), 2);

    let spec_names: Vec<String> = specs.iter().map(|s| s.name.clone()).collect();
    assert!(spec_names.contains(&"feature_a".to_string()));
    assert!(spec_names.contains(&"feature_b".to_string()));

    // Cleanup
    cleanup_test_project(&fs_manager, &project_name1).await;
    cleanup_test_project(&fs_manager, &project_name2).await;
}
