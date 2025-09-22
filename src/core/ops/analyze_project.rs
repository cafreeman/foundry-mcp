//! Core op for analyze_project (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::foundry;
use crate::core::validation::{ContentType, validate_content};
use crate::types::project::ProjectConfig;
use crate::types::responses::{AnalyzeProjectResponse, FoundryResponse, ValidationStatus};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<AnalyzeProjectResponse>> {
    let foundry = foundry::get_default_foundry()?;
    
    validate_project_name(&input.project_name).with_context(|| "Project name validation failed")?;

    validate_content_sizes(&input.vision, &input.tech_stack, &input.summary)
        .with_context(|| "Content size validation failed")?;

    if foundry.project_exists(&input.project_name).await
        .with_context(|| format!("Failed to check if project '{}' exists", input.project_name))?
    {
        return Err(anyhow::anyhow!(
            "Project '{}' already exists. Use MCP to discover existing projects: {{\"name\": \"list_projects\", \"arguments\": {{}}}} or choose a different name.",
            input.project_name
        ));
    }

    let vision_validation = validate_content(ContentType::Vision, &input.vision);
    let tech_stack_validation = validate_content(ContentType::TechStack, &input.tech_stack);
    let summary_validation = validate_content(ContentType::Summary, &input.summary);

    let mut validation_errors = Vec::new();
    if !vision_validation.is_valid {
        validation_errors.extend(vision_validation.errors);
    }
    if !tech_stack_validation.is_valid {
        validation_errors.extend(tech_stack_validation.errors);
    }
    if !summary_validation.is_valid {
        validation_errors.extend(summary_validation.errors);
    }

    if !validation_errors.is_empty() {
        let error_count = validation_errors.len();
        return Err(anyhow::anyhow!(
            "Content validation failed with {} error(s):\n{}",
            error_count,
            validation_errors.join("\n")
        ));
    }

    // Create project using the foundry façade
    let project_config = ProjectConfig {
        name: input.project_name.clone(),
        vision: input.vision,
        tech_stack: input.tech_stack,
        summary: input.summary,
    };
    
    foundry.create_project(project_config).await
        .with_context(|| format!("Failed to create project '{}'", input.project_name))?;

    let files_created = vec![
        "vision.md".to_string(),
        "tech-stack.md".to_string(),
        "summary.md".to_string(),
        "specs/".to_string(),
    ];

    let response_data = AnalyzeProjectResponse {
        project_name: input.project_name.clone(),
        files_created,
    };

    let next_steps = vec![
        format!(
            "Project '{}' analyzed and structure created from your codebase analysis",
            input.project_name
        ),
        "Your analyzed content has been structured and is ready for development work".to_string(),
        format!(
            "Create a spec: {{\"name\": \"create_spec\", \"arguments\": {{\"project_name\": \"{}\", \"feature_name\": \"<feature>\", \"spec\": \"...\", \"tasks\": \"...\", \"notes\": \"...\"}}}}; Load project: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}; Or continue codebase analysis",
            input.project_name, input.project_name
        ),
    ];

    let mut workflow_hints: Vec<String> = vec![
        "Project structure created based on your codebase analysis".to_string(),
        format!("Create a spec when you identify a feature: {{\"name\": \"create_spec\", \"arguments\": {{\"project_name\": \"{}\", \"feature_name\": \"<feature>\", \"spec\": \"...\", \"tasks\": \"...\", \"notes\": \"...\"}}}}", input.project_name),
        format!("Load full project context: {{\"name\": \"load_project\", \"arguments\": {{\"project_name\": \"{}\"}}}}", input.project_name),
        "You can continue using codebase_search, grep_search, read_file for deeper analysis".to_string(),
        "Tool selection guidance: {\"name\": \"get_foundry_help\", \"arguments\": {\"topic\": \"decision-points\"}}".to_string(),
    ];

    workflow_hints.extend(vision_validation.suggestions);
    workflow_hints.extend(tech_stack_validation.suggestions);
    workflow_hints.extend(summary_validation.suggestions);

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    })
}

fn validate_project_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Project name cannot be empty. Please provide a descriptive project name."
        ));
    }
    if name.len() > 100 {
        return Err(anyhow::anyhow!(
            "Project name too long ({} characters). Please keep it under 100 characters for filesystem compatibility.",
            name.len()
        ));
    }
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if let Some(invalid_char) = name.chars().find(|c| invalid_chars.contains(c)) {
        return Err(anyhow::anyhow!(
            "Project name contains invalid character '{}'. Please use only letters, numbers, hyphens, and underscores.",
            invalid_char
        ));
    }
    let reserved_names = ["CON", "PRN", "AUX", "NUL"];
    let upper_name = name.to_uppercase();
    if reserved_names.contains(&upper_name.as_str()) {
        return Err(anyhow::anyhow!(
            "Project name '{}' is reserved by the operating system. Please choose a different name.",
            name
        ));
    }
    Ok(())
}

fn validate_content_sizes(vision: &str, tech_stack: &str, summary: &str) -> Result<()> {
    const MAX_CONTENT_SIZE: usize = 50_000;
    if vision.len() > MAX_CONTENT_SIZE {
        return Err(anyhow::anyhow!(
            "Vision content too large ({} characters). Please keep it under {} characters for optimal performance.",
            vision.len(),
            MAX_CONTENT_SIZE
        ));
    }
    if tech_stack.len() > MAX_CONTENT_SIZE {
        return Err(anyhow::anyhow!(
            "Tech stack content too large ({} characters). Please keep it under {} characters for optimal performance.",
            tech_stack.len(),
            MAX_CONTENT_SIZE
        ));
    }
    if summary.len() > MAX_CONTENT_SIZE {
        return Err(anyhow::anyhow!(
            "Summary content too large ({} characters). Please keep it under {} characters for optimal performance.",
            summary.len(),
            MAX_CONTENT_SIZE
        ));
    }
    Ok(())
}
