//! Linear backend scaffolding (Phase A)
//!
//! This module sets up a self-contained GraphQL client for Linear using a
//! preconfigured reqwest::Client supplied to Cynic. In Phase A, the backend
//! methods intentionally return Unsupported to avoid changing runtime behavior.

mod config;
mod graphql;
mod helpers;
pub mod ops;
pub use config::LinearConfig;

use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use crate::core::backends::FoundryBackend;
use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecFileType, SpecMetadata},
};

/// Linear backend (Phase A skeleton)
#[derive(Debug, Clone)]
pub struct LinearBackend {
    _endpoint: Url,
    _client: reqwest::Client,
}

impl LinearBackend {
    pub fn new(cfg: &LinearConfig) -> Result<Self> {
        let (client, endpoint) = graphql::build_client(cfg)?;
        Ok(Self { _endpoint: endpoint, _client: client })
    }
}

#[async_trait]
impl FoundryBackend for LinearBackend {
    async fn create_project(&self, _config: ProjectConfig) -> Result<Project> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn project_exists(&self, _name: &str) -> Result<bool> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn load_project(&self, _name: &str) -> Result<Project> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn create_spec(&self, _config: SpecConfig) -> Result<Spec> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn list_specs(&self, _project_name: &str) -> Result<Vec<SpecMetadata>> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn load_spec(&self, _project_name: &str, _spec_name: &str) -> Result<Spec> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn update_spec_content(
        &self,
        _project_name: &str,
        _spec_name: &str,
        _file_type: SpecFileType,
        _content: &str,
    ) -> Result<()> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn delete_spec(&self, _project_name: &str, _spec_name: &str) -> Result<()> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn get_latest_spec(&self, _project_name: &str) -> Result<Option<SpecMetadata>> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    async fn count_specs(&self, _project_name: &str) -> Result<usize> {
        Err(anyhow::anyhow!("LinearBackend not implemented (Phase A)"))
    }

    fn capabilities(&self) -> crate::core::backends::BackendCapabilities {
        crate::core::backends::BackendCapabilities {
            supports_documents: true,
            supports_subtasks: true,
            url_deeplinks: true,
            atomic_replace: false,
            strong_consistency: false,
        }
    }
}