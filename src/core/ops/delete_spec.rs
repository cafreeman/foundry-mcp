//! Core op for deleting a spec (tool-agnostic)

use anyhow::{Context, Result};
use std::fs;

use crate::core::{project, spec};
use crate::types::responses::{DeleteSpecResponse, FoundryResponse, ValidationStatus};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub spec_name: String,
    pub confirm: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<DeleteSpecResponse>> {
    validate_args(&input)?;
    validate_project_exists(&input.project_name)?;

    if !spec::spec_exists(&input.project_name, &input.spec_name)? {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'. Use 'mcp_foundry_load_project {}' to see available specs.",
            input.spec_name,
            input.project_name,
            input.project_name
        ));
    }

    let spec_path = spec::get_spec_path(&input.project_name, &input.spec_name)?;
    let files_to_delete = get_spec_files(&spec_path)?;

    if input.confirm.to_lowercase() != "true" {
        return Err(anyhow::anyhow!(
            "Deletion not confirmed. Set --confirm true to proceed with deleting spec '{}' and all its files. Got: '{}'",
            input.spec_name,
            input.confirm
        ));
    }

    spec::delete_spec(&input.project_name, &input.spec_name)
        .with_context(|| format!("Failed to delete spec '{}'", input.spec_name))?;

    let response_data = DeleteSpecResponse {
        project_name: input.project_name.clone(),
        spec_name: input.spec_name.clone(),
        spec_path: spec_path.to_string_lossy().to_string(),
        files_deleted: files_to_delete,
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

fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'mcp_foundry_list_projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

fn get_spec_files(spec_path: &std::path::Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    if !spec_path.exists() {
        return Ok(files);
    }
    let expected_files = ["spec.md", "task-list.md", "notes.md"];
    for file_name in &expected_files {
        let file_path = spec_path.join(file_name);
        if file_path.exists() {
            files.push(file_path.to_string_lossy().to_string());
        }
    }
    if let Ok(entries) = fs::read_dir(spec_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                if !expected_files.contains(&file_name) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(files)
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
