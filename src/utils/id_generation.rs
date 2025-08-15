//! ID generation utilities for projects and specifications

use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Generate a timestamped specification ID in the format YYYYMMDD_name
pub fn generate_spec_id(spec_name: &str) -> String {
    let now: DateTime<Utc> = Utc::now();
    let date_str = now.format("%Y%m%d").to_string();

    // Convert spec_name to snake_case if it isn't already
    let snake_case_name = to_snake_case(spec_name);

    format!("{}_{}", date_str, snake_case_name)
}

/// Generate a unique task ID
pub fn generate_task_id() -> String {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.timestamp_millis();
    let random_suffix = (timestamp % 10000) as u32;

    format!("task_{}_{:04}", timestamp, random_suffix)
}

/// Generate a unique note ID
pub fn generate_note_id() -> String {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.timestamp_millis();
    let random_suffix = (timestamp % 10000) as u32;

    format!("note_{}_{:04}", timestamp, random_suffix)
}

/// Convert a string to snake_case format
pub fn to_snake_case(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect::<String>()
        .trim_start_matches('_')
        .to_string()
}

/// Validate that a specification name follows the correct format
pub fn validate_spec_name(spec_name: &str) -> Result<(), String> {
    if spec_name.is_empty() {
        return Err("Specification name cannot be empty".to_string());
    }

    if spec_name.len() > 100 {
        return Err("Specification name cannot exceed 100 characters".to_string());
    }

    // Check for invalid characters
    let invalid_chars: HashSet<char> = ['/', '\\', ':', '*', '?', '"', '<', '>', '|']
        .into_iter()
        .collect();
    for c in spec_name.chars() {
        if invalid_chars.contains(&c) {
            return Err(format!(
                "Specification name contains invalid character: '{}'",
                c
            ));
        }
    }

    // Check for reserved names
    let reserved_names = [
        "project",
        "specs",
        "tech-stack",
        "vision",
        "spec",
        "task-list",
        "notes",
    ];
    if reserved_names.contains(&spec_name.to_lowercase().as_str()) {
        return Err(format!(
            "'{}' is a reserved name and cannot be used for specifications",
            spec_name
        ));
    }

    Ok(())
}

/// Validate that a project name follows the correct format
pub fn validate_project_name(project_name: &str) -> Result<(), String> {
    if project_name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if project_name.len() > 100 {
        return Err("Project name cannot exceed 100 characters".to_string());
    }

    // Check for invalid characters
    let invalid_chars: HashSet<char> = ['/', '\\', ':', '*', '?', '"', '<', '>', '|']
        .into_iter()
        .collect();
    for c in project_name.chars() {
        if invalid_chars.contains(&c) {
            return Err(format!("Project name contains invalid character: '{}'", c));
        }
    }

    // Check for reserved names
    let reserved_names = [
        "project",
        "specs",
        "tech-stack",
        "vision",
        "spec",
        "task-list",
        "notes",
    ];
    if reserved_names.contains(&project_name.to_lowercase().as_str()) {
        return Err(format!(
            "'{}' is a reserved name and cannot be used for projects",
            project_name
        ));
    }

    Ok(())
}

/// Check if a specification ID is valid
pub fn validate_spec_id(spec_id: &str) -> Result<(), String> {
    if spec_id.is_empty() {
        return Err("Specification ID cannot be empty".to_string());
    }

    // Check format: YYYYMMDD_name
    let parts: Vec<&str> = spec_id.split('_').collect();
    if parts.len() < 2 {
        return Err("Specification ID must be in format YYYYMMDD_name".to_string());
    }

    let date_part = parts[0];
    if date_part.len() != 8 || !date_part.chars().all(|c| c.is_ascii_digit()) {
        return Err("Date part must be 8 digits in format YYYYMMDD".to_string());
    }

    // Validate date components
    let year: u32 = date_part[0..4]
        .parse()
        .map_err(|_| "Invalid year in date")?;
    let month: u32 = date_part[4..6]
        .parse()
        .map_err(|_| "Invalid month in date")?;
    let day: u32 = date_part[6..8].parse().map_err(|_| "Invalid day in date")?;

    if year < 2020 || year > 2100 {
        return Err("Year must be between 2020 and 2100".to_string());
    }

    if month < 1 || month > 12 {
        return Err("Month must be between 1 and 12".to_string());
    }

    if day < 1 || day > 31 {
        return Err("Day must be between 1 and 31".to_string());
    }

    // Check name part
    let name_part = parts[1..].join("_");
    if name_part.is_empty() {
        return Err("Name part cannot be empty".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_spec_id() {
        let spec_id = generate_spec_id("test_spec");
        assert!(spec_id.starts_with(&chrono::Utc::now().format("%Y%m%d").to_string()));
        assert!(spec_id.ends_with("test_spec"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("TestSpec"), "test_spec");
        assert_eq!(to_snake_case("test_spec"), "test_spec");
        assert_eq!(to_snake_case("Test"), "test");
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_validate_spec_name() {
        assert!(validate_spec_name("valid_spec").is_ok());
        assert!(validate_spec_name("valid-spec").is_ok());
        assert!(validate_spec_name("validSpec").is_ok());

        assert!(validate_spec_name("").is_err());
        assert!(validate_spec_name("project").is_err());
        assert!(validate_spec_name("specs").is_err());
        assert!(validate_spec_name("invalid/name").is_err());
    }

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("valid_project").is_ok());
        assert!(validate_project_name("valid-project").is_ok());
        assert!(validate_project_name("validProject").is_ok());

        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("project").is_err());
        assert!(validate_project_name("specs").is_err());
        assert!(validate_project_name("invalid/name").is_err());
    }

    #[test]
    fn test_validate_spec_id() {
        let today = chrono::Utc::now().format("%Y%m%d").to_string();
        let valid_id = format!("{}_test_spec", today);

        assert!(validate_spec_id(&valid_id).is_ok());
        assert!(validate_spec_id("20240101_test").is_ok());

        assert!(validate_spec_id("").is_err());
        assert!(validate_spec_id("invalid").is_err());
        assert!(validate_spec_id("20240101").is_err());
        assert!(validate_spec_id("20241301_test").is_err()); // Invalid month
        assert!(validate_spec_id("20240132_test").is_err()); // Invalid day
    }
}
