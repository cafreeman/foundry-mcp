//! Implementation of the list_projects command

use crate::cli::args::ListProjectsArgs;
use crate::core::project;
use crate::types::responses::{FoundryResponse, ListProjectsResponse, ProjectInfo};
use crate::utils::formatting::format_count;
use crate::utils::response::build_success_response;
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
                "No projects found in foundry directory - ready for project creation".to_string(),
                "You can create your first project with: foundry mcp create-project".to_string(),
            ],
            vec![
                "You can use 'foundry mcp get-foundry-help workflows' for getting started guidance"
                    .to_string(),
                "Projects are stored in ~/.foundry/ directory for easy access".to_string(),
            ],
        )
    } else {
        let project_count = response_data.projects.len();
        (
            vec![
                format_count(project_count, "project", "projects"),
                "You can use 'foundry mcp create-spec <project_name> <feature_name>' to add specifications"
                    .to_string(),
                "You can use 'foundry mcp load-spec <project_name>' to view existing specifications"
                    .to_string(),
            ],
            vec![
                "Each project can contain multiple timestamped specifications for organized development".to_string(),
                "You can use 'foundry mcp analyze-project' to add project analysis to existing codebases"
                    .to_string(),
            ],
        )
    };

    Ok(build_success_response(
        response_data,
        next_steps,
        workflow_hints,
    ))
}
