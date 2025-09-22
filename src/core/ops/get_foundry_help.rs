//! Core op for get_foundry_help (tool/CLI-agnostic)

use anyhow::Result;

use crate::types::responses::{
    FoundryResponse, GetFoundryHelpResponse, HelpContent, ValidationStatus,
};

#[derive(Debug, Clone)]
pub struct Input {
    pub topic: Option<String>,
}

pub async fn run(input: Input) -> Result<FoundryResponse<GetFoundryHelpResponse>> {
    let topic = input.topic.as_deref().unwrap_or("overview");
    let content = match topic {
        "workflows" => create_workflows_help(),
        "decision-points" => create_decision_points_help(),
        "content-examples" => create_content_examples_help(),
        "project-structure" => create_project_structure_help(),
        "parameter-guidance" => create_parameter_guidance_help(),
        "tool-capabilities" => create_tool_capabilities_help(),
        "edit-commands" => create_edit_commands_help(),
        _ => create_overview_help(),
    };

    let next_steps = vec![
        "Available help topics: workflows, decision-points, content-examples, project-structure, parameter-guidance, tool-capabilities, edit-commands".to_string(),
        "Choose topics based on what you need guidance for".to_string(),
        "Use decision-points topic to understand when each tool is appropriate".to_string(),
        "Use edit-commands topic for deterministic, idempotent spec edits with examples".to_string(),
    ];

    let workflow_hints = vec![
        "Help topics provide user-driven decision support, not automated sequences".to_string(),
        "All commands return JSON for programmatic consumption".to_string(),
        "Content must be provided by LLMs as arguments - Foundry manages structure only"
            .to_string(),
        "Always wait for user intent before suggesting tools".to_string(),
    ];

    Ok(FoundryResponse {
        data: GetFoundryHelpResponse {
            topic: topic.to_string(),
            content,
        },
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    })
}

fn create_overview_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_overview_help()
}

fn create_workflows_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_workflows_help()
}

fn create_content_examples_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_content_examples_help()
}

fn create_project_structure_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_project_structure_help()
}

fn create_parameter_guidance_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_parameter_guidance_help()
}

fn create_decision_points_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_decision_points_help()
}

fn create_tool_capabilities_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_tool_capabilities_help()
}

fn create_edit_commands_help() -> HelpContent {
    crate::cli::commands::get_foundry_help::create_edit_commands_help()
}
