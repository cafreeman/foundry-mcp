//! Path manipulation utilities

use anyhow::Result;
use std::path::Path;

/// Normalize project name to kebab-case
pub fn normalize_project_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Validate project name format
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    if name.len() > 50 {
        return Err(anyhow::anyhow!(
            "Project name cannot be longer than 50 characters"
        ));
    }

    // Check for valid characters (alphanumeric, hyphens, underscores)
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            return Err(anyhow::anyhow!(
                "Project name can only contain alphanumeric characters, hyphens, and underscores"
            ));
        }
    }

    // Check for consecutive special characters
    if name.contains("--") || name.contains("__") || name.contains("_-") || name.contains("-_") {
        return Err(anyhow::anyhow!(
            "Project name cannot contain consecutive special characters"
        ));
    }

    // Check that it starts and ends with alphanumeric
    let first_char_valid = name
        .chars()
        .next()
        .map(|c| c.is_alphanumeric())
        .unwrap_or(false);
    let last_char_valid = name
        .chars()
        .last()
        .map(|c| c.is_alphanumeric())
        .unwrap_or(false);

    if !first_char_valid || !last_char_valid {
        return Err(anyhow::anyhow!(
            "Project name must start and end with alphanumeric characters"
        ));
    }

    Ok(())
}

/// Validate feature name for specs
pub fn validate_feature_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Feature name cannot be empty"));
    }

    if name.len() > 50 {
        return Err(anyhow::anyhow!(
            "Feature name cannot be longer than 50 characters"
        ));
    }

    // Feature names should be snake_case (lowercase alphanumeric with underscores)
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '_' {
            return Err(anyhow::anyhow!(
                "Feature name can only contain alphanumeric characters and underscores"
            ));
        }
        if c.is_alphabetic() && c.is_uppercase() {
            return Err(anyhow::anyhow!(
                "Feature name must be in snake_case (lowercase letters, numbers, and underscores only)"
            ));
        }
    }

    // Can't start or end with underscore
    if name.starts_with('_') || name.ends_with('_') {
        return Err(anyhow::anyhow!(
            "Feature name cannot start or end with an underscore"
        ));
    }

    // Can't have consecutive underscores
    if name.contains("__") {
        return Err(anyhow::anyhow!(
            "Feature name cannot contain consecutive underscores"
        ));
    }

    Ok(())
}

/// Get relative path from foundry directory
pub fn relative_to_foundry(path: &Path) -> Result<String> {
    let foundry_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".foundry");

    let relative_path = path
        .strip_prefix(&foundry_dir)
        .map_err(|_| anyhow::anyhow!("Path is not within foundry directory"))?;

    Ok(relative_path.to_string_lossy().to_string())
}

/// Ensure path is safe (doesn't escape the foundry directory)
pub fn ensure_safe_path(path: &Path) -> Result<()> {
    let foundry_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".foundry");

    if !path.starts_with(&foundry_dir) {
        return Err(anyhow::anyhow!("Path is outside of foundry directory"));
    }

    // Check for path traversal attempts
    let path_str = path.to_string_lossy();
    if path_str.contains("../") || path_str.contains("..\\") {
        return Err(anyhow::anyhow!("Path contains directory traversal"));
    }

    Ok(())
}
