//! Specification repository for data access operations

use crate::cache::ProjectManagerCache;
use crate::errors::{self, Result};
use crate::filesystem::FileSystemManager;
use crate::models::{Note, NoteCategory, Specification, Task, TaskList, TaskPriority, TaskStatus};
use chrono::Utc;
use serde_json;

/// Repository for specification data access operations
#[derive(Clone)]
pub struct SpecificationRepository {
    fs_manager: FileSystemManager,
    cache: ProjectManagerCache,
}

impl SpecificationRepository {
    /// Create a new SpecificationRepository instance
    pub fn new(fs_manager: FileSystemManager) -> Self {
        Self { 
            fs_manager,
            cache: ProjectManagerCache::new(),
        }
    }

    /// Create a new SpecificationRepository instance with shared cache
    pub fn with_cache(fs_manager: FileSystemManager, cache: ProjectManagerCache) -> Self {
        Self { fs_manager, cache }
    }

    /// Create a new specification
    pub async fn create_spec(
        &self,
        project_name: &str,
        spec_name: &str,
        description: &str,
        content: &str,
    ) -> Result<Specification> {
        // Validate spec name format (snake_case)
        if !self.is_valid_spec_name(spec_name) {
            return Err(errors::helpers::invalid_spec_name(spec_name));
        }

        let spec_id = self.generate_spec_id(spec_name);
        let spec_path = self.fs_manager.spec_dir(project_name, &spec_id);

        // Check if spec already exists
        if self.fs_manager.spec_exists(project_name, &spec_id) {
            return Err(errors::helpers::spec_already_exists(&spec_id, project_name));
        }

        // Create spec directory structure
        self.fs_manager
            .create_spec_structure(project_name, &spec_id)?;

        // Create specification
        let spec = Specification {
            id: spec_id.clone(),
            name: spec_name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: crate::models::SpecStatus::Draft,
        };

        // Save spec metadata
        let metadata_path = spec_path.join("spec.json");
        let metadata_content = serde_json::to_string_pretty(&spec).map_err(|e| {
            errors::helpers::serialization_error("serialize spec metadata", "spec data", e)
        })?;
        self.fs_manager
            .write_file_safe(&metadata_path, &metadata_content)?;

        // Save spec content
        let content_path = spec_path.join("spec.md");
        self.fs_manager.write_file_safe(&content_path, content)?;

        // Create initial task list
        let task_list = TaskList {
            tasks: Vec::new(),
            last_updated: Utc::now(),
        };
        let task_list_content = self.render_task_list(&task_list);
        let task_list_path = spec_path.join("task-list.md");
        self.fs_manager
            .write_file_safe(&task_list_path, &task_list_content)?;

        // Create initial notes file
        let notes_content = self.render_notes(&Vec::new());
        let notes_path = spec_path.join("notes.md");
        self.fs_manager
            .write_file_safe(&notes_path, &notes_content)?;

        // Cache the newly created specification
        let cache_key = format!("{}:{}", project_name, spec_id);
        self.cache.cache_specification(&cache_key, spec.clone());
        
        // Invalidate spec lists since we added a new spec
        self.cache.invalidate_specification(project_name, &spec_id);

        Ok(spec)
    }

    /// Load a specification from the file system
    pub async fn load_spec(&self, project_name: &str, spec_id: &str) -> Result<Specification> {
        let cache_key = format!("{}:{}", project_name, spec_id);
        
        // Check cache first
        if let Some(cached_spec) = self.cache.get_specification(&cache_key) {
            tracing::debug!("Retrieved specification '{}:{}' from cache", project_name, spec_id);
            return Ok(cached_spec);
        }

        if !self.fs_manager.spec_exists(project_name, spec_id) {
            return Err(errors::helpers::spec_not_found(spec_id, project_name));
        }

        tracing::debug!("Loading specification '{}:{}' from filesystem", project_name, spec_id);

        let spec_path = self.fs_manager.spec_dir(project_name, spec_id);
        let metadata_path = spec_path.join("spec.json");
        let metadata_content = self.fs_manager.read_file(&metadata_path)?;
        let spec: Specification = serde_json::from_str(&metadata_content).map_err(|e| {
            errors::helpers::serialization_error("parse spec metadata", &metadata_content, e)
        })?;

        // Cache the loaded specification
        self.cache.cache_specification(&cache_key, spec.clone());

        Ok(spec)
    }

    /// Check if a specification exists
    pub async fn spec_exists(&self, project_name: &str, spec_id: &str) -> bool {
        self.fs_manager.spec_exists(project_name, spec_id)
    }

    /// List all specifications for a project
    pub async fn list_specs(&self, project_name: &str) -> Result<Vec<Specification>> {
        // Check if we have a cached list of spec IDs
        let spec_ids = if let Some(cached_ids) = self.cache.get_spec_list(project_name) {
            tracing::debug!("Retrieved spec list for '{}' from cache", project_name);
            cached_ids
        } else {
            tracing::debug!("Loading spec list for '{}' from filesystem", project_name);
            let ids = self.fs_manager.list_specs(project_name)?;
            self.cache.cache_spec_list(project_name, ids.clone());
            ids
        };

        let mut specs = Vec::new();

        for spec_id in spec_ids {
            match self.load_spec(project_name, &spec_id).await {
                Ok(spec) => specs.push(spec),
                Err(e) => {
                    // Log the error but continue with other specs
                    tracing::warn!("Failed to load spec '{}': {}", spec_id, e);
                }
            }
        }

        Ok(specs)
    }

    /// Load task list for a specification
    pub async fn load_task_list(&self, project_name: &str, spec_id: &str) -> Result<TaskList> {
        let task_list_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("task-list.md");
        let content = self.fs_manager.read_file(&task_list_path)?;
        self.parse_task_list(&content)
    }

    /// Load notes for a specification
    pub async fn load_notes(&self, project_name: &str, spec_id: &str) -> Result<Vec<Note>> {
        let notes_path = self
            .fs_manager
            .spec_dir(project_name, spec_id)
            .join("notes.md");
        let content = self.fs_manager.read_file(&notes_path)?;
        self.parse_notes(&content)
    }

    /// Add a new task to a specification
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

    /// Remove a task from a specification
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

    /// Update an existing task in a specification
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
            return Err(errors::helpers::spec_not_found(&task.id, spec_id));
        }
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
            task.status = status.clone();
            task_list.last_updated = Utc::now();
            let task_list_content = self.render_task_list(&task_list);
            let task_list_path = self
                .fs_manager
                .spec_dir(project_name, spec_id)
                .join("task-list.md");
            self.fs_manager
                .write_file_safe(&task_list_path, &task_list_content)?;
        } else {
            return Err(errors::helpers::spec_not_found(task_id, spec_id));
        }
        Ok(())
    }

    /// Add a new note to a specification
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

    /// Reorder tasks by priority
    pub async fn reorder_tasks(&self, project_name: &str, spec_id: &str) -> Result<()> {
        let mut task_list = self.load_task_list(project_name, spec_id).await?;
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

    /// Update an existing specification
    pub async fn update_spec(&self, project_name: &str, spec: &Specification) -> Result<()> {
        let spec_path = self.fs_manager.spec_dir(project_name, &spec.id);
        
        if !spec_path.exists() {
            return Err(errors::helpers::spec_not_found(&spec.id, project_name));
        }

        // Update spec metadata
        let spec_metadata_path = spec_path.join("spec.json");
        let spec_json = serde_json::to_string_pretty(spec)
            .map_err(|e| errors::helpers::serialization_error("update_spec", "specification", e))?;
        self.fs_manager.write_file_safe(&spec_metadata_path, &spec_json)?;

        // Update spec content
        let spec_content_path = spec_path.join("spec.md");
        self.fs_manager.write_file_safe(&spec_content_path, &spec.content)?;

        Ok(())
    }

    /// Generate a unique spec ID
    fn generate_spec_id(&self, spec_name: &str) -> String {
        let date = Utc::now().format("%Y%m%d");
        format!("{}_{}", date, spec_name)
    }

    /// Validate spec name format (snake_case)
    fn is_valid_spec_name(&self, name: &str) -> bool {
        name.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            && !name.starts_with('_')
            && !name.ends_with('_')
            && !name.contains("__")
    }

    /// Parse task list from markdown content
    fn parse_task_list(&self, content: &str) -> Result<TaskList> {
        let mut tasks = Vec::new();
        let mut current_task: Option<Task> = None;
        let mut current_description = String::new();

        for line in content.lines() {
            if line.starts_with("## ") {
                // Save previous task if exists
                if let Some(mut task) = current_task.take() {
                    task.description = current_description.trim().to_string();
                    tasks.push(task);
                    current_description.clear();
                }

                // Parse new task header
                let header = line.trim_start_matches("## ");
                if let Some(task) = self.parse_task_header(header) {
                    current_task = Some(task);
                }
            } else if line.starts_with("- ") && current_task.is_some() {
                // Parse task metadata
                self.parse_task_metadata(line, current_task.as_mut().unwrap());
            } else if current_task.is_some() {
                // Add to description
                current_description.push_str(line);
                current_description.push('\n');
            }
        }

        // Add the last task
        if let Some(mut task) = current_task {
            task.description = current_description.trim().to_string();
            tasks.push(task);
        }

        Ok(TaskList {
            tasks,
            last_updated: Utc::now(),
        })
    }

    /// Parse task header to extract basic info
    fn parse_task_header(&self, header: &str) -> Option<Task> {
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let id = parts[0].to_string();
        let title = parts[1..].join(" ");

        Some(Task {
            id,
            title,
            description: String::new(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            dependencies: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Parse task metadata from markdown list items
    fn parse_task_metadata(&self, line: &str, task: &mut Task) {
        let line = line.trim_start_matches("- ");
        if line.starts_with("Status: ") {
            if let Some(status_str) = line.strip_prefix("Status: ") {
                task.status = self.parse_status(status_str);
            }
        } else if line.starts_with("Priority: ") {
            if let Some(priority_str) = line.strip_prefix("Priority: ") {
                task.priority = self.parse_priority(priority_str);
            }
        } else if line.starts_with("Dependencies: ")
            && let Some(deps_str) = line.strip_prefix("Dependencies: ") {
                task.dependencies = deps_str.split(',').map(|s| s.trim().to_string()).collect();
            }
    }

    /// Parse status from string
    fn parse_status(&self, status: &str) -> TaskStatus {
        match status.to_lowercase().as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "blocked" => TaskStatus::Blocked,
            _ => TaskStatus::Todo,
        }
    }

    /// Parse priority from string
    fn parse_priority(&self, priority: &str) -> TaskPriority {
        match priority.to_lowercase().as_str() {
            "critical" => TaskPriority::Critical,
            "high" => TaskPriority::High,
            "medium" => TaskPriority::Medium,
            "low" => TaskPriority::Low,
            _ => TaskPriority::Medium,
        }
    }

    /// Parse notes from markdown content
    fn parse_notes(&self, content: &str) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        let mut current_note: Option<Note> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            if line.starts_with("## ") {
                // Save previous note if exists
                if let Some(mut note) = current_note.take() {
                    note.content = current_content.trim().to_string();
                    notes.push(note);
                    current_content.clear();
                }

                // Parse new note header
                let header = line.trim_start_matches("## ");
                if let Some(note) = self.parse_note_header(header) {
                    current_note = Some(note);
                }
            } else if line.starts_with("- ") && current_note.is_some() {
                // Parse note metadata
                self.parse_note_metadata(line, current_note.as_mut().unwrap());
            } else if current_note.is_some() {
                // Add to content
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Add the last note
        if let Some(mut note) = current_note {
            note.content = current_content.trim().to_string();
            notes.push(note);
        }

        Ok(notes)
    }

    /// Parse note header to extract basic info
    fn parse_note_header(&self, header: &str) -> Option<Note> {
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let id = parts[0].to_string();
        let content = parts[1..].join(" ");

        Some(Note {
            id,
            content,
            category: NoteCategory::Other,
            created_at: Utc::now(),
        })
    }

    /// Parse note metadata from markdown list items
    fn parse_note_metadata(&self, line: &str, note: &mut Note) {
        let line = line.trim_start_matches("- ");
        if line.starts_with("Category: ")
            && let Some(category_str) = line.strip_prefix("Category: ") {
                note.category = self.parse_category(category_str);
            }
    }

    /// Parse category from string
    fn parse_category(&self, category: &str) -> NoteCategory {
        match category.to_lowercase().as_str() {
            "implementation" => NoteCategory::Implementation,
            "decision" => NoteCategory::Decision,
            "question" => NoteCategory::Question,
            "bug" => NoteCategory::Bug,
            "enhancement" => NoteCategory::Enhancement,
            _ => NoteCategory::Other,
        }
    }

    /// Render task list to markdown
    fn render_task_list(&self, task_list: &TaskList) -> String {
        let mut content = String::new();
        content.push_str("# Task List\n\n");
        content.push_str(&format!(
            "Last updated: {}\n\n",
            task_list.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        for task in &task_list.tasks {
            content.push_str(&format!("## {} {}\n", task.id, task.title));
            content.push_str(&format!(
                "- Status: {}\n",
                self.status_to_string(&task.status)
            ));
            content.push_str(&format!(
                "- Priority: {}\n",
                self.priority_to_string(&task.priority)
            ));
            if !task.dependencies.is_empty() {
                content.push_str(&format!(
                    "- Dependencies: {}\n",
                    task.dependencies.join(", ")
                ));
            }
            content.push_str(&format!(
                "- Created: {}\n",
                task.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            content.push_str(&format!(
                "- Updated: {}\n",
                task.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            content.push('\n');
            content.push_str(&task.description);
            content.push_str("\n\n");
        }

        content
    }

    /// Render notes to markdown
    fn render_notes(&self, notes: &[Note]) -> String {
        let mut content = String::new();
        content.push_str("# Notes\n\n");

        for note in notes {
            content.push_str(&format!(
                "## {} {}\n",
                note.id,
                note.content.split('\n').next().unwrap_or("")
            ));
            content.push_str(&format!(
                "- Category: {}\n",
                self.category_to_string(&note.category)
            ));
            content.push_str(&format!(
                "- Created: {}\n",
                note.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            content.push('\n');
            content.push_str(&note.content);
            content.push_str("\n\n");
        }

        content
    }

    /// Convert status to string
    fn status_to_string(&self, status: &TaskStatus) -> String {
        match status {
            TaskStatus::Todo => "todo".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
            TaskStatus::Blocked => "blocked".to_string(),
        }
    }

    /// Convert priority to string
    fn priority_to_string(&self, priority: &TaskPriority) -> String {
        match priority {
            TaskPriority::Critical => "critical".to_string(),
            TaskPriority::High => "high".to_string(),
            TaskPriority::Medium => "medium".to_string(),
            TaskPriority::Low => "low".to_string(),
        }
    }

    /// Convert category to string
    fn category_to_string(&self, category: &NoteCategory) -> String {
        match category {
            NoteCategory::Implementation => "implementation".to_string(),
            NoteCategory::Decision => "decision".to_string(),
            NoteCategory::Question => "question".to_string(),
            NoteCategory::Bug => "bug".to_string(),
            NoteCategory::Enhancement => "enhancement".to_string(),
            NoteCategory::Other => "other".to_string(),
        }
    }
}
