//! Core op for creating a project (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::{foundry, validation};
use crate::types::project::ProjectConfig;
use crate::types::responses::{CreateProjectResponse, FoundryResponse};
use crate::utils::response::{build_incomplete_response, build_success_response};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<CreateProjectResponse>> {
    let foundry = foundry::get_default_foundry()?;
    
    validate_project_preconditions(&foundry, &input.project_name).await?;

    let suggestions = process_content_validation(&input)?;

    let project_config = build_project_config(input);
    let created_project = foundry
        .create_project(project_config)
        .await
        .context("Failed to create project structure")?;

    Ok(build_response(created_project, suggestions))
}

async fn validate_project_preconditions(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<()> {
    validate_project_name(project_name)?;

    if foundry.project_exists(project_name).await? {
        return Err(anyhow::anyhow!("Project '{}' already exists", project_name));
    }

    Ok(())
}

fn process_content_validation(input: &Input) -> Result<Vec<String>> {
    let validation_results = validate_content(input)?;

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

    if !validation_errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Content validation failed:\n{}",
            validation_errors.join("\n")
        ));
    }

    Ok(suggestions)
}

fn build_project_config(input: Input) -> ProjectConfig {
    ProjectConfig {
        name: input.project_name,
        vision: input.vision,
        tech_stack: input.tech_stack,
        summary: input.summary,
    }
}

fn build_response(
    created_project: crate::types::project::Project,
    suggestions: Vec<String>,
) -> FoundryResponse<CreateProjectResponse> {
    let files_created = vec![
        "vision.md".to_string(),
        "tech-stack.md".to_string(),
        "summary.md".to_string(),
        "specs/".to_string(),
    ];

    let response_data = CreateProjectResponse {
        project_name: created_project.name.clone(),
        created_at: created_project.created_at,
        project_path: created_project.path.to_string_lossy().to_string(),
        files_created,
    };

    let next_steps = vec![
        format!("Project '{}' created successfully", created_project.name),
        "Project structure is ready for development".to_string(),
        format!(
            "Next → create a spec: {{\"name\": \"create_spec\", \"arguments\": {{\"project_name\": \"{}\", \"feature_name\": \"<feature>\", \"spec\": \"...\", \"tasks\": \"...\", \"notes\": \"...\"}}}}; load project: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}; list projects: {{\"name\": \"list_projects\", \"arguments\": {{}}}}",
            created_project.name, created_project.name
        ),
    ];

    let workflow_hints = if !suggestions.is_empty() {
        let mut enhanced_suggestions = vec![
            "📋 DOCUMENT PURPOSE: Your content serves as COMPLETE CONTEXT for future implementation".to_string(),
            "🎯 CONTEXT TEST: Could someone with no prior knowledge implement this using only your documents?".to_string(),
        ];
        enhanced_suggestions.extend(suggestions.clone());
        enhanced_suggestions
    } else {
        vec![
            "📋 DOCUMENT PURPOSE: Your content serves as COMPLETE CONTEXT for future implementation".to_string(),
            "🎯 CONTEXT TEST: Could someone with no prior knowledge implement this using only your documents?".to_string(),
            "Consider what you want to work on next".to_string(),
            // Guidance preserved from previous implementation
            // Create spec / Load project / Help
            // These strings are intentionally identical to avoid behavior drift
            // during the refactor.
            //
            // clippy: allow identical strings — intentional UX
            format!("Create a spec: {{\"name\": \"create_spec\", \"arguments\": {{\"project_name\": \"{}\", \"feature_name\": \"<feature>\", \"spec\": \"...\", \"tasks\": \"...\", \"notes\": \"...\"}}}}", created_project.name),
            format!("Load project: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}", created_project.name),
            "Tool selection guidance: {\"name\": \"get_foundry_help\", {\"topic\": \"decision-points\"}}".to_string(),
        ]
    };

    if suggestions.is_empty() {
        build_success_response(response_data, next_steps, workflow_hints)
    } else {
        build_incomplete_response(response_data, next_steps, workflow_hints)
    }
}

fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow::anyhow!(
            "Project name must be in kebab-case format (lowercase letters, numbers, and hyphens only)"
        ));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow::anyhow!(
            "Project name cannot start or end with a hyphen"
        ));
    }

    if name.contains("--") {
        return Err(anyhow::anyhow!(
            "Project name cannot contain consecutive hyphens"
        ));
    }

    Ok(())
}

fn validate_content(input: &Input) -> Result<Vec<(&'static str, validation::ValidationResult)>> {
    let validations = vec![
        (
            "Vision",
            validation::validate_content(validation::ContentType::Vision, &input.vision),
        ),
        (
            "Tech Stack",
            validation::validate_content(validation::ContentType::TechStack, &input.tech_stack),
        ),
        (
            "Summary",
            validation::validate_content(validation::ContentType::Summary, &input.summary),
        ),
    ];

    Ok(validations)
}
