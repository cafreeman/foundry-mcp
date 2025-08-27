//! Implementation of the update_spec command

use crate::cli::args::UpdateSpecArgs;
use crate::core::{filesystem, project, spec};
use crate::types::responses::{
    FileUpdateResult, FoundryResponse, UpdateSpecResponse, ValidationStatus,
};
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

    // Build list of files to update
    let files_to_update = build_update_list(&args)?;

    // Process each file update
    let mut results = Vec::new();
    for file_update in files_to_update {
        let result = update_single_file(&args, &file_update)?;
        results.push(result);
    }

    let total_files_updated = results.len();

    let response_data = UpdateSpecResponse {
        project_name: args.project_name.clone(),
        spec_name: args.spec_name.clone(),
        files_updated: results,
        total_files_updated,
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&args),
        validation_status: ValidationStatus::Complete,
        workflow_hints: generate_workflow_hints(&args),
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

    // Validate operation is required and valid
    if args.operation.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Operation is required. Must be either 'replace' or 'append'"
        ));
    }

    if !matches!(args.operation.to_lowercase().as_str(), "replace" | "append") {
        return Err(anyhow::anyhow!(
            "Invalid operation '{}'. Must be either 'replace' or 'append'",
            args.operation
        ));
    }

    // Validate at least one content parameter is provided
    let has_spec = args.spec.as_ref().map_or(false, |s| !s.trim().is_empty());
    let has_tasks = args.tasks.as_ref().map_or(false, |s| !s.trim().is_empty());
    let has_notes = args.notes.as_ref().map_or(false, |s| !s.trim().is_empty());

    if !has_spec && !has_tasks && !has_notes {
        return Err(anyhow::anyhow!(
            "At least one content parameter must be provided. Use --spec, --tasks, or --notes to specify content for the files you want to update."
        ));
    }

    // Validate non-empty content for provided parameters
    if let Some(ref spec_content) = args.spec {
        if spec_content.trim().is_empty() {
            return Err(anyhow::anyhow!("Spec content cannot be empty"));
        }
    }

    if let Some(ref tasks_content) = args.tasks {
        if tasks_content.trim().is_empty() {
            return Err(anyhow::anyhow!("Tasks content cannot be empty"));
        }
    }

    if let Some(ref notes_content) = args.notes {
        if notes_content.trim().is_empty() {
            return Err(anyhow::anyhow!("Notes content cannot be empty"));
        }
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

/// Represents a single file update operation
#[derive(Debug)]
struct FileUpdate {
    file_type: SpecFileType,
    file_type_str: String,
    content: String,
}

/// Build list of files to update based on provided content arguments
fn build_update_list(args: &UpdateSpecArgs) -> Result<Vec<FileUpdate>> {
    let mut updates = Vec::new();

    if let Some(ref spec_content) = args.spec {
        updates.push(FileUpdate {
            file_type: SpecFileType::Spec,
            file_type_str: "spec".to_string(),
            content: spec_content.clone(),
        });
    }

    if let Some(ref tasks_content) = args.tasks {
        updates.push(FileUpdate {
            file_type: SpecFileType::TaskList,
            file_type_str: "tasks".to_string(),
            content: tasks_content.clone(),
        });
    }

    if let Some(ref notes_content) = args.notes {
        updates.push(FileUpdate {
            file_type: SpecFileType::Notes,
            file_type_str: "notes".to_string(),
            content: notes_content.clone(),
        });
    }

    Ok(updates)
}

/// Process a single file update and return the result
fn update_single_file(args: &UpdateSpecArgs, file_update: &FileUpdate) -> Result<FileUpdateResult> {
    let file_path = get_file_path(&args.project_name, &args.spec_name, &file_update.file_type)?;

    match perform_file_update(args, file_update) {
        Ok(content_length) => Ok(FileUpdateResult {
            file_type: file_update.file_type_str.clone(),
            operation: args.operation.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            content_length,
            success: true,
            error_message: None,
        }),
        Err(error) => Ok(FileUpdateResult {
            file_type: file_update.file_type_str.clone(),
            operation: args.operation.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            content_length: 0,
            success: false,
            error_message: Some(error.to_string()),
        }),
    }
}

/// Perform the actual file update operation
fn perform_file_update(args: &UpdateSpecArgs, file_update: &FileUpdate) -> Result<usize> {
    let final_content = if args.operation.to_lowercase() == "append" {
        let current_content =
            get_current_content(&args.project_name, &args.spec_name, &file_update.file_type)?;
        if current_content.trim().is_empty() {
            file_update.content.clone()
        } else {
            format!("{}\n\n{}", current_content, file_update.content)
        }
    } else {
        file_update.content.clone()
    };

    spec::update_spec_content(
        &args.project_name,
        &args.spec_name,
        file_update.file_type.clone(),
        &final_content,
    )?;

    Ok(final_content.len())
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
    let mut steps = vec![
        format!(
            "Successfully updated spec '{}' in project '{}' with {} operation",
            args.spec_name, args.project_name, args.operation
        ),
        format!(
            "Load updated spec: foundry load_spec {} {}",
            args.project_name, args.spec_name
        ),
        "Use 'foundry get-foundry-help content-examples' for formatting guidance".to_string(),
    ];

    // Add operation-specific guidance
    if args.operation.to_lowercase() == "append" {
        steps.push("Content was appended to preserve existing data".to_string());
        steps.push(format!(
            "Continue iterating: foundry update-spec {} {} --operation append",
            args.project_name, args.spec_name
        ));
    } else {
        steps.push("Content was completely replaced".to_string());
        steps.push(
            "Use --operation append for future updates to preserve existing content".to_string(),
        );
    }

    steps
}

/// Generate workflow hints for the response
fn generate_workflow_hints(args: &UpdateSpecArgs) -> Vec<String> {
    let mut hints = vec![format!(
        "Operation: {} content across multiple files",
        args.operation
    )];

    // Add hints about which files were updated
    let mut file_hints = Vec::new();
    if args.spec.is_some() {
        file_hints.push("spec.md");
    }
    if args.tasks.is_some() {
        file_hints.push("task-list.md");
    }
    if args.notes.is_some() {
        file_hints.push("notes.md");
    }

    if !file_hints.is_empty() {
        hints.push(format!("Updated files: {}", file_hints.join(", ")));
    }

    // Add operation-specific guidance
    if args.operation.to_lowercase() == "append" {
        hints.push("Content was appended to preserve existing data".to_string());
        hints.push("Use append operations to iteratively build up specifications".to_string());
    } else {
        hints.push("Content was completely replaced".to_string());
        hints.push("Use replace operations for major restructuring or rewrites".to_string());
    }

    // Add general guidance
    hints.push("Load the spec to see all updated content".to_string());
    hints.push("Use --operation append for iterative development".to_string());
    hints.push("Use --operation replace for major changes".to_string());

    hints
}
