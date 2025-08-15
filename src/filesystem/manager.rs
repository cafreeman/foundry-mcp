//! File system manager for project directory operations

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

    /// Get the project directory path
    pub fn project_dir(&self, project_name: &str) -> PathBuf {
        self.base_dir.join(project_name)
    }

    /// Get the project info directory path (tech-stack.md, vision.md)
    pub fn project_info_dir(&self, project_name: &str) -> PathBuf {
        self.project_dir(project_name).join("project")
    }

    /// Get the specifications directory path for a project
    pub fn specs_dir(&self, project_name: &str) -> PathBuf {
        self.project_dir(project_name).join("specs")
    }

    /// Get the specification directory path
    pub fn spec_dir(&self, project_name: &str, spec_id: &str) -> PathBuf {
        self.specs_dir(project_name).join(spec_id)
    }

    /// Create the base directory structure if it doesn't exist
    pub fn ensure_base_dir(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)
                .with_context(|| format!("Failed to create base directory: {:?}", self.base_dir))?;
        }
        Ok(())
    }

    /// Create the complete project directory structure
    pub fn create_project_structure(&self, project_name: &str) -> Result<()> {
        self.ensure_base_dir()?;

        let project_path = self.project_dir(project_name);
        let project_info_path = self.project_info_dir(project_name);
        let specs_path = self.specs_dir(project_name);

        // Create project directory
        fs::create_dir_all(&project_path)
            .with_context(|| format!("Failed to create project directory: {:?}", project_path))?;

        // Create project info directory
        fs::create_dir_all(&project_info_path).with_context(|| {
            format!(
                "Failed to create project info directory: {:?}",
                project_info_path
            )
        })?;

        // Create specs directory
        fs::create_dir_all(&specs_path)
            .with_context(|| format!("Failed to create specs directory: {:?}", specs_path))?;

        Ok(())
    }

    /// Create the specification directory structure
    pub fn create_spec_structure(&self, project_name: &str, spec_id: &str) -> Result<()> {
        let spec_path = self.spec_dir(project_name, spec_id);

        fs::create_dir_all(&spec_path)
            .with_context(|| format!("Failed to create spec directory: {:?}", spec_path))?;

        Ok(())
    }

    /// Check if a project exists
    pub fn project_exists(&self, project_name: &str) -> bool {
        self.project_dir(project_name).exists()
    }

    /// Check if a specification exists
    pub fn spec_exists(&self, project_name: &str, spec_id: &str) -> bool {
        self.spec_dir(project_name, spec_id).exists()
    }

    /// List all projects
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.ensure_base_dir()?;

        let mut projects = Vec::new();

        if self.base_dir.exists() {
            for entry in fs::read_dir(&self.base_dir)
                .with_context(|| format!("Failed to read base directory: {:?}", self.base_dir))?
            {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir()
                    && !path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with('.')
                {
                    if let Some(name) = path.file_name() {
                        projects.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        projects.sort();
        Ok(projects)
    }

    /// List all specifications for a project
    pub fn list_specs(&self, project_name: &str) -> Result<Vec<String>> {
        let specs_path = self.specs_dir(project_name);
        let mut specs = Vec::new();

        if specs_path.exists() {
            for entry in fs::read_dir(&specs_path)
                .with_context(|| format!("Failed to read specs directory: {:?}", specs_path))?
            {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir()
                    && !path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with('.')
                {
                    if let Some(name) = path.file_name() {
                        specs.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        specs.sort();
        Ok(specs)
    }

    /// Write content to a file with atomic operation and backup
    pub fn write_file_safe(&self, file_path: &Path, content: &str) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
        }

        // Create backup if file exists
        if file_path.exists() {
            let backup_path = self.create_backup_path(file_path)?;
            fs::copy(file_path, &backup_path)
                .with_context(|| format!("Failed to create backup: {:?}", backup_path))?;
        }

        // Write to temporary file first
        let temp_path = self.create_temp_path(file_path)?;
        let mut temp_file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;

        temp_file
            .write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to temp file: {:?}", temp_path))?;
        temp_file
            .flush()
            .with_context(|| format!("Failed to flush temp file: {:?}", temp_path))?;

        // Atomic move from temp to final location
        fs::rename(&temp_path, file_path).with_context(|| {
            format!(
                "Failed to move temp file to final location: {:?}",
                file_path
            )
        })?;

        Ok(())
    }

    /// Read content from a file
    pub fn read_file(&self, file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        Ok(content)
    }

    /// Check if a file exists
    pub fn file_exists(&self, file_path: &Path) -> bool {
        file_path.exists() && file_path.is_file()
    }

    /// Get file modification time
    pub fn get_file_modified_time(&self, file_path: &Path) -> Result<SystemTime> {
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("Failed to get metadata for file: {:?}", file_path))?;

        metadata
            .modified()
            .with_context(|| format!("Failed to get modification time for file: {:?}", file_path))
    }

    /// Create a backup path for a file
    fn create_backup_path(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_name = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", file_path))?
            .to_string_lossy();

        let backup_name = format!("{}.backup.{}", file_name, timestamp);
        let backup_path = file_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", file_path))?
            .join(backup_name);

        Ok(backup_path)
    }

    /// Create a temporary path for a file
    fn create_temp_path(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_name = file_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", file_path))?
            .to_string_lossy();

        let temp_name = format!("{}.tmp.{}", file_name, timestamp);
        let temp_path = file_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: {:?}", file_path))?
            .join(temp_name);

        Ok(temp_path)
    }

    /// Clean up temporary and backup files older than specified age
    pub fn cleanup_old_files(&self, max_age_seconds: u64) -> Result<()> {
        let current_time = SystemTime::now();
        let max_age = std::time::Duration::from_secs(max_age_seconds);

        self.cleanup_directory(&self.base_dir, current_time, max_age)?;

        Ok(())
    }

    /// Recursively clean up old temporary and backup files
    fn cleanup_directory(
        &self,
        dir_path: &Path,
        current_time: SystemTime,
        max_age: std::time::Duration,
    ) -> Result<()> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.cleanup_directory(&path, current_time, max_age)?;
            } else if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if it's a temp or backup file
                if file_name.contains(".tmp.") || file_name.contains(".backup.") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(age) = current_time.duration_since(modified) {
                                if age > max_age {
                                    fs::remove_file(&path).with_context(|| {
                                        format!("Failed to remove old file: {:?}", path)
                                    })?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
