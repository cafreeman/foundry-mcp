//! Enhanced validation utilities with detailed error reporting and security checks

use crate::models::{Project, Specification, Task, TaskList, TechStack, Vision};
use chrono::Utc;
use std::collections::HashMap;

/// Validation error with context and suggestions
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub value: String,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    Required,
    TooLong,
    TooShort,
    InvalidFormat,
    InvalidCharacters,
    AlreadyExists,
    SecurityRisk,
    InvalidRange,
    InvalidTimestamp,
}

impl ValidationError {
    pub fn new(field: &str, value: &str, error_type: ValidationErrorType, message: &str) -> Self {
        Self {
            field: field.to_string(),
            value: value.to_string(),
            error_type,
            message: message.to_string(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn format_error(&self) -> String {
        let mut error_msg = format!("Field '{}': {}", self.field, self.message);

        if let Some(suggestion) = &self.suggestion {
            error_msg.push_str(&format!("\nSuggestion: {}", suggestion));
        }

        error_msg
    }
}

/// Validation result with detailed errors
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Enhanced validation context
pub struct ValidationContext {
    pub existing_projects: Vec<String>,
    pub existing_specs: HashMap<String, Vec<String>>, // project_name -> spec_names
    pub strict_mode: bool,
    pub max_content_length: usize,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            existing_projects: Vec::new(),
            existing_specs: HashMap::new(),
            strict_mode: false,
            max_content_length: 50_000,
        }
    }
}

/// Enhanced project validation with detailed error reporting
pub fn validate_project_enhanced(
    project: &Project,
    context: &ValidationContext,
) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate project name
    if project.name.trim().is_empty() {
        errors.push(
            ValidationError::new(
                "name",
                &project.name,
                ValidationErrorType::Required,
                "Project name is required"
            ).with_suggestion("Provide a descriptive name in lowercase with hyphens or underscores, e.g., 'my-awesome-project'")
        );
    } else {
        // Check length
        if project.name.len() > 100 {
            errors.push(
                ValidationError::new(
                    "name",
                    &project.name,
                    ValidationErrorType::TooLong,
                    "Project name cannot exceed 100 characters",
                )
                .with_suggestion("Use a shorter, more concise name"),
            );
        }

        // Check format
        if !project
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            errors.push(
                ValidationError::new(
                    "name",
                    &project.name,
                    ValidationErrorType::InvalidCharacters,
                    "Project name can only contain letters, numbers, hyphens, and underscores",
                )
                .with_suggestion("Replace special characters with hyphens or underscores"),
            );
        }

        // Check for uppercase letters
        if project.name.chars().any(|c| c.is_uppercase()) {
            errors.push(
                ValidationError::new(
                    "name",
                    &project.name,
                    ValidationErrorType::InvalidFormat,
                    "Project name should be lowercase",
                )
                .with_suggestion("Convert the name to lowercase"),
            );
        }

        // Check availability
        if context.existing_projects.contains(&project.name) {
            errors.push(
                ValidationError::new(
                    "name",
                    &project.name,
                    ValidationErrorType::AlreadyExists,
                    "Project name already exists",
                )
                .with_suggestion("Choose a different project name or load the existing project"),
            );
        }

        // Security check
        if let Err(security_error) = validate_input_security(&project.name) {
            errors.push(
                ValidationError::new(
                    "name",
                    &project.name,
                    ValidationErrorType::SecurityRisk,
                    &security_error,
                )
                .with_suggestion("Use only safe characters in project names"),
            );
        }
    }

    // Validate description
    if project.description.trim().is_empty() {
        errors.push(
            ValidationError::new(
                "description",
                &project.description,
                ValidationErrorType::Required,
                "Project description is required",
            )
            .with_suggestion("Provide a brief description of what this project does"),
        );
    } else if project.description.len() > 2000 {
        errors.push(
            ValidationError::new(
                "description",
                &project.description,
                ValidationErrorType::TooLong,
                "Project description cannot exceed 2000 characters",
            )
            .with_suggestion(
                "Shorten the description or move detailed information to the vision overview",
            ),
        );
    }

    // Security check for description
    if let Err(security_error) = validate_input_security(&project.description) {
        errors.push(
            ValidationError::new(
                "description",
                &project.description,
                ValidationErrorType::SecurityRisk,
                &security_error,
            )
            .with_suggestion("Remove any potentially dangerous content from the description"),
        );
    }

    // Validate timestamps
    if project.created_at > project.updated_at {
        errors.push(
            ValidationError::new(
                "timestamps",
                &format!(
                    "created: {}, updated: {}",
                    project.created_at, project.updated_at
                ),
                ValidationErrorType::InvalidTimestamp,
                "Created timestamp cannot be after updated timestamp",
            )
            .with_suggestion(
                "Ensure the updated timestamp is equal to or after the created timestamp",
            ),
        );
    }

    // Validate tech stack and vision with enhanced validation
    if let Err(tech_stack_errors) = validate_tech_stack_enhanced(&project.tech_stack, context) {
        errors.extend(tech_stack_errors);
    }

    if let Err(vision_errors) = validate_vision_enhanced(&project.vision, context) {
        errors.extend(vision_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Enhanced tech stack validation
pub fn validate_tech_stack_enhanced(
    tech_stack: &TechStack,
    context: &ValidationContext,
) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate languages
    if tech_stack.languages.is_empty() && context.strict_mode {
        errors.push(
            ValidationError::new(
                "languages",
                "",
                ValidationErrorType::Required,
                "At least one programming language should be specified",
            )
            .with_suggestion("Add the main programming language(s) for this project"),
        );
    }

    for (i, language) in tech_stack.languages.iter().enumerate() {
        let field_name = format!("languages[{}]", i);
        if language.trim().is_empty() {
            errors.push(
                ValidationError::new(
                    &field_name,
                    language,
                    ValidationErrorType::Required,
                    "Language name cannot be empty",
                )
                .with_suggestion("Remove empty entries or provide a valid language name"),
            );
        } else if language.len() > 50 {
            errors.push(
                ValidationError::new(
                    &field_name,
                    language,
                    ValidationErrorType::TooLong,
                    "Language name cannot exceed 50 characters",
                )
                .with_suggestion("Use standard language names like 'JavaScript', 'Python', 'Rust'"),
            );
        } else if let Err(security_error) = validate_input_security(language) {
            errors.push(
                ValidationError::new(
                    &field_name,
                    language,
                    ValidationErrorType::SecurityRisk,
                    &security_error,
                )
                .with_suggestion("Use only standard programming language names"),
            );
        }
    }

    // Similar validation for frameworks, databases, tools, and deployment
    validate_string_list(
        &tech_stack.frameworks,
        "frameworks",
        100,
        &mut errors,
        context,
    );
    validate_string_list(
        &tech_stack.databases,
        "databases",
        100,
        &mut errors,
        context,
    );
    validate_string_list(&tech_stack.tools, "tools", 100, &mut errors, context);
    validate_string_list(
        &tech_stack.deployment,
        "deployment",
        100,
        &mut errors,
        context,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Enhanced vision validation
pub fn validate_vision_enhanced(vision: &Vision, context: &ValidationContext) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate overview
    if vision.overview.trim().is_empty() {
        errors.push(
            ValidationError::new(
                "overview",
                &vision.overview,
                ValidationErrorType::Required,
                "Vision overview is required",
            )
            .with_suggestion("Provide a clear overview of the project's purpose and scope"),
        );
    } else if vision.overview.len() > 5000 {
        errors.push(
            ValidationError::new(
                "overview",
                &vision.overview,
                ValidationErrorType::TooLong,
                "Vision overview cannot exceed 5000 characters",
            )
            .with_suggestion(
                "Keep the overview concise and move detailed information to specifications",
            ),
        );
    }

    // Security check
    if let Err(security_error) = validate_input_security(&vision.overview) {
        errors.push(
            ValidationError::new(
                "overview",
                &vision.overview,
                ValidationErrorType::SecurityRisk,
                &security_error,
            )
            .with_suggestion("Remove any potentially dangerous content from the overview"),
        );
    }

    // Validate goals, target_users, and success_criteria
    validate_string_list(&vision.goals, "goals", 500, &mut errors, context);
    validate_string_list(
        &vision.target_users,
        "target_users",
        200,
        &mut errors,
        context,
    );
    validate_string_list(
        &vision.success_criteria,
        "success_criteria",
        500,
        &mut errors,
        context,
    );

    // Check minimum requirements in strict mode
    if context.strict_mode {
        if vision.goals.is_empty() {
            errors.push(
                ValidationError::new(
                    "goals",
                    "",
                    ValidationErrorType::Required,
                    "At least one goal must be specified",
                )
                .with_suggestion("Define clear, measurable goals for the project"),
            );
        }

        if vision.target_users.is_empty() {
            errors.push(
                ValidationError::new(
                    "target_users",
                    "",
                    ValidationErrorType::Required,
                    "At least one target user must be specified",
                )
                .with_suggestion("Identify who will use or benefit from this project"),
            );
        }

        if vision.success_criteria.is_empty() {
            errors.push(
                ValidationError::new(
                    "success_criteria",
                    "",
                    ValidationErrorType::Required,
                    "At least one success criterion must be specified",
                )
                .with_suggestion("Define how you will measure the success of this project"),
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Helper function to validate string lists with consistent error handling
fn validate_string_list(
    list: &[String],
    field_name: &str,
    max_length: usize,
    errors: &mut Vec<ValidationError>,
    _context: &ValidationContext,
) {
    for (i, item) in list.iter().enumerate() {
        let item_field = format!("{}[{}]", field_name, i);

        if item.trim().is_empty() {
            errors.push(
                ValidationError::new(
                    &item_field,
                    item,
                    ValidationErrorType::Required,
                    &format!("{} item cannot be empty", field_name),
                )
                .with_suggestion("Remove empty entries or provide valid content"),
            );
        } else if item.len() > max_length {
            errors.push(
                ValidationError::new(
                    &item_field,
                    item,
                    ValidationErrorType::TooLong,
                    &format!(
                        "{} item cannot exceed {} characters",
                        field_name, max_length
                    ),
                )
                .with_suggestion("Use shorter, more concise descriptions"),
            );
        } else if let Err(security_error) = validate_input_security(item) {
            errors.push(
                ValidationError::new(
                    &item_field,
                    item,
                    ValidationErrorType::SecurityRisk,
                    &security_error,
                )
                .with_suggestion("Remove any potentially dangerous content"),
            );
        }
    }
}

/// Security validation for input strings
pub fn validate_input_security(input: &str) -> Result<(), String> {
    // Check for script injection attempts
    let dangerous_patterns = [
        "<script",
        "</script>",
        "javascript:",
        "data:",
        "vbscript:",
        "onload=",
        "onerror=",
        "onclick=",
        "onmouseover=",
        "eval(",
        "setTimeout(",
        "setInterval(",
        "document.cookie",
        "document.location",
        "window.location",
        "innerHTML",
        "outerHTML",
        "document.write",
        "exec(",
        "system(",
        "shell_exec(",
        "passthru(",
        "file_get_contents(",
        "file_put_contents(",
        "fopen(",
        "include(",
        "include_once(",
        "require(",
        "require_once(",
        "<?php",
        "<?=",
        "<%",
        "%>",
        "<%=",
        "DROP TABLE",
        "DELETE FROM",
        "INSERT INTO",
        "UPDATE SET",
        "UNION SELECT",
        "OR 1=1",
        "' OR '1'='1",
        "\" OR \"1\"=\"1",
        "../",
        "..\\",
        "/etc/",
        "/var/",
        "/usr/",
        "/bin/",
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Users",
    ];

    let input_lower = input.to_lowercase();
    for pattern in &dangerous_patterns {
        if input_lower.contains(&pattern.to_lowercase()) {
            return Err(format!(
                "Input contains potentially dangerous pattern: {}",
                pattern
            ));
        }
    }

    // Check for unusual Unicode characters that could be used for attacks
    if input
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
    {
        return Err("Input contains control characters".to_string());
    }

    // Check for excessively long lines (potential DoS)
    if input.lines().any(|line| line.len() > 10000) {
        return Err("Input contains excessively long lines".to_string());
    }

    Ok(())
}

/// Backward compatibility function
pub fn validate_project(project: &Project) -> Result<(), Vec<String>> {
    let context = ValidationContext::default();
    match validate_project_enhanced(project, &context) {
        Ok(()) => Ok(()),
        Err(errors) => Err(errors.into_iter().map(|e| e.format_error()).collect()),
    }
}

/// Validate a tech stack structure
pub fn validate_tech_stack(tech_stack: &TechStack) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate languages
    errors.extend(
        tech_stack
            .languages
            .iter()
            .enumerate()
            .filter_map(|(i, language)| {
                if language.trim().is_empty() {
                    Some(format!("Language {} cannot be empty", i + 1))
                } else if language.len() > 50 {
                    Some(format!(
                        "Language '{}' cannot exceed 50 characters",
                        language
                    ))
                } else {
                    None
                }
            }),
    );

    // Validate frameworks
    errors.extend(
        tech_stack
            .frameworks
            .iter()
            .enumerate()
            .filter_map(|(i, framework)| {
                if framework.trim().is_empty() {
                    Some(format!("Framework {} cannot be empty", i + 1))
                } else if framework.len() > 100 {
                    Some(format!(
                        "Framework '{}' cannot exceed 100 characters",
                        framework
                    ))
                } else {
                    None
                }
            }),
    );

    // Validate databases
    errors.extend(
        tech_stack
            .databases
            .iter()
            .enumerate()
            .filter_map(|(i, database)| {
                if database.trim().is_empty() {
                    Some(format!("Database {} cannot be empty", i + 1))
                } else if database.len() > 100 {
                    Some(format!(
                        "Database '{}' cannot exceed 100 characters",
                        database
                    ))
                } else {
                    None
                }
            }),
    );

    // Validate tools
    errors.extend(tech_stack.tools.iter().enumerate().filter_map(|(i, tool)| {
        if tool.trim().is_empty() {
            Some(format!("Tool {} cannot be empty", i + 1))
        } else if tool.len() > 100 {
            Some(format!("Tool '{}' cannot exceed 100 characters", tool))
        } else {
            None
        }
    }));

    // Validate deployment
    errors.extend(
        tech_stack
            .deployment
            .iter()
            .enumerate()
            .filter_map(|(i, deployment)| {
                if deployment.trim().is_empty() {
                    Some(format!("Deployment {} cannot be empty", i + 1))
                } else if deployment.len() > 100 {
                    Some(format!(
                        "Deployment '{}' cannot exceed 100 characters",
                        deployment
                    ))
                } else {
                    None
                }
            }),
    );

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

    errors.extend(vision.goals.iter().enumerate().filter_map(|(i, goal)| {
        if goal.trim().is_empty() {
            Some(format!("Goal {} cannot be empty", i + 1))
        } else if goal.len() > 500 {
            Some(format!("Goal '{}' cannot exceed 500 characters", goal))
        } else {
            None
        }
    }));

    // Validate target users
    if vision.target_users.is_empty() {
        errors.push("At least one target user must be specified".to_string());
    }

    errors.extend(
        vision
            .target_users
            .iter()
            .enumerate()
            .filter_map(|(i, user)| {
                if user.trim().is_empty() {
                    Some(format!("Target user {} cannot be empty", i + 1))
                } else if user.len() > 200 {
                    Some(format!(
                        "Target user '{}' cannot exceed 200 characters",
                        user
                    ))
                } else {
                    None
                }
            }),
    );

    // Validate success criteria
    if vision.success_criteria.is_empty() {
        errors.push("At least one success criterion must be specified".to_string());
    }

    errors.extend(
        vision
            .success_criteria
            .iter()
            .enumerate()
            .filter_map(|(i, criterion)| {
                if criterion.trim().is_empty() {
                    Some(format!("Success criterion {} cannot be empty", i + 1))
                } else if criterion.len() > 500 {
                    Some(format!(
                        "Success criterion '{}' cannot exceed 500 characters",
                        criterion
                    ))
                } else {
                    None
                }
            }),
    );

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
    errors.extend(
        task.dependencies
            .iter()
            .enumerate()
            .filter_map(|(i, dependency)| {
                if dependency.trim().is_empty() {
                    Some(format!("Dependency {} cannot be empty", i + 1))
                } else if dependency.len() > 100 {
                    Some(format!(
                        "Dependency '{}' cannot exceed 100 characters",
                        dependency
                    ))
                } else {
                    None
                }
            }),
    );

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
    errors.extend(task_list.tasks.iter().enumerate().flat_map(|(i, task)| {
        validate_task(task)
            .map(|_| Vec::<String>::new())
            .unwrap_or_else(|task_errors| {
                task_errors
                    .into_iter()
                    .map(|task_error| format!("Task {}: {}", i + 1, task_error))
                    .collect::<Vec<_>>()
            })
    }));

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
    if dangerous_patterns
        .iter()
        .any(|pattern| path.to_lowercase().contains(pattern))
    {
        return Err(format!(
            "Path contains potentially dangerous pattern: {}",
            dangerous_patterns
                .iter()
                .find(|pattern| path.to_lowercase().contains(*pattern))
                .unwrap()
        ));
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
