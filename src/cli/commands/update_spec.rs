//! Implementation of the update_spec command

use crate::cli::args::UpdateSpecArgs;
use crate::core::{context_patch::ContextMatcher, filesystem, project, spec};
use crate::types::responses::{
    FileUpdateResult, FoundryResponse, UpdateSpecResponse, ValidationStatus,
};
use crate::types::spec::{ContextOperation, ContextPatch, SpecFileType};
use anyhow::Result;

pub async fn execute(args: UpdateSpecArgs) -> Result<FoundryResponse<UpdateSpecResponse>> {
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

    // Branch based on operation type
    let (results, total_files_updated) = match args.operation.to_lowercase().as_str() {
        "context_patch" => execute_context_patch(&args).await?,
        "replace" | "append" => execute_traditional_update(&args).await?,
        _ => return Err(anyhow::anyhow!("Invalid operation: {}", args.operation)),
    };

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
            "Operation is required. Must be 'replace', 'append', or 'context_patch'"
        ));
    }

    match args.operation.to_lowercase().as_str() {
        "replace" | "append" => validate_traditional_args(args)?,
        "context_patch" => validate_context_patch_args(args)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid operation '{}'. Must be 'replace', 'append', or 'context_patch'",
                args.operation
            ));
        }
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
            operation_performed: args.operation.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            content_length,
            success: true,
            error_message: None,
            lines_modified: None,
            patch_type: None,
            match_confidence: None,
        }),
        Err(error) => Ok(FileUpdateResult {
            file_type: file_update.file_type_str.clone(),
            operation_performed: args.operation.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            content_length: 0,
            success: false,
            error_message: Some(error.to_string()),
            lines_modified: None,
            patch_type: None,
            match_confidence: None,
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
            "Load updated spec: {{\"name\": \"load_spec\", \"arguments\": {{\"project_name\": \"{}\", \"spec_name\": \"{}\"}}}}",
            args.project_name, args.spec_name
        ),
        "Use get_foundry_help tool for formatting guidance: {\"name\": \"get_foundry_help\", \"arguments\": {\"topic\": \"content-examples\"}}".to_string(),
    ];

    // Add operation-specific guidance
    if args.operation.to_lowercase() == "append" {
        steps.push("Content was appended to preserve existing data".to_string());
        steps.push(format!(
            "Continue iterating: {{\"name\": \"update_spec\", \"arguments\": {{\"project_name\": \"{}\", \"spec_name\": \"{}\", \"operation\": \"append\", \"tasks\": \"new task content\"}}}}",
            args.project_name, args.spec_name
        ));
    } else {
        steps.push("Content was completely replaced".to_string());
        steps.push(
            "Use append operation for future updates to preserve existing content".to_string(),
        );
    }

    steps
}

/// Generate workflow hints with efficiency guidance
fn generate_workflow_hints(args: &UpdateSpecArgs) -> Vec<String> {
    let mut hints = vec![
        "📋 DOCUMENT PURPOSE: Your updates serve as COMPLETE CONTEXT for future implementation".to_string(),
        "🎯 CONTEXT TEST: Could someone with no prior knowledge implement using your updated documents?".to_string(),
    ];

    // Add operation-specific efficiency guidance
    match args.operation.to_lowercase().as_str() {
        "context_patch" => {
            hints.push(
                "✅ EFFICIENT CHOICE: Context patch uses ~90% fewer tokens than replace"
                    .to_string(),
            );
            hints
                .push("🎯 PRECISE UPDATES: Targeted changes preserve existing content".to_string());
        }
        "replace" => {
            hints.push(
                "⚠️  HIGH TOKEN COST: Replace operation uses full document tokens".to_string(),
            );
            hints.push(
                "💡 EFFICIENCY TIP: Use context_patch for small targeted changes next time"
                    .to_string(),
            );
            hints.push(
                "📊 TOKEN COMPARISON: Context patch saves 80-95% tokens vs replace".to_string(),
            );
        }
        "append" => {
            hints.push(
                "📝 ADDITIVE APPROACH: Content appended to preserve existing data".to_string(),
            );
            hints.push(
                "💡 PRECISION OPTION: Consider context_patch for insertions in specific locations"
                    .to_string(),
            );
        }
        _ => {}
    }

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
        hints.push(format!("📁 Updated files: {}", file_hints.join(", ")));
    }

    // Add strategic recommendations based on operation used
    match args.operation.to_lowercase().as_str() {
        "replace" => {
            hints.push("🚀 NEXT TIME: For marking tasks complete, try context_patch with specific task text".to_string());
            hints.push("🎯 EXAMPLE: {\"name\": \"update_spec\", \"arguments\": {\"project_name\": \"project\", \"spec_name\": \"spec\", \"operation\": \"context_patch\", \"context_patch\": \"{\\\"file_type\\\":\\\"tasks\\\",\\\"operation\\\":\\\"replace\\\",\\\"before_context\\\":[\\\"- [ ] Implement feature X\\\"],\\\"after_context\\\":[\\\"- [ ] Add validation\\\"],\\\"content\\\":\\\"- [x] Implement feature X\\\"}\"}}".to_string());
        }
        "append" => {
            hints.push(
                "📍 PRECISE INSERTIONS: Use context_patch to add content at specific locations"
                    .to_string(),
            );
            hints.push(
                "⚡ EFFICIENCY: Context patch can place content exactly where needed".to_string(),
            );
        }
        "context_patch" => {
            hints.push("🎉 EXCELLENT: You're using the most efficient update method!".to_string());
            hints.push("📈 PATTERN: Save this context pattern for similar updates".to_string());
        }
        _ => {}
    }

    // Add general guidance
    hints.push("📖 Load the spec to see all updated content".to_string());
    hints.push("🔄 ITERATIVE: Use append for building up content over time".to_string());
    hints.push("🎯 TARGETED: Use context_patch for precise changes".to_string());
    hints.push("🔄 MAJOR CHANGES: Use replace only for complete rewrites".to_string());

    hints
}

/// Execute context-based patch operation
async fn execute_context_patch(args: &UpdateSpecArgs) -> Result<(Vec<FileUpdateResult>, usize)> {
    // Parse the context patch JSON
    let context_patch_json = args.context_patch.as_ref().ok_or_else(|| {
        anyhow::anyhow!("context_patch parameter is required for context_patch operation")
    })?;

    let patch: ContextPatch = serde_json::from_str(context_patch_json)
        .map_err(|e| anyhow::anyhow!("Invalid context patch JSON: {}", e))?;

    // Determine which file to update based on the patch
    let file_path = match patch.file_type {
        SpecFileType::Spec => spec::get_spec_file_path(&args.project_name, &args.spec_name)?,
        SpecFileType::TaskList => {
            spec::get_task_list_file_path(&args.project_name, &args.spec_name)?
        }
        SpecFileType::Notes => spec::get_notes_file_path(&args.project_name, &args.spec_name)?,
    };

    // Read current file content
    let current_content = filesystem::read_file(&file_path)?;

    // Apply the context patch
    let mut matcher = ContextMatcher::new(current_content.clone());
    let patch_result = matcher.apply_patch(&patch)?;

    let mut results = Vec::new();

    if patch_result.success {
        // Write the updated content back to the file
        filesystem::write_file_atomic(&file_path, matcher.get_content())?;

        // Calculate and show efficiency gains
        let original_content_size = current_content.len();
        let patch_content_size = patch.content.len() + 200; // estimate context overhead
        let efficiency_percentage =
            (1.0 - (patch_content_size as f32 / original_content_size as f32).min(1.0)) * 100.0;

        // Success amplification messaging
        if let Some(confidence) = patch_result.match_confidence {
            eprintln!(
                "✅ Context patch succeeded with {:.1}% match confidence",
                confidence * 100.0
            );
        }
        eprintln!(
            "⚡ Token efficiency: Saved ~{:.0}% tokens vs replace operation",
            efficiency_percentage
        );
        eprintln!(
            "🎯 Modified {} lines with precise targeting",
            patch_result.lines_modified
        );

        results.push(FileUpdateResult {
            file_type: format!("{:?}", patch.file_type),
            file_path: file_path.to_string_lossy().to_string(),
            content_length: matcher.get_content().len(),
            operation_performed: args.operation.clone(),
            success: true,
            error_message: None,
            lines_modified: Some(patch_result.lines_modified),
            patch_type: Some(patch_result.patch_type),
            match_confidence: patch_result.match_confidence,
        });
    } else {
        // Patch failed - return error information
        results.push(FileUpdateResult {
            file_type: format!("{:?}", patch.file_type),
            file_path: file_path.to_string_lossy().to_string(),
            content_length: 0,
            operation_performed: args.operation.clone(),
            success: false,
            error_message: patch_result
                .error_message
                .or_else(|| Some("Context patch failed".to_string())),
            lines_modified: Some(0),
            patch_type: Some(patch_result.patch_type),
            match_confidence: None,
        });
    }

    let total_files = results.len();
    Ok((results, total_files))
}

/// Execute traditional update operation (replace/append)
async fn execute_traditional_update(
    args: &UpdateSpecArgs,
) -> Result<(Vec<FileUpdateResult>, usize)> {
    // Build list of files to update
    let files_to_update = build_update_list(args)?;

    // Process each file update
    let mut results = Vec::new();
    for file_update in files_to_update {
        let result = update_single_file(args, &file_update)?;
        results.push(result);
    }

    let total_files_updated = results.len();
    Ok((results, total_files_updated))
}

/// Validate arguments for traditional operations (replace/append) with efficiency warnings
fn validate_traditional_args(args: &UpdateSpecArgs) -> Result<()> {
    // Validate at least one content parameter is provided
    let has_spec = args.spec.as_ref().is_some_and(|s| !s.trim().is_empty());
    let has_tasks = args.tasks.as_ref().is_some_and(|s| !s.trim().is_empty());
    let has_notes = args.notes.as_ref().is_some_and(|s| !s.trim().is_empty());

    if !has_spec && !has_tasks && !has_notes {
        return Err(anyhow::anyhow!(
            "At least one content parameter must be provided. Use spec, tasks, or notes parameters to specify content for the files you want to update."
        ));
    }

    // Add strategic friction for replace operations
    if args.operation.to_lowercase() == "replace" {
        eprintln!("⚠️  TOKEN EFFICIENCY WARNING:");
        eprintln!("   Replace operation will use ~10-20x more tokens than context_patch");
        eprintln!("   Consider using context_patch for targeted changes:");
        eprintln!("   • Mark tasks complete: use context_patch with specific task text");
        eprintln!("   • Add single requirements: use context_patch to insert at specific location");
        eprintln!("   • Fix specific content: use context_patch with surrounding context");
        eprintln!();
        eprintln!("   Example context_patch for task completion:");
        eprintln!("   {{\"name\": \"update_spec\", \"arguments\": {{");
        eprintln!("     \"project_name\": \"project\", \"spec_name\": \"spec\",");
        eprintln!("     \"operation\": \"context_patch\",");
        eprintln!(
            "     \"context_patch\": \"{{\\\"file_type\\\":\\\"tasks\\\",\\\"operation\\\":\\\"replace\\\",\\\"before_context\\\":[\\\"- [ ] Implement user authentication\\\"],\\\"after_context\\\":[\\\"- [ ] Add password validation\\\"],\\\"content\\\":\\\"- [x] Implement user authentication\\\"}}\""
        );
        eprintln!("   }}}}");
        eprintln!();
    }

    Ok(())
}

/// Validate arguments for context patch operations
fn validate_context_patch_args(args: &UpdateSpecArgs) -> Result<()> {
    // Ensure context_patch parameter is provided
    let context_patch_json = args.context_patch.as_ref().ok_or_else(|| {
        anyhow::anyhow!("context_patch parameter is required for context_patch operation")
    })?;

    if context_patch_json.trim().is_empty() {
        return Err(anyhow::anyhow!("context_patch parameter cannot be empty"));
    }

    // Validate JSON structure
    let patch: ContextPatch = serde_json::from_str(context_patch_json)
        .map_err(|e| anyhow::anyhow!("Invalid context patch JSON: {}. Expected format: {{\"file_type\":\"spec|tasks|notes\",\"operation\":\"insert|replace|delete\",\"before_context\":[\"line1\"],\"after_context\":[\"line2\"],\"content\":\"new content\"}}", e))?;

    // Validate context requirements
    if patch.before_context.is_empty() && patch.after_context.is_empty() {
        return Err(anyhow::anyhow!(
            "At least one of 'before_context' or 'after_context' must be provided in context patch"
        ));
    }

    // Validate content requirements based on operation
    match patch.operation {
        ContextOperation::Insert | ContextOperation::Replace => {
            if patch.content.trim().is_empty() {
                return Err(anyhow::anyhow!(
                    "'content' field cannot be empty for insert/replace operations"
                ));
            }
        }
        ContextOperation::Delete => {
            // Delete operations don't require content
        }
    }

    // Ensure traditional content parameters are not provided with context_patch
    if args.spec.is_some() || args.tasks.is_some() || args.notes.is_some() {
        return Err(anyhow::anyhow!(
            "Cannot use spec, tasks, or notes parameters with context_patch operation. Use context_patch JSON parameter instead."
        ));
    }

    Ok(())
}
