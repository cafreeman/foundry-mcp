//! Project management core logic
//!
//! This module now delegates to the backend abstraction instead of direct I/O.
//! The functions here maintain backward compatibility while using the Foundry façade.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::core::foundry::get_default_foundry;
use crate::types::project::{Project, ProjectConfig, ProjectMetadata};

/// Helper function to run async operations in sync context
fn run_async<F, R>(f: F) -> Result<R>
where
    F: std::future::Future<Output = Result<R>>,
{
    // Use tokio runtime to block on futures
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;
    rt.block_on(f)
}

/// Create a new project structure
pub fn create_project(config: ProjectConfig) -> Result<Project> {
    let foundry = get_default_foundry()?;
    run_async(foundry.create_project(config))
}

/// Get project directory path
pub fn get_project_path(project_name: &str) -> Result<PathBuf> {
    let foundry_dir = crate::core::filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name))
}

/// Check if a project exists
pub fn project_exists(project_name: &str) -> Result<bool> {
    let foundry = get_default_foundry()?;
    run_async(foundry.project_exists(project_name))
}

/// List all projects
pub fn list_projects() -> Result<Vec<ProjectMetadata>> {
    let foundry = get_default_foundry()?;
    run_async(foundry.list_projects())
}

/// Load project by name
pub fn load_project(project_name: &str) -> Result<Project> {
    let foundry = get_default_foundry()?;
    run_async(foundry.load_project(project_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_environment::TestEnvironment;

    #[test]
    fn test_list_projects_rfc3339_timestamps() {
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            // Create a test project
            env.create_test_project("test-rfc3339-timestamps")
                .await
                .unwrap();

            // Use the facade directly in async context
            let foundry = get_default_foundry().unwrap();
            let projects = foundry.list_projects().await.unwrap();
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

            // Use the facade directly in async context
            let foundry = get_default_foundry().unwrap();
            let project = foundry.load_project(project_name).await.unwrap();

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
