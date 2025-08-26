//! Implementation of the load_spec command

use crate::cli::args::LoadSpecArgs;
use crate::core::{filesystem, project, spec};
use crate::types::responses::{
    FoundryResponse, LoadSpecResponse, SpecContent, SpecInfo, ValidationStatus,
};
use anyhow::{Context, Result};

pub async fn execute(args: LoadSpecArgs) -> Result<FoundryResponse<LoadSpecResponse>> {
    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Load project summary for context
    let project_summary = load_project_summary(&args.project_name)?;

    // Handle two cases: list specs or load specific spec
    match args.spec_name {
        None => {
            // List available specs
            let specs = spec::list_specs(&args.project_name)?;
            let available_specs: Vec<SpecInfo> = specs
                .into_iter()
                .map(|spec_meta| SpecInfo {
                    name: spec_meta.name,
                    feature_name: spec_meta.feature_name,
                    created_at: spec_meta.created_at,
                })
                .collect();

            let response_data = LoadSpecResponse {
                project_name: args.project_name.clone(),
                project_summary,
                spec_name: None,
                created_at: None,
                spec_content: None,
                available_specs: available_specs.clone(),
            };

            Ok(FoundryResponse {
                data: response_data,
                next_steps: generate_listing_next_steps(&args.project_name, &available_specs),
                validation_status: if available_specs.is_empty() {
                    ValidationStatus::Incomplete
                } else {
                    ValidationStatus::Complete
                },
                workflow_hints: generate_listing_workflow_hints(&available_specs),
            })
        }
        Some(spec_name) => {
            // Load specific spec
            let spec_data = spec::load_spec(&args.project_name, &spec_name)
                .with_context(|| format!("Failed to load spec '{}'", spec_name))?;

            let spec_content = SpecContent {
                spec: spec_data.spec_content,
                notes: spec_data.notes,
                task_list: spec_data.tasks,
            };

            let response_data = LoadSpecResponse {
                project_name: args.project_name.clone(),
                project_summary,
                spec_name: Some(spec_data.name.clone()),
                created_at: Some(spec_data.created_at.clone()),
                spec_content: Some(spec_content),
                available_specs: Vec::new(), // Empty when loading specific spec
            };

            Ok(FoundryResponse {
                data: response_data,
                next_steps: generate_spec_next_steps(&args.project_name, &spec_data.name),
                validation_status: ValidationStatus::Complete,
                workflow_hints: generate_spec_workflow_hints(&spec_data.name),
            })
        }
    }
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

/// Load project summary for context
fn load_project_summary(project_name: &str) -> Result<String> {
    let project_path = project::get_project_path(project_name)?;
    let summary_path = project_path.join("summary.md");

    Ok(filesystem::read_file(summary_path).unwrap_or_else(|_| {
        "No project summary available. Consider updating the project summary for better context.".to_string()
    }))
}

/// Generate next steps for spec listing
fn generate_listing_next_steps(project_name: &str, available_specs: &[SpecInfo]) -> Vec<String> {
    if available_specs.is_empty() {
        vec![
            "No specifications found for this project".to_string(),
            format!(
                "Create your first specification: foundry create-spec {} <feature_name>",
                project_name
            ),
            "Use 'foundry load-project' to see full project context".to_string(),
        ]
    } else {
        let mut steps = vec![
            format!(
                "Found {} specification(s) in project",
                available_specs.len()
            ),
            format!(
                "Load specific spec: foundry load-spec {} <spec_name>",
                project_name
            ),
        ];

        if available_specs.len() <= 5 {
            steps.push("Available specs:".to_string());
            for spec in available_specs {
                steps.push(format!("  - {} ({})", spec.name, spec.feature_name));
            }
        }

        steps.push(format!(
            "Create new spec: foundry create-spec {} <feature_name>",
            project_name
        ));

        steps
    }
}

/// Generate next steps for specific spec loading
fn generate_spec_next_steps(project_name: &str, spec_name: &str) -> Vec<String> {
    vec![
        format!("Spec '{}' loaded successfully", spec_name),
        "Review the specification content and tasks for implementation guidance".to_string(),
        "Use the project summary for additional context".to_string(),
        format!(
            "Create new spec: foundry create-spec {} <feature_name>",
            project_name
        ),
        format!("List all specs: foundry load-spec {}", project_name),
    ]
}

/// Generate workflow hints for spec listing
fn generate_listing_workflow_hints(available_specs: &[SpecInfo]) -> Vec<String> {
    let mut hints = vec![
        "Project summary provides context for all specifications".to_string(),
        "Specifications are timestamped and organized by feature".to_string(),
    ];

    if available_specs.is_empty() {
        hints.push("Start by creating specifications to track development features".to_string());
        hints.push("Each spec includes implementation notes and task lists".to_string());
    } else {
        hints.push(format!("Total specs: {}", available_specs.len()));
        hints.push("Load individual specs to see detailed implementation plans".to_string());
        hints.push("Specs include specification content, notes, and task lists".to_string());
    }

    hints
}

/// Generate workflow hints for specific spec loading
fn generate_spec_workflow_hints(spec_name: &str) -> Vec<String> {
    vec![
        format!("Loaded spec: {}", spec_name),
        "Update task-list.md as work progresses".to_string(),
        "Add notes for design decisions".to_string(),
        "Spec content provides detailed feature requirements".to_string(),
        "Use project summary for broader context during implementation".to_string(),
    ]
}
