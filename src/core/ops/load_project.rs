//! Core op for loading full project context (tool-agnostic)

use anyhow::Result;

use crate::core::foundry;
use crate::types::responses::{
    FoundryResponse, LoadProjectResponse, ProjectContext, ValidationStatus,
};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<LoadProjectResponse>> {
    let foundry = foundry::get_default_foundry()?;

    validate_project_exists(&foundry, &input.project_name).await?;

    let project = foundry.load_project(&input.project_name).await?;
    let specs = foundry.list_specs(&input.project_name).await?;

    let project_context = build_project_context(project, specs);
    let specs_available = project_context.specs_available.clone();

    let response_data = LoadProjectResponse {
        project: project_context,
    };

    let validation_status = if specs_available.is_empty() {
        ValidationStatus::Incomplete
    } else {
        ValidationStatus::Complete
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&input.project_name, &specs_available),
        validation_status,
        workflow_hints: generate_workflow_hints(&specs_available),
    })
}

async fn validate_project_exists(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<()> {
    if !foundry.project_exists(project_name).await? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'mcp_foundry_list_projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

fn build_project_context(
    project: crate::types::project::Project,
    specs: Vec<crate::types::spec::SpecMetadata>,
) -> ProjectContext {
    let specs_available = specs.into_iter().map(|s| s.name).collect();

    ProjectContext {
        name: project.name,
        vision: project.vision.unwrap_or_default(),
        tech_stack: project.tech_stack.unwrap_or_default(),
        summary: project.summary.unwrap_or_default(),
        specs_available,
        created_at: project.created_at,
    }
}

fn generate_next_steps(project_name: &str, specs_available: &[String]) -> Vec<String> {
    if specs_available.is_empty() {
        vec![
            "Project context loaded successfully - ready for specification creation".to_string(),
            format!(
                "You can create your first specification: mcp_foundry_create_spec {} <feature_name>",
                project_name
            ),
            "Your loaded project context provides comprehensive background for development decisions".to_string(),
        ]
    } else {
        vec![
            format!(
                "Project context loaded with {} specification(s) available",
                specs_available.len()
            ),
            format!(
                "You can load a specific spec: mcp_foundry_load_spec {} <spec_name>",
                project_name
            ),
            format!(
                "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
                project_name
            ),
        ]
    }
}

fn generate_workflow_hints(specs_available: &[String]) -> Vec<String> {
    let mut hints = vec![
        "You can use the project summary for quick context in conversations".to_string(),
        "The full vision provides comprehensive background and goals for your work".to_string(),
        "Tech stack details guide your implementation decisions and technology choices".to_string(),
        "You can skip list-projects calls when you know the project name - load_project is more efficient".to_string(),
    ];

    if specs_available.is_empty() {
        hints.push(
            "You can create specifications to track specific features as you identify them"
                .to_string(),
        );
        hints.push(
            "You can prompt the user about creating specifications to track specific features"
                .to_string(),
        );
    } else {
        hints.push(format!("Available specs: {}", specs_available.join(", ")));
        hints.push(
            "You can load individual specs to see detailed implementation plans and progress"
                .to_string(),
        );
        hints.push("You can update existing specs with progress as work continues".to_string());
    }

    hints.push(
        "You can use mcp_foundry_get_foundry_help decision-points to understand tool selection"
            .to_string(),
    );

    hints
}
