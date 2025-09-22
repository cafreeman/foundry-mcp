//! Core op for listing specs in a project (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::{project, spec};
use crate::types::responses::{FoundryResponse, ListSpecsResponse, SpecInfo};
use crate::utils::response::{build_incomplete_response, build_success_response};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<ListSpecsResponse>> {
    validate_project_exists(&input.project_name)?;

    let specs = spec::list_specs(&input.project_name)
        .with_context(|| format!("Failed to list specs for project '{}'", input.project_name))?;

    let spec_infos: Vec<SpecInfo> = specs
        .into_iter()
        .map(|spec_meta| SpecInfo {
            name: spec_meta.name,
            feature_name: spec_meta.feature_name,
            created_at: spec_meta.created_at,
        })
        .collect();

    let response_data = ListSpecsResponse {
        project_name: input.project_name.clone(),
        specs: spec_infos.clone(),
        total_count: spec_infos.len(),
    };

    if response_data.specs.is_empty() {
        let next_steps = vec![
            "No specifications found for this project - ready for specification creation"
                .to_string(),
            format!(
                "You can create your first specification: mcp_foundry_create_spec {} <feature_name>",
                input.project_name
            ),
            "You can use 'mcp_foundry_load_project' to see full project context".to_string(),
        ];

        let workflow_hints = vec![
            "You can start by creating specifications to track development features".to_string(),
            "Each spec includes implementation notes and task lists for comprehensive planning"
                .to_string(),
        ];

        Ok(build_incomplete_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    } else {
        let spec_count = response_data.specs.len();
        let mut next_steps = vec![
            format!("Found {} specification(s) in project", spec_count),
            format!(
                "You can load a specific spec: mcp_foundry_load_spec {} <spec_name>",
                input.project_name
            ),
        ];

        if spec_count <= 5 {
            next_steps.push("Available specs:".to_string());
            for spec in &response_data.specs {
                next_steps.push(format!("  - {} ({})", spec.name, spec.feature_name));
            }
        }

        next_steps.push(format!(
            "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
            input.project_name
        ));

        let workflow_hints = vec![
            "Specifications are timestamped and organized by feature for easy navigation"
                .to_string(),
            format!("Total specs: {}", spec_count),
            "You can load individual specs to see detailed implementation plans".to_string(),
            "Specs include specification content, notes, and task lists for complete context"
                .to_string(),
        ];

        Ok(build_success_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    }
}

fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'mcp_foundry_list_projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}
