//! Specification repository for data access operations

use crate::filesystem::FileSystemManager;
use crate::models::{Note, NoteCategory, Specification, Task, TaskList, TaskPriority, TaskStatus};
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;

/// Repository for specification data access operations
#[derive(Clone)]
pub struct SpecificationRepository {
    fs_manager: FileSystemManager,
}

impl SpecificationRepository {
    /// Create a new SpecificationRepository instance
    pub fn new(fs_manager: FileSystemManager) -> Self {
        Self { fs_manager }
    }

    /// Create a new specification
    pub async fn create_spec(&self, project_name: &str, spec: Specification) -> Result<()> {
        // Verify project exists
        if !self.fs_manager.project_exists(project_name) {
            return Err(anyhow::anyhow!("Project '{}' does not exist", project_name));
        }

        // Check if spec already exists
        if self.fs_manager.spec_exists(project_name, &spec.id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' already exists in project '{}'",
                spec.id,
                project_name
            ));
        }

        // Create spec directory structure
        self.fs_manager
            .create_spec_structure(project_name, &spec.id)?;

        // Save spec metadata
        self.save_spec_metadata(project_name, &spec)?;

        // Generate and save spec.md
        let spec_content = self.render_spec_content(&spec);
        let spec_path = self
            .fs_manager
            .spec_dir(project_name, &spec.id)
            .join("spec.md");
        self.fs_manager.write_file_safe(&spec_path, &spec_content)?;

        // Create empty task-list.md
        let empty_task_list = TaskList {
            tasks: Vec::new(),
            last_updated: Utc::now(),
        };
        let task_list_content = self.render_task_list(&empty_task_list);
        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, &spec.id)
            .join("task-list.md");
        self.fs_manager
            .write_file_safe(&task_list_path, &task_list_content)?;

        // Create empty notes.md
        let notes_content = self.render_notes(&[]);
        let notes_path = self
            .fs_manager
            .spec_dir(project_name, &spec.id)
            .join("notes.md");
        self.fs_manager
            .write_file_safe(&notes_path, &notes_content)?;

        Ok(())
    }

    /// Load a specification from the file system
    pub async fn load_spec(&self, project_name: &str, spec_id: &str) -> Result<Specification> {
        if !self.fs_manager.spec_exists(project_name, spec_id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec_id,
                project_name
            ));
        }

        // Load spec metadata
        let metadata_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("spec.json");
        let metadata_content = self.fs_manager.read_file(&metadata_path)?;
        let spec: Specification = serde_json::from_str(&metadata_content)
            .with_context(|| format!("Failed to parse spec metadata for '{}'", spec_id))?;

        Ok(spec)
    }

    /// Update an existing specification
    pub async fn update_spec(&self, project_name: &str, spec: &Specification) -> Result<()> {
        if !self.fs_manager.spec_exists(project_name, &spec.id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec.id,
                project_name
            ));
        }

        // Save updated spec metadata
        self.save_spec_metadata(project_name, spec)?;

        // Update spec.md
        let spec_content = self.render_spec_content(spec);
        let spec_path = self
            .fs_manager
            .spec_dir(project_name, &spec.id)
            .join("spec.md");
        self.fs_manager.write_file_safe(&spec_path, &spec_content)?;

        Ok(())
    }

    /// List all specifications for a project
    pub async fn list_specs_for_project(&self, project_name: &str) -> Result<Vec<String>> {
        self.fs_manager.list_specs(project_name)
    }

    /// Delete a specification and all its contents
    pub async fn delete_spec(
        &self,
        project_name: &str,
        spec_id: &str,
        confirm: bool,
    ) -> Result<()> {
        if !confirm {
            return Err(anyhow::anyhow!("Specification deletion not confirmed"));
        }

        if !self.fs_manager.spec_exists(project_name, spec_id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec_id,
                project_name
            ));
        }

        let spec_path = self.fs_manager.spec_dir(project_name, spec_id);

        // Remove the entire spec directory
        std::fs::remove_dir_all(&spec_path)
            .with_context(|| format!("Failed to delete spec directory: {:?}", spec_path))?;

        Ok(())
    }

    /// Load task list for a specification
    pub async fn load_task_list(&self, project_name: &str, spec_id: &str) -> Result<TaskList> {
        if !self.fs_manager.spec_exists(project_name, spec_id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec_id,
                project_name
            ));
        }

        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("task-list.md");

        if !self.fs_manager.file_exists(&task_list_path) {
            // Return empty task list if file doesn't exist
            return Ok(TaskList {
                tasks: Vec::new(),
                last_updated: Utc::now(),
            });
        }

        // For now, return empty task list - parsing markdown would require additional dependencies
        // TODO: Implement markdown parsing for task list
        Ok(TaskList {
            tasks: Vec::new(),
            last_updated: Utc::now(),
        })
    }

    /// Load notes for a specification
    pub async fn load_notes(&self, project_name: &str, spec_id: &str) -> Result<Vec<Note>> {
        if !self.fs_manager.spec_exists(project_name, spec_id) {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'",
                spec_id,
                project_name
            ));
        }

        let notes_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("notes.md");

        if !self.fs_manager.file_exists(&notes_path) {
            return Ok(Vec::new());
        }

        // For now, return empty notes - parsing markdown would require additional dependencies
        // TODO: Implement markdown parsing for notes
        Ok(Vec::new())
    }

    /// Add a task to the task list
    pub async fn add_task(&self, project_name: &str, spec_id: &str, task: Task) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;
        task_list.tasks.push(task);
        task_list.last_updated = Utc::now();

        let task_list_content = self.render_task_list(&task_list);
        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("task-list.md");
        self.fs_manager
            .write_file_safe(&task_list_path, &task_list_content)?;

        Ok(())
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        project_name: &str,
        spec_id: &str,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;

        if let Some(task) = task_list.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = status;
            task.updated_at = Utc::now();
            task_list.last_updated = Utc::now();

            let task_list_content = self.render_task_list(&task_list);
            let task_list_path = self
                .fs_manager
                .spec_dir(project_name, spec_id)
                .join("task-list.md");
            self.fs_manager
                .write_file_safe(&task_list_path, &task_list_content)?;
        }

        Ok(())
    }

    /// Get the next available task
    pub async fn get_next_task(&self, project_name: &str, spec_id: &str) -> Result<Option<Task>> {
        let task_list = self.load_task_list(project_name, spec_id).await?;

        // Find the first task that's not completed
        let next_task = task_list
            .tasks
            .iter()
            .find(|task| !matches!(task.status, TaskStatus::Completed))
            .cloned();

        Ok(next_task)
    }

    /// Render specification content as markdown
    pub fn render_spec_content(&self, spec: &Specification) -> String {
        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", spec.name));

        if !spec.description.is_empty() {
            content.push_str(&format!("## Description\n\n{}\n\n", spec.description));
        }

        content.push_str(&format!("## Status\n\n{:?}\n\n", spec.status));
        content.push_str(&format!("## Content\n\n{}\n\n", spec.content));
        content.push_str(&format!(
            "Created: {}\n",
            spec.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        content.push_str(&format!(
            "Updated: {}\n",
            spec.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        content
    }

    /// Render task list as markdown
    pub fn render_task_list(&self, task_list: &TaskList) -> String {
        let mut content = String::new();
        content.push_str("# Task List\n\n");
        content.push_str(&format!(
            "Last Updated: {}\n\n",
            task_list.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if task_list.tasks.is_empty() {
            content.push_str("No tasks yet.\n");
            return content;
        }

        // Group tasks by status
        let mut tasks_by_status: HashMap<TaskStatus, Vec<&Task>> = HashMap::new();
        for task in &task_list.tasks {
            tasks_by_status
                .entry(task.status.clone())
                .or_default()
                .push(task);
        }

        // Render tasks by status
        let status_order = [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
        ];

        for status in status_order.iter() {
            if let Some(tasks) = tasks_by_status.get(status)
                && !tasks.is_empty()
            {
                content.push_str(&format!("## {:?}\n", status));
                for task in tasks {
                    content.push_str(&format!(
                        "- **{}** (Priority: {:?})\n",
                        task.title, task.priority
                    ));
                    if !task.description.is_empty() {
                        content.push_str(&format!("  {}\n", task.description));
                    }
                    if !task.dependencies.is_empty() {
                        content.push_str(&format!(
                            "  Dependencies: {}\n",
                            task.dependencies.join(", ")
                        ));
                    }
                    content.push('\n');
                }
            }
        }

        content
    }

    /// Render notes as markdown
    pub fn render_notes(&self, notes: &[Note]) -> String {
        let mut content = String::new();
        content.push_str("# Notes\n\n");

        if notes.is_empty() {
            content.push_str("No notes yet.\n");
            return content;
        }

        // Group notes by category
        let mut notes_by_category: HashMap<NoteCategory, Vec<&Note>> = HashMap::new();
        for note in notes {
            notes_by_category
                .entry(note.category.clone())
                .or_default()
                .push(note);
        }

        // Render notes by category
        for (category, category_notes) in notes_by_category.iter() {
            content.push_str(&format!("## {:?}\n", category));
            for note in category_notes {
                content.push_str(&format!("- {}\n", note.content));
                content.push_str(&format!(
                    "  *Added: {}*\n\n",
                    note.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));
            }
        }

        content
    }

    /// Save specification metadata to JSON file
    fn save_spec_metadata(&self, project_name: &str, spec: &Specification) -> Result<()> {
        let metadata_path = self
            .fs_manager
            .spec_dir(project_name, &spec.id)
            .join("spec.json");
        let metadata_content = serde_json::to_string_pretty(spec)
            .with_context(|| "Failed to serialize spec metadata")?;

        self.fs_manager
            .write_file_safe(&metadata_path, &metadata_content)
    }

    /// Check if a specification exists
    pub async fn spec_exists(&self, project_name: &str, spec_id: &str) -> bool {
        self.fs_manager.spec_exists(project_name, spec_id)
    }

    /// Add a note to the specification
    pub async fn add_note(&self, project_name: &str, spec_id: &str, note: Note) -> Result<()> {
        let mut notes = self.load_notes(project_name, spec_id).await?;
        notes.push(note);

        let notes_content = self.render_notes(&notes);
        let notes_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("notes.md");
        self.fs_manager
            .write_file_safe(&notes_path, &notes_content)?;

        Ok(())
    }

    /// Remove a task from the task list
    pub async fn remove_task(
        &self,
        project_name: &str,
        spec_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;
        task_list.tasks.retain(|t| t.id != task_id);
        task_list.last_updated = Utc::now();

        let task_list_content = self.render_task_list(&task_list);
        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("task-list.md");
        self.fs_manager
            .write_file_safe(&task_list_path, &task_list_content)?;

        Ok(())
    }

    /// Update an existing task
    pub async fn update_task(&self, project_name: &str, spec_id: &str, task: Task) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;

        if let Some(existing_task) = task_list.tasks.iter_mut().find(|t| t.id == task.id) {
            *existing_task = task;
            task_list.last_updated = Utc::now();

            let task_list_content = self.render_task_list(&task_list);
            let task_list_path = self
                .fs_manager
                .spec_dir(project_name, spec_id)
                .join("task-list.md");
            self.fs_manager
                .write_file_safe(&task_list_path, &task_list_content)?;
        } else {
            return Err(anyhow::anyhow!("Task with ID '{}' not found", task.id));
        }

        Ok(())
    }

    /// Reorder tasks by priority
    pub async fn reorder_tasks(&self, project_name: &str, spec_id: &str) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;

        // Sort tasks by priority (Critical -> High -> Medium -> Low)
        task_list.tasks.sort_by(|a, b| {
            let priority_order = |p: &TaskPriority| match p {
                TaskPriority::Critical => 0,
                TaskPriority::High => 1,
                TaskPriority::Medium => 2,
                TaskPriority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        task_list.last_updated = Utc::now();

        let task_list_content = self.render_task_list(&task_list);
        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("task-list.md");
        self.fs_manager
            .write_file_safe(&task_list_path, &task_list_content)?;

        Ok(())
    }
}
