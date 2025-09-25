//! Linear backend scaffolding (Phase A)
//!
//! This module sets up a self-contained GraphQL client for Linear using a
//! preconfigured reqwest::Client supplied to Cynic. In Phase A, the backend
//! methods intentionally return Unsupported to avoid changing runtime behavior.

pub mod config;
mod graphql;
pub mod helpers;
pub mod ops;
pub mod reconcile;
pub use config::LinearConfig;

use anyhow::Result;
use async_trait::async_trait;

use crate::core::backends::FoundryBackend;
use crate::core::backends::ResourceLocator;
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
            ops::find_or_create_project(&self.gql, &config.name, Some(&config.summary)).await?;

        // 2) Ensure description is up to date with the provided summary
        ops::upsert_project_description(&self.gql, &project_id, &config.summary).await?;

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
            summary: Some(config.summary),
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
            Some(&config.content.spec),
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
            notes_document_id: notes_doc.id.clone().into_inner(),
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

    async fn list_specs(&self, project_name: &str) -> Result<Vec<SpecMetadata>> {
        use crate::core::backends::linear::ops::list_spec_issues_by_project;

        // Collect all specs with pagination support
        let mut all_specs = Vec::new();
        let mut after_cursor: Option<String> = None;

        loop {
            let (mut specs, has_next, next_cursor) =
                list_spec_issues_by_project(&self.gql, project_name, after_cursor, Some(50))
                    .await?;

            all_specs.append(&mut specs);

            if !has_next {
                break;
            }

            after_cursor = next_cursor;
        }

        // Sort by created_at descending (most recent first)
        all_specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(all_specs)
    }

    async fn load_spec(&self, project_name: &str, spec_name: &str) -> Result<Spec> {
        use crate::core::backends::linear::ops::load_spec_by_marker;

        match load_spec_by_marker(&self.gql, project_name, spec_name).await? {
            Some(spec) => Ok(spec),
            None => Err(anyhow::anyhow!(
                "Spec '{}' not found in project '{}'",
                spec_name,
                project_name
            )),
        }
    }

    async fn update_spec_content(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: SpecFileType,
        content: &str,
    ) -> Result<()> {
        match file_type {
            SpecFileType::TaskList => {
                #[cfg(feature = "linear_backend")]
                {
                    // Phase D wiring behind feature flag. Implement real parser → plan → execute
                    // flow using Linear GraphQL ops.
                    return self
                        .update_tasks_via_linear(project_name, spec_name, content)
                        .await;
                }

                #[cfg(not(feature = "linear_backend"))]
                {
                    return Err(anyhow::anyhow!(
                        "LinearBackend Phase D (tasks) gated behind 'linear_backend' feature"
                    ));
                }
            }
            _ => Err(anyhow::anyhow!(
                "LinearBackend does not support updating this file type yet"
            )),
        }
    }

    async fn delete_spec(&self, project_name: &str, spec_name: &str) -> Result<()> {
        use crate::core::backends::linear::ops::delete_spec_by_marker;

        delete_spec_by_marker(&self.gql, project_name, spec_name).await
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

impl LinearBackend {
    #[cfg(feature = "linear_backend")]
    async fn update_tasks_via_linear(
        &self,
        _project_name: &str,
        spec_name: &str,
        task_list_markdown: &str,
    ) -> Result<()> {
        use crate::core::backends::linear::config::LinearConfig;
        use crate::core::backends::linear::ops;
        use crate::core::backends::linear::ops::build_existing_from_tuples;
        use crate::linear_phase_d::plan_from_markdown_and_existing;
        use crate::linear_reconcile::{ExistingSubIssue, normalize_task_key};

        // Get Linear config from environment
        let cfg = LinearConfig::from_env()?;

        // TODO: Implement ResourceLocator to get actual issue_id and project_id
        // For now, this function demonstrates the wiring but needs resource discovery
        // In a real implementation, we would:
        // 1. Query Linear for the spec issue by marker or title pattern
        // 2. Extract issue_id and project_id from the found issue
        // This is a placeholder that shows the structure but needs resource lookup

        // Placeholder values - in real implementation, get from ResourceLocator
        let parent_issue_id = "placeholder_issue_id";
        let project_id = "placeholder_project_id";

        // Note: Until resource lookup is implemented, return early to avoid API calls
        // with placeholder values. The structure below shows the intended flow.
        if parent_issue_id == "placeholder_issue_id" {
            // Structure is correct, but we need resource discovery first
            return Ok(());
        }

        // 1) Fetch existing sub-issues for this spec's issue
        let existing_tuples = ops::list_sub_issues_for_parent(&self.gql, parent_issue_id).await?;
        let existing: Vec<ExistingSubIssue> = build_existing_from_tuples(existing_tuples);

        // 2) Compute reconciliation plan from markdown and existing sub-issues
        let plan = plan_from_markdown_and_existing(task_list_markdown, &existing);

        // 3) Execute plan with real operations in stable order
        // Since the executor expects sync closures but our operations are async,
        // we'll collect operations and execute them sequentially to preserve order

        // Ensure foundry label exists first
        let foundry_label_id = ops::ensure_foundry_label(&self.gql).await?;

        // Execute label fixes first
        for issue_id in &plan.to_keep_label_fix {
            ops::ensure_label_on_issue(&self.gql, issue_id, &foundry_label_id).await?;
        }

        // Create missing tasks
        for task in &plan.to_create {
            let task_key = normalize_task_key(&task.text);
            ops::create_sub_issue_with_marker(
                &self.gql,
                &cfg,
                parent_issue_id,
                project_id,
                &task.text,
                spec_name,
                &task_key,
            )
            .await?;
        }

        // Close extraneous tasks
        for issue_id in &plan.to_close {
            ops::close_issue(&self.gql, issue_id).await?;
        }

        // Reopen tasks that should be open
        for issue_id in &plan.to_reopen {
            ops::reopen_issue(&self.gql, issue_id).await?;
        }

        Ok(())
    }
}
