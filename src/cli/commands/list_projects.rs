//! Implementation of the list_projects command

use crate::cli::args::ListProjectsArgs;
use crate::core::project;
use crate::types::responses::{
    FoundryResponse, ListProjectsResponse, ProjectInfo, ValidationStatus,
};
use anyhow::{Context, Result};

pub async fn execute(_args: ListProjectsArgs) -> Result<FoundryResponse<ListProjectsResponse>> {
    // Get all projects from the foundry directory
    let project_metadata_list =
        project::list_projects().context("Failed to list projects from foundry directory")?;

    // Convert to response format
    let projects: Vec<ProjectInfo> = project_metadata_list
        .into_iter()
        .map(|metadata| {
            let project_path = project::get_project_path(&metadata.name)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "Unknown".to_string());

            ProjectInfo {
                name: metadata.name,
                created_at: metadata.created_at,
                spec_count: metadata.spec_count,
                path: project_path,
            }
        })
        .collect();

    let response_data = ListProjectsResponse { projects };

    // Generate appropriate response based on project count
    let (next_steps, workflow_hints) = if response_data.projects.is_empty() {
        (
            vec![
                "No projects found in foundry directory".to_string(),
                "Create your first project with: foundry create-project".to_string(),
            ],
            vec![
                "Use 'foundry get_foundry_help workflows' for getting started guidance".to_string(),
                "Projects are stored in ~/.foundry/ directory".to_string(),
            ],
        )
    } else {
        let project_count = response_data.projects.len();
        (
            vec![
                format!("Found {} project(s)", project_count),
                "Use 'foundry create-spec <project_name> <feature_name>' to add specifications"
                    .to_string(),
                "Use 'foundry load-spec <project_name>' to view existing specifications"
                    .to_string(),
            ],
            vec![
                "Each project can contain multiple timestamped specifications".to_string(),
                "Use 'foundry analyze-project' to add project analysis to existing codebases"
                    .to_string(),
            ],
        )
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    })
}
