//! Implementation of the update_spec command

use crate::cli::args::UpdateSpecArgs;
use crate::core::{filesystem, project, spec};
use crate::types::responses::{FoundryResponse, UpdateSpecResponse, ValidationStatus};
use crate::types::spec::SpecFileType;
use anyhow::Result;

pub async fn execute(args: UpdateSpecArgs) -> Result<FoundryResponse<UpdateSpecResponse>> {
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

    // Parse file type
    let file_type = parse_file_type(&args.file_type)?;

    // Get current content for append operations
    let final_content = if args.operation.to_lowercase() == "append" {
        let current_content = get_current_content(&args.project_name, &args.spec_name, &file_type)?;
        if current_content.trim().is_empty() {
            args.content.clone()
        } else {
            format!("{}\n\n{}", current_content, args.content)
        }
    } else {
        args.content.clone()
    };

    // Update the spec content
    spec::update_spec_content(
        &args.project_name,
        &args.spec_name,
        file_type.clone(),
        &final_content,
    )?;

    // Get file path for response
    let file_path = get_file_path(&args.project_name, &args.spec_name, &file_type)?;

    let response_data = UpdateSpecResponse {
        project_name: args.project_name.clone(),
        spec_name: args.spec_name.clone(),
        file_type: args.file_type.clone(),
        operation: args.operation.clone(),
        file_path: file_path.to_string_lossy().to_string(),
        content_length: final_content.len(),
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&args),
        validation_status: ValidationStatus::Complete,
        workflow_hints: generate_workflow_hints(&args, final_content.len()),
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

    if !matches!(
        args.file_type.to_lowercase().as_str(),
        "spec" | "task-list" | "tasks" | "notes"
    ) {
        return Err(anyhow::anyhow!(
            "Invalid file_type '{}'. Must be one of: 'spec', 'task-list', 'tasks', or 'notes'",
            args.file_type
        ));
    }

    if !matches!(args.operation.to_lowercase().as_str(), "replace" | "append") {
        return Err(anyhow::anyhow!(
            "Invalid operation '{}'. Must be either 'replace' or 'append'",
            args.operation
        ));
    }

    if args.content.trim().is_empty() {
        return Err(anyhow::anyhow!("Content cannot be empty"));
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

/// Parse file type string to SpecFileType enum
fn parse_file_type(file_type_str: &str) -> Result<SpecFileType> {
    match file_type_str.to_lowercase().as_str() {
        "spec" => Ok(SpecFileType::Spec),
        "task-list" | "tasks" => Ok(SpecFileType::TaskList),
        "notes" => Ok(SpecFileType::Notes),
        _ => Err(anyhow::anyhow!(
            "Invalid file_type '{}'. Must be one of: 'spec', 'task-list', 'tasks', or 'notes'",
            file_type_str
        )),
    }
}

/// Get current content of a spec file for append operations
fn get_current_content(
    project_name: &str,
    spec_name: &str,
    file_type: &SpecFileType,
) -> Result<String> {
    let file_path = get_file_path(project_name, spec_name, file_type)?;

    filesystem::read_file(&file_path).or_else(|_| Ok(String::new())) // Return empty string if file doesn't exist or can't be read
}

/// Get the file path for a specific spec file type
fn get_file_path(
    project_name: &str,
    spec_name: &str,
    file_type: &SpecFileType,
) -> Result<std::path::PathBuf> {
    let spec_path = spec::get_spec_path(project_name, spec_name)?;

    let filename = match file_type {
        SpecFileType::Spec => "spec.md",
        SpecFileType::Notes => "notes.md",
        SpecFileType::TaskList => "task-list.md",
    };

    Ok(spec_path.join(filename))
}

/// Generate next steps for the response
fn generate_next_steps(args: &UpdateSpecArgs) -> Vec<String> {
    vec![
        format!(
            "Successfully updated {} for spec '{}'",
            args.file_type, args.spec_name
        ),
        format!(
            "Content {} with {} operation",
            if args.operation == "append" {
                "appended"
            } else {
                "replaced"
            },
            args.operation
        ),
        format!(
            "Load updated spec: foundry load-spec {} {}",
            args.project_name, args.spec_name
        ),
        format!(
            "Continue iterating: foundry update-spec {} {} --operation append",
            args.project_name, args.spec_name
        ),
        "Use 'foundry get-foundry-help content-examples' for formatting guidance".to_string(),
    ]
}

/// Generate workflow hints for the response
fn generate_workflow_hints(args: &UpdateSpecArgs, content_length: usize) -> Vec<String> {
    let mut hints = vec![
        format!("Updated {}: {} characters", args.file_type, content_length),
        format!("Operation: {} content", args.operation),
    ];

    match args.file_type.to_lowercase().as_str() {
        "task-list" | "tasks" => {
            hints.push("Use '- [ ]' format for new tasks, '- [x]' for completed tasks".to_string());
            hints.push("Update task lists frequently to track progress".to_string());
        }
        "spec" => {
            hints.push("Use clear headers and structured content for specifications".to_string());
            hints.push(
                "Include Requirements, Acceptance Criteria, and Implementation sections"
                    .to_string(),
            );
        }
        "notes" => {
            hints.push("Add design decisions, context, and implementation notes".to_string());
            hints.push("Use append to build up notes over time during development".to_string());
        }
        _ => {}
    }

    hints.push("Load the spec to see all updated content".to_string());
    hints.push("Continue iterative development with append operations".to_string());

    hints
}
