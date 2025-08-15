//! Validation utilities for project and specification data

use crate::models::{Project, Specification, Task, TaskList, TechStack, Vision};
use chrono::Utc;

/// Validate a complete project structure
pub fn validate_project(project: &Project) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate project name
    if let Err(e) = crate::utils::id_generation::validate_project_name(&project.name) {
        errors.push(format!("Project name: {}", e));
    }

    // Validate description
    if project.description.trim().is_empty() {
        errors.push("Project description cannot be empty".to_string());
    }

    if project.description.len() > 1000 {
        errors.push("Project description cannot exceed 1000 characters".to_string());
    }

    // Validate timestamps
    if project.created_at > project.updated_at {
        errors.push("Created timestamp cannot be after updated timestamp".to_string());
    }

    // Validate tech stack
    if let Err(e) = validate_tech_stack(&project.tech_stack) {
        errors.extend(e);
    }

    // Validate vision
    if let Err(e) = validate_vision(&project.vision) {
        errors.extend(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a tech stack structure
pub fn validate_tech_stack(tech_stack: &TechStack) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate languages
    for (i, language) in tech_stack.languages.iter().enumerate() {
        if language.trim().is_empty() {
            errors.push(format!("Language {} cannot be empty", i + 1));
        }
        if language.len() > 50 {
            errors.push(format!(
                "Language '{}' cannot exceed 50 characters",
                language
            ));
        }
    }

    // Validate frameworks
    for (i, framework) in tech_stack.frameworks.iter().enumerate() {
        if framework.trim().is_empty() {
            errors.push(format!("Framework {} cannot be empty", i + 1));
        }
        if framework.len() > 100 {
            errors.push(format!(
                "Framework '{}' cannot exceed 100 characters",
                framework
            ));
        }
    }

    // Validate databases
    for (i, database) in tech_stack.databases.iter().enumerate() {
        if database.trim().is_empty() {
            errors.push(format!("Database {} cannot be empty", i + 1));
        }
        if database.len() > 100 {
            errors.push(format!(
                "Database '{}' cannot exceed 100 characters",
                database
            ));
        }
    }

    // Validate tools
    for (i, tool) in tech_stack.tools.iter().enumerate() {
        if tool.trim().is_empty() {
            errors.push(format!("Tool {} cannot be empty", i + 1));
        }
        if tool.len() > 100 {
            errors.push(format!("Tool '{}' cannot exceed 100 characters", tool));
        }
    }

    // Validate deployment
    for (i, deployment) in tech_stack.deployment.iter().enumerate() {
        if deployment.trim().is_empty() {
            errors.push(format!("Deployment {} cannot be empty", i + 1));
        }
        if deployment.len() > 100 {
            errors.push(format!(
                "Deployment '{}' cannot exceed 100 characters",
                deployment
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a vision structure
pub fn validate_vision(vision: &Vision) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate overview
    if vision.overview.trim().is_empty() {
        errors.push("Vision overview cannot be empty".to_string());
    }

    if vision.overview.len() > 2000 {
        errors.push("Vision overview cannot exceed 2000 characters".to_string());
    }

    // Validate goals
    if vision.goals.is_empty() {
        errors.push("At least one goal must be specified".to_string());
    }

    for (i, goal) in vision.goals.iter().enumerate() {
        if goal.trim().is_empty() {
            errors.push(format!("Goal {} cannot be empty", i + 1));
        }
        if goal.len() > 500 {
            errors.push(format!("Goal '{}' cannot exceed 500 characters", goal));
        }
    }

    // Validate target users
    if vision.target_users.is_empty() {
        errors.push("At least one target user must be specified".to_string());
    }

    for (i, user) in vision.target_users.iter().enumerate() {
        if user.trim().is_empty() {
            errors.push(format!("Target user {} cannot be empty", i + 1));
        }
        if user.len() > 200 {
            errors.push(format!(
                "Target user '{}' cannot exceed 200 characters",
                user
            ));
        }
    }

    // Validate success criteria
    if vision.success_criteria.is_empty() {
        errors.push("At least one success criterion must be specified".to_string());
    }

    for (i, criterion) in vision.success_criteria.iter().enumerate() {
        if criterion.trim().is_empty() {
            errors.push(format!("Success criterion {} cannot be empty", i + 1));
        }
        if criterion.len() > 500 {
            errors.push(format!(
                "Success criterion '{}' cannot exceed 500 characters",
                criterion
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a specification structure
pub fn validate_specification(spec: &Specification) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate ID
    if let Err(e) = crate::utils::id_generation::validate_spec_id(&spec.id) {
        errors.push(format!("Specification ID: {}", e));
    }

    // Validate name
    if spec.name.trim().is_empty() {
        errors.push("Specification name cannot be empty".to_string());
    }

    if spec.name.len() > 100 {
        errors.push("Specification name cannot exceed 100 characters".to_string());
    }

    // Validate description
    if spec.description.trim().is_empty() {
        errors.push("Specification description cannot be empty".to_string());
    }

    if spec.description.len() > 1000 {
        errors.push("Specification description cannot exceed 1000 characters".to_string());
    }

    // Validate timestamps
    if spec.created_at > spec.updated_at {
        errors.push("Created timestamp cannot be after updated timestamp".to_string());
    }

    // Validate content
    if spec.content.len() > 10000 {
        errors.push("Specification content cannot exceed 10000 characters".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a task structure
pub fn validate_task(task: &Task) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate ID
    if task.id.trim().is_empty() {
        errors.push("Task ID cannot be empty".to_string());
    }

    if task.id.len() > 100 {
        errors.push("Task ID cannot exceed 100 characters".to_string());
    }

    // Validate title
    if task.title.trim().is_empty() {
        errors.push("Task title cannot be empty".to_string());
    }

    if task.title.len() > 200 {
        errors.push("Task title cannot exceed 200 characters".to_string());
    }

    // Validate description
    if task.description.len() > 1000 {
        errors.push("Task description cannot exceed 1000 characters".to_string());
    }

    // Validate timestamps
    if task.created_at > task.updated_at {
        errors.push("Created timestamp cannot be after updated timestamp".to_string());
    }

    // Validate dependencies
    for (i, dependency) in task.dependencies.iter().enumerate() {
        if dependency.trim().is_empty() {
            errors.push(format!("Dependency {} cannot be empty", i + 1));
        }
        if dependency.len() > 100 {
            errors.push(format!(
                "Dependency '{}' cannot exceed 100 characters",
                dependency
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate a task list structure
pub fn validate_task_list(task_list: &TaskList) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate tasks
    for (i, task) in task_list.tasks.iter().enumerate() {
        if let Err(task_errors) = validate_task(task) {
            for task_error in task_errors {
                errors.push(format!("Task {}: {}", i + 1, task_error));
            }
        }
    }

    // Validate last updated timestamp
    let now = Utc::now();
    if task_list.last_updated > now {
        errors.push("Last updated timestamp cannot be in the future".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate that a project name is available (not already taken)
pub fn validate_project_name_availability(
    project_name: &str,
    existing_projects: &[String],
) -> Result<(), String> {
    if existing_projects.iter().any(|name| name == project_name) {
        Err(format!("Project name '{}' is already taken", project_name))
    } else {
        Ok(())
    }
}

/// Validate that a specification name is available within a project
pub fn validate_spec_name_availability(
    spec_name: &str,
    existing_specs: &[String],
) -> Result<(), String> {
    if existing_specs.iter().any(|name| name == spec_name) {
        Err(format!(
            "Specification name '{}' is already taken in this project",
            spec_name
        ))
    } else {
        Ok(())
    }
}

/// Validate file path safety (prevent directory traversal attacks)
pub fn validate_file_path_safety(path: &str) -> Result<(), String> {
    if path.contains("..") {
        return Err("Path contains directory traversal attempt".to_string());
    }

    if path.starts_with('/') || path.starts_with('\\') {
        return Err("Path cannot be absolute".to_string());
    }

    if path.contains('\0') {
        return Err("Path contains null character".to_string());
    }

    // Check for other potentially dangerous patterns
    let dangerous_patterns = ["/etc/", "/var/", "/usr/", "/bin/", "/sbin/", "C:\\", "D:\\"];
    for pattern in &dangerous_patterns {
        if path.to_lowercase().contains(pattern) {
            return Err(format!(
                "Path contains potentially dangerous pattern: {}",
                pattern
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SpecStatus;

    fn create_test_project() -> Project {
        Project {
            name: "test_project".to_string(),
            description: "A test project".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tech_stack: TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Actix".to_string()],
                databases: vec!["PostgreSQL".to_string()],
                tools: vec!["Cargo".to_string()],
                deployment: vec!["Docker".to_string()],
            },
            vision: Vision {
                overview: "A test project overview".to_string(),
                goals: vec!["Goal 1".to_string()],
                target_users: vec!["Developer".to_string()],
                success_criteria: vec!["Criterion 1".to_string()],
            },
        }
    }

    fn create_test_spec() -> Specification {
        Specification {
            id: "20240101_test_spec".to_string(),
            name: "test_spec".to_string(),
            description: "A test specification".to_string(),
            status: SpecStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: "Test content".to_string(),
        }
    }

    fn create_test_task() -> Task {
        Task {
            id: "task_1".to_string(),
            title: "Test Task".to_string(),
            description: "A test task".to_string(),
            status: crate::models::TaskStatus::Todo,
            priority: crate::models::TaskPriority::Medium,
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_validate_project() {
        let project = create_test_project();
        assert!(validate_project(&project).is_ok());
    }

    #[test]
    fn test_validate_specification() {
        let spec = create_test_spec();
        assert!(validate_specification(&spec).is_ok());
    }

    #[test]
    fn test_validate_task() {
        let task = create_test_task();
        assert!(validate_task(&task).is_ok());
    }

    #[test]
    fn test_validate_project_name_availability() {
        let existing = vec!["project1".to_string(), "project2".to_string()];

        assert!(validate_project_name_availability("new_project", &existing).is_ok());
        assert!(validate_project_name_availability("project1", &existing).is_err());
    }

    #[test]
    fn test_validate_file_path_safety() {
        assert!(validate_file_path_safety("valid/path").is_ok());
        assert!(validate_file_path_safety("../dangerous").is_err());
        assert!(validate_file_path_safety("/absolute/path").is_err());
        assert!(validate_file_path_safety("path\0with\0nulls").is_err());
    }
}
