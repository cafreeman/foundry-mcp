//! Core op for creating a spec (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::{foundry, validation};
use crate::types::responses::{CreateSpecResponse, FoundryResponse, ValidationStatus};
use crate::types::spec::{SpecConfig, SpecContentData};
use crate::utils::paths;

/// Input for create_spec operation (decoupled from interface-specific args)
#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub feature_name: String,
    pub spec: String,
    pub notes: String,
    pub tasks: String,
}

/// Execute the create_spec operation and return a structured response
pub async fn run(input: Input) -> Result<FoundryResponse<CreateSpecResponse>> {
    let foundry = foundry::get_default_foundry()?;

    // Validate project exists
    validate_project_exists(&foundry, &input.project_name).await?;

    // Validate feature name
    validate_feature_name(&input.feature_name)?;

    // Validate content
    let content_validation = validate_content(&input)?;
    let has_validation_warnings = content_validation
        .iter()
        .any(|(_, result)| !result.is_valid);

    // Create the spec
    let spec_config = build_spec_config(input);
    let created_spec = foundry
        .create_spec(spec_config)
        .await
        .context("Failed to create specification")?;

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
async fn validate_project_exists(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<()> {
    if !foundry.project_exists(project_name).await? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use list_projects via MCP to see available projects: {{\"name\": \"list_projects\", \"arguments\": {{}}}}",
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
fn validate_content(input: &Input) -> Result<Vec<(&'static str, validation::ValidationResult)>> {
    let validations = vec![
        (
            "Spec Content",
            validation::validate_content(validation::ContentType::Spec, &input.spec),
        ),
        (
            "Implementation Notes",
            validation::validate_content(validation::ContentType::Notes, &input.notes),
        ),
        (
            "Task List",
            validation::validate_content(validation::ContentType::Tasks, &input.tasks),
        ),
    ];

    Ok(validations)
}

/// Build spec config from input
fn build_spec_config(input: Input) -> SpecConfig {
    SpecConfig {
        project_name: input.project_name,
        feature_name: input.feature_name,
        content: SpecContentData {
            spec: input.spec,
            notes: input.notes,
            tasks: input.tasks,
        },
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
            "Load spec: {{\"name\": \"load_spec\", \"arguments\": {{\"project_name\": \"{}\", \"spec_name\": \"{}\"}}}}; Load project: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}",
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
        "Review spec: {\"name\": \"load_spec\", \"arguments\": {\"project_name\": \"<project>\", \"spec_name\": \"<spec>\"}}".to_string(),
        "Load project: {\"name\": \"load_project\", \"arguments\": {\"project_name\": \"<project>\"}}".to_string(),
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

    hints.push("Tool selection guidance: {\"name\": \"get_foundry_help\", \"arguments\": {\"topic\": \"decision-points\"}}".to_string());

    hints
}
