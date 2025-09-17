//! Project management core logic

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;

use crate::core::filesystem;
use crate::types::project::{Project, ProjectConfig, ProjectMetadata};

/// Create a new project structure
pub fn create_project(config: ProjectConfig) -> Result<Project> {
    let foundry_dir = filesystem::foundry_dir()?;
    let project_path = foundry_dir.join(&config.name);
    let created_at = Utc::now().to_rfc3339();

    // Create project directory structure
    filesystem::create_dir_all(&project_path)?;
    filesystem::create_dir_all(project_path.join("specs"))?;

    // Write project files
    filesystem::write_file_atomic(project_path.join("vision.md"), &config.vision)?;
    filesystem::write_file_atomic(project_path.join("tech-stack.md"), &config.tech_stack)?;
    filesystem::write_file_atomic(project_path.join("summary.md"), &config.summary)?;

    Ok(Project {
        name: config.name,
        created_at,
        path: project_path,
        vision: Some(config.vision),
        tech_stack: Some(config.tech_stack),
        summary: Some(config.summary),
    })
}

/// Get project directory path
pub fn get_project_path(project_name: &str) -> Result<PathBuf> {
    let foundry_dir = filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name))
}

/// Check if a project exists
pub fn project_exists(project_name: &str) -> Result<bool> {
    let project_path = get_project_path(project_name)?;
    Ok(project_path.exists())
}

/// List all projects
pub fn list_projects() -> Result<Vec<ProjectMetadata>> {
    let foundry_dir = filesystem::foundry_dir()?;

    if !foundry_dir.exists() {
        return Ok(Vec::new());
    }

    let projects: Vec<ProjectMetadata> = fs::read_dir(foundry_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                Some(entry)
            } else {
                None
            }
        })
        .map(|entry| {
            let project_name = entry.file_name().to_string_lossy().to_string();
            let project_path = entry.path();

            // Count specs using fold
            let specs_dir = project_path.join("specs");
            let spec_count = if specs_dir.exists() {
                fs::read_dir(specs_dir)
                    .ok()
                    .map(|dir| {
                        dir.filter_map(|e| e.ok())
                            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                            .fold(0, |acc, _| acc + 1)
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            // Get creation time (use directory creation time as fallback)
            let created_at = entry
                .metadata()
                .ok()
                .and_then(|m| m.created().ok())
                .map(DateTime::<Utc>::from)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());

            ProjectMetadata {
                name: project_name,
                created_at: created_at.clone(),
                spec_count,
                last_modified: created_at, // TODO: Use actual last modified time
            }
        })
        .collect();

    Ok(projects)
}

/// Load project by name
pub fn load_project(project_name: &str) -> Result<Project> {
    let project_path = get_project_path(project_name)?;

    if !project_path.exists() {
        return Err(anyhow::anyhow!("Project '{}' not found", project_name));
    }

    // Read project files
    let vision = filesystem::read_file(project_path.join("vision.md")).ok();
    let tech_stack = filesystem::read_file(project_path.join("tech-stack.md")).ok();
    let summary = filesystem::read_file(project_path.join("summary.md")).ok();

    // Get creation time from directory metadata
    let created_at = DateTime::<Utc>::from(fs::metadata(&project_path)?.created()?).to_rfc3339();

    Ok(Project {
        name: project_name.to_string(),
        created_at,
        path: project_path,
        vision,
        tech_stack,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestEnvironment;

    #[test]
    fn test_list_projects_rfc3339_timestamps() {
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            // Create a test project
            env.create_test_project("test-rfc3339-timestamps")
                .await
                .unwrap();

            // List projects and verify timestamp format
            let projects = list_projects().unwrap();
            assert_eq!(projects.len(), 1);

            let project = &projects[0];
            assert_eq!(project.name, "test-rfc3339-timestamps");

            // Verify RFC3339 format (should contain timezone offset or 'Z')
            assert!(project.created_at.contains('+') || project.created_at.contains('Z'));
            assert!(project.created_at.contains('T')); // ISO format separator
            assert!(project.created_at.len() >= 20); // Minimum RFC3339 length

            // Verify it can be parsed as a valid DateTime
            let parsed = chrono::DateTime::parse_from_rfc3339(&project.created_at)
                .or_else(|_| {
                    chrono::DateTime::parse_from_str(&project.created_at, "%Y-%m-%dT%H:%M:%SZ")
                })
                .unwrap();
            assert!(parsed.timestamp() > 0); // Should be a valid timestamp
        });
    }

    #[test]
    fn test_load_project_rfc3339_timestamps() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-load-rfc3339";

        env.with_env_async(|| async {
            // Create a test project
            env.create_test_project(project_name).await.unwrap();

            // Load project and verify timestamp format
            let project = load_project(project_name).unwrap();

            // Verify RFC3339 format
            assert!(project.created_at.contains('+') || project.created_at.contains('Z'));
            assert!(project.created_at.contains('T'));

            // Verify it can be parsed as a valid DateTime
            let parsed = chrono::DateTime::parse_from_rfc3339(&project.created_at)
                .or_else(|_| {
                    chrono::DateTime::parse_from_str(&project.created_at, "%Y-%m-%dT%H:%M:%SZ")
                })
                .unwrap();
            assert!(parsed.timestamp() > 0);
        });
    }
}
