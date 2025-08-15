//! Project repository for data access operations

use crate::filesystem::FileSystemManager;
use crate::models::{Project, TechStack, Vision};
use anyhow::{Context, Result};
use serde_json;

/// Repository for project data access operations
#[derive(Clone)]
pub struct ProjectRepository {
    fs_manager: FileSystemManager,
}

impl ProjectRepository {
    /// Create a new ProjectRepository instance
    pub fn new(fs_manager: FileSystemManager) -> Self {
        Self { fs_manager }
    }

    /// Create a new project
    pub async fn create_project(&self, project: Project) -> Result<()> {
        // Check if project already exists
        if self.fs_manager.project_exists(&project.name) {
            return Err(anyhow::anyhow!("Project '{}' already exists", project.name));
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

        Ok(())
    }

    /// Load a project from the file system
    pub async fn load_project(&self, project_name: &str) -> Result<Project> {
        if !self.fs_manager.project_exists(project_name) {
            return Err(anyhow::anyhow!("Project '{}' does not exist", project_name));
        }

        // Load project metadata
        let metadata_path = self
            .fs_manager
            .project_dir(project_name)
            .join("project.json");
        let metadata_content = self.fs_manager.read_file(&metadata_path)?;
        let project: Project = serde_json::from_str(&metadata_content)
            .with_context(|| format!("Failed to parse project metadata for '{}'", project_name))?;

        Ok(project)
    }

    /// Update an existing project
    pub async fn update_project(&self, project: &Project) -> Result<()> {
        if !self.fs_manager.project_exists(&project.name) {
            return Err(anyhow::anyhow!("Project '{}' does not exist", project.name));
        }

        // Save updated project metadata
        self.save_project_metadata(&project.name, project)?;

        // Update tech-stack.md
        let tech_stack_content = self.render_tech_stack(&project.tech_stack);
        let tech_stack_path = self
            .fs_manager
            .project_info_dir(&project.name)
            .join("tech-stack.md");
        self.fs_manager
            .write_file_safe(&tech_stack_path, &tech_stack_content)?;

        // Update vision.md
        let vision_content = self.render_vision(&project.vision);
        let vision_path = self
            .fs_manager
            .project_info_dir(&project.name)
            .join("vision.md");
        self.fs_manager
            .write_file_safe(&vision_path, &vision_content)?;

        Ok(())
    }

    /// Delete a project and all its contents
    pub async fn delete_project(&self, project_name: &str, confirm: bool) -> Result<()> {
        if !confirm {
            return Err(anyhow::anyhow!("Project deletion not confirmed"));
        }

        if !self.fs_manager.project_exists(project_name) {
            return Err(anyhow::anyhow!("Project '{}' does not exist", project_name));
        }

        let project_path = self.fs_manager.project_dir(project_name);

        // Remove the entire project directory
        fs::remove_dir_all(&project_path)
            .with_context(|| format!("Failed to delete project directory: {:?}", project_path))?;

        Ok(())
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<String>> {
        self.fs_manager.list_projects()
    }

    /// Check if a project exists
    pub async fn project_exists(&self, project_name: &str) -> bool {
        self.fs_manager.project_exists(project_name)
    }

    /// Render tech stack as markdown
    pub fn render_tech_stack(&self, tech_stack: &TechStack) -> String {
        let mut content = String::new();
        content.push_str("# Technology Stack\n\n");

        if !tech_stack.languages.is_empty() {
            content.push_str("## Languages\n");
            for language in &tech_stack.languages {
                content.push_str(&format!("- {}\n", language));
            }
            content.push('\n');
        }

        if !tech_stack.frameworks.is_empty() {
            content.push_str("## Frameworks\n");
            for framework in &tech_stack.frameworks {
                content.push_str(&format!("- {}\n", framework));
            }
            content.push('\n');
        }

        if !tech_stack.databases.is_empty() {
            content.push_str("## Databases\n");
            for database in &tech_stack.databases {
                content.push_str(&format!("- {}\n", database));
            }
            content.push('\n');
        }

        if !tech_stack.tools.is_empty() {
            content.push_str("## Tools\n");
            for tool in &tech_stack.tools {
                content.push_str(&format!("- {}\n", tool));
            }
            content.push('\n');
        }

        if !tech_stack.deployment.is_empty() {
            content.push_str("## Deployment\n");
            for deployment in &tech_stack.deployment {
                content.push_str(&format!("- {}\n", deployment));
            }
            content.push('\n');
        }

        content
    }

    /// Render vision as markdown
    pub fn render_vision(&self, vision: &Vision) -> String {
        let mut content = String::new();
        content.push_str("# Project Vision\n\n");

        content.push_str("## Overview\n");
        content.push_str(&vision.overview);
        content.push_str("\n\n");

        if !vision.goals.is_empty() {
            content.push_str("## Goals\n");
            for goal in &vision.goals {
                content.push_str(&format!("- {}\n", goal));
            }
            content.push('\n');
        }

        if !vision.target_users.is_empty() {
            content.push_str("## Target Users\n");
            for goal in &vision.target_users {
                content.push_str(&format!("- {}\n", goal));
            }
            content.push('\n');
        }

        if !vision.success_criteria.is_empty() {
            content.push_str("## Success Criteria\n");
            for criterion in &vision.success_criteria {
                content.push_str(&format!("- {}\n", criterion));
            }
            content.push('\n');
        }

        content
    }

    /// Save project metadata to JSON file
    fn save_project_metadata(&self, project_name: &str, project: &Project) -> Result<()> {
        let metadata_path = self
            .fs_manager
            .project_dir(project_name)
            .join("project.json");
        let metadata_content = serde_json::to_string_pretty(project)
            .with_context(|| "Failed to serialize project metadata")?;

        self.fs_manager
            .write_file_safe(&metadata_path, &metadata_content)
    }
}

use std::fs;
