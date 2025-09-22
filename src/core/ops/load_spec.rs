//! Core op for loading specs or listing available specs (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::{foundry, spec};
use crate::types::responses::{
    FoundryResponse, LoadSpecResponse, SpecContent, SpecInfo, ValidationStatus,
};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub spec_name: Option<String>,
}

pub async fn run(input: Input) -> Result<FoundryResponse<LoadSpecResponse>> {
    let foundry = foundry::get_default_foundry()?;
    
    validate_project_exists(&foundry, &input.project_name).await?;

    let project_summary = load_project_summary(&foundry, &input.project_name).await?;

    match &input.spec_name {
        None => {
            let specs = foundry.list_specs(&input.project_name).await?;
            let available_specs: Vec<SpecInfo> = specs
                .into_iter()
                .map(|spec_meta| SpecInfo {
                    name: spec_meta.name,
                    feature_name: spec_meta.feature_name,
                    created_at: spec_meta.created_at,
                })
                .collect();

            let response_data = LoadSpecResponse {
                project_name: input.project_name.clone(),
                project_summary,
                spec_name: None,
                created_at: None,
                spec_content: None,
                available_specs: available_specs.clone(),
                match_info: None,
            };

            Ok(FoundryResponse {
                data: response_data,
                next_steps: generate_listing_next_steps(&input.project_name, &available_specs),
                validation_status: if available_specs.is_empty() {
                    ValidationStatus::Incomplete
                } else {
                    ValidationStatus::Complete
                },
                workflow_hints: generate_listing_workflow_hints(&available_specs),
            })
        }
        Some(spec_name) => {
            let match_strategy = foundry
                .find_spec_match(&input.project_name, spec_name)
                .await?;
                
            let (spec_data, match_strategy) = match match_strategy {
                spec::SpecMatchStrategy::None => {
                    return Err(anyhow::anyhow!(
                        "No spec found matching '{}' in project '{}'",
                        spec_name,
                        input.project_name
                    ));
                }
                spec::SpecMatchStrategy::Multiple(candidates) => {
                    return Err(anyhow::anyhow!(
                        "Multiple specs match '{}': {}. Please be more specific.",
                        spec_name,
                        candidates.join(", ")
                    ));
                }
                spec::SpecMatchStrategy::Exact(actual_name) => {
                    let spec_data = foundry
                        .load_spec(&input.project_name, &actual_name)
                        .await
                        .with_context(|| format!("Failed to load spec '{}'", actual_name))?;
                    (spec_data, spec::SpecMatchStrategy::Exact(actual_name))
                }
                spec::SpecMatchStrategy::FeatureExact(actual_name) => {
                    let spec_data = foundry
                        .load_spec(&input.project_name, &actual_name)
                        .await
                        .with_context(|| format!("Failed to load spec '{}'", actual_name))?;
                    (spec_data, spec::SpecMatchStrategy::FeatureExact(actual_name))
                }
                spec::SpecMatchStrategy::FeatureFuzzy(actual_name) => {
                    let spec_data = foundry
                        .load_spec(&input.project_name, &actual_name)
                        .await
                        .with_context(|| format!("Failed to load spec '{}'", actual_name))?;
                    (spec_data, spec::SpecMatchStrategy::FeatureFuzzy(actual_name))
                }
                spec::SpecMatchStrategy::NameFuzzy(actual_name) => {
                    let spec_data = foundry
                        .load_spec(&input.project_name, &actual_name)
                        .await
                        .with_context(|| format!("Failed to load spec '{}'", actual_name))?;
                    (spec_data, spec::SpecMatchStrategy::NameFuzzy(actual_name))
                }
            };

            let spec_content = SpecContent {
                content: spec_data.content,
            };

            let match_info = match match_strategy {
                spec::SpecMatchStrategy::Exact(_) => None,
                _ => Some(crate::types::responses::MatchInfo {
                    requested_spec: spec_name.clone(),
                    matched_spec: spec_data.name.clone(),
                    match_type: match match_strategy {
                        spec::SpecMatchStrategy::FeatureExact(_) => "feature_exact".to_string(),
                        spec::SpecMatchStrategy::FeatureFuzzy(_) => "feature_fuzzy".to_string(),
                        spec::SpecMatchStrategy::NameFuzzy(_) => "name_fuzzy".to_string(),
                        _ => "exact".to_string(),
                    },
                    confidence: 1.0,
                }),
            };

            let response_data = LoadSpecResponse {
                project_name: input.project_name.clone(),
                project_summary,
                spec_name: Some(spec_data.name.clone()),
                created_at: Some(spec_data.created_at.clone()),
                spec_content: Some(spec_content),
                available_specs: Vec::new(),
                match_info,
            };

            Ok(FoundryResponse {
                data: response_data,
                next_steps: generate_spec_next_steps(&input.project_name, &spec_data.name),
                validation_status: ValidationStatus::Complete,
                workflow_hints: generate_spec_workflow_hints(&spec_data.name),
            })
        }
    }
}

async fn validate_project_exists(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<()> {
    if !foundry.project_exists(project_name).await? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'mcp_foundry_list_projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

async fn load_project_summary(
    foundry: &foundry::Foundry<crate::core::backends::filesystem::FilesystemBackend>,
    project_name: &str,
) -> Result<String> {
    let project = foundry.load_project(project_name).await?;
    
    Ok(project.summary.unwrap_or_else(|| {
        "No project summary available. Consider updating the project summary for better context.".to_string()
    }))
}

fn generate_listing_next_steps(project_name: &str, available_specs: &[SpecInfo]) -> Vec<String> {
    if available_specs.is_empty() {
        vec![
            "No specifications found for this project - ready for specification creation"
                .to_string(),
            format!(
                "You can create your first specification: mcp_foundry_create_spec {} <feature_name>",
                project_name
            ),
            "You can use 'mcp_foundry_load_project' to see full project context".to_string(),
        ]
    } else {
        let mut steps = vec![
            format!(
                "Found {} specification(s) in project",
                available_specs.len()
            ),
            format!(
                "You can load a specific spec: mcp_foundry_load_spec {} <spec_name>",
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
            "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
            project_name
        ));

        steps
    }
}

fn generate_spec_next_steps(project_name: &str, spec_name: &str) -> Vec<String> {
    vec![
        format!("Spec '{}' loaded successfully", spec_name),
        "You can review the specification content and tasks for implementation guidance"
            .to_string(),
        "You can use the project summary for additional context".to_string(),
        format!(
            "You can create a new spec: mcp_foundry_create_spec {} <feature_name>",
            project_name
        ),
        format!(
            "You can list all specs: mcp_foundry_load_spec {}",
            project_name
        ),
    ]
}

fn generate_listing_workflow_hints(available_specs: &[SpecInfo]) -> Vec<String> {
    let mut hints = vec![
        "You can use the project summary for context about all specifications".to_string(),
        "Specifications are timestamped and organized by feature for easy navigation".to_string(),
    ];

    if available_specs.is_empty() {
        hints.push(
            "You can start by creating specifications to track development features".to_string(),
        );
        hints.push(
            "Each spec includes implementation notes and task lists for comprehensive planning"
                .to_string(),
        );
    } else {
        hints.push(format!("Total specs: {}", available_specs.len()));
        hints
            .push("You can load individual specs to see detailed implementation plans".to_string());
        hints.push(
            "Specs include specification content, notes, and task lists for complete context"
                .to_string(),
        );
    }

    hints
}

fn generate_spec_workflow_hints(spec_name: &str) -> Vec<String> {
    vec![
        format!("Loaded spec: {}", spec_name),
        "You must update task-list.md as work progresses".to_string(),
        "You can add notes for design decisions and implementation details".to_string(),
        "Spec content provides detailed feature requirements and acceptance criteria".to_string(),
        "You can use the project summary for broader context during implementation".to_string(),
    ]
}
