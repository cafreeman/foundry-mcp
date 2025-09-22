//! Filesystem backend implementation
//!
//! Implements the FoundryBackend trait using direct filesystem operations.
//! This backend preserves the existing directory structure and atomic write semantics.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;
use tracing::warn;

use crate::core::backends::{BackendCapabilities, FoundryBackend, ResourceLocator};
use crate::core::filesystem;
use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecContentData, SpecFileType, SpecMetadata},
};
use crate::utils::timestamp;

/// Filesystem backend implementation
///
/// Implements the FoundryBackend trait using direct filesystem operations.
/// Preserves existing directory structure, atomic writes, and timestamp formats.
pub struct FilesystemBackend;

impl FilesystemBackend {
    pub fn new() -> Self {
        Self
    }

    fn get_project_path(&self, name: &str) -> Result<PathBuf> {
        let foundry_dir = filesystem::foundry_dir()?;
        Ok(foundry_dir.join(name))
    }

    fn get_spec_path(&self, project_name: &str, spec_name: &str) -> Result<PathBuf> {
        let foundry_dir = filesystem::foundry_dir()?;
        Ok(foundry_dir.join(project_name).join("specs").join(spec_name))
    }

    fn capabilities() -> BackendCapabilities {
        BackendCapabilities {
            supports_documents: true,
            supports_subtasks: true,
            url_deeplinks: false,
            atomic_replace: true,
            strong_consistency: true,
        }
    }
}

#[async_trait]
impl FoundryBackend for FilesystemBackend {
    async fn create_project(&self, config: ProjectConfig) -> Result<Project> {
        let project_path = self.get_project_path(&config.name)?;
        let created_at = Utc::now().to_rfc3339();

        // Create project directory structure
        filesystem::create_dir_all(&project_path)?;
        filesystem::create_dir_all(project_path.join("specs"))?;

        // Write project files
        filesystem::write_file_atomic(project_path.join("vision.md"), &config.vision)?;
        filesystem::write_file_atomic(project_path.join("tech-stack.md"), &config.tech_stack)?;
        filesystem::write_file_atomic(project_path.join("summary.md"), &config.summary)?;

        let path_string = project_path.to_string_lossy().to_string();
        Ok(Project {
            name: config.name,
            created_at,
            path: project_path, // Keep for backward compatibility
            location_hint: Some(path_string.clone()),
            locator: Some(ResourceLocator::FilesystemPath(path_string)),
            vision: Some(config.vision),
            tech_stack: Some(config.tech_stack),
            summary: Some(config.summary),
        })
    }

    async fn project_exists(&self, name: &str) -> Result<bool> {
        let project_path = self.get_project_path(name)?;
        Ok(project_path.exists())
    }

    async fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        let foundry_dir = filesystem::foundry_dir()?;

        if !foundry_dir.exists() {
            return Ok(Vec::new());
        }

        let projects: Vec<ProjectMetadata> = fs::read_dir(foundry_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    Some(entry)
                } else {
                    None
                }
            })
            .map(|entry| {
                let project_name = entry.file_name().to_string_lossy().to_string();
                let project_path = entry.path();

                // Count specs using fold
                let specs_dir = project_path.join("specs");
                let spec_count = if specs_dir.exists() {
                    fs::read_dir(specs_dir)
                        .ok()
                        .map(|dir| {
                            dir.filter_map(|e| e.ok())
                                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                                .fold(0, |acc, _| acc + 1)
                        })
                        .unwrap_or(0)
                } else {
                    0
                };

                // Get creation time (use directory creation time as fallback)
                let created_at = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.created().ok())
                    .map(DateTime::<Utc>::from)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());

                ProjectMetadata {
                    name: project_name,
                    created_at: created_at.clone(),
                    spec_count,
                    last_modified: created_at, // TODO: Use actual last modified time
                }
            })
            .collect();

        Ok(projects)
    }

    async fn load_project(&self, name: &str) -> Result<Project> {
        let project_path = self.get_project_path(name)?;

        if !project_path.exists() {
            return Err(anyhow::anyhow!("Project '{}' not found", name));
        }

        // Read project files
        let vision = filesystem::read_file(project_path.join("vision.md")).ok();
        let tech_stack = filesystem::read_file(project_path.join("tech-stack.md")).ok();
        let summary = filesystem::read_file(project_path.join("summary.md")).ok();

        // Get creation time from directory metadata
        let created_at =
            DateTime::<Utc>::from(fs::metadata(&project_path)?.created()?).to_rfc3339();

        let path_string = project_path.to_string_lossy().to_string();
        Ok(Project {
            name: name.to_string(),
            created_at,
            path: project_path, // Keep for backward compatibility
            location_hint: Some(path_string.clone()),
            locator: Some(ResourceLocator::FilesystemPath(path_string)),
            vision,
            tech_stack,
            summary,
        })
    }

    async fn create_spec(&self, config: SpecConfig) -> Result<Spec> {
        let foundry_dir = filesystem::foundry_dir()?;
        let project_path = foundry_dir.join(&config.project_name);
        let specs_dir = project_path.join("specs");
        let spec_name =
            crate::core::foundry::Foundry::<Self>::generate_spec_name(&config.feature_name);
        let spec_path = specs_dir.join(&spec_name);
        let created_at = Utc::now().to_rfc3339();

        // Ensure specs directory exists
        filesystem::create_dir_all(&spec_path)?;

        // Write spec files
        filesystem::write_file_atomic(spec_path.join("spec.md"), &config.content.spec)?;
        filesystem::write_file_atomic(spec_path.join("notes.md"), &config.content.notes)?;
        filesystem::write_file_atomic(spec_path.join("task-list.md"), &config.content.tasks)?;

        let path_string = spec_path.to_string_lossy().to_string();
        Ok(Spec {
            name: spec_name,
            created_at,
            path: spec_path, // Keep for backward compatibility
            project_name: config.project_name,
            location_hint: Some(path_string.clone()),
            locator: Some(ResourceLocator::FilesystemPath(path_string)),
            content: config.content,
        })
    }

    async fn list_specs(&self, project_name: &str) -> Result<Vec<SpecMetadata>> {
        let foundry_dir = filesystem::foundry_dir()?;
        let specs_dir = foundry_dir.join(project_name).join("specs");

        if !specs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut specs = Vec::new();
        let mut malformed_count = 0;

        for entry in fs::read_dir(specs_dir)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let spec_name = entry.file_name().to_string_lossy().to_string();

                    // Use enhanced timestamp parsing with better error handling
                    match (
                        timestamp::parse_spec_timestamp(&spec_name),
                        timestamp::extract_feature_name(&spec_name),
                    ) {
                        (Some(timestamp_str), Some(feature_name)) => {
                            // Convert timestamp to ISO format for consistent storage
                            let created_at = timestamp::spec_timestamp_to_iso(&timestamp_str)
                                .unwrap_or_else(|_| timestamp::iso_timestamp());

                            specs.push(SpecMetadata {
                                name: spec_name.clone(),
                                created_at,
                                feature_name,
                                project_name: project_name.to_string(),
                            });
                        }
                        _ => {
                            malformed_count += 1;
                            warn!("Skipping malformed spec directory: '{}'", spec_name);
                        }
                    }
                }
            } else {
                warn!(
                    "Failed to determine file type for entry: {:?}",
                    entry.path()
                );
            }
        }

        // Log summary of malformed specs if any were found
        if malformed_count > 0 {
            warn!(
                "Skipped {} malformed spec directories in project '{}'",
                malformed_count, project_name
            );
        }

        // Sort by creation time (newest first)
        specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(specs)
    }

    async fn load_spec(&self, project_name: &str, spec_name: &str) -> Result<Spec> {
        // Validate spec name format first
        crate::core::foundry::Foundry::<Self>::validate_spec_name(spec_name)
            .with_context(|| format!("Invalid spec name: {}", spec_name))?;

        let foundry_dir = filesystem::foundry_dir()?;
        let spec_path = foundry_dir.join(project_name).join("specs").join(spec_name);

        if !spec_path.exists() {
            return Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            ));
        }

        // Read spec files
        let spec_content = filesystem::read_file(spec_path.join("spec.md"))?;
        let notes = filesystem::read_file(spec_path.join("notes.md"))?;
        let task_list = filesystem::read_file(spec_path.join("task-list.md"))?;

        // Get creation time from spec name timestamp (more reliable than filesystem metadata)
        let created_at = timestamp::parse_spec_timestamp(spec_name).map_or_else(
            || {
                // Fallback to filesystem metadata if timestamp parsing fails
                fs::metadata(&spec_path)
                    .and_then(|metadata| metadata.created())
                    .map_err(anyhow::Error::from)
                    .and_then(|time| {
                        time.duration_since(std::time::UNIX_EPOCH)
                            .map_err(anyhow::Error::from)
                    })
                    .map(|duration| {
                        chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                            .unwrap_or_else(chrono::Utc::now)
                            .to_rfc3339()
                    })
                    .unwrap_or_else(|_| timestamp::iso_timestamp())
            },
            |timestamp_str| {
                timestamp::spec_timestamp_to_iso(&timestamp_str)
                    .unwrap_or_else(|_| timestamp::iso_timestamp())
            },
        );

        let path_string = spec_path.to_string_lossy().to_string();
        Ok(Spec {
            name: spec_name.to_string(),
            created_at,
            path: spec_path, // Keep for backward compatibility
            project_name: project_name.to_string(),
            location_hint: Some(path_string.clone()),
            locator: Some(ResourceLocator::FilesystemPath(path_string)),
            content: SpecContentData {
                spec: spec_content,
                notes,
                tasks: task_list,
            },
        })
    }

    async fn update_spec_content(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        new_content: &str,
    ) -> Result<()> {
        // Validate spec exists
        crate::core::foundry::Foundry::<Self>::validate_spec_name(spec_name)?;
        if !self.spec_exists(project_name, spec_name).await? {
            return Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            ));
        }

        let foundry_dir = filesystem::foundry_dir()?;
        let spec_path = foundry_dir.join(project_name).join("specs").join(spec_name);

        let file_path = match file_type {
            SpecFileType::Spec => spec_path.join("spec.md"),
            SpecFileType::Notes => spec_path.join("notes.md"),
            SpecFileType::TaskList => spec_path.join("task-list.md"),
        };

        filesystem::write_file_atomic(&file_path, new_content).with_context(|| {
            format!("Failed to update {:?} for spec '{}'", file_type, spec_name)
        })?;

        Ok(())
    }

    async fn delete_spec(&self, project_name: &str, spec_name: &str) -> Result<()> {
        crate::core::foundry::Foundry::<Self>::validate_spec_name(spec_name)?;

        let spec_path = self.get_spec_path(project_name, spec_name)?;

        if !spec_path.exists() {
            return Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            ));
        }

        std::fs::remove_dir_all(&spec_path).with_context(|| {
            format!(
                "Failed to delete spec '{}' from project '{}'",
                spec_name, project_name
            )
        })?;

        Ok(())
    }

    async fn get_latest_spec(&self, project_name: &str) -> Result<Option<SpecMetadata>> {
        let specs = self.list_specs(project_name).await?;
        Ok(specs.into_iter().next()) // Already sorted by creation time (newest first)
    }

    async fn count_specs(&self, project_name: &str) -> Result<usize> {
        let specs = self.list_specs(project_name).await?;
        Ok(specs.len())
    }

    fn capabilities(&self) -> BackendCapabilities {
        Self::capabilities()
    }
}

impl FilesystemBackend {
    /// Check if a spec exists
    pub async fn spec_exists(&self, project_name: &str, spec_name: &str) -> Result<bool> {
        let spec_path = self.get_spec_path(project_name, spec_name)?;
        Ok(spec_path.exists() && spec_path.is_dir())
    }
}
