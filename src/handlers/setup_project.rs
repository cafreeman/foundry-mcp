//! Setup project tool handler

use crate::errors::{self, Result};
use crate::models::{Project, TechStack, Vision};
use crate::repository::ProjectRepository;
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
            .ok_or_else(|| errors::helpers::invalid_project_name("missing"))?;

        let description = arguments["description"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("description", "missing", "Description is required")
        })?;

        let overview = arguments["overview"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("overview", "missing", "Overview is required")
        })?;

        // Validate project name (no special characters)
        if !self.is_valid_project_name(project_name) {
            return Err(errors::helpers::invalid_project_name(project_name));
        }

        // Check if project already exists
        if self.project_repo.project_exists(project_name).await {
            return Err(errors::helpers::project_already_exists(project_name));
        }

        // Parse optional arrays with defaults
        let languages = self.parse_string_array(&arguments["languages"], vec![]);
        let frameworks = self.parse_string_array(&arguments["frameworks"], vec![]);
        let databases = self.parse_string_array(&arguments["databases"], vec![]);
        let tools = self.parse_string_array(&arguments["tools"], vec![]);
        let deployment = self.parse_string_array(&arguments["deployment"], vec![]);

        // Parse goals, target users, and success criteria
        let goals = self.parse_string_array(&arguments["goals"], vec![]);
        let target_users = self.parse_string_array(&arguments["target_users"], vec![]);
        let success_criteria = self.parse_string_array(&arguments["success_criteria"], vec![]);

        // Create tech stack
        let tech_stack = TechStack {
            languages,
            frameworks,
            databases,
            tools,
            deployment,
        };

        // Create vision
        let vision = Vision {
            overview: overview.to_string(),
            goals,
            target_users,
            success_criteria,
        };

        // Create project
        let project = Project {
            name: project_name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tech_stack,
            vision,
        };

        // Save project
        self.project_repo.create_project(project).await?;

        Ok(format!(
            "Successfully created project '{}' with tech stack and vision documents",
            project_name
        ))
    }

    /// Validate project name (no special characters)
    fn is_valid_project_name(&self, name: &str) -> bool {
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.starts_with('_')
            && !name.ends_with('_')
            && !name.is_empty()
    }

    /// Parse string array from JSON value with default
    fn parse_string_array(&self, value: &Value, default: Vec<String>) -> Vec<String> {
        if let Some(array) = value.as_array() {
            array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            default
        }
    }
}
