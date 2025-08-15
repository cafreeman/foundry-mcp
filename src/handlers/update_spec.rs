//! Update specification tool handler

use crate::models::{Note, NoteCategory, Task, TaskPriority, TaskStatus};
use crate::repository::{ProjectRepository, SpecificationRepository};
use anyhow::Result;
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
        let project_name = arguments["project_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing project_name"))?;
        let spec_id = arguments["spec_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing spec_id"))?;

        // Verify project and spec exist
        if !self.project_repo.project_exists(project_name).await {
            return Err(anyhow::anyhow!("Project '{}' does not exist", project_name));
        }

        if !self.spec_repo.spec_exists(project_name, spec_id).await {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec_id,
                project_name
            ));
        }

        // Determine the operation type
        let operation = arguments["operation"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;

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
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation)),
        }
    }

    /// Handle adding a new task
    async fn handle_add_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_data = arguments["task"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Missing task data"))?;

        let title = task_data["title"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing task title"))?;
        let description = task_data["description"].as_str().unwrap_or("").to_string();

        let priority = match task_data["priority"].as_str().unwrap_or("medium") {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };

        let dependencies = if let Some(deps) = task_data["dependencies"].as_array() {
            deps.iter()
                .filter_map(|d| d.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            description,
            status: TaskStatus::Todo,
            priority,
            dependencies,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.spec_repo
            .add_task(project_name, spec_id, task.clone())
            .await?;

        Ok(format!(
            "Task '{}' added successfully with ID: {}",
            task.title, task.id
        ))
    }

    /// Handle updating an existing task
    async fn handle_update_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_data = arguments["task"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Missing task data"))?;

        let task_id = task_data["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing task ID"))?;

        // Load existing task to preserve unchanged fields
        let task_list = self.spec_repo.load_task_list(project_name, spec_id).await?;
        let existing_task = task_list
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task with ID '{}' not found", task_id))?;

        let mut updated_task = existing_task.clone();

        // Update fields if provided
        if let Some(title) = task_data["title"].as_str() {
            updated_task.title = title.to_string();
        }
        if let Some(description) = task_data["description"].as_str() {
            updated_task.description = description.to_string();
        }
        if let Some(priority_str) = task_data["priority"].as_str() {
            updated_task.priority = match priority_str {
                "low" => TaskPriority::Low,
                "medium" => TaskPriority::Medium,
                "high" => TaskPriority::High,
                "critical" => TaskPriority::Critical,
                _ => updated_task.priority,
            };
        }
        if let Some(status_str) = task_data["status"].as_str() {
            updated_task.status = match status_str {
                "todo" => TaskStatus::Todo,
                "in_progress" => TaskStatus::InProgress,
                "completed" => TaskStatus::Completed,
                "blocked" => TaskStatus::Blocked,
                _ => updated_task.status,
            };
        }
        if let Some(deps) = task_data["dependencies"].as_array() {
            updated_task.dependencies = deps
                .iter()
                .filter_map(|d| d.as_str())
                .map(|s| s.to_string())
                .collect();
        }

        updated_task.updated_at = Utc::now();

        self.spec_repo
            .update_task(project_name, spec_id, updated_task.clone())
            .await?;

        Ok(format!(
            "Task '{}' updated successfully",
            updated_task.title
        ))
    }

    /// Handle removing a task
    async fn handle_remove_task(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_id = arguments["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?;

        self.spec_repo
            .remove_task(project_name, spec_id, task_id)
            .await?;

        Ok(format!("Task '{}' removed successfully", task_id))
    }

    /// Handle updating task status
    async fn handle_update_task_status(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let task_id = arguments["task_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing task_id"))?;
        let status_str = arguments["status"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing status"))?;

        let status = match status_str {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            _ => return Err(anyhow::anyhow!("Invalid status: {}", status_str)),
        };

        self.spec_repo
            .update_task_status(project_name, spec_id, task_id, status.clone())
            .await?;

        Ok(format!("Task '{}' status updated to {:?}", task_id, status))
    }

    /// Handle adding a note
    async fn handle_add_note(
        &self,
        project_name: &str,
        spec_id: &str,
        arguments: &Value,
    ) -> Result<String> {
        let content = arguments["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing note content"))?;

        let category_str = arguments["category"].as_str().unwrap_or("other");

        let category = match category_str {
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

        Ok(format!("Note added successfully with ID: {}", note.id))
    }

    /// Handle reordering tasks by priority
    async fn handle_reorder_tasks(&self, project_name: &str, spec_id: &str) -> Result<String> {
        self.spec_repo.reorder_tasks(project_name, spec_id).await?;

        Ok("Tasks reordered successfully by priority".to_string())
    }
}
