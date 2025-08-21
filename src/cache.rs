//! Caching system for frequently accessed data

use crate::models::{Project, Specification};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache entry with timestamp for expiration
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// In-memory cache for project manager data
#[derive(Clone)]
pub struct ProjectManagerCache {
    projects: Arc<DashMap<String, CacheEntry<Project>>>,
    specifications: Arc<DashMap<String, CacheEntry<Specification>>>,
    project_lists: Arc<DashMap<String, CacheEntry<Vec<String>>>>,
    spec_lists: Arc<DashMap<String, CacheEntry<Vec<String>>>>,
    file_contents: Arc<DashMap<String, CacheEntry<String>>>,
}

impl ProjectManagerCache {
    /// Create a new cache instance
    pub fn new() -> Self {
        Self {
            projects: Arc::new(DashMap::new()),
            specifications: Arc::new(DashMap::new()),
            project_lists: Arc::new(DashMap::new()),
            spec_lists: Arc::new(DashMap::new()),
            file_contents: Arc::new(DashMap::new()),
        }
    }

    /// Cache a project with default TTL of 5 minutes
    pub fn cache_project(&self, name: &str, project: Project) {
        self.cache_project_with_ttl(name, project, Duration::from_secs(300));
    }

    /// Cache a project with custom TTL
    pub fn cache_project_with_ttl(&self, name: &str, project: Project, ttl: Duration) {
        self.projects
            .insert(name.to_string(), CacheEntry::new(project, ttl));
    }

    /// Get a cached project if not expired
    pub fn get_project(&self, name: &str) -> Option<Project> {
        self.projects.get(name).and_then(|entry| {
            if entry.is_expired() {
                self.projects.remove(name);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Cache a specification with default TTL of 10 minutes
    pub fn cache_specification(&self, key: &str, spec: Specification) {
        self.cache_specification_with_ttl(key, spec, Duration::from_secs(600));
    }

    /// Cache a specification with custom TTL
    pub fn cache_specification_with_ttl(&self, key: &str, spec: Specification, ttl: Duration) {
        self.specifications
            .insert(key.to_string(), CacheEntry::new(spec, ttl));
    }

    /// Get a cached specification if not expired
    pub fn get_specification(&self, key: &str) -> Option<Specification> {
        self.specifications.get(key).and_then(|entry| {
            if entry.is_expired() {
                self.specifications.remove(key);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Cache a project list with default TTL of 2 minutes
    pub fn cache_project_list(&self, key: &str, projects: Vec<String>) {
        self.cache_project_list_with_ttl(key, projects, Duration::from_secs(120));
    }

    /// Cache a project list with custom TTL
    pub fn cache_project_list_with_ttl(&self, key: &str, projects: Vec<String>, ttl: Duration) {
        self.project_lists
            .insert(key.to_string(), CacheEntry::new(projects, ttl));
    }

    /// Get a cached project list if not expired
    pub fn get_project_list(&self, key: &str) -> Option<Vec<String>> {
        self.project_lists.get(key).and_then(|entry| {
            if entry.is_expired() {
                self.project_lists.remove(key);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Cache a spec list with default TTL of 5 minutes
    pub fn cache_spec_list(&self, key: &str, specs: Vec<String>) {
        self.cache_spec_list_with_ttl(key, specs, Duration::from_secs(300));
    }

    /// Cache a spec list with custom TTL
    pub fn cache_spec_list_with_ttl(&self, key: &str, specs: Vec<String>, ttl: Duration) {
        self.spec_lists
            .insert(key.to_string(), CacheEntry::new(specs, ttl));
    }

    /// Get a cached spec list if not expired
    pub fn get_spec_list(&self, key: &str) -> Option<Vec<String>> {
        self.spec_lists.get(key).and_then(|entry| {
            if entry.is_expired() {
                self.spec_lists.remove(key);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Cache file contents with default TTL of 1 minute
    pub fn cache_file_content(&self, path: &str, content: String) {
        self.cache_file_content_with_ttl(path, content, Duration::from_secs(60));
    }

    /// Cache file contents with custom TTL
    pub fn cache_file_content_with_ttl(&self, path: &str, content: String, ttl: Duration) {
        self.file_contents
            .insert(path.to_string(), CacheEntry::new(content, ttl));
    }

    /// Get cached file contents if not expired
    pub fn get_file_content(&self, path: &str) -> Option<String> {
        self.file_contents.get(path).and_then(|entry| {
            if entry.is_expired() {
                self.file_contents.remove(path);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Invalidate all cached data for a specific project
    pub fn invalidate_project(&self, project_name: &str) {
        // Remove project cache
        self.projects.remove(project_name);

        // Remove related specifications
        self.specifications
            .retain(|key, _| !key.starts_with(&format!("{}:", project_name)));

        // Remove spec lists for this project
        self.spec_lists.remove(project_name);

        // Remove project lists (since they might include this project)
        self.project_lists.clear();

        // Remove file contents for this project
        let project_path = format!("/{}/", project_name);
        self.file_contents
            .retain(|key, _| !key.contains(&project_path));
    }

    /// Invalidate all cached data for a specific specification
    pub fn invalidate_specification(&self, project_name: &str, spec_id: &str) {
        let spec_key = format!("{}:{}", project_name, spec_id);
        self.specifications.remove(&spec_key);

        // Remove spec lists for this project since they might be outdated
        self.spec_lists.remove(project_name);

        // Remove related file contents
        let spec_path = format!("/{}/{}/", project_name, spec_id);
        self.file_contents
            .retain(|key, _| !key.contains(&spec_path));
    }

    /// Clear all expired entries
    pub fn cleanup_expired(&self) {
        self.projects.retain(|_, entry| !entry.is_expired());
        self.specifications.retain(|_, entry| !entry.is_expired());
        self.project_lists.retain(|_, entry| !entry.is_expired());
        self.spec_lists.retain(|_, entry| !entry.is_expired());
        self.file_contents.retain(|_, entry| !entry.is_expired());
    }

    /// Clear all cached data
    pub fn clear_all(&self) {
        self.projects.clear();
        self.specifications.clear();
        self.project_lists.clear();
        self.spec_lists.clear();
        self.file_contents.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            projects_count: self.projects.len(),
            specifications_count: self.specifications.len(),
            project_lists_count: self.project_lists.len(),
            spec_lists_count: self.spec_lists.len(),
            file_contents_count: self.file_contents.len(),
        }
    }
}

impl Default for ProjectManagerCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub projects_count: usize,
    pub specifications_count: usize,
    pub project_lists_count: usize,
    pub spec_lists_count: usize,
    pub file_contents_count: usize,
}

impl CacheStats {
    pub fn total_entries(&self) -> usize {
        self.projects_count
            + self.specifications_count
            + self.project_lists_count
            + self.spec_lists_count
            + self.file_contents_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Project, SpecStatus, Specification, TechStack, Vision};
    use std::thread;

    fn create_test_project() -> Project {
        Project {
            name: "test-project".to_string(),
            description: "Test project".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tech_stack: TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Actix".to_string()],
                databases: vec!["PostgreSQL".to_string()],
                tools: vec!["Cargo".to_string()],
                deployment: vec!["Docker".to_string()],
            },
            vision: Vision {
                overview: "Test vision".to_string(),
                goals: vec!["Goal 1".to_string()],
                target_users: vec!["User 1".to_string()],
                success_criteria: vec!["Criterion 1".to_string()],
            },
        }
    }

    fn create_test_spec() -> Specification {
        Specification {
            id: "test-spec".to_string(),
            name: "Test Spec".to_string(),
            description: "Test description".to_string(),
            status: SpecStatus::Draft,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            content: "Test content".to_string(),
        }
    }

    #[test]
    fn test_cache_and_get_project() {
        let cache = ProjectManagerCache::new();
        let project = create_test_project();

        // Cache should be empty initially
        assert!(cache.get_project("test-project").is_none());

        // Cache the project
        cache.cache_project("test-project", project.clone());

        // Should be able to retrieve it
        let cached_project = cache.get_project("test-project");
        assert!(cached_project.is_some());
        assert_eq!(cached_project.unwrap().name, project.name);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ProjectManagerCache::new();
        let project = create_test_project();

        // Cache with very short TTL
        cache.cache_project_with_ttl("test-project", project, Duration::from_millis(10));

        // Should be available immediately
        assert!(cache.get_project("test-project").is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Should be expired and removed
        assert!(cache.get_project("test-project").is_none());
    }

    #[test]
    fn test_cache_specification() {
        let cache = ProjectManagerCache::new();
        let spec = create_test_spec();
        let key = format!("test-project:{}", spec.id);

        cache.cache_specification(&key, spec.clone());

        let cached_spec = cache.get_specification(&key);
        assert!(cached_spec.is_some());
        assert_eq!(cached_spec.unwrap().id, spec.id);
    }

    #[test]
    fn test_cache_lists() {
        let cache = ProjectManagerCache::new();
        let projects = vec!["project1".to_string(), "project2".to_string()];
        let specs = vec!["spec1".to_string(), "spec2".to_string()];

        cache.cache_project_list("all_projects", projects.clone());
        cache.cache_spec_list("project1_specs", specs.clone());

        assert_eq!(cache.get_project_list("all_projects"), Some(projects));
        assert_eq!(cache.get_spec_list("project1_specs"), Some(specs));
    }

    #[test]
    fn test_cache_file_content() {
        let cache = ProjectManagerCache::new();
        let path = "/path/to/file.txt";
        let content = "file contents";

        cache.cache_file_content(path, content.to_string());

        let cached_content = cache.get_file_content(path);
        assert_eq!(cached_content, Some(content.to_string()));
    }

    #[test]
    fn test_invalidate_project() {
        let cache = ProjectManagerCache::new();
        let project = create_test_project();
        let spec = create_test_spec();

        // Cache project and related data
        cache.cache_project("test-project", project);
        cache.cache_specification("test-project:test-spec", spec);
        cache.cache_spec_list("test-project", vec!["spec1".to_string()]);
        cache.cache_file_content("/test-project/file.txt", "content".to_string());

        // Verify they're cached
        assert!(cache.get_project("test-project").is_some());
        assert!(cache.get_specification("test-project:test-spec").is_some());
        assert!(cache.get_spec_list("test-project").is_some());
        assert!(cache.get_file_content("/test-project/file.txt").is_some());

        // Invalidate project
        cache.invalidate_project("test-project");

        // All related data should be gone
        assert!(cache.get_project("test-project").is_none());
        assert!(cache.get_specification("test-project:test-spec").is_none());
        assert!(cache.get_spec_list("test-project").is_none());
        assert!(cache.get_file_content("/test-project/file.txt").is_none());
    }

    #[test]
    fn test_invalidate_specification() {
        let cache = ProjectManagerCache::new();
        let spec1 = create_test_spec();
        let mut spec2 = create_test_spec();
        spec2.id = "test-spec-2".to_string();

        cache.cache_specification("test-project:test-spec", spec1);
        cache.cache_specification("test-project:test-spec-2", spec2);
        cache.cache_spec_list(
            "test-project",
            vec!["spec1".to_string(), "spec2".to_string()],
        );

        // Invalidate one specification
        cache.invalidate_specification("test-project", "test-spec");

        // First spec should be gone, second should remain
        assert!(cache.get_specification("test-project:test-spec").is_none());
        assert!(
            cache
                .get_specification("test-project:test-spec-2")
                .is_some()
        );
        // Spec list should be cleared since it might be outdated
        assert!(cache.get_spec_list("test-project").is_none());
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = ProjectManagerCache::new();
        let project1 = create_test_project();
        let mut project2 = create_test_project();
        project2.name = "project2".to_string();

        // Cache one with short TTL, one with long TTL
        cache.cache_project_with_ttl("project1", project1, Duration::from_millis(10));
        cache.cache_project_with_ttl("project2", project2, Duration::from_secs(3600));

        // Wait for first to expire
        thread::sleep(Duration::from_millis(20));

        // Before cleanup, expired entry still exists in map
        assert_eq!(cache.stats().projects_count, 2);

        // After cleanup, only non-expired entry should remain
        cache.cleanup_expired();
        assert_eq!(cache.stats().projects_count, 1);
        assert!(cache.get_project("project2").is_some());
    }

    #[test]
    fn test_clear_all() {
        let cache = ProjectManagerCache::new();
        let project = create_test_project();
        let spec = create_test_spec();

        cache.cache_project("test-project", project);
        cache.cache_specification("test-project:test-spec", spec);
        cache.cache_project_list("all", vec!["project1".to_string()]);
        cache.cache_spec_list("project1", vec!["spec1".to_string()]);
        cache.cache_file_content("/file.txt", "content".to_string());

        assert!(cache.stats().total_entries() > 0);

        cache.clear_all();

        assert_eq!(cache.stats().total_entries(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = ProjectManagerCache::new();

        let initial_stats = cache.stats();
        assert_eq!(initial_stats.total_entries(), 0);

        cache.cache_project("project1", create_test_project());
        cache.cache_specification("project1:spec1", create_test_spec());
        cache.cache_project_list("all", vec!["project1".to_string()]);
        cache.cache_spec_list("project1", vec!["spec1".to_string()]);
        cache.cache_file_content("/file.txt", "content".to_string());

        let stats = cache.stats();
        assert_eq!(stats.projects_count, 1);
        assert_eq!(stats.specifications_count, 1);
        assert_eq!(stats.project_lists_count, 1);
        assert_eq!(stats.spec_lists_count, 1);
        assert_eq!(stats.file_contents_count, 1);
        assert_eq!(stats.total_entries(), 5);
    }
}
