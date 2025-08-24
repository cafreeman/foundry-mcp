//! Content validation logic

use crate::utils::validation::{
    conditional_error, conditional_suggestion, conditional_suggestions,
};

/// Content types that can be validated
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    Vision,
    TechStack,
    Summary,
    Spec,
    Notes,
}

/// Validation result
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Validate content based on type
pub fn validate_content(content_type: ContentType, content: &str) -> ValidationResult {
    match content_type {
        ContentType::Vision => validate_vision_content(content),
        ContentType::TechStack => validate_tech_stack_content(content),
        ContentType::Summary => validate_summary_content(content),
        ContentType::Spec => validate_spec_content(content),
        ContentType::Notes => validate_notes_content(content),
    }
}

/// Validate vision content (2-4 paragraphs, 200+ characters)
fn validate_vision_content(content: &str) -> ValidationResult {
    let errors = conditional_error(
        content.len() < 200,
        "Vision content must be at least 200 characters",
    );

    let paragraphs_count = content
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .count();

    let lower_content = content.to_lowercase();

    let suggestions = conditional_suggestions(&[
        (
            paragraphs_count < 2,
            "Consider adding more paragraphs to provide comprehensive vision coverage",
        ),
        (
            !lower_content.contains("problem") && !lower_content.contains("solve"),
            "Consider including what problem this solves",
        ),
        (
            !lower_content.contains("target") && !lower_content.contains("user"),
            "Consider specifying target users or audience",
        ),
    ]);

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate tech stack content (150+ characters)
fn validate_tech_stack_content(content: &str) -> ValidationResult {
    let errors = conditional_error(
        content.len() < 150,
        "Tech stack content must be at least 150 characters",
    );

    let lower_content = content.to_lowercase();
    let tech_keywords = [
        "language",
        "framework",
        "database",
        "deployment",
        "infrastructure",
    ];
    let has_tech = tech_keywords
        .iter()
        .any(|&keyword| lower_content.contains(keyword));

    let suggestions = conditional_suggestion(
        !has_tech,
        "Consider including specific technologies, frameworks, or deployment platforms",
    );

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate summary content (100+ characters, concise)
fn validate_summary_content(content: &str) -> ValidationResult {
    let errors = conditional_error(
        content.len() < 100,
        "Summary content must be at least 100 characters",
    );

    let suggestions = conditional_suggestion(
        content.len() > 500,
        "Consider making the summary more concise (under 500 characters)",
    );

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate spec content
fn validate_spec_content(content: &str) -> ValidationResult {
    let errors = conditional_error(
        content.len() < 100,
        "Spec content must be at least 100 characters",
    );

    let lower_content = content.to_lowercase();
    let structure_keywords = ["requirements", "functionality", "behavior", "interface"];
    let has_structure = structure_keywords
        .iter()
        .any(|&keyword| lower_content.contains(keyword));

    let suggestions = conditional_suggestion(
        !has_structure,
        "Consider adding requirements, functionality, or behavioral specifications",
    );

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate notes content
fn validate_notes_content(content: &str) -> ValidationResult {
    let errors = conditional_error(
        content.len() < 50,
        "Notes content must be at least 50 characters",
    );

    let suggestions = Vec::new();

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Convert content type string to enum
pub fn parse_content_type(content_type: &str) -> anyhow::Result<ContentType> {
    match content_type.to_lowercase().as_str() {
        "vision" => Ok(ContentType::Vision),
        "tech-stack" => Ok(ContentType::TechStack),
        "summary" => Ok(ContentType::Summary),
        "spec" => Ok(ContentType::Spec),
        "notes" => Ok(ContentType::Notes),
        _ => Err(anyhow::anyhow!("Unknown content type: {}", content_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_vision_content_too_short() {
        let content = "Too short";
        let result = validate_vision_content(content);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("200 characters"));
    }

    #[test]
    fn test_validate_tech_stack_content_too_short() {
        let content = "Too short";
        let result = validate_tech_stack_content(content);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("150 characters"));
    }

    #[test]
    fn test_validate_summary_content_too_short() {
        let content = "Too short";
        let result = validate_summary_content(content);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("100 characters"));
    }

    #[test]
    fn test_validate_spec_content_too_short() {
        let content = "Too short";
        let result = validate_spec_content(content);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("100 characters"));
    }

    #[test]
    fn test_validate_notes_content_too_short() {
        let content = "Too short";
        let result = validate_notes_content(content);

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("50 characters"));
    }

    #[test]
    fn test_parse_content_type_valid() {
        assert!(matches!(
            parse_content_type("vision"),
            Ok(ContentType::Vision)
        ));
        assert!(matches!(
            parse_content_type("tech-stack"),
            Ok(ContentType::TechStack)
        ));
        assert!(matches!(
            parse_content_type("summary"),
            Ok(ContentType::Summary)
        ));
        assert!(matches!(parse_content_type("spec"), Ok(ContentType::Spec)));
        assert!(matches!(
            parse_content_type("notes"),
            Ok(ContentType::Notes)
        ));
    }

    #[test]
    fn test_parse_content_type_case_insensitive() {
        assert!(matches!(
            parse_content_type("VISION"),
            Ok(ContentType::Vision)
        ));
        assert!(matches!(
            parse_content_type("Tech-Stack"),
            Ok(ContentType::TechStack)
        ));
        assert!(matches!(
            parse_content_type("SUMMARY"),
            Ok(ContentType::Summary)
        ));
    }

    #[test]
    fn test_parse_content_type_invalid() {
        assert!(parse_content_type("invalid").is_err());
        assert!(parse_content_type("").is_err());
        assert!(parse_content_type("random").is_err());
    }
}
