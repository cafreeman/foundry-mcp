//! Implementation of the create_spec command

use crate::cli::args::CreateSpecArgs;
use crate::core::{project, spec, validation};
use crate::types::responses::{CreateSpecResponse, FoundryResponse, ValidationStatus};
use crate::types::spec::SpecConfig;
use crate::utils::paths;
use anyhow::{Context, Result};

pub async fn execute(args: CreateSpecArgs) -> Result<FoundryResponse<CreateSpecResponse>> {
    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Validate feature name
    validate_feature_name(&args.feature_name)?;

    // Validate content
    let content_validation = validate_content(&args)?;
    let has_validation_warnings = content_validation
        .iter()
        .any(|(_, result)| !result.is_valid);

    // Create the spec
    let spec_config = build_spec_config(args);
    let created_spec = spec::create_spec(spec_config).context("Failed to create specification")?;

    // Build response
    let response_data = CreateSpecResponse {
        project_name: created_spec.project_name.clone(),
        spec_name: created_spec.name.clone(),
        created_at: created_spec.created_at.clone(),
        spec_path: created_spec.path.to_string_lossy().to_string(),
        files_created: vec![
            format!("{}/spec.md", created_spec.name),
            format!("{}/notes.md", created_spec.name),
            format!("{}/task-list.md", created_spec.name),
        ],
    };

    let validation_status = if has_validation_warnings {
        ValidationStatus::Incomplete
    } else {
        ValidationStatus::Complete
    };

    let next_steps = generate_next_steps(&created_spec.project_name, &created_spec.name);
    let workflow_hints = generate_workflow_hints(&content_validation);

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status,
        workflow_hints,
    })
}

/// Validate that project exists
fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'foundry list-projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

/// Validate feature name format
fn validate_feature_name(feature_name: &str) -> Result<()> {
    paths::validate_feature_name(feature_name).context("Feature name validation failed")
}

/// Validate content according to schema requirements
fn validate_content(
    args: &CreateSpecArgs,
) -> Result<Vec<(&'static str, validation::ValidationResult)>> {
    let validations = vec![
        (
            "Spec Content",
            validation::validate_content(validation::ContentType::Spec, &args.spec),
        ),
        (
            "Implementation Notes",
            validation::validate_content(validation::ContentType::Notes, &args.notes),
        ),
        (
            "Task List",
            validation::validate_content(validation::ContentType::Tasks, &args.tasks),
        ),
    ];

    Ok(validations)
}

/// Build spec config from CLI arguments
fn build_spec_config(args: CreateSpecArgs) -> SpecConfig {
    SpecConfig {
        project_name: args.project_name,
        feature_name: args.feature_name,
        spec_content: args.spec,
        notes: args.notes,
        tasks: args.tasks,
    }
}

/// Generate next steps for the response
fn generate_next_steps(project_name: &str, spec_name: &str) -> Vec<String> {
    vec![
        format!(
            "Specification '{}' created successfully from your provided content",
            spec_name
        ),
        "Your specification content has been structured and is ready for implementation work"
            .to_string(),
        format!(
            "You can load the full spec: foundry load_spec {} {} (to review your content), foundry load_project {} (to see project context), or begin implementation",
            project_name, spec_name, project_name
        ),
    ]
}

/// Generate workflow hints based on validation results
fn generate_workflow_hints(
    validation_results: &[(&'static str, validation::ValidationResult)],
) -> Vec<String> {
    let mut hints = vec![
        "📋 DOCUMENT PURPOSE: Your spec content serves as COMPLETE CONTEXT for future implementation".to_string(),
        "🎯 CONTEXT TEST: Could someone with no prior knowledge implement this feature using only your spec documents?".to_string(),
        "Your specification content has been structured with task-list.md for implementation tracking".to_string(),
        "You can use foundry load_spec to review your full specification content and notes".to_string(),
        "You can use foundry load_project to see project context before implementation".to_string(),
    ];

    // Add validation-specific hints
    let invalid_content: Vec<&str> = validation_results
        .iter()
        .filter_map(|(name, result)| if !result.is_valid { Some(*name) } else { None })
        .collect();

    if !invalid_content.is_empty() {
        hints.push(format!(
            "You might consider reviewing content quality for: {}",
            invalid_content.join(", ")
        ));
    }

    hints.push(
        "You can use foundry get_foundry_help decision-points to understand tool options"
            .to_string(),
    );

    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create test arguments for spec creation
    fn create_test_spec_args() -> CreateSpecArgs {
        CreateSpecArgs {
            project_name: "test-project".to_string(),
            feature_name: "user_authentication".to_string(),
            spec: "Implement user authentication system with JWT tokens. Users should be able to register, login, logout, and reset passwords. The system should include email verification and role-based access control with proper security measures.".to_string(),
            notes: "Consider using bcrypt for password hashing. JWT tokens should expire after 24 hours. Need to implement rate limiting for login attempts and proper session management.".to_string(),
            tasks: "Create user registration endpoint, Implement password hashing with bcrypt, Add JWT token generation and validation, Create login/logout endpoints, Implement email verification system, Add role-based middleware for access control, Create password reset flow with email verification".to_string(),
        }
    }

    #[test]
    fn test_validate_feature_name_valid() {
        let valid_names = vec![
            "user_authentication",
            "api_endpoints",
            "database_schema",
            "test_feature",
            "feature123",
        ];

        for name in valid_names {
            assert!(
                validate_feature_name(name).is_ok(),
                "Feature name '{}' should be valid",
                name
            );
        }
    }

    #[test]
    fn test_validate_feature_name_invalid() {
        let invalid_names = vec![
            "",              // empty
            "Feature-Name",  // kebab-case instead of snake_case
            "featureName",   // camelCase
            "feature name",  // spaces
            "feature.name",  // dots
            "feature__name", // double underscores
            "_feature",      // starts with underscore
            "feature_",      // ends with underscore
            "FEATURE_NAME",  // all uppercase
        ];

        for name in invalid_names {
            assert!(
                validate_feature_name(name).is_err(),
                "Feature name '{}' should be invalid",
                name
            );
        }
    }

    #[test]
    fn test_validate_content_structure() {
        let args = create_test_spec_args();
        let validations = validate_content(&args).unwrap();

        assert_eq!(validations.len(), 3);

        // Check that all content types are present
        let content_types: Vec<&str> = validations.iter().map(|(t, _)| *t).collect();
        assert!(content_types.contains(&"Spec Content"));
        assert!(content_types.contains(&"Implementation Notes"));
        assert!(content_types.contains(&"Task List"));
    }

    #[test]
    fn test_validate_project_exists_missing_project() {
        // Test with a project that definitely doesn't exist
        let result = validate_project_exists("non-existent-project-12345");

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("not found"));
        assert!(error_message.contains("list-projects"));
    }
}
