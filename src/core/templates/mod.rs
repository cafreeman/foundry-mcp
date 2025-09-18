//! Template system for client-specific configuration files
//!
//! This module provides a trait-based system for managing embedded templates
//! that are installed alongside MCP server configurations.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Trait for client-specific template implementations
///
/// Each client (Cursor, Claude Code, etc.) implements this trait to provide
/// their specific template content and file paths.
pub trait ClientTemplate {
    /// Get the embedded template content as a static string
    ///
    /// This content is compiled into the binary and should contain all
    /// necessary configuration and guidance for the specific client.
    fn content() -> &'static str;

    /// Get the file path where this template should be installed
    ///
    /// # Arguments
    /// * `config_dir` - The base configuration directory for the client
    ///
    /// # Returns
    /// The full path where the template file should be created
    fn file_path(config_dir: &Path) -> Result<PathBuf>;

    /// Whether the parent directory should be created if it doesn't exist
    ///
    /// Most templates require their parent directories to be created.
    /// This allows for flexibility in cases where directory creation
    /// might not be desired.
    fn should_create_dir() -> bool {
        true
    }
}

// Re-export template implementations
pub mod claude_subagent;
pub mod commands;
pub mod cursor_rules;
