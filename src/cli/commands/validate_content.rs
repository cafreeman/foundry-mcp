//! Implementation of the validate_content command
//!
//! This command validates LLM-provided content against Foundry's validation rules
//! and provides improvement suggestions.

use crate::cli::args::ValidateContentArgs;
use crate::core::validation::{parse_content_type, validate_content};
use crate::types::responses::{FoundryResponse, ValidateContentResponse, ValidationStatus};
use anyhow::{Context, Result};

/// Validate input arguments for validate_content command
fn validate_input_args(content_type: &str, content: &str) -> Result<()> {
    // Check for empty content type
    if content_type.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Content type cannot be empty. Supported types: vision, tech-stack, summary, spec, notes, tasks"
        ));
    }

    // Check for extremely large content
    const MAX_VALIDATION_SIZE: usize = 100_000; // 100KB for validation
    if content.len() > MAX_VALIDATION_SIZE {
        return Err(anyhow::anyhow!(
            "Content too large for validation ({} characters). Maximum size for validation is {} characters.",
            content.len(),
            MAX_VALIDATION_SIZE
        ));
    }

    // Check for binary content (basic check)
    if content.contains('\0') {
        return Err(anyhow::anyhow!(
            "Content appears to contain binary data. Only text content can be validated."
        ));
    }

    Ok(())
}

pub async fn execute(
    args: ValidateContentArgs,
) -> Result<FoundryResponse<ValidateContentResponse>> {
    // Enhanced input validation
    validate_input_args(&args.content_type, &args.content)
        .with_context(|| "Input validation failed")?;

    // Parse content type with enhanced error handling
    let content_type = parse_content_type(&args.content_type)
        .with_context(|| {
            format!(
                "Invalid content type '{}'. Supported types are: vision, tech-stack, summary, spec, notes, tasks",
                args.content_type
            )
        })?;

    // Validate content with context
    let validation_result = validate_content(content_type, &args.content);

    // Prepare enhanced response with additional context
    let response_data = ValidateContentResponse {
        content_type: args.content_type.clone(),
        is_valid: validation_result.is_valid,
        validation_errors: validation_result.errors.clone(),
        suggestions: validation_result.suggestions.clone(),
    };

    // Determine validation status with enhanced logic
    let validation_status = if validation_result.is_valid {
        ValidationStatus::Complete // Valid content, regardless of suggestions
    } else {
        ValidationStatus::Error
    };

    // Generate enhanced next steps based on validation results
    let next_steps = if validation_result.is_valid {
        let mut steps =
            vec!["Content validation passed - ready to use in project creation".to_string()];

        if !validation_result.suggestions.is_empty() {
            steps.push(format!(
                "Consider incorporating {} suggestions to improve content quality",
                validation_result.suggestions.len()
            ));
        }

        steps.push(
            "Use this content with 'foundry create_project' or 'foundry analyze_project'"
                .to_string(),
        );
        steps
    } else {
        let error_count = validation_result.errors.len();
        let suggestion_count = validation_result.suggestions.len();

        let mut steps = vec![format!(
            "Fix {} validation error(s) before using this content",
            error_count
        )];

        if suggestion_count > 0 {
            steps.push(format!(
                "Review {} suggestion(s) for improvement guidance",
                suggestion_count
            ));
        }

        steps.push("Re-run validation after making changes".to_string());
        steps
    };

    // Enhanced workflow hints with content-specific guidance
    let mut workflow_hints = vec![
        "Use this command to pre-validate content before project operations".to_string(),
        "Validation helps ensure content meets Foundry's structural requirements".to_string(),
    ];

    // Add content-type specific hints
    match args.content_type.as_str() {
        "vision" => workflow_hints.push(
            "Vision should describe the problem, target users, and value proposition".to_string(),
        ),
        "tech-stack" => workflow_hints.push(
            "Tech stack should include languages, frameworks, and deployment decisions".to_string(),
        ),
        "summary" => workflow_hints
            .push("Summary should be concise but capture key project insights".to_string()),
        "spec" => workflow_hints.push(
            "Spec should include clear requirements and functionality descriptions".to_string(),
        ),
        "notes" => workflow_hints
            .push("Notes provide additional context and implementation considerations".to_string()),
        "tasks" => workflow_hints
            .push("Tasks should be actionable items with clear completion criteria".to_string()),
        _ => workflow_hints
            .push("Follow the content guidelines for your specific content type".to_string()),
    }

    workflow_hints
        .push("Content validation is performed client-side for immediate feedback".to_string());

    Ok(FoundryResponse {
        data: response_data,
        next_steps,
        validation_status,
        workflow_hints,
    })
}
