//! In-memory backend implementation for testing
//!
//! This backend provides a lightweight, fast implementation of FoundryBackend
//! that stores all data in memory for contract testing and development.

use anyhow::{Result, anyhow};
use chrono::{Datelike, Timelike, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::backends::{BackendCapabilities, FoundryBackend, ResourceLocator};
use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecFileType, SpecMetadata},
};

/// In-memory storage for projects and specs
#[derive(Debug, Clone, Default)]
struct MemoryStore {
    projects: HashMap<String, Project>,
    specs: HashMap<String, HashMap<String, Spec>>, // project_name -> spec_name -> spec
}

/// In-memory backend implementation for testing
#[derive(Debug, Clone)]
pub struct InMemoryBackend {
    store: Arc<RwLock<MemoryStore>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(MemoryStore::default())),
        }
    }

    /// Clear all data (useful for test cleanup)
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.projects.clear();
        store.specs.clear();
    }

    /// Get project count (useful for testing)
    pub async fn project_count(&self) -> usize {
        let store = self.store.read().await;
        store.projects.len()
    }

    /// Get spec count for a project (useful for testing)
    pub async fn spec_count_for_project(&self, project_name: &str) -> usize {
        let store = self.store.read().await;
        store
            .specs
            .get(project_name)
            .map(|specs| specs.len())
            .unwrap_or(0)
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FoundryBackend for InMemoryBackend {
    // Project operations
    async fn create_project(&self, config: ProjectConfig) -> Result<Project> {
        let mut store = self.store.write().await;

        // Check if project already exists
        if store.projects.contains_key(&config.name) {
            return Err(anyhow!("Project '{}' already exists", config.name));
        }

        let created_at = Utc::now().to_rfc3339();
        let project = Project {
            name: config.name.clone(),
            created_at,
            path: std::path::PathBuf::from(format!("/memory/{}", config.name)), // Fake path for compatibility
            location_hint: Some(format!("memory://{}", config.name)),
            locator: Some(ResourceLocator::FilesystemPath(format!(
                "memory://{}",
                config.name
            ))),
            vision: Some(config.vision),
            tech_stack: Some(config.tech_stack),
            summary: Some(config.summary),
        };

        store.projects.insert(config.name.clone(), project.clone());
        store.specs.insert(config.name, HashMap::new());

        Ok(project)
    }

    async fn project_exists(&self, name: &str) -> Result<bool> {
        let store = self.store.read().await;
        Ok(store.projects.contains_key(name))
    }

    async fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        let store = self.store.read().await;
        let mut projects: Vec<ProjectMetadata> = store
            .projects
            .values()
            .map(|project| ProjectMetadata {
                name: project.name.clone(),
                created_at: project.created_at.clone(),
                last_modified: project.created_at.clone(), // Use created_at as last_modified for simplicity
                spec_count: store
                    .specs
                    .get(&project.name)
                    .map(|specs| specs.len())
                    .unwrap_or(0),
            })
            .collect();

        // Sort by created_at descending (newest first)
        projects.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(projects)
    }

    async fn load_project(&self, name: &str) -> Result<Project> {
        let store = self.store.read().await;
        store
            .projects
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Project '{}' not found", name))
    }

    // Spec operations
    async fn create_spec(&self, config: SpecConfig) -> Result<Spec> {
        let mut store = self.store.write().await;

        // Check if project exists
        if !store.projects.contains_key(&config.project_name) {
            return Err(anyhow!("Project '{}' not found", config.project_name));
        }

        // Generate spec name
        let now = Utc::now();
        let spec_name = format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}_{}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            config.feature_name
        );

        let created_at = now.to_rfc3339();
        let spec = Spec {
            name: spec_name.clone(),
            created_at,
            path: std::path::PathBuf::from(format!(
                "/memory/{}/specs/{}",
                config.project_name, spec_name
            )),
            project_name: config.project_name.clone(),
            location_hint: Some(format!(
                "memory://{}/specs/{}",
                config.project_name, spec_name
            )),
            locator: Some(ResourceLocator::FilesystemPath(format!(
                "memory://{}/specs/{}",
                config.project_name, spec_name
            ))),
            content: config.content,
        };

        store
            .specs
            .get_mut(&config.project_name)
            .unwrap()
            .insert(spec_name, spec.clone());

        Ok(spec)
    }

    async fn list_specs(&self, project_name: &str) -> Result<Vec<SpecMetadata>> {
        let store = self.store.read().await;

        // Check if project exists
        if !store.projects.contains_key(project_name) {
            return Err(anyhow!("Project '{}' not found", project_name));
        }

        let specs = store.specs.get(project_name).unwrap();
        let mut spec_list: Vec<SpecMetadata> = specs
            .values()
            .map(|spec| {
                // Extract feature name from spec name (format: YYYYMMDD_HHMMSS_feature_name)
                let feature_name = spec.name.split('_').skip(2).collect::<Vec<_>>().join("_");

                SpecMetadata {
                    name: spec.name.clone(),
                    created_at: spec.created_at.clone(),
                    feature_name,
                    project_name: spec.project_name.clone(),
                }
            })
            .collect();

        // Sort by created_at descending (newest first)
        spec_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(spec_list)
    }

    async fn load_spec(&self, project_name: &str, spec_name: &str) -> Result<Spec> {
        let store = self.store.read().await;

        let specs = store
            .specs
            .get(project_name)
            .ok_or_else(|| anyhow!("Project '{}' not found", project_name))?;

        specs.get(spec_name).cloned().ok_or_else(|| {
            anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            )
        })
    }

    async fn update_spec_content(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        content: &str,
    ) -> Result<()> {
        let mut store = self.store.write().await;

        let specs = store
            .specs
            .get_mut(project_name)
            .ok_or_else(|| anyhow!("Project '{}' not found", project_name))?;

        let spec = specs.get_mut(spec_name).ok_or_else(|| {
            anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            )
        })?;

        match file_type {
            SpecFileType::Spec => spec.content.spec = content.to_string(),
            SpecFileType::Notes => spec.content.notes = content.to_string(),
            SpecFileType::TaskList => spec.content.tasks = content.to_string(),
        }

        Ok(())
    }

    async fn delete_spec(&self, project_name: &str, spec_name: &str) -> Result<()> {
        let mut store = self.store.write().await;

        let specs = store
            .specs
            .get_mut(project_name)
            .ok_or_else(|| anyhow!("Project '{}' not found", project_name))?;

        specs.remove(spec_name).ok_or_else(|| {
            anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            )
        })?;

        Ok(())
    }

    // Helper operations
    async fn get_latest_spec(&self, project_name: &str) -> Result<Option<SpecMetadata>> {
        let specs = self.list_specs(project_name).await?;
        Ok(specs.into_iter().next()) // Already sorted by created_at desc
    }

    async fn count_specs(&self, project_name: &str) -> Result<usize> {
        let store = self.store.read().await;

        // Check if project exists
        if !store.projects.contains_key(project_name) {
            return Err(anyhow!("Project '{}' not found", project_name));
        }

        let count = store
            .specs
            .get(project_name)
            .map(|specs| specs.len())
            .unwrap_or(0);

        Ok(count)
    }

    // Capabilities introspection
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_documents: true,
            supports_subtasks: true,
            url_deeplinks: false,
            atomic_replace: true,
            strong_consistency: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_backend_creation() {
        let backend = InMemoryBackend::new();
        let capabilities = backend.capabilities();
        assert!(capabilities.supports_documents);
        assert!(capabilities.supports_subtasks);
        assert!(capabilities.atomic_replace);
        assert!(capabilities.strong_consistency);
    }

    #[test]
    fn test_memory_backend_basic_operations() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for test");

        rt.block_on(async {
            let backend = InMemoryBackend::new();

            // Test project creation
            let config = ProjectConfig {
                name: "test-project".to_string(),
                vision: "Test vision".to_string(),
                tech_stack: "Test tech stack".to_string(),
                summary: "Test summary".to_string(),
            };

            let project = backend.create_project(config).await.unwrap();
            assert_eq!(project.name, "test-project");

            // Test project exists
            assert!(backend.project_exists("test-project").await.unwrap());
            assert!(!backend.project_exists("nonexistent").await.unwrap());

            // Test clear
            backend.clear().await;
            assert!(!backend.project_exists("test-project").await.unwrap());
        });
    }
}
