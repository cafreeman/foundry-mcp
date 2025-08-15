//! File system manager for project directory operations

use anyhow::Result;
use std::path::PathBuf;

/// Manages file system operations for project directories
pub struct FileSystemManager {
    base_dir: PathBuf,
}

impl FileSystemManager {
    /// Create a new FileSystemManager instance
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let base_dir = home_dir.join(".project-manager-mcp");

        Ok(Self { base_dir })
    }

    /// Get the base directory for all projects
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}
