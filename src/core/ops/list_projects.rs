//! Core op for listing projects (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::project;
use crate::types::responses::{FoundryResponse, ListProjectsResponse, ProjectInfo};
use crate::utils::formatting::format_count;
use crate::utils::response::build_success_response;

#[derive(Debug, Clone)]
pub struct Input;

pub async fn run(_input: Input) -> Result<FoundryResponse<ListProjectsResponse>> {
    let project_metadata_list =
        project::list_projects().context("Failed to list projects from foundry directory")?;

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

    let (next_steps, workflow_hints) = if response_data.projects.is_empty() {
        (
            vec![
                "No projects found in foundry directory - ready for project creation".to_string(),
                "You can create your first project with: mcp_foundry_create_project".to_string(),
            ],
            vec![
                "You can use 'mcp_foundry_get_foundry_help workflows' for getting started guidance"
                    .to_string(),
                "Projects are stored in ~/.foundry/ directory for easy access".to_string(),
            ],
        )
    } else {
        let project_count = response_data.projects.len();
        (
            vec![
                format_count(project_count, "project", "projects"),
                "You can use 'mcp_foundry_create_spec <project_name> <feature_name>' to add specifications".to_string(),
                "You can use 'mcp_foundry_load_spec <project_name>' to view existing specifications".to_string(),
            ],
            vec![
                "Each project can contain multiple timestamped specifications for organized development".to_string(),
                "You can use 'mcp_foundry_analyze_project' to add project analysis to existing codebases".to_string(),
            ],
        )
    };

    Ok(build_success_response(
        response_data,
        next_steps,
        workflow_hints,
    ))
}
