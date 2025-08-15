//! Validation utilities

/// Validate project name format
pub fn validate_project_name(name: &str) -> bool {
    // TODO: Implement project name validation
    !name.is_empty() && !name.contains('/')
}

/// Validate spec name format (snake_case)
pub fn validate_spec_name(name: &str) -> bool {
    // TODO: Implement spec name validation
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}
