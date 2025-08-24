//! Implementation of the create_project command

use crate::cli::args::CreateProjectArgs;
use crate::core::{project, validation};
use crate::types::project::ProjectConfig;
use crate::types::responses::{CreateProjectResponse, FoundryResponse};
use crate::utils::response::{build_incomplete_response, build_success_response};
use anyhow::{Context, Result};

pub async fn execute(args: CreateProjectArgs) -> Result<FoundryResponse<CreateProjectResponse>> {
    // Validate project preconditions
    validate_project_preconditions(&args.project_name)?;

    // Validate and process content
    let suggestions = process_content_validation(&args)?;

    // Create the project
    let project_config = build_project_config(args);
    let created_project =
        project::create_project(project_config).context("Failed to create project structure")?;

    // Build and return response
    Ok(build_response(created_project, suggestions))
}

/// Validate project preconditions (name format and existence)
fn validate_project_preconditions(project_name: &str) -> Result<()> {
    validate_project_name(project_name)?;

    if project::project_exists(project_name)? {
        return Err(anyhow::anyhow!("Project '{}' already exists", project_name));
    }

    Ok(())
}

/// Process content validation and return suggestions
fn process_content_validation(args: &CreateProjectArgs) -> Result<Vec<String>> {
    let validation_results = validate_content(args)?;

    let (validation_errors, suggestions): (Vec<String>, Vec<String>) =
        validation_results.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut errors, mut suggestions), (content_type, result)| {
                if !result.is_valid {
                    errors.extend(
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
                (errors, suggestions)
            },
        );

    // If there are validation errors, return them
    if !validation_errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Content validation failed:\n{}",
            validation_errors.join("\n")
        ));
    }

    Ok(suggestions)
}

/// Build project configuration from args
fn build_project_config(args: CreateProjectArgs) -> ProjectConfig {
    ProjectConfig {
        name: args.project_name,
        vision: args.vision,
        tech_stack: args.tech_stack,
        summary: args.summary,
    }
}

/// Build the final response
fn build_response(
    created_project: crate::types::project::Project,
    suggestions: Vec<String>,
) -> FoundryResponse<CreateProjectResponse> {
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

    let next_steps = vec![
        format!("Project '{}' created successfully", created_project.name),
        "You can now create specifications using: foundry create_spec".to_string(),
        "View your project with: foundry list_projects".to_string(),
    ];

    let workflow_hints = if !suggestions.is_empty() {
        suggestions.clone()
    } else {
        vec![
            "Consider creating your first specification to start development".to_string(),
            "Use 'foundry get_foundry_help workflows' for workflow guidance".to_string(),
        ]
    };

    if suggestions.is_empty() {
        build_success_response(response_data, next_steps, workflow_hints)
    } else {
        build_incomplete_response(response_data, next_steps, workflow_hints)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Mock project args for testing
    fn create_test_args() -> CreateProjectArgs {
        CreateProjectArgs {
            project_name: "test-project".to_string(),
            vision: "This is a test vision that is long enough to meet the minimum requirements. It should contain at least 200 characters to pass validation. This includes multiple sentences and provides comprehensive coverage of what the project aims to achieve.".to_string(),
            tech_stack: "This project uses Rust as the primary language with tokio for async runtime, serde for serialization, and clap for command line argument parsing. It follows functional programming principles and modern Rust 2024 practices.".to_string(),
            summary: "A comprehensive CLI tool for deterministic project management and AI coding assistant integration with modern Rust patterns.".to_string(),
        }
    }

    #[test]
    fn test_validate_project_name_valid() {
        let valid_names = vec!["my-project", "project123", "my-awesome-project", "test"];

        for name in valid_names {
            assert!(
                validate_project_name(name).is_ok(),
                "Name '{}' should be valid",
                name
            );
        }
    }

    #[test]
    fn test_validate_project_name_invalid() {
        let invalid_names = vec![
            "",            // empty
            "-project",    // starts with hyphen
            "project-",    // ends with hyphen
            "my--project", // consecutive hyphens
            "MyProject",   // uppercase
            "my project",  // space
            "my.project",  // invalid character
        ];

        for name in invalid_names {
            assert!(
                validate_project_name(name).is_err(),
                "Name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_validate_content_structure() {
        let args = create_test_args();
        let validations = validate_content(&args).unwrap();

        assert_eq!(validations.len(), 3);

        // Check that all content types are present
        let content_types: Vec<&str> = validations.iter().map(|(t, _)| *t).collect();
        assert!(content_types.contains(&"Vision"));
        assert!(content_types.contains(&"Tech Stack"));
        assert!(content_types.contains(&"Summary"));
    }

    #[test]
    fn test_build_project_config() {
        let args = create_test_args();
        let config = build_project_config(args);

        assert_eq!(config.name, "test-project");
        assert!(!config.vision.is_empty());
        assert!(!config.tech_stack.is_empty());
        assert!(!config.summary.is_empty());
    }

    #[test]
    fn test_build_response_without_suggestions() {
        let project = crate::types::project::Project {
            name: "test-project".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            path: std::path::PathBuf::from("/tmp/test"),
            vision: Some("test vision".to_string()),
            tech_stack: Some("test tech".to_string()),
            summary: Some("test summary".to_string()),
        };

        let suggestions = Vec::new();
        let response = build_response(project, suggestions);

        // Should be a success response (no suggestions)
        assert!(matches!(
            response.validation_status,
            crate::types::responses::ValidationStatus::Complete
        ));
        assert_eq!(response.data.project_name, "test-project");
        assert_eq!(response.data.files_created.len(), 4);
        assert!(
            response
                .next_steps
                .iter()
                .any(|s| s.contains("created successfully"))
        );
    }

    #[test]
    fn test_build_response_with_suggestions() {
        let project = crate::types::project::Project {
            name: "test-project".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            path: std::path::PathBuf::from("/tmp/test"),
            vision: Some("test vision".to_string()),
            tech_stack: Some("test tech".to_string()),
            summary: Some("test summary".to_string()),
        };

        let suggestions = vec!["Add more detail".to_string()];
        let response = build_response(project, suggestions);

        // Should be an incomplete response (has suggestions)
        assert!(matches!(
            response.validation_status,
            crate::types::responses::ValidationStatus::Incomplete
        ));
        assert_eq!(response.workflow_hints, vec!["Add more detail"]);
    }
}
