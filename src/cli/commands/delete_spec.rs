//! Implementation of the delete_spec command

use crate::cli::args::DeleteSpecArgs;
use crate::core::{project, spec};
use crate::types::responses::{DeleteSpecResponse, FoundryResponse, ValidationStatus};
use anyhow::{Context, Result};
use std::fs;

pub async fn execute(args: DeleteSpecArgs) -> Result<FoundryResponse<DeleteSpecResponse>> {
    // Validate inputs
    validate_args(&args)?;

    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Validate spec exists
    if !spec::spec_exists(&args.project_name, &args.spec_name)? {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'. Use 'foundry load-project {}' to see available specs.",
            args.spec_name,
            args.project_name,
            args.project_name
        ));
    }

    // Get spec path and files before deletion for response
    let spec_path = spec::get_spec_path(&args.project_name, &args.spec_name)?;
    let files_to_delete = get_spec_files(&spec_path)?;

    // Validate confirmation
    if args.confirm.to_lowercase() != "true" {
        return Err(anyhow::anyhow!(
            "Deletion not confirmed. Set --confirm true to proceed with deleting spec '{}' and all its files. Got: '{}'",
            args.spec_name,
            args.confirm
        ));
    }

    // Delete the spec
    spec::delete_spec(&args.project_name, &args.spec_name)
        .with_context(|| format!("Failed to delete spec '{}'", args.spec_name))?;

    let response_data = DeleteSpecResponse {
        project_name: args.project_name.clone(),
        spec_name: args.spec_name.clone(),
        spec_path: spec_path.to_string_lossy().to_string(),
        files_deleted: files_to_delete,
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&args),
        validation_status: ValidationStatus::Complete,
        workflow_hints: generate_workflow_hints(&args),
    })
}

/// Validate command arguments
fn validate_args(args: &DeleteSpecArgs) -> Result<()> {
    if args.project_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    if args.spec_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Spec name cannot be empty"));
    }

    // Validate spec name format (basic check)
    if !args.spec_name.contains('_') {
        return Err(anyhow::anyhow!(
            "Invalid spec name format '{}'. Expected format: YYYYMMDD_HHMMSS_feature_name",
            args.spec_name
        ));
    }

    Ok(())
}

/// Validate that project exists
fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'foundry list-projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

/// Get list of files that will be deleted from a spec directory
fn get_spec_files(spec_path: &std::path::Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    if !spec_path.exists() {
        return Ok(files);
    }

    // Check for standard spec files
    let expected_files = ["spec.md", "task-list.md", "notes.md"];

    for file_name in &expected_files {
        let file_path = spec_path.join(file_name);
        if file_path.exists() {
            files.push(file_path.to_string_lossy().to_string());
        }
    }

    // Also check for any other files in the spec directory
    if let Ok(entries) = fs::read_dir(spec_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                // Only add if not already in our expected files list
                if !expected_files.contains(&file_name) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(files)
}

/// Generate next steps for the response
fn generate_next_steps(args: &DeleteSpecArgs) -> Vec<String> {
    vec![
        format!(
            "Successfully deleted spec '{}' from project '{}'",
            args.spec_name, args.project_name
        ),
        "All spec files have been permanently removed".to_string(),
        format!(
            "View remaining specs: foundry load-project {}",
            args.project_name
        ),
        format!(
            "Create new spec: foundry create-spec {} <feature_name>",
            args.project_name
        ),
        "Deletion cannot be undone - consider backing up important specs before deletion"
            .to_string(),
    ]
}

/// Generate workflow hints for the response
fn generate_workflow_hints(args: &DeleteSpecArgs) -> Vec<String> {
    vec![
        format!("Deleted spec: {}", args.spec_name),
        "This action cannot be undone".to_string(),
        "All associated files (spec.md, task-list.md, notes.md) have been removed".to_string(),
        "Use 'foundry list-projects' to see project status after deletion".to_string(),
        "Consider archiving completed specs rather than deleting for future reference".to_string(),
        "Create new specs to continue feature development".to_string(),
    ]
}
