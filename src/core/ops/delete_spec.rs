//! Core op for deleting a spec (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::foundry;
use crate::types::responses::{DeleteSpecResponse, FoundryResponse, ValidationStatus};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub spec_name: String,
    pub confirm: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<DeleteSpecResponse>> {
    let foundry = foundry::get_default_foundry()?;

    validate_args(&input)?;
    validate_project_exists(&foundry, &input.project_name).await?;

    // Check if spec exists by trying to load it
    let response_data = match foundry
        .load_spec(&input.project_name, &input.spec_name)
        .await
    {
        Ok(spec) => {
            let files_to_delete = vec![
                format!("{}/spec.md", spec.name),
                format!("{}/task-list.md", spec.name),
                format!("{}/notes.md", spec.name),
            ];

            if input.confirm.to_lowercase() != "true" {
                return Err(anyhow::anyhow!(
                    "Deletion not confirmed. Set --confirm true to proceed with deleting spec '{}' and all its files. Got: '{}'",
                    input.spec_name,
                    input.confirm
                ));
            }

            foundry
                .delete_spec(&input.project_name, &input.spec_name)
                .await
                .with_context(|| format!("Failed to delete spec '{}'", input.spec_name))?;

            DeleteSpecResponse {
                project_name: input.project_name.clone(),
                spec_name: input.spec_name.clone(),
                spec_path: format!(
                    "~/.foundry/{}/specs/{}",
                    input.project_name, input.spec_name
                ),
                files_deleted: files_to_delete,
            }
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'. Use 'mcp_foundry_load_project {}' to see available specs.",
                input.spec_name,
                input.project_name,
                input.project_name
            ));
        }
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&input),
        validation_status: ValidationStatus::Complete,
        workflow_hints: generate_workflow_hints(&input),
    })
}

fn validate_args(input: &Input) -> Result<()> {
    if input.project_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }
    if input.spec_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Spec name cannot be empty"));
    }
    if !input.spec_name.contains('_') {
        return Err(anyhow::anyhow!(
            "Invalid spec name format '{}'. Expected format: YYYYMMDD_HHMMSS_feature_name",
            input.spec_name
        ));
    }
    Ok(())
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

fn generate_next_steps(input: &Input) -> Vec<String> {
    vec![
        format!(
            "Successfully deleted spec '{}' from project '{}'",
            input.spec_name, input.project_name
        ),
        "All spec files have been permanently removed".to_string(),
        format!(
            "You can view remaining specs: mcp_foundry_load_project {}",
            input.project_name
        ),
        format!(
            "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
            input.project_name
        ),
        "Deletion cannot be undone - you might consider backing up important specs before deletion"
            .to_string(),
    ]
}

fn generate_workflow_hints(input: &Input) -> Vec<String> {
    vec![
        format!("Deleted spec: {}", input.spec_name),
        "This action cannot be undone".to_string(),
        "All associated files (spec.md, task-list.md, notes.md) have been removed".to_string(),
        "You can use 'mcp_foundry_list_projects' to see project status after deletion".to_string(),
        "You might consider archiving completed specs rather than deleting for future reference"
            .to_string(),
        "You can create new specs to continue feature development".to_string(),
    ]
}
