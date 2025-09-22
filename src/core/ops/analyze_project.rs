//! Core op for analyze_project (tool-agnostic)

use anyhow::{Context, Result};

use crate::core::filesystem::{foundry_dir, write_file_atomic};
use crate::core::project::project_exists;
use crate::core::validation::{ContentType, validate_content};
use crate::types::responses::{AnalyzeProjectResponse, FoundryResponse, ValidationStatus};

#[derive(Debug, Clone)]
pub struct Input {
    pub project_name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
}

pub async fn run(input: Input) -> Result<FoundryResponse<AnalyzeProjectResponse>> {
    validate_project_name(&input.project_name).with_context(|| "Project name validation failed")?;

    validate_content_sizes(&input.vision, &input.tech_stack, &input.summary)
        .with_context(|| "Content size validation failed")?;

    if project_exists(&input.project_name)
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

    let foundry_path = foundry_dir().with_context(
        || "Failed to access or create foundry directory (~/.foundry/). Check file permissions.",
    )?;
    let project_path = foundry_path.join(&input.project_name);

    std::fs::create_dir_all(&project_path).with_context(|| {
        format!(
            "Failed to create project directory '{}'. Possible causes:\n- Insufficient disk space\n- Permission denied\n- Invalid project name characters\n- Path too long for filesystem",
            project_path.display()
        )
    })?;

    let specs_dir = project_path.join("specs");
    std::fs::create_dir_all(&specs_dir).with_context(|| {
        format!(
            "Failed to create specs directory '{}'. The project directory was created, but specs creation failed. Check disk space and permissions.",
            specs_dir.display()
        )
    })?;

    let vision_path = project_path.join("vision.md");
    let tech_stack_path = project_path.join("tech-stack.md");
    let summary_path = project_path.join("summary.md");

    write_file_atomic(&vision_path, &input.vision).with_context(|| {
        format!(
            "Failed to write vision.md ({}). File creation failed after directory setup. Check disk space and file permissions.",
            vision_path.display()
        )
    })?;

    write_file_atomic(&tech_stack_path, &input.tech_stack).with_context(|| {
        format!(
            "Failed to write tech-stack.md ({}). Vision file was created successfully. Check disk space and file permissions.",
            tech_stack_path.display()
        )
    })?;

    write_file_atomic(&summary_path, &input.summary).with_context(|| {
        format!(
            "Failed to write summary.md ({}). Vision and tech-stack files were created successfully. Check disk space and file permissions.",
            summary_path.display()
        )
    })?;

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
