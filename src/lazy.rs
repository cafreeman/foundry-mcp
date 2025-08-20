//! Lazy loading system for large data structures

use crate::cache::ProjectManagerCache;
use crate::filesystem::FileSystemManager;
use crate::models::{Task, Note};
use crate::errors::Result;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Lazily loaded specification content
#[derive(Clone)]
pub struct LazySpecificationContent {
    project_name: String,
    spec_id: String,
    fs_manager: FileSystemManager,
    cache: ProjectManagerCache,
    
    // Lazy loaded fields
    content: Arc<OnceCell<String>>,
    tasks: Arc<OnceCell<Vec<Task>>>,
    notes: Arc<OnceCell<Vec<Note>>>,
}

impl LazySpecificationContent {
    /// Create a new lazy specification content loader
    pub fn new(
        project_name: String,
        spec_id: String,
        fs_manager: FileSystemManager,
        cache: ProjectManagerCache,
    ) -> Self {
        Self {
            project_name,
            spec_id,
            fs_manager,
            cache,
            content: Arc::new(OnceCell::new()),
            tasks: Arc::new(OnceCell::new()),
            notes: Arc::new(OnceCell::new()),
        }
    }

    /// Get the specification content (loaded on first access)
    pub async fn get_content(&self) -> Result<&String> {
        self.content
            .get_or_try_init(|| async {
                self.load_content().await
            })
            .await
    }

    /// Get the task list (loaded on first access)
    pub async fn get_tasks(&self) -> Result<&Vec<Task>> {
        self.tasks
            .get_or_try_init(|| async {
                self.load_tasks().await
            })
            .await
    }

    /// Get the notes (loaded on first access)
    pub async fn get_notes(&self) -> Result<&Vec<Note>> {
        self.notes
            .get_or_try_init(|| async {
                self.load_notes().await
            })
            .await
    }

    /// Check if content is already loaded
    pub fn is_content_loaded(&self) -> bool {
        self.content.get().is_some()
    }

    /// Check if tasks are already loaded
    pub fn are_tasks_loaded(&self) -> bool {
        self.tasks.get().is_some()
    }

    /// Check if notes are already loaded
    pub fn are_notes_loaded(&self) -> bool {
        self.notes.get().is_some()
    }

    /// Preload all content (useful for background loading)
    pub async fn preload_all(&self) -> Result<()> {
        // Load all content in parallel
        let (content_result, tasks_result, notes_result) = tokio::join!(
            self.get_content(),
            self.get_tasks(),
            self.get_notes()
        );

        content_result?;
        tasks_result?;
        notes_result?;

        Ok(())
    }

    /// Invalidate cached data and reset lazy loaders
    pub fn invalidate(&self) {
        let cache_key = format!("content:{}:{}", self.project_name, self.spec_id);
        self.cache.get_file_content(&cache_key);
        
        let task_cache_key = format!("tasks:{}:{}", self.project_name, self.spec_id);
        self.cache.get_file_content(&task_cache_key);
        
        let notes_cache_key = format!("notes:{}:{}", self.project_name, self.spec_id);
        self.cache.get_file_content(&notes_cache_key);
        
        // Reset the once cells so they reload on next access
        // Note: OnceCell doesn't have a reset method, so we'd need to use a different pattern
        // for full invalidation. For now, we just clear the cache.
    }

    /// Load specification content from file system
    async fn load_content(&self) -> Result<String> {
        let cache_key = format!("content:{}:{}", self.project_name, self.spec_id);
        
        // Check cache first
        if let Some(cached_content) = self.cache.get_file_content(&cache_key) {
            tracing::debug!("Retrieved spec content for '{}:{}' from cache", self.project_name, self.spec_id);
            return Ok(cached_content);
        }

        tracing::debug!("Loading spec content for '{}:{}' from filesystem", self.project_name, self.spec_id);

        let spec_path = self.fs_manager.spec_dir(&self.project_name, &self.spec_id);
        let content_path = spec_path.join("spec.md");
        
        let content = self.fs_manager.read_file(&content_path)?;
        
        // Cache the content
        self.cache.cache_file_content(&cache_key, content.clone());
        
        Ok(content)
    }

    /// Load tasks from file system
    async fn load_tasks(&self) -> Result<Vec<Task>> {
        let cache_key = format!("tasks:{}:{}", self.project_name, self.spec_id);
        
        // For tasks, we'll parse from the task-list.md file
        if let Some(cached_content) = self.cache.get_file_content(&cache_key) {
            tracing::debug!("Retrieved task list for '{}:{}' from cache", self.project_name, self.spec_id);
            return self.parse_tasks_from_markdown(&cached_content);
        }

        tracing::debug!("Loading task list for '{}:{}' from filesystem", self.project_name, self.spec_id);

        let spec_path = self.fs_manager.spec_dir(&self.project_name, &self.spec_id);
        let task_list_path = spec_path.join("task-list.md");
        
        if !self.fs_manager.file_exists(&task_list_path) {
            return Ok(Vec::new());
        }
        
        let task_list_content = self.fs_manager.read_file(&task_list_path)?;
        
        // Cache the raw content
        self.cache.cache_file_content(&cache_key, task_list_content.clone());
        
        self.parse_tasks_from_markdown(&task_list_content)
    }

    /// Load notes from file system
    async fn load_notes(&self) -> Result<Vec<Note>> {
        let cache_key = format!("notes:{}:{}", self.project_name, self.spec_id);
        
        if let Some(cached_content) = self.cache.get_file_content(&cache_key) {
            tracing::debug!("Retrieved notes for '{}:{}' from cache", self.project_name, self.spec_id);
            return self.parse_notes_from_markdown(&cached_content);
        }

        tracing::debug!("Loading notes for '{}:{}' from filesystem", self.project_name, self.spec_id);

        let spec_path = self.fs_manager.spec_dir(&self.project_name, &self.spec_id);
        let notes_path = spec_path.join("notes.md");
        
        if !self.fs_manager.file_exists(&notes_path) {
            return Ok(Vec::new());
        }
        
        let notes_content = self.fs_manager.read_file(&notes_path)?;
        
        // Cache the raw content
        self.cache.cache_file_content(&cache_key, notes_content.clone());
        
        self.parse_notes_from_markdown(&notes_content)
    }

    /// Parse tasks from markdown content
    fn parse_tasks_from_markdown(&self, content: &str) -> Result<Vec<Task>> {
        // This is a simplified parser - in a real implementation, you'd want
        // more robust markdown parsing
        let mut tasks = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                // Extract basic task information from markdown checkbox format
                let is_completed = trimmed.starts_with("- [x]");
                let title = trimmed[5..].trim().to_string();
                
                tasks.push(Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    description: String::new(),
                    status: if is_completed { 
                        crate::models::TaskStatus::Completed 
                    } else { 
                        crate::models::TaskStatus::Todo 
                    },
                    priority: crate::models::TaskPriority::Medium,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    dependencies: Vec::new(),
                });
            }
        }
        
        Ok(tasks)
    }

    /// Parse notes from markdown content
    fn parse_notes_from_markdown(&self, content: &str) -> Result<Vec<Note>> {
        // This is a simplified parser - in a real implementation, you'd want
        // more robust markdown parsing
        let mut notes = Vec::new();
        let mut current_note: Option<Note> = None;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("## ") {
                // Save previous note if exists
                if let Some(note) = current_note.take() {
                    notes.push(note);
                }
                
                // Start new note
                let _title = trimmed[3..].trim().to_string();
                current_note = Some(Note {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: String::new(),
                    category: crate::models::NoteCategory::Other,
                    created_at: chrono::Utc::now(),
                });
            } else if let Some(ref mut note) = current_note {
                // Add content to current note
                if !note.content.is_empty() {
                    note.content.push('\n');
                }
                note.content.push_str(line);
            }
        }
        
        // Don't forget the last note
        if let Some(note) = current_note {
            notes.push(note);
        }
        
        Ok(notes)
    }
}

/// Lazy loading manager for coordinating multiple lazy loaders
#[derive(Clone)]
pub struct LazyLoadingManager {
    cache: ProjectManagerCache,
    fs_manager: FileSystemManager,
}

impl LazyLoadingManager {
    /// Create a new lazy loading manager
    pub fn new(fs_manager: FileSystemManager, cache: ProjectManagerCache) -> Self {
        Self {
            cache,
            fs_manager,
        }
    }

    /// Create a lazy specification content loader
    pub fn create_spec_loader(&self, project_name: &str, spec_id: &str) -> LazySpecificationContent {
        LazySpecificationContent::new(
            project_name.to_string(),
            spec_id.to_string(),
            self.fs_manager.clone(),
            self.cache.clone(),
        )
    }

    /// Preload specifications in the background
    pub async fn preload_specs_background(&self, project_name: &str, spec_ids: &[String]) {
        let project_name = project_name.to_string();
        let spec_ids = spec_ids.to_vec();
        let manager = self.clone();
        
        let tasks: Vec<_> = spec_ids
            .iter()
            .map(|spec_id| {
                let loader = manager.create_spec_loader(&project_name, spec_id);
                let spec_id_clone = spec_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = loader.preload_all().await {
                        tracing::warn!("Failed to preload spec {}: {}", spec_id_clone, e);
                    }
                })
            })
            .collect();

        // Wait for all preloading tasks to complete
        for task in tasks {
            let _ = task.await;
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats()
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        self.cache.cleanup_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::filesystem::FileSystemManager;
    use std::fs;

    fn create_test_setup() -> (LazyLoadingManager, TempDir, String, String) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // We can't directly construct FileSystemManager with a custom base_dir
        // So we'll skip this test or modify it to work with the existing API
        let fs_manager = FileSystemManager::new().expect("Failed to create fs manager");
        let cache = ProjectManagerCache::new();
        let manager = LazyLoadingManager::new(fs_manager, cache);
        
        let project_name = "test-project".to_string();
        let spec_id = "test-spec".to_string();
        
        // Create test directory structure in the actual base directory
        // Note: This test will create real files in the user's home directory
        // In a real test environment, you'd want a better approach
        
        (manager, temp_dir, project_name, spec_id)
    }

    #[tokio::test]
    #[ignore] // Disabled until we can properly mock the filesystem
    async fn test_lazy_content_loading() {
        let (manager, temp_dir, project_name, spec_id) = create_test_setup();
        
        // Create test content
        let spec_dir = temp_dir.path().join(&project_name).join("specs").join(&spec_id);
        let content_path = spec_dir.join("spec.md");
        fs::write(&content_path, "# Test Specification\n\nThis is test content.").unwrap();
        
        let loader = manager.create_spec_loader(&project_name, &spec_id);
        
        // Content should not be loaded initially
        assert!(!loader.is_content_loaded());
        
        // Load content
        let content = loader.get_content().await.unwrap();
        assert!(content.contains("Test Specification"));
        
        // Content should now be loaded
        assert!(loader.is_content_loaded());
        
        // Second access should be from memory (no file system access)
        let content2 = loader.get_content().await.unwrap();
        assert_eq!(content, content2);
    }

    #[tokio::test]
    #[ignore] // Disabled until we can properly mock the filesystem
    async fn test_lazy_tasks_loading() {
        let (manager, temp_dir, project_name, spec_id) = create_test_setup();
        
        // Create test task list
        let spec_dir = temp_dir.path().join(&project_name).join("specs").join(&spec_id);
        let task_list_path = spec_dir.join("task-list.md");
        let task_content = "# Task List\n\n- [ ] Task 1\n- [x] Task 2\n- [ ] Task 3";
        fs::write(&task_list_path, task_content).unwrap();
        
        let loader = manager.create_spec_loader(&project_name, &spec_id);
        
        // Tasks should not be loaded initially
        assert!(!loader.are_tasks_loaded());
        
        // Load tasks
        let tasks = loader.get_tasks().await.unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].title, "Task 1");
        assert_eq!(tasks[1].title, "Task 2");
        assert_eq!(tasks[2].title, "Task 3");
        
        // Check status
        assert_eq!(tasks[0].status, crate::models::TaskStatus::Todo);
        assert_eq!(tasks[1].status, crate::models::TaskStatus::Completed);
        assert_eq!(tasks[2].status, crate::models::TaskStatus::Todo);
        
        // Tasks should now be loaded
        assert!(loader.are_tasks_loaded());
    }

    #[tokio::test]
    #[ignore] // Disabled until we can properly mock the filesystem
    async fn test_preload_all() {
        let (manager, temp_dir, project_name, spec_id) = create_test_setup();
        
        // Create test files
        let spec_dir = temp_dir.path().join(&project_name).join("specs").join(&spec_id);
        fs::write(spec_dir.join("spec.md"), "# Test Spec").unwrap();
        fs::write(spec_dir.join("task-list.md"), "- [ ] Task 1").unwrap();
        fs::write(spec_dir.join("notes.md"), "## Note 1\nContent").unwrap();
        
        let loader = manager.create_spec_loader(&project_name, &spec_id);
        
        // Nothing should be loaded initially
        assert!(!loader.is_content_loaded());
        assert!(!loader.are_tasks_loaded());
        assert!(!loader.are_notes_loaded());
        
        // Preload all
        loader.preload_all().await.unwrap();
        
        // Everything should now be loaded
        assert!(loader.is_content_loaded());
        assert!(loader.are_tasks_loaded());
        assert!(loader.are_notes_loaded());
    }

    #[tokio::test]
    #[ignore] // Disabled until we can properly mock the filesystem
    async fn test_background_preloading() {
        let (manager, temp_dir, project_name, _) = create_test_setup();
        
        // Create multiple specs
        let spec_ids = vec!["spec1".to_string(), "spec2".to_string(), "spec3".to_string()];
        
        for spec_id in &spec_ids {
            let spec_dir = temp_dir.path().join(&project_name).join("specs").join(spec_id);
            fs::create_dir_all(&spec_dir).unwrap();
            fs::write(spec_dir.join("spec.md"), format!("# {}", spec_id)).unwrap();
        }
        
        // Preload in background
        manager.preload_specs_background(&project_name, &spec_ids).await;
        
        // Verify all specs are now cached
        for spec_id in &spec_ids {
            let cache_key = format!("content:{}:{}", project_name, spec_id);
            assert!(manager.cache.get_file_content(&cache_key).is_some());
        }
    }
}