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
use crate::core::backends::linear::helpers::humanize_title;
use crate::types::{
    project::{Project, ProjectConfig, ProjectMetadata},
    spec::{Spec, SpecConfig, SpecFileType, SpecMetadata},
};

/// Linear backend (Phase A skeleton)
#[derive(Debug, Clone)]
pub struct LinearBackend {
    gql: crate::core::backends::linear::graphql::LinearGraphQl,
}

impl LinearBackend {
    pub fn new(cfg: &LinearConfig) -> Result<Self> {
        let gql = crate::core::backends::linear::graphql::LinearGraphQl::new(cfg)?;
        Ok(Self { gql })
    }
}

#[async_trait]
impl FoundryBackend for LinearBackend {
    async fn create_project(&self, config: ProjectConfig) -> Result<Project> {
        use crate::core::backends::linear::ops;
        use chrono::Utc;

        // 1) Find or create the project in Linear
        let (project_id, project_name, _existing_desc) =
            ops::find_or_create_project(&self.gql, &config.name, config.summary.as_deref()).await?;

        // 2) Ensure description is up to date with the provided summary
        if let Some(summary) = config.summary.as_ref() {
            ops::upsert_project_description(&self.gql, &project_id, summary).await?;
        }

        // 3) Upsert project documents: Vision and Tech Stack, with hidden marker
        ops::upsert_project_documents(
            &self.gql,
            &project_id,
            &config.name,
            &config.vision,
            &config.tech_stack,
        )
        .await?;

        // 4) Return a Project struct. We don't have a direct URL for the project page here; leave hints empty.
        let created_at = Utc::now().to_rfc3339();
        Ok(Project {
            name: project_name,
            created_at,
            path: std::path::PathBuf::from(format!("linear://project/{}", project_id)),
            location_hint: None,
            locator: None,
            vision: Some(config.vision),
            tech_stack: Some(config.tech_stack),
            summary: config.summary,
        })
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

    async fn create_spec(&self, config: SpecConfig) -> Result<Spec> {
        use crate::core::backends::linear::ops;
        use chrono::Utc;

        // 1) Find or create the project in Linear
        let (project_id, _project_name, _existing_desc) = ops::find_or_create_project(
            &self.gql,
            &config.project_name,
            config.content.spec.clone(),
        )
        .await?;

        // 2) Generate spec name (timestamped)
        let spec_name =
            crate::core::foundry::Foundry::<Self>::generate_spec_name(&config.feature_name);
        let created_at = Utc::now().to_rfc3339();

        // 3) Create the Notes document first
        let notes_marker = format!("<!-- foundry:specId={}; type=notes; v=1 -->\n", spec_name);
        let notes_content = format!("{}{}", notes_marker, config.content.notes);

        let notes_title = format!("{} — Notes", humanize_title(&config.feature_name));
        let notes_doc =
            ops::create_document(&self.gql, &notes_title, &notes_content, &project_id).await?;

        // 4) Create the spec issue with reference to notes
        let (issue_id, issue_url) = ops::create_spec_issue(
            &self.gql,
            &LinearConfig::from_env()?,
            &project_id,
            &spec_name,
            &config.content.spec,
            &notes_doc.url,
        )
        .await?;

        // 5) Return a Spec struct with Linear-specific locator
        let linear_locator = ResourceLocator::Linear {
            project_id: project_id.clone(),
            issue_id: issue_id.clone(),
            notes_document_id: notes_doc.id.clone(),
            issue_url: issue_url.clone(),
            notes_url: notes_doc.url.clone(),
        };

        Ok(Spec {
            name: spec_name,
            created_at,
            path: std::path::PathBuf::from(format!("linear://spec/{}", issue_id)),
            project_name: config.project_name,
            location_hint: Some(issue_url.clone()),
            locator: Some(linear_locator),
            content: config.content,
        })
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
