//! Backend abstraction for pluggable storage systems

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecFileType, SpecMetadata},
};

/// Core backend trait defining storage contracts
#[async_trait::async_trait]
pub trait FoundryBackend: Send + Sync {
    // Project operations
    async fn create_project(&self, config: ProjectConfig) -> Result<Project>;
    async fn project_exists(&self, name: &str) -> Result<bool>;
    async fn list_projects(&self) -> Result<Vec<ProjectMetadata>>;
    async fn load_project(&self, name: &str) -> Result<Project>;

    // Spec operations
    async fn create_spec(&self, config: SpecConfig) -> Result<Spec>;
    async fn list_specs(&self, project_name: &str) -> Result<Vec<SpecMetadata>>;
    async fn load_spec(&self, project_name: &str, spec_name: &str) -> Result<Spec>;
    async fn update_spec_content(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        content: &str,
    ) -> Result<()>;
    async fn delete_spec(&self, project_name: &str, spec_name: &str) -> Result<()>;

    // Helper operations
    async fn get_latest_spec(&self, project_name: &str) -> Result<Option<SpecMetadata>>;
    async fn count_specs(&self, project_name: &str) -> Result<usize>;

    // Capabilities introspection
    fn capabilities(&self) -> BackendCapabilities;
}

/// Backend capability flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub supports_documents: bool,
    pub supports_subtasks: bool,
    pub url_deeplinks: bool,
    pub atomic_replace: bool,
    pub strong_consistency: bool,
}

/// Resource locator for different backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLocator {
    FilesystemPath(String),
    // Future: Linear { project_id: String, issue_id: String, urls: Vec<String> },
}

/// Content store abstraction for EditEngine I/O via façade
#[async_trait::async_trait]
pub trait SpecContentStore: Send + Sync {
    async fn read_spec_file(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
    ) -> Result<String>;

    async fn write_spec_file(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        content: &str,
    ) -> Result<()>;

    async fn is_file_modified(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        new_content: &str,
    ) -> Result<bool>;
}

// Re-export filesystem backend
pub mod filesystem;

// Re-export memory backend for testing
pub mod memory;

// Backend testing infrastructure
mod tests;

// Re-export factory functions
pub use crate::core::foundry::get_default_foundry;
