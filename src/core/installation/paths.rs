//! Platform-specific path detection for MCP server configuration files

use crate::core::installation::utils::{ensure_directory_exists, get_home_dir};
use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Get the configuration directory path for Claude Code
///
/// Claude Code stores user settings in ~/.claude/
/// Can be overridden with CLAUDE_CONFIG_DIR environment variable for testing
pub fn get_claude_code_config_dir() -> Result<PathBuf> {
    if let Ok(test_dir) = env::var("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(test_dir));
    }
    let home = get_home_dir()?;
    Ok(home.join(".claude"))
}

/// Get the MCP configuration file path for Claude Code
///
/// Claude Code uses ~/.claude.json for MCP server configurations.
/// This returns the MCP config file at ~/.claude.json
pub fn get_claude_code_mcp_config_path() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join(".claude.json"))
}

/// Get the configuration directory path for Cursor
///
/// Cursor stores MCP configurations in ~/.cursor/
/// Can be overridden with CURSOR_CONFIG_DIR environment variable for testing
pub fn get_cursor_config_dir() -> Result<PathBuf> {
    if let Ok(test_dir) = env::var("CURSOR_CONFIG_DIR") {
        return Ok(PathBuf::from(test_dir));
    }
    let home = get_home_dir()?;
    Ok(home.join(".cursor"))
}

/// Get the MCP configuration file path for Cursor
pub fn get_cursor_mcp_config_path() -> Result<PathBuf> {
    let config_dir = get_cursor_config_dir()?;
    ensure_directory_exists(&config_dir)?;
    Ok(config_dir.join("mcp.json"))
}

/// Get all supported MCP configuration paths
///
/// Returns the configuration file paths for both Claude Code and Cursor.
/// Claude Code uses ~/.claude.json for MCP server configurations.
pub fn get_all_config_paths() -> Vec<(String, PathBuf)> {
    vec![
        (
            "claude-code".to_string(),
            get_claude_code_mcp_config_path().unwrap_or_default(),
        ),
        (
            "cursor".to_string(),
            get_cursor_mcp_config_path().unwrap_or_default(),
        ),
    ]
}

/// Validate that a configuration directory is writable
pub fn validate_config_dir_writable(config_path: &Path) -> Result<()> {
    let parent_dir = config_path
        .parent()
        .context("Configuration path has no parent directory")?;

    // Try to create the parent directory if it doesn't exist
    ensure_directory_exists(&parent_dir.to_path_buf())?;

    // Test if we can create a temporary file to check write permissions
    let temp_file = parent_dir.join(".foundry_test_write");
    match std::fs::write(&temp_file, b"test") {
        Ok(_) => {
            // Clean up the test file
            let _ = std::fs::remove_file(&temp_file);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(
            "Configuration directory is not writable: {}. Error: {}",
            parent_dir.display(),
            e
        )),
    }
}

/// Get platform-specific information for display
pub fn get_platform_info() -> String {
    format!(
        "{} {} ({})",
        env::consts::OS,
        env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claude_code_config_dir() {
        let result = get_claude_code_config_dir();
        assert!(
            result.is_ok(),
            "Should be able to get Claude Code config dir"
        );
        let path = result.unwrap();
        assert!(path.ends_with(".claude"));
    }

    #[test]
    fn test_get_claude_code_mcp_config_path() {
        let result = get_claude_code_mcp_config_path();
        assert!(
            result.is_ok(),
            "Should be able to get Claude Code MCP config path"
        );
        let path = result.unwrap();
        assert!(path.ends_with(".claude.json"));
        assert!(path.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_get_cursor_config_dir() {
        let result = get_cursor_config_dir();
        assert!(result.is_ok(), "Should be able to get Cursor config dir");
        let path = result.unwrap();
        assert!(path.ends_with(".cursor"));
    }

    #[test]
    fn test_get_cursor_mcp_config_path() {
        let result = get_cursor_mcp_config_path();
        assert!(
            result.is_ok(),
            "Should be able to get Cursor MCP config path"
        );
        let path = result.unwrap();
        assert!(path.ends_with("mcp.json"));
    }

    #[test]
    fn test_get_all_config_paths() {
        let paths = get_all_config_paths();
        assert_eq!(paths.len(), 2, "Should return paths for both environments");

        let environment_names: Vec<&String> = paths.iter().map(|(name, _)| name).collect();
        assert!(environment_names.contains(&&"claude-code".to_string()));
        assert!(environment_names.contains(&&"cursor".to_string()));
    }

    #[test]
    fn test_get_platform_info() {
        let info = get_platform_info();
        assert!(!info.is_empty(), "Platform info should not be empty");
        assert!(
            info.contains(env::consts::OS),
            "Platform info should contain OS"
        );
        assert!(
            info.contains(env::consts::ARCH),
            "Platform info should contain architecture"
        );
    }
}
