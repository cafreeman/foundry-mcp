//! Load specification tool handler

use crate::errors::{self, Result};
use crate::repository::{ProjectRepository, SpecificationRepository};
use serde_json::Value;

/// Handler for loading specifications with full context
pub struct LoadSpecHandler {
    project_repo: ProjectRepository,
    spec_repo: SpecificationRepository,
}

impl LoadSpecHandler {
    /// Create a new LoadSpecHandler instance
    pub fn new(project_repo: ProjectRepository, spec_repo: SpecificationRepository) -> Self {
        Self {
            project_repo,
            spec_repo,
        }
    }

    /// Handle the load_spec tool call
    pub async fn handle_load_spec(&self, arguments: &Value) -> Result<String> {
        // Parse and validate arguments
        let project_name = arguments["project_name"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("project_name", "missing", "Project name is required")
        })?;

        let spec_id = arguments["spec_id"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("spec_id", "missing", "Spec ID is required")
        })?;

        // Validate project exists
        if !self.project_repo.project_exists(project_name).await {
            return Err(errors::helpers::project_not_found(project_name));
        }

        // Validate spec exists
        if !self.spec_repo.spec_exists(project_name, spec_id).await {
            return Err(errors::helpers::spec_not_found(spec_id, project_name));
        }

        // Load project context
        let project = self.project_repo.load_project(project_name).await?;

        // Load specification
        let spec = self.spec_repo.load_spec(project_name, spec_id).await?;

        // Load task list
        let task_list = self.spec_repo.load_task_list(project_name, spec_id).await?;

        // Load notes
        let notes = self.spec_repo.load_notes(project_name, spec_id).await?;

        // Format response
        let response = format!(
            "📋 Specification: {}\n\
             📁 Project: {}\n\
             📝 Description: {}\n\
             📅 Created: {}\n\
             📅 Updated: {}\n\
             📊 Status: {:?}\n\n\
             📋 Content:\n{}\n\n\
             📋 Tasks ({}):\n{}\n\n\
             📝 Notes ({}):\n{}",
            spec.name,
            project.name,
            spec.description,
            spec.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            spec.updated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            spec.status,
            spec.content,
            task_list.tasks.len(),
            if task_list.tasks.is_empty() {
                "No tasks yet".to_string()
            } else {
                task_list
                    .tasks
                    .iter()
                    .map(|t| format!("- {}: {} ({:?}, {:?})", t.id, t.title, t.status, t.priority))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            notes.len(),
            if notes.is_empty() {
                "No notes yet".to_string()
            } else {
                notes
                    .iter()
                    .map(|n| {
                        format!(
                            "- {}: {} ({:?})",
                            n.id,
                            n.content.split('\n').next().unwrap_or(""),
                            n.category
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );

        Ok(response)
    }
}
