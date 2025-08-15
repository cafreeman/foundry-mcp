//! Update specification tool handler

use crate::errors::{self, Result};
use crate::models::{Note, NoteCategory, Task, TaskPriority, TaskStatus};
use crate::repository::{ProjectRepository, SpecificationRepository};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

/// Handler for updating specifications
#[derive(Clone)]
pub struct UpdateSpecHandler {
    project_repo: ProjectRepository,
    spec_repo: SpecificationRepository,
}

impl UpdateSpecHandler {
    /// Create a new UpdateSpecHandler instance
    pub fn new(project_repo: ProjectRepository, spec_repo: SpecificationRepository) -> Self {
        Self {
            project_repo,
            spec_repo,
        }
    }

    /// Handle update_spec tool calls
    pub async fn handle_update_spec(&self, arguments: &Value) -> Result<String> {
        let project_name = arguments["project_name"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("project_name", "missing", "Project name is required")
        })?;
        let spec_id = arguments["spec_id"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("spec_id", "missing", "Spec ID is required")
        })?;

        // Verify project and spec exist
        if !self.project_repo.project_exists(project_name).await {
            return Err(errors::helpers::project_not_found(project_name));
        }

        if !self.spec_repo.spec_exists(project_name, spec_id).await {
            return Err(errors::helpers::spec_not_found(spec_id, project_name));
        }

        // Determine the operation type
        let operation = arguments["operation"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("operation", "missing", "Operation is required")
        })?;

        match operation {
            "add_task" => self.handle_add_task(project_name, spec_id, arguments).await,
            "update_task" => {
                self.handle_update_task(project_name, spec_id, arguments)
                    .await
            }
            "remove_task" => {
                self.handle_remove_task(project_name, spec_id, arguments)
                    .await
            }
            "update_task_status" => {
                self.handle_update_task_status(project_name, spec_id, arguments)
                    .await
            }
            "add_note" => self.handle_add_note(project_name, spec_id, arguments).await,
            "reorder_tasks" => self.handle_reorder_tasks(project_name, spec_id).await,
            _ => Err(errors::helpers::validation_error(
                "operation",
                operation,
                "Unknown operation",
            )),
        }
    }

    async fn handle_add_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let title = arguments["title"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("title", "missing", "Task title is required")
        })?;
        let description = arguments["description"]
            .as_str()
            .unwrap_or("No description provided");
        let priority = arguments["priority"].as_str().unwrap_or("medium");

        let priority_enum = match priority.to_lowercase().as_str() {
            "critical" => TaskPriority::Critical,
            "high" => TaskPriority::High,
            "medium" => TaskPriority::Medium,
            "low" => TaskPriority::Low,
            _ => TaskPriority::Medium,
        };

        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Todo,
            priority: priority_enum,
            dependencies: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.spec_repo
            .add_task(project_name, spec_id, task.clone())
            .await?;

        Ok(format!(
            "Successfully added task '{}' to specification '{}'",
            task.title, spec_id
        ))
    }

    async fn handle_update_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_id = arguments["task_id"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("task_id", "missing", "Task ID is required")
        })?;
        let title = arguments["title"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("title", "missing", "Task title is required")
        })?;
        let description = arguments["description"]
            .as_str()
            .unwrap_or("No description provided");

        // Load existing task to preserve other fields
        let task_list = self.spec_repo.load_task_list(project_name, spec_id).await?;
        let existing_task = task_list
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| errors::helpers::spec_not_found(task_id, spec_id))?;

        let updated_task = Task {
            id: task_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: existing_task.status.clone(),
            priority: existing_task.priority.clone(),
            dependencies: existing_task.dependencies.clone(),
            created_at: existing_task.created_at,
            updated_at: Utc::now(),
        };

        self.spec_repo
            .update_task(project_name, spec_id, updated_task.clone())
            .await?;

        Ok(format!(
            "Successfully updated task '{}' in specification '{}'",
            updated_task.title, spec_id
        ))
    }

    async fn handle_remove_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_id = arguments["task_id"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("task_id", "missing", "Task ID is required")
        })?;

        self.spec_repo
            .remove_task(project_name, spec_id, task_id)
            .await?;

        Ok(format!(
            "Successfully removed task '{}' from specification '{}'",
            task_id, spec_id
        ))
    }

    async fn handle_update_task_status(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_id = arguments["task_id"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("task_id", "missing", "Task ID is required")
        })?;
        let status_str = arguments["status"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("status", "missing", "Task status is required")
        })?;

        let status = match status_str.to_lowercase().as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            _ => {
                return Err(errors::helpers::validation_error(
                    "status",
                    status_str,
                    "Invalid status value",
                ));
            }
        };

        self.spec_repo
            .update_task_status(project_name, spec_id, task_id, status.clone())
            .await?;

        Ok(format!(
            "Successfully updated task '{}' status to '{:?}' in specification '{}'",
            task_id, status, spec_id
        ))
    }

    async fn handle_add_note(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let content = arguments["content"].as_str().ok_or_else(|| {
            errors::helpers::validation_error("content", "missing", "Note content is required")
        })?;
        let category_str = arguments["category"].as_str().unwrap_or("other");

        let category = match category_str.to_lowercase().as_str() {
            "implementation" => NoteCategory::Implementation,
            "decision" => NoteCategory::Decision,
            "question" => NoteCategory::Question,
            "bug" => NoteCategory::Bug,
            "enhancement" => NoteCategory::Enhancement,
            _ => NoteCategory::Other,
        };

        let note = Note {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            category,
            created_at: Utc::now(),
        };

        self.spec_repo
            .add_note(project_name, spec_id, note.clone())
            .await?;

        Ok(format!(
            "Successfully added note to specification '{}'",
            spec_id
        ))
    }

    async fn handle_reorder_tasks(&self, project_name: &str, spec_id: &str) -> Result<String> {
        self.spec_repo.reorder_tasks(project_name, spec_id).await?;

        Ok(format!(
            "Successfully reordered tasks in specification '{}' by priority",
            spec_id
        ))
    }
}
