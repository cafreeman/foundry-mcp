//! File system manager for project directory operations

use crate::errors::{self, ProjectManagerError, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Manages file system operations for project directories
#[derive(Clone)]
pub struct FileSystemManager {
    base_dir: PathBuf,
}

impl FileSystemManager {
    /// Create a new FileSystemManager instance
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| ProjectManagerError::Configuration {
            setting: "home_directory".to_string(),
            value: "unknown".to_string(),
            reason: "Could not determine home directory".to_string(),
        })?;
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
            fs::create_dir_all(&self.base_dir).map_err(|e| {
                errors::helpers::file_system_error("create base directory", &self.base_dir, e)
            })?;
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
        if !project_path.exists() {
            fs::create_dir(&project_path).map_err(|e| {
                errors::helpers::file_system_error("create project directory", &project_path, e)
            })?;
        }

        // Create project info directory
        if !project_info_path.exists() {
            fs::create_dir(&project_info_path).map_err(|e| {
                errors::helpers::file_system_error(
                    "create project info directory",
                    &project_info_path,
                    e,
                )
            })?;
        }

        // Create specs directory
        if !specs_path.exists() {
            fs::create_dir(&specs_path).map_err(|e| {
                errors::helpers::file_system_error("create specs directory", &specs_path, e)
            })?;
        }

        Ok(())
    }

    /// Create specification directory structure
    pub fn create_spec_structure(&self, project_name: &str, spec_id: &str) -> Result<()> {
        let spec_path = self.spec_dir(project_name, spec_id);
        if !spec_path.exists() {
            fs::create_dir(&spec_path).map_err(|e| {
                errors::helpers::file_system_error("create spec directory", &spec_path, e)
            })?;
        }
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
            for entry in fs::read_dir(&self.base_dir).map_err(|e| {
                errors::helpers::file_system_error("read base directory", &self.base_dir, e)
            })? {
                let entry = entry.map_err(|e| {
                    errors::helpers::file_system_error("read directory entry", &self.base_dir, e)
                })?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            projects.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(projects)
    }

    /// List specifications for a project
    pub fn list_specs(&self, project_name: &str) -> Result<Vec<String>> {
        let specs_path = self.specs_dir(project_name);
        if !specs_path.exists() {
            return Ok(Vec::new());
        }

        let mut specs = Vec::new();
        for entry in fs::read_dir(&specs_path).map_err(|e| {
            errors::helpers::file_system_error("read specs directory", &specs_path, e)
        })? {
            let entry = entry.map_err(|e| {
                errors::helpers::file_system_error("read specs directory entry", &specs_path, e)
            })?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        specs.push(name_str.to_string());
                    }
                }
            }
        }

        // Sort specs by creation date (newest first)
        specs.sort_by(|a, b| {
            let a_path = self.spec_dir(project_name, a);
            let b_path = self.spec_dir(project_name, b);
            let a_time = a_path
                .metadata()
                .and_then(|m| m.created())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let b_time = b_path
                .metadata()
                .and_then(|m| m.created())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        Ok(specs)
    }

    /// Write content to a file with atomic operation and backup
    pub fn write_file_safe(&self, file_path: &Path, content: &str) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                errors::helpers::file_system_error("create parent directory", parent, e)
            })?;
        }

        // Create backup if file exists
        if file_path.exists() {
            let backup_path = self.create_backup_path(file_path)?;
            fs::copy(file_path, &backup_path)
                .map_err(|e| errors::helpers::file_system_error("create backup", file_path, e))?;
        }

        // Write to temporary file first
        let temp_path = self.create_temp_path(file_path)?;
        let mut temp_file = File::create(&temp_path)
            .map_err(|e| errors::helpers::file_system_error("create temp file", &temp_path, e))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| errors::helpers::file_system_error("write to temp file", &temp_path, e))?;
        temp_file
            .flush()
            .map_err(|e| errors::helpers::file_system_error("flush temp file", &temp_path, e))?;

        // Atomic move from temp to final location
        fs::rename(&temp_path, file_path).map_err(|e| {
            errors::helpers::file_system_error("move temp file to final location", file_path, e)
        })?;

        Ok(())
    }

    /// Read content from a file
    pub fn read_file(&self, file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)
            .map_err(|e| errors::helpers::file_system_error("open file", file_path, e))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| errors::helpers::file_system_error("read file", file_path, e))?;

        Ok(content)
    }

    /// Check if a file exists
    pub fn file_exists(&self, file_path: &Path) -> bool {
        file_path.exists() && file_path.is_file()
    }

    /// Get file modification time
    pub fn get_file_modified_time(&self, file_path: &Path) -> Result<SystemTime> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| errors::helpers::file_system_error("get file metadata", file_path, e))?;
        metadata.modified().map_err(|e| {
            errors::helpers::file_system_error("get file modification time", file_path, e)
        })
    }

    /// Create a backup path for a file
    fn create_backup_path(&self, file_path: &Path) -> Result<PathBuf> {
        let file_name = file_path
            .file_name()
            .ok_or_else(|| ProjectManagerError::Internal {
                operation: "create backup path".to_string(),
                details: "Could not get file name".to_string(),
                source: None,
            })?;

        let backup_name = format!(
            "{}.backup.{}",
            file_name.to_string_lossy(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let parent = file_path
            .parent()
            .ok_or_else(|| ProjectManagerError::Internal {
                operation: "create backup path".to_string(),
                details: "Could not get parent directory".to_string(),
                source: None,
            })?;

        Ok(parent.join(backup_name))
    }

    /// Create a temporary path for a file
    fn create_temp_path(&self, file_path: &Path) -> Result<PathBuf> {
        let file_name = file_path
            .file_name()
            .ok_or_else(|| ProjectManagerError::Internal {
                operation: "create temp path".to_string(),
                details: "Could not get file name".to_string(),
                source: None,
            })?;

        let temp_name = format!(
            ".{}.tmp.{}",
            file_name.to_string_lossy(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let parent = file_path
            .parent()
            .ok_or_else(|| ProjectManagerError::Internal {
                operation: "create temp path".to_string(),
                details: "Could not get parent directory".to_string(),
                source: None,
            })?;

        Ok(parent.join(temp_name))
    }

    /// Clean up temporary and backup files older than specified age
    pub fn cleanup_old_files(&self, max_age_seconds: u64) -> Result<()> {
        let current_time = SystemTime::now();
        let max_age = std::time::Duration::from_secs(max_age_seconds);

        Self::cleanup_directory(&self.base_dir, current_time, max_age)?;

        Ok(())
    }

    /// Recursively clean up old temporary and backup files
    fn cleanup_directory(
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
                Self::cleanup_directory(&path, current_time, max_age)?;
            } else if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if it's a temp or backup file
                if (file_name.contains(".tmp.") || file_name.contains(".backup."))
                    && let Ok(metadata) = fs::metadata(&path)
                    && let Ok(modified) = metadata.modified()
                    && let Ok(age) = current_time.duration_since(modified)
                    && age > max_age
                {
                    fs::remove_file(&path).map_err(|e| {
                        errors::helpers::file_system_error("remove old file", &path, e)
                    })?;
                }
            }
        }

        Ok(())
    }
}
