//! Core op for applying edit commands to a spec (tool-agnostic)

use anyhow::Result;

use crate::core::foundry;
use crate::types::edit_commands::EditCommand;
use crate::types::responses::{EditCommandsResponsePayload, FoundryResponse, ValidationStatus};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub spec_name: String,
    pub commands_json: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<EditCommandsResponsePayload>> {
    let foundry = foundry::get_default_foundry()?;

    validate_args(&input)?;
    validate_project_exists(&foundry, &input.project_name).await?;

    // Check if spec exists by trying to load it
    foundry.load_spec(&input.project_name, &input.spec_name).await.map_err(|_| {
        anyhow::anyhow!(
            "Spec '{}' not found in project '{}'. Use load_project tool to see available specs: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}",
            input.spec_name,
            input.project_name,
            input.project_name
        )
    })?;

    let commands: Vec<EditCommand> = serde_json::from_str(&input.commands_json)
        .map_err(|e| anyhow::anyhow!("Invalid commands JSON: {}", e))?;

    let result = foundry
        .apply_edit_commands(&input.project_name, &input.spec_name, &commands)
        .await?;

    let response_data = EditCommandsResponsePayload {
        applied_count: result.applied_count,
        skipped_idempotent_count: result.skipped_idempotent_count,
        file_updates: result.file_updates,
        errors: if result.errors.is_empty() {
            None
        } else {
            Some(result.errors)
        },
        preview_diff: result.preview_diff,
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: result.next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints: result.workflow_hints,
    })
}

fn validate_args(input: &Input) -> Result<()> {
    if input.project_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }
    if input.spec_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Spec name cannot be empty"));
    }
    if input.commands_json.trim().is_empty() {
        return Err(anyhow::anyhow!("'commands' parameter is required"));
    }
    Ok(())
}

async fn validate_project_exists(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<()> {
    if !foundry.project_exists(project_name).await? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use list_projects tool to see available projects: {{\"name\": \"list_projects\", \"arguments\": {{}}}}",
            project_name
        ));
    }
    Ok(())
}
