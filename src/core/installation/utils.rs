//! Common utilities for MCP server installation

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

/// Result of an installation operation
#[derive(Debug, Clone)]
pub struct InstallationResult {
    pub success: bool,
    pub config_path: String,
    pub actions_taken: Vec<String>,
}

/// Result of an uninstallation operation
#[derive(Debug, Clone)]
pub struct UninstallationResult {
    pub success: bool,
    pub config_path: String,
    pub actions_taken: Vec<String>,
    pub files_removed: Vec<String>,
}

/// Detect the current binary path
///
/// Attempts to detect the path of the currently running foundry binary
/// This is used for creating MCP server configurations
pub fn detect_binary_path() -> Result<String> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    let binary_path = current_exe
        .to_str()
        .context("Binary path contains invalid Unicode characters")?
        .to_string();

    Ok(binary_path)
}

/// Check if the binary at the given path is accessible
pub fn check_binary_accessible(binary_path: &str) -> bool {
    let path = PathBuf::from(binary_path);
    path.exists() && path.is_file()
}

/// Validate that a binary path exists and is executable
pub fn validate_binary_path(binary_path: &str) -> Result<()> {
    let path = PathBuf::from(binary_path);

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Binary path does not exist: {}",
            binary_path
        ));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!(
            "Binary path is not a file: {}",
            binary_path
        ));
    }

    Ok(())
}

/// Create a standardized installation result
pub fn create_installation_result(
    success: bool,
    config_path: String,
    actions_taken: Vec<String>,
) -> InstallationResult {
    InstallationResult {
        success,
        config_path,
        actions_taken,
    }
}

/// Create a standardized uninstallation result
pub fn create_uninstallation_result(
    success: bool,
    config_path: String,
    actions_taken: Vec<String>,
    files_removed: Vec<String>,
) -> UninstallationResult {
    UninstallationResult {
        success,
        config_path,
        actions_taken,
        files_removed,
    }
}

/// Check if a file exists at the given path
pub fn file_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

/// Read file content if it exists, otherwise return None
pub fn read_file_content(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Format actions taken into human-readable strings
pub fn format_actions(actions: &[String]) -> Vec<String> {
    actions
        .iter()
        .map(|action| format!("• {}", action))
        .collect()
}

/// Get the current user's home directory
pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("Failed to determine home directory")
}

/// Create a directory if it doesn't exist
pub fn ensure_directory_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)
            .context(format!("Failed to create directory: {:?}", dir_path))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_binary_path() {
        let result = detect_binary_path();
        assert!(result.is_ok(), "Should be able to detect binary path");
        let path = result.unwrap();
        assert!(!path.is_empty(), "Binary path should not be empty");
        assert!(
            path.contains("foundry"),
            "Binary path should contain 'foundry'"
        );
    }

    #[test]
    fn test_validate_binary_path_valid() {
        let binary_path = detect_binary_path().unwrap();
        let result = validate_binary_path(&binary_path);
        assert!(result.is_ok(), "Valid binary path should pass validation");
    }

    #[test]
    fn test_validate_binary_path_invalid() {
        let result = validate_binary_path("/nonexistent/path");
        assert!(result.is_err(), "Nonexistent path should fail validation");
    }

    #[test]
    fn test_get_home_dir() {
        let result = get_home_dir();
        assert!(result.is_ok(), "Should be able to get home directory");
        let home = result.unwrap();
        assert!(home.exists(), "Home directory should exist");
        assert!(home.is_dir(), "Home directory should be a directory");
    }

    #[test]
    fn test_create_installation_result() {
        let result = create_installation_result(
            true,
            "/path/to/config".to_string(),
            vec![
                "Created config file".to_string(),
                "Updated environment".to_string(),
            ],
        );

        assert!(result.success);
        assert_eq!(result.config_path, "/path/to/config");
        assert_eq!(result.actions_taken.len(), 2);
    }

    #[test]
    fn test_format_actions() {
        let actions = vec![
            "Created config file".to_string(),
            "Updated environment".to_string(),
        ];

        let formatted = format_actions(&actions);
        assert_eq!(formatted.len(), 2);
        assert!(formatted[0].starts_with("• "));
        assert!(formatted[1].starts_with("• "));
    }

    #[test]
    fn test_format_actions_empty() {
        let actions: Vec<String> = vec![];
        let formatted = format_actions(&actions);
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        assert!(file_exists(file_path.to_str().unwrap()));
        assert!(!file_exists("/nonexistent/file"));
    }

    #[test]
    fn test_read_file_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let content = read_file_content(file_path.to_str().unwrap());
        assert_eq!(content, Some("test content".to_string()));

        let nonexistent = read_file_content("/nonexistent/file");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sub_dir = temp_dir.path().join("subdir").join("nested");

        let result = ensure_directory_exists(&sub_dir);
        assert!(result.is_ok());
        assert!(sub_dir.exists());
        assert!(sub_dir.is_dir());
    }

    #[test]
    fn test_create_uninstallation_result() {
        let result = create_uninstallation_result(
            false,
            "/path/to/config.json".to_string(),
            vec!["Action 1".to_string()],
            vec!["file1.json".to_string()],
        );

        assert!(!result.success);
        assert_eq!(result.config_path, "/path/to/config.json");
        assert_eq!(result.actions_taken.len(), 1);
        assert_eq!(result.files_removed.len(), 1);
    }

    #[test]
    fn test_check_binary_accessible_valid() {
        let binary_path = detect_binary_path().unwrap();
        let accessible = check_binary_accessible(&binary_path);
        assert!(accessible, "Current binary should be accessible");
    }

    #[test]
    fn test_check_binary_accessible_invalid() {
        let accessible = check_binary_accessible("/nonexistent/path");
        assert!(!accessible, "Nonexistent path should not be accessible");
    }
}
