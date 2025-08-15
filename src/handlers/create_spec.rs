//! Create specification tool handler

use crate::errors::{self, Result};
use crate::repository::{ProjectRepository, SpecificationRepository};
use serde_json::Value;

/// Handler for creating new specifications
pub struct CreateSpecHandler {
    project_repo: ProjectRepository,
    spec_repo: SpecificationRepository,
}

impl CreateSpecHandler {
    /// Create a new CreateSpecHandler instance
    pub fn new(project_repo: ProjectRepository, spec_repo: SpecificationRepository) -> Self {
        Self {
            project_repo,
            spec_repo,
        }
    }

    /// Handle the create_spec tool call
    pub async fn handle_create_spec(&self, arguments: &Value) -> Result<String> {
        // Parse and validate arguments
        let project_name = arguments["project_name"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("project_name", "missing", "Project name is required")
        })?;

        let spec_name = arguments["spec_name"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("spec_name", "missing", "Spec name is required")
        })?;

        let description = arguments["description"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("description", "missing", "Description is required")
        })?;

        let content = arguments["content"]
            .as_str()
            .unwrap_or("Initial specification content");

        // Validate project exists
        if !self.project_repo.project_exists(project_name).await {
            return Err(errors::helpers::project_not_found(project_name));
        }

        // Validate spec name format (snake_case)
        if !self.is_valid_spec_name(spec_name) {
            return Err(errors::helpers::invalid_spec_name(spec_name));
        }

        // Create specification
        let spec = self
            .spec_repo
            .create_spec(project_name, spec_name, description, content)
            .await?;

        Ok(format!(
            "Successfully created specification '{}' in project '{}'",
            spec.name, project_name
        ))
    }

    /// Validate spec name format (snake_case)
    fn is_valid_spec_name(&self, name: &str) -> bool {
        name.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            && !name.starts_with('_')
            && !name.ends_with('_')
            && !name.contains("__")
    }
}
