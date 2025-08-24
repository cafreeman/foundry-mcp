//! Content validation logic

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
    let mut errors = Vec::new();
    let mut suggestions = Vec::new();

    // Check length
    if content.len() < 200 {
        errors.push("Vision content must be at least 200 characters".to_string());
    }

    // Check for multiple paragraphs
    let paragraphs: Vec<&str> = content
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    if paragraphs.len() < 2 {
        suggestions.push(
            "Consider adding more paragraphs to provide comprehensive vision coverage".to_string(),
        );
    }

    // Check for key elements
    let lower_content = content.to_lowercase();
    if !lower_content.contains("problem") && !lower_content.contains("solve") {
        suggestions.push("Consider including what problem this solves".to_string());
    }

    if !lower_content.contains("target") || !lower_content.contains("user") {
        suggestions.push("Consider specifying target users or audience".to_string());
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate tech stack content (150+ characters)
fn validate_tech_stack_content(content: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut suggestions = Vec::new();

    // Check length
    if content.len() < 150 {
        errors.push("Tech stack content must be at least 150 characters".to_string());
    }

    // Check for technology mentions
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

    if !has_tech {
        suggestions.push(
            "Consider including specific technologies, frameworks, or deployment platforms"
                .to_string(),
        );
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate summary content (100+ characters, concise)
fn validate_summary_content(content: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut suggestions = Vec::new();

    // Check length
    if content.len() < 100 {
        errors.push("Summary content must be at least 100 characters".to_string());
    }

    if content.len() > 500 {
        suggestions
            .push("Consider making the summary more concise (under 500 characters)".to_string());
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate spec content
fn validate_spec_content(content: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut suggestions = Vec::new();

    // Check length
    if content.len() < 100 {
        errors.push("Spec content must be at least 100 characters".to_string());
    }

    // Check for structure
    let lower_content = content.to_lowercase();
    let structure_keywords = ["requirements", "functionality", "behavior", "interface"];
    let has_structure = structure_keywords
        .iter()
        .any(|&keyword| lower_content.contains(keyword));

    if !has_structure {
        suggestions.push(
            "Consider adding requirements, functionality, or behavioral specifications".to_string(),
        );
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        suggestions,
    }
}

/// Validate notes content
fn validate_notes_content(content: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let suggestions = Vec::new();

    // Notes can be shorter but should have some substance
    if content.len() < 50 {
        errors.push("Notes content must be at least 50 characters".to_string());
    }

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
