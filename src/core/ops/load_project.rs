//! Core op for loading full project context (tool-agnostic)

use anyhow::{Context, Result};
use std::fs;

use crate::core::{filesystem, project};
use crate::types::responses::{
    FoundryResponse, LoadProjectResponse, ProjectContext, ValidationStatus,
};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<LoadProjectResponse>> {
    validate_project_exists(&input.project_name)?;

    let project_path = project::get_project_path(&input.project_name)?;
    let project_context = load_project_context(&input.project_name, &project_path)?;
    let specs_available = project_context.specs_available.clone();

    let response_data = LoadProjectResponse {
        project: project_context,
    };

    let validation_status = if specs_available.is_empty() {
        ValidationStatus::Incomplete
    } else {
        ValidationStatus::Complete
    };

    Ok(FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&input.project_name, &specs_available),
        validation_status,
        workflow_hints: generate_workflow_hints(&specs_available),
    })
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

fn load_project_context(
    project_name: &str,
    project_path: &std::path::Path,
) -> Result<ProjectContext> {
    let vision =
        filesystem::read_file(project_path.join("vision.md")).unwrap_or_else(|_| String::new());
    let tech_stack =
        filesystem::read_file(project_path.join("tech-stack.md")).unwrap_or_else(|_| String::new());
    let summary =
        filesystem::read_file(project_path.join("summary.md")).unwrap_or_else(|_| String::new());

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

fn generate_next_steps(project_name: &str, specs_available: &[String]) -> Vec<String> {
    if specs_available.is_empty() {
        vec![
            "Project context loaded successfully - ready for specification creation".to_string(),
            format!(
                "You can create your first specification: mcp_foundry_create_spec {} <feature_name>",
                project_name
            ),
            "Your loaded project context provides comprehensive background for development decisions".to_string(),
        ]
    } else {
        vec![
            format!(
                "Project context loaded with {} specification(s) available",
                specs_available.len()
            ),
            format!(
                "You can load a specific spec: mcp_foundry_load_spec {} <spec_name>",
                project_name
            ),
            format!(
                "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
                project_name
            ),
        ]
    }
}

fn generate_workflow_hints(specs_available: &[String]) -> Vec<String> {
    let mut hints = vec![
        "You can use the project summary for quick context in conversations".to_string(),
        "The full vision provides comprehensive background and goals for your work".to_string(),
        "Tech stack details guide your implementation decisions and technology choices".to_string(),
        "You can skip list-projects calls when you know the project name - load_project is more efficient".to_string(),
    ];

    if specs_available.is_empty() {
        hints.push(
            "You can create specifications to track specific features as you identify them"
                .to_string(),
        );
        hints.push(
            "You can prompt the user about creating specifications to track specific features"
                .to_string(),
        );
    } else {
        hints.push(format!("Available specs: {}", specs_available.join(", ")));
        hints.push(
            "You can load individual specs to see detailed implementation plans and progress"
                .to_string(),
        );
        hints.push("You can update existing specs with progress as work continues".to_string());
    }

    hints.push(
        "You can use mcp_foundry_get_foundry_help decision-points to understand tool selection"
            .to_string(),
    );

    hints
}
