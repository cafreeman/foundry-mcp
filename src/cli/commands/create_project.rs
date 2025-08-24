//! Implementation of the create_project command

use crate::cli::args::CreateProjectArgs;
use crate::core::{project, validation};
use crate::types::project::ProjectConfig;
use crate::types::responses::{CreateProjectResponse, FoundryResponse, ValidationStatus};
use anyhow::{Context, Result};

pub async fn execute(args: CreateProjectArgs) -> Result<FoundryResponse<CreateProjectResponse>> {
    // Validate project name format (kebab-case)
    validate_project_name(&args.project_name)?;

    // Check if project already exists
    if project::project_exists(&args.project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' already exists",
            args.project_name
        ));
    }

    // Validate content according to schema requirements
    let validation_results = validate_content(&args)?;
    let mut validation_errors = Vec::new();
    let mut suggestions = Vec::new();

    for (content_type, result) in validation_results {
        if !result.is_valid {
            validation_errors.extend(
                result
                    .errors
                    .into_iter()
                    .map(|e| format!("{}: {}", content_type, e)),
            );
        }
        suggestions.extend(
            result
                .suggestions
                .into_iter()
                .map(|s| format!("{}: {}", content_type, s)),
        );
    }

    // If there are validation errors, return them
    if !validation_errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Content validation failed:\n{}",
            validation_errors.join("\n")
        ));
    }

    // Create project configuration
    let project_config = ProjectConfig {
        name: args.project_name.clone(),
        vision: args.vision,
        tech_stack: args.tech_stack,
        summary: args.summary,
    };

    // Create the project
    let created_project =
        project::create_project(project_config).context("Failed to create project structure")?;

    // Build response
    let files_created = vec![
        "project/vision.md".to_string(),
        "project/tech-stack.md".to_string(),
        "project/summary.md".to_string(),
        "project/specs/".to_string(),
    ];

    let response_data = CreateProjectResponse {
        project_name: created_project.name.clone(),
        created_at: created_project.created_at,
        project_path: created_project.path.to_string_lossy().to_string(),
        files_created,
    };

    let validation_status = if suggestions.is_empty() {
        ValidationStatus::Complete
    } else {
        ValidationStatus::Incomplete
    };

    let next_steps = vec![
        format!("Project '{}' created successfully", created_project.name),
        "You can now create specifications using: foundry create_spec".to_string(),
        "View your project with: foundry list_projects".to_string(),
    ];

    let workflow_hints = if !suggestions.is_empty() {
        suggestions
    } else {
        vec![
            "Consider creating your first specification to start development".to_string(),
            "Use 'foundry get_foundry_help workflows' for workflow guidance".to_string(),
        ]
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status,
        workflow_hints,
    })
}

/// Validate project name format (kebab-case)
fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    // Check for kebab-case format
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow::anyhow!(
            "Project name must be in kebab-case format (lowercase letters, numbers, and hyphens only)"
        ));
    }

    // Can't start or end with hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow::anyhow!(
            "Project name cannot start or end with a hyphen"
        ));
    }

    // Can't have consecutive hyphens
    if name.contains("--") {
        return Err(anyhow::anyhow!(
            "Project name cannot contain consecutive hyphens"
        ));
    }

    Ok(())
}

/// Validate all content according to schema requirements
fn validate_content(
    args: &CreateProjectArgs,
) -> Result<Vec<(&'static str, validation::ValidationResult)>> {
    let validations = vec![
        (
            "Vision",
            validation::validate_content(validation::ContentType::Vision, &args.vision),
        ),
        (
            "Tech Stack",
            validation::validate_content(validation::ContentType::TechStack, &args.tech_stack),
        ),
        (
            "Summary",
            validation::validate_content(validation::ContentType::Summary, &args.summary),
        ),
    ];

    Ok(validations)
}
