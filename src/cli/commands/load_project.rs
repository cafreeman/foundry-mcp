//! Implementation of the load_project command

use crate::cli::args::LoadProjectArgs;
use crate::core::{filesystem, project};
use crate::types::responses::{
    FoundryResponse, LoadProjectResponse, ProjectContext, ValidationStatus,
};
use anyhow::{Context, Result};
use std::fs;

pub async fn execute(args: LoadProjectArgs) -> Result<FoundryResponse<LoadProjectResponse>> {
    // Check if project exists
    if !project::project_exists(&args.project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'foundry list-projects' to see available projects.",
            args.project_name
        ));
    }

    // Get project path
    let project_path = project::get_project_path(&args.project_name)?.join("project");

    // Read project files - handle missing files gracefully
    let vision =
        filesystem::read_file(project_path.join("vision.md")).unwrap_or_else(|_| String::new());

    let tech_stack =
        filesystem::read_file(project_path.join("tech-stack.md")).unwrap_or_else(|_| String::new());

    let summary =
        filesystem::read_file(project_path.join("summary.md")).unwrap_or_else(|_| String::new());

    // Get creation time from directory metadata
    let created_at = fs::metadata(&project_path)
        .and_then(|metadata| metadata.created())
        .map_err(anyhow::Error::from)
        .and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .map_err(anyhow::Error::from)
        })
        .map(|duration| {
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    // Scan specs directory for available specifications
    let specs_dir = project_path.join("specs");
    let specs_available = if specs_dir.exists() {
        fs::read_dir(&specs_dir)
            .context("Failed to read specs directory")?
            .filter_map(|entry| {
                entry
                    .ok()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
            })
            .collect()
    } else {
        Vec::new()
    };

    // Build response
    let project_context = ProjectContext {
        name: args.project_name.clone(),
        vision,
        tech_stack,
        summary,
        specs_available: specs_available.clone(),
        created_at,
    };

    let response_data = LoadProjectResponse {
        project: project_context,
    };

    // Generate next steps based on available specs
    let next_steps = if specs_available.is_empty() {
        vec![
            format!(
                "Loaded project '{}' context successfully",
                args.project_name
            ),
            "No specifications found. Create your first spec with: foundry create-spec".to_string(),
            "Continue development using the loaded project context".to_string(),
        ]
    } else {
        vec![
            format!(
                "Loaded project '{}' with {} specification(s)",
                args.project_name,
                specs_available.len()
            ),
            "Load a specific spec with: foundry load-spec <project_name> <spec_name>".to_string(),
            "Create a new spec with: foundry create-spec".to_string(),
        ]
    };

    let workflow_hints = vec![
        "Use project summary for quick context in conversations".to_string(),
        "Full vision provides comprehensive background and goals".to_string(),
        "Tech stack details guide implementation decisions".to_string(),
        if !specs_available.is_empty() {
            format!("Available specs: {}", specs_available.join(", "))
        } else {
            "Consider creating specifications to track specific features".to_string()
        },
    ];

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    })
}
