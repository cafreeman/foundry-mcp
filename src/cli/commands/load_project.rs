//! Implementation of the load_project command

use crate::cli::args::LoadProjectArgs;
use crate::core::{filesystem, project};
use crate::types::responses::{FoundryResponse, LoadProjectResponse, ProjectContext};
use crate::utils::response::build_success_response;
use anyhow::{Context, Result};
use std::fs;

pub async fn execute(args: LoadProjectArgs) -> Result<FoundryResponse<LoadProjectResponse>> {
    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Load project data
    let project_path = project::get_project_path(&args.project_name)?.join("project");
    let project_context = load_project_context(&args.project_name, &project_path)?;
    let specs_available = project_context.specs_available.clone();

    // Build response
    let response_data = LoadProjectResponse {
        project: project_context,
    };

    Ok(build_success_response(
        response_data,
        generate_next_steps(&args.project_name, &specs_available),
        generate_workflow_hints(&specs_available),
    ))
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

/// Load project context from files
fn load_project_context(
    project_name: &str,
    project_path: &std::path::Path,
) -> Result<ProjectContext> {
    // Read project files - handle missing files gracefully
    let vision =
        filesystem::read_file(project_path.join("vision.md")).unwrap_or_else(|_| String::new());
    let tech_stack =
        filesystem::read_file(project_path.join("tech-stack.md")).unwrap_or_else(|_| String::new());
    let summary =
        filesystem::read_file(project_path.join("summary.md")).unwrap_or_else(|_| String::new());

    // Get creation time from directory metadata
    let created_at = fs::metadata(project_path)
        .and_then(|metadata| metadata.created())
        .map_err(anyhow::Error::from)
        .and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .map_err(anyhow::Error::from)
        })
        .map(|duration| {
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
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

    Ok(ProjectContext {
        name: project_name.to_string(),
        vision,
        tech_stack,
        summary,
        specs_available,
        created_at,
    })
}

/// Generate next steps based on available specs
fn generate_next_steps(project_name: &str, specs_available: &[String]) -> Vec<String> {
    if specs_available.is_empty() {
        vec![
            format!("Loaded project '{}' context successfully", project_name),
            "No specifications found. Create your first spec with: foundry create-spec".to_string(),
            "Continue development using the loaded project context".to_string(),
        ]
    } else {
        vec![
            format!(
                "Loaded project '{}' with {} specification(s)",
                project_name,
                specs_available.len()
            ),
            "Load a specific spec with: foundry load-spec <project_name> <spec_name>".to_string(),
            "Create a new spec with: foundry create-spec".to_string(),
        ]
    }
}

/// Generate workflow hints based on available specs
fn generate_workflow_hints(specs_available: &[String]) -> Vec<String> {
    vec![
        "Use project summary for quick context in conversations".to_string(),
        "Full vision provides comprehensive background and goals".to_string(),
        "Tech stack details guide implementation decisions".to_string(),
        if !specs_available.is_empty() {
            format!("Available specs: {}", specs_available.join(", "))
        } else {
            "Consider creating specifications to track specific features".to_string()
        },
    ]
}
