//! Create specification tool handler

use crate::models::{SpecStatus, Specification};
use crate::repository::{ProjectRepository, SpecificationRepository};
use anyhow::Result;
use chrono::Utc;
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
        let project_name = arguments["project_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: project_name"))?;

        let spec_name = arguments["spec_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: spec_name"))?;

        let description = arguments["description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: description"))?;

        let content = arguments["content"]
            .as_str()
            .unwrap_or("Initial specification content");

        // Validate project exists
        if !self.project_repo.project_exists(project_name).await {
            return Err(anyhow::anyhow!(
                "Project '{}' does not exist. Please create the project first using setup_project.",
                project_name
            ));
        }

        // Validate spec name format (snake_case)
        if !self.is_valid_spec_name(spec_name) {
            return Err(anyhow::anyhow!(
                "Invalid specification name: '{}'. Specification names must be in snake_case format (e.g., 'user_authentication', 'api_endpoints').",
                spec_name
            ));
        }

        // Generate spec ID (YYYYMMDD_name format)
        let spec_id = self.generate_spec_id(spec_name);

        // Check if spec already exists
        if self.spec_repo.spec_exists(project_name, &spec_id).await {
            return Err(anyhow::anyhow!(
                "Specification '{}' already exists in project '{}'.",
                spec_id,
                project_name
            ));
        }

        // Create the specification
        let spec = Specification {
            id: spec_id.clone(),
            name: spec_name.to_string(),
            description: description.to_string(),
            status: SpecStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: content.to_string(),
        };

        // Save the specification
        self.spec_repo.create_spec(project_name, spec).await?;

        Ok(format!(
            "✅ Specification '{}' created successfully!\n\n\
            📁 Project: {}\n\
            🆔 Spec ID: {}\n\
            📋 Status: Draft\n\
            📝 Description: {}\n\
            🚀 You can now load this specification using the load_spec tool.",
            spec_name, project_name, spec_id, description
        ))
    }

    /// Validate spec name format (snake_case)
    fn is_valid_spec_name(&self, name: &str) -> bool {
        if name.is_empty() || name.starts_with('_') || name.ends_with('_') {
            return false;
        }

        name.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            && !name.contains("__") // No consecutive underscores
    }

    /// Generate spec ID in YYYYMMDD_name format
    fn generate_spec_id(&self, name: &str) -> String {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        format!("{}_{}", date_str, name)
    }
}
