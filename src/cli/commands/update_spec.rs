//! Implementation of the update_spec command

use crate::cli::args::UpdateSpecArgs;
use crate::core::edit_engine::EditEngine;
use crate::core::{project, spec};
use crate::types::edit_commands::EditCommand;
use crate::types::responses::{EditCommandsResponsePayload, FoundryResponse, ValidationStatus};
use anyhow::Result;

pub async fn execute(args: UpdateSpecArgs) -> Result<FoundryResponse<EditCommandsResponsePayload>> {
    // Validate inputs
    validate_args(&args)?;

    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Validate spec exists
    if !spec::spec_exists(&args.project_name, &args.spec_name)? {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'. Use load_project tool to see available specs: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}",
            args.spec_name,
            args.project_name,
            args.project_name
        ));
    }

    let commands: Vec<EditCommand> = serde_json::from_str(&args.commands)
        .map_err(|e| anyhow::anyhow!("Invalid commands JSON: {}", e))?;

    let result = EditEngine::apply_edit_commands(&args.project_name, &args.spec_name, &commands)?;

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

/// Validate command arguments
fn validate_args(args: &UpdateSpecArgs) -> Result<()> {
    if args.project_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    if args.spec_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Spec name cannot be empty"));
    }

    if args.commands.trim().is_empty() {
        return Err(anyhow::anyhow!("'commands' parameter is required"));
    }

    Ok(())
}

/// Validate that project exists
fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use list_projects tool to see available projects: {{\"name\": \"list_projects\", \"arguments\": {{}}}}",
            project_name
        ));
    }
    Ok(())
}
