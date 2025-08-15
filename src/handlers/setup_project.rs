//! Setup project tool handler

use crate::models::{Project, TechStack, Vision};
use crate::repository::ProjectRepository;
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

/// Handler for setting up new projects
pub struct SetupProjectHandler {
    project_repo: ProjectRepository,
}

impl SetupProjectHandler {
    /// Create a new SetupProjectHandler instance
    pub fn new(project_repo: ProjectRepository) -> Self {
        Self { project_repo }
    }

    /// Handle the setup_project tool call
    pub async fn handle_setup_project(&self, arguments: &Value) -> Result<String> {
        // Parse and validate arguments
        let project_name = arguments["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: name"))?;

        let description = arguments["description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: description"))?;

        let overview = arguments["overview"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: overview"))?;

        // Validate project name (no special characters)
        if !self.is_valid_project_name(project_name) {
            return Err(anyhow::anyhow!(
                "Invalid project name: '{}'. Project names cannot contain special characters or spaces.",
                project_name
            ));
        }

        // Check if project already exists
        if self.project_repo.project_exists(project_name).await {
            return Err(anyhow::anyhow!(
                "Project '{}' already exists. Please choose a different name.",
                project_name
            ));
        }

        // Parse optional arrays with defaults
        let languages = self.parse_string_array(&arguments["languages"], vec![]);
        let frameworks = self.parse_string_array(&arguments["frameworks"], vec![]);
        let databases = self.parse_string_array(&arguments["databases"], vec![]);
        let tools = self.parse_string_array(&arguments["tools"], vec![]);
        let deployment = self.parse_string_array(&arguments["deployment"], vec![]);
        let goals = self.parse_string_array(&arguments["goals"], vec![]);
        let target_users = self.parse_string_array(&arguments["target_users"], vec![]);
        let success_criteria = self.parse_string_array(&arguments["success_criteria"], vec![]);

        // Create the project
        let project = Project {
            name: project_name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tech_stack: TechStack {
                languages,
                frameworks,
                databases,
                tools,
                deployment,
            },
            vision: Vision {
                overview: overview.to_string(),
                goals,
                target_users,
                success_criteria,
            },
        };

        // Save the project
        self.project_repo.create_project(project).await?;

        Ok(format!(
            "✅ Project '{}' created successfully!\n\n\
            📁 Project directory: ~/.project-manager-mcp/{}\n\
            📋 Tech stack and vision files have been generated.\n\
            🚀 You can now create specifications using the create_spec tool.",
            project_name, project_name
        ))
    }

    /// Validate project name (no special characters or spaces)
    fn is_valid_project_name(&self, name: &str) -> bool {
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.is_empty()
            && !name.starts_with('-')
            && !name.starts_with('_')
    }

    /// Parse a JSON array of strings, returning default if not present
    fn parse_string_array(&self, value: &Value, default: Vec<String>) -> Vec<String> {
        value
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
                    .into()
            })
            .unwrap_or(default)
    }
}
