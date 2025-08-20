//! Project repository for data access operations

use crate::cache::ProjectManagerCache;
use crate::errors::{self, Result};
use crate::filesystem::FileSystemManager;
use crate::models::{Project, TechStack, Vision};

/// Repository for project data access operations
#[derive(Clone)]
pub struct ProjectRepository {
    fs_manager: FileSystemManager,
    cache: ProjectManagerCache,
}

impl ProjectRepository {
    /// Create a new ProjectRepository instance
    pub fn new(fs_manager: FileSystemManager) -> Self {
        Self { 
            fs_manager,
            cache: ProjectManagerCache::new(),
        }
    }

    /// Create a new ProjectRepository instance with shared cache
    pub fn with_cache(fs_manager: FileSystemManager, cache: ProjectManagerCache) -> Self {
        Self { fs_manager, cache }
    }

    /// Create a new project
    pub async fn create_project(&self, project: Project) -> Result<()> {
        // Check if project already exists
        if self.fs_manager.project_exists(&project.name) {
            return Err(errors::helpers::project_already_exists(&project.name));
        }

        // Create project directory structure
        self.fs_manager.create_project_structure(&project.name)?;

        // Save project metadata
        self.save_project_metadata(&project.name, &project)?;

        // Generate and save tech-stack.md
        let tech_stack_content = self.render_tech_stack(&project.tech_stack);
        let tech_stack_path = self
            .fs_manager
            .project_info_dir(&project.name)
            .join("tech-stack.md");
        self.fs_manager
            .write_file_safe(&tech_stack_path, &tech_stack_content)?;

        // Generate and save vision.md
        let vision_content = self.render_vision(&project.vision);
        let vision_path = self
            .fs_manager
            .project_info_dir(&project.name)
            .join("vision.md");
        self.fs_manager
            .write_file_safe(&vision_path, &vision_content)?;

        // Cache the newly created project
        self.cache.cache_project(&project.name, project.clone());
        
        // Invalidate project lists since we added a new project
        self.cache.get_project_list("all_projects");

        Ok(())
    }

    /// Load a project from the file system
    pub async fn load_project(&self, project_name: &str) -> Result<Project> {
        // Check cache first
        if let Some(cached_project) = self.cache.get_project(project_name) {
            tracing::debug!("Retrieved project '{}' from cache", project_name);
            return Ok(cached_project);
        }

        if !self.fs_manager.project_exists(project_name) {
            return Err(errors::helpers::project_not_found(project_name));
        }

        tracing::debug!("Loading project '{}' from filesystem", project_name);

        // Load project metadata
        let metadata_path = self
            .fs_manager
            .project_dir(project_name)
            .join("project.json");
        let metadata_content = self.fs_manager.read_file(&metadata_path)?;
        let project: Project = serde_json::from_str(&metadata_content).map_err(|e| {
            errors::helpers::serialization_error("parse project metadata", &metadata_content, e)
        })?;

        // Cache the loaded project
        self.cache.cache_project(project_name, project.clone());

        Ok(project)
    }

    /// Check if a project exists
    pub async fn project_exists(&self, project_name: &str) -> bool {
        self.fs_manager.project_exists(project_name)
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        // Check if we have a cached list of project names
        let project_names = if let Some(cached_names) = self.cache.get_project_list("all_projects") {
            tracing::debug!("Retrieved project list from cache");
            cached_names
        } else {
            tracing::debug!("Loading project list from filesystem");
            let names = self.fs_manager.list_projects()?;
            self.cache.cache_project_list("all_projects", names.clone());
            names
        };

        let mut projects = Vec::new();

        for name in project_names {
            match self.load_project(&name).await {
                Ok(project) => projects.push(project),
                Err(e) => {
                    // Log the error but continue with other projects
                    tracing::warn!("Failed to load project '{}': {}", name, e);
                }
            }
        }

        Ok(projects)
    }

    /// Save project metadata to JSON file
    fn save_project_metadata(&self, project_name: &str, project: &Project) -> Result<()> {
        let metadata_path = self
            .fs_manager
            .project_dir(project_name)
            .join("project.json");
        let metadata_content = serde_json::to_string_pretty(project).map_err(|e| {
            errors::helpers::serialization_error("serialize project metadata", "project data", e)
        })?;
        self.fs_manager
            .write_file_safe(&metadata_path, &metadata_content)?;
        Ok(())
    }

    /// Render tech stack to markdown
    fn render_tech_stack(&self, tech_stack: &TechStack) -> String {
        format!(
            "# Tech Stack\n\n## Languages\n{}\n\n## Frameworks\n{}\n\n## Databases\n{}\n\n## Tools\n{}\n\n## Deployment\n{}\n",
            tech_stack.languages.join("\n- "),
            tech_stack.frameworks.join("\n- "),
            tech_stack.databases.join("\n- "),
            tech_stack.tools.join("\n- "),
            tech_stack.deployment.join("\n- ")
        )
    }

    /// Render vision to markdown
    fn render_vision(&self, vision: &Vision) -> String {
        format!(
            "# Project Vision\n\n## Overview\n{}\n\n## Goals\n{}\n\n## Target Users\n{}\n\n## Success Criteria\n{}\n",
            vision.overview,
            vision.goals.join("\n- "),
            vision.target_users.join("\n- "),
            vision.success_criteria.join("\n- ")
        )
    }
}
