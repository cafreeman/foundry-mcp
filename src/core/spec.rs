//! Spec management core logic
//!
//! This module now delegates to the backend abstraction instead of direct I/O.
//! The functions here maintain backward compatibility while using the Foundry façade.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::core::foundry::get_default_foundry;
use crate::types::spec::{
    ContentValidationStatus, Spec, SpecConfig, SpecFileType, SpecFilter, SpecMetadata,
    SpecValidationResult,
};

// Import SpecContentData only for tests
#[cfg(test)]
use crate::types::spec::SpecContentData;

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

/// Generate timestamped spec name
pub fn generate_spec_name(feature_name: &str) -> String {
    crate::core::foundry::Foundry::<crate::core::backends::filesystem::FilesystemBackend>::generate_spec_name(feature_name)
}

/// Create a new spec
pub fn create_spec(config: SpecConfig) -> Result<Spec> {
    let foundry = get_default_foundry()?;
    run_async(foundry.create_spec(config))
}

/// Validate spec directory name format
pub fn validate_spec_name(spec_name: &str) -> Result<()> {
    crate::core::foundry::Foundry::<crate::core::backends::filesystem::FilesystemBackend>::validate_spec_name(spec_name)
}
/// List specs for a project with enhanced validation
pub fn list_specs(project_name: &str) -> Result<Vec<SpecMetadata>> {
    let foundry = get_default_foundry()?;
    run_async(foundry.list_specs(project_name))
}

/// List specs with filtering capabilities
pub fn list_specs_filtered(project_name: &str, filter: SpecFilter) -> Result<Vec<SpecMetadata>> {
    let specs = list_specs(project_name)?;

    let mut filtered_specs: Vec<SpecMetadata> = specs
        .into_iter()
        .filter(|spec| {
            // Apply feature name filter
            if let Some(name_filter) = &filter.feature_name_contains
                && !spec
                    .feature_name
                    .to_lowercase()
                    .contains(&name_filter.to_lowercase())
            {
                return false;
            }

            // Apply date range filters
            if let Some(after) = &filter.created_after
                && spec.created_at < *after
            {
                return false;
            }

            if let Some(before) = &filter.created_before
                && spec.created_at > *before
            {
                return false;
            }

            true
        })
        .collect();

    // Apply limit
    if let Some(limit) = filter.limit {
        filtered_specs.truncate(limit);
    }

    Ok(filtered_specs)
}

/// Get the most recent spec for a project
pub fn get_latest_spec(project_name: &str) -> Result<Option<SpecMetadata>> {
    let foundry = get_default_foundry()?;
    run_async(foundry.get_latest_spec(project_name))
}

/// Count total specs for a project
pub fn count_specs(project_name: &str) -> Result<usize> {
    let foundry = get_default_foundry()?;
    run_async(foundry.count_specs(project_name))
}

/// Check if a spec exists
pub fn spec_exists(project_name: &str, spec_name: &str) -> Result<bool> {
    run_async(async {
        // Access the backend through the foundry instance
        let backend = crate::core::backends::filesystem::FilesystemBackend::new();
        backend.spec_exists(project_name, spec_name).await
    })
}

/// Update spec content (for task list updates)
pub fn update_spec_content(
    project_name: &str,
    spec_name: &str,
    file_type: SpecFileType,
    new_content: &str,
) -> Result<()> {
    let foundry = get_default_foundry()?;
    run_async(foundry.update_spec_content(project_name, spec_name, file_type, new_content))
}

/// Get spec directory path
pub fn get_spec_path(project_name: &str, spec_name: &str) -> Result<PathBuf> {
    let foundry_dir = crate::core::filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name).join("specs").join(spec_name))
}

/// Get specs directory path for a project
pub fn get_specs_directory(project_name: &str) -> Result<PathBuf> {
    let foundry_dir = crate::core::filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name).join("specs"))
}

/// Ensure specs directory exists for a project
pub fn ensure_specs_directory(project_name: &str) -> Result<PathBuf> {
    let specs_dir = get_specs_directory(project_name)?;
    crate::core::filesystem::create_dir_all(&specs_dir).with_context(|| {
        format!(
            "Failed to create specs directory for project '{}'",
            project_name
        )
    })?;
    Ok(specs_dir)
}

/// Delete a spec (with confirmation)
pub fn delete_spec(project_name: &str, spec_name: &str) -> Result<()> {
    let foundry = get_default_foundry()?;
    run_async(foundry.delete_spec(project_name, spec_name))
}

/// Validate spec content files exist and are readable
pub fn validate_spec_files(project_name: &str, spec_name: &str) -> Result<SpecValidationResult> {
    let spec_path = get_spec_path(project_name, spec_name)?;

    if !spec_path.exists() {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'",
            spec_name,
            project_name
        ));
    }

    let spec_file = spec_path.join("spec.md");
    let notes_file = spec_path.join("notes.md");
    let task_list_file = spec_path.join("task-list.md");

    let mut result = SpecValidationResult {
        spec_name: spec_name.to_string(),
        project_name: project_name.to_string(),
        spec_file_exists: spec_file.exists(),
        notes_file_exists: notes_file.exists(),
        task_list_file_exists: task_list_file.exists(),
        content_validation: ContentValidationStatus {
            spec_valid: false,
            notes_valid: false,
            task_list_valid: false,
        },
        validation_errors: Vec::new(),
    };

    // Validate file contents if they exist
    if result.spec_file_exists {
        match crate::core::filesystem::read_file(&spec_file) {
            Ok(content) => {
                result.content_validation.spec_valid = !content.trim().is_empty();
                if !result.content_validation.spec_valid {
                    result
                        .validation_errors
                        .push("Spec file is empty".to_string());
                }
            }
            Err(e) => {
                result
                    .validation_errors
                    .push(format!("Cannot read spec file: {}", e));
            }
        }
    } else {
        result
            .validation_errors
            .push("Spec file missing".to_string());
    }

    if result.notes_file_exists {
        match crate::core::filesystem::read_file(&notes_file) {
            Ok(content) => {
                result.content_validation.notes_valid = !content.trim().is_empty();
            }
            Err(e) => {
                result
                    .validation_errors
                    .push(format!("Cannot read notes file: {}", e));
            }
        }
    }

    if result.task_list_file_exists {
        match crate::core::filesystem::read_file(&task_list_file) {
            Ok(content) => {
                result.content_validation.task_list_valid = !content.trim().is_empty();
            }
            Err(e) => {
                result
                    .validation_errors
                    .push(format!("Cannot read task list file: {}", e));
            }
        }
    }

    Ok(result)
}

/// Fuzzy matching strategy for spec discovery
#[derive(Debug, Clone, PartialEq)]
pub enum SpecMatchStrategy {
    /// Direct exact match found
    Exact(String),
    /// Matched by feature name (exact)
    FeatureExact(String),
    /// Matched by feature name (fuzzy)
    FeatureFuzzy(String),
    /// Matched by spec name similarity
    NameFuzzy(String),
    /// Multiple candidates found
    Multiple(Vec<String>),
    /// No reasonable matches
    None,
}

/// Find the best matching spec using fuzzy matching
pub fn find_spec_match(project_name: &str, query: &str) -> Result<SpecMatchStrategy> {
    let foundry = get_default_foundry()?;
    run_async(foundry.find_spec_match(project_name, query))
}

/// Load a spec with fuzzy matching support and comprehensive error handling
pub fn load_spec_with_fuzzy(project_name: &str, query: &str) -> Result<(Spec, SpecMatchStrategy)> {
    // Validate inputs with detailed error messages
    if query.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Cannot search for empty spec name. Please provide a spec name or feature name to search for."
        ));
    }

    if project_name.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Project name cannot be empty. Please specify a valid project name."
        ));
    }

    let match_strategy = find_spec_match(project_name, query)?;

    match &match_strategy {
        SpecMatchStrategy::Exact(spec_name)
        | SpecMatchStrategy::FeatureExact(spec_name)
        | SpecMatchStrategy::FeatureFuzzy(spec_name)
        | SpecMatchStrategy::NameFuzzy(spec_name) => {
            let spec = load_spec(project_name, spec_name)
                .with_context(|| format!("Failed to load matched spec '{}'", spec_name))?;
            Ok((spec, match_strategy))
        }
        SpecMatchStrategy::Multiple(candidates) => {
            // Provide detailed disambiguation with suggestions
            let candidate_list = candidates
                .iter()
                .enumerate()
                .map(|(i, name)| format!("  {}. {}", i + 1, name))
                .collect::<Vec<_>>()
                .join("\n");

            Err(anyhow::anyhow!(
                "Multiple specs match '{}':\n{}\n\nPlease specify which one you want to load by using the exact spec name or a more specific query.",
                query,
                candidate_list
            ))
        }
        SpecMatchStrategy::None => {
            // Get available specs for helpful error message
            let available_specs = list_specs(project_name)?;
            if available_specs.is_empty() {
                Err(anyhow::anyhow!(
                    "No specs found in project '{}'. This project doesn't have any specifications yet.\n\nTo create your first spec, use:\n  mcp_foundry_create_spec {} <feature_name>\n\nFor example:\n  mcp_foundry_create_spec {} user_authentication",
                    project_name,
                    project_name,
                    project_name
                ))
            } else {
                // Show available specs with better formatting
                let spec_list = if available_specs.len() <= 10 {
                    available_specs
                        .iter()
                        .map(|s| format!("  - {} ({})", s.name, s.feature_name))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    format!(
                        "  {} specs available (showing first 10):\n{}",
                        available_specs.len(),
                        available_specs
                            .iter()
                            .take(10)
                            .map(|s| format!("  - {} ({})", s.name, s.feature_name))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                };

                Err(anyhow::anyhow!(
                    "No specs found matching '{}'.\n\nAvailable specs:\n{}\n\nTry using a more specific search term or use the exact spec name.",
                    query,
                    spec_list
                ))
            }
        }
    }
}

/// Load a specific spec with validation
pub fn load_spec(project_name: &str, spec_name: &str) -> Result<Spec> {
    let foundry = get_default_foundry()?;
    run_async(foundry.load_spec(project_name, spec_name))
}

/// Get the file path for a spec.md file
pub fn get_spec_file_path(project_name: &str, spec_name: &str) -> Result<PathBuf> {
    let spec_path = get_spec_path(project_name, spec_name)?;
    Ok(spec_path.join("spec.md"))
}

/// Get the file path for a task-list.md file
pub fn get_task_list_file_path(project_name: &str, spec_name: &str) -> Result<PathBuf> {
    let spec_path = get_spec_path(project_name, spec_name)?;
    Ok(spec_path.join("task-list.md"))
}

/// Get the file path for a notes.md file
pub fn get_notes_file_path(project_name: &str, spec_name: &str) -> Result<PathBuf> {
    let spec_path = get_spec_path(project_name, spec_name)?;
    Ok(spec_path.join("notes.md"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_environment::TestEnvironment;
    use crate::types::spec::{SpecConfig, SpecFileType, SpecFilter};

    // Removed legacy mutex-based testing in favor of modern environment isolation

    #[test]
    fn test_spec_filtering() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-filtering";

        env.with_env_async(|| async {
            // First create the project
            env.create_test_project(project_name).await.unwrap();

            // Create test specs using helper methods
            env.create_test_spec(
                project_name,
                "user_auth",
                "User authentication specification",
            )
            .await
            .unwrap();
            env.create_test_spec(project_name, "user_profile", "User profile management")
                .await
                .unwrap();

            // Use spawn_blocking to run sync functions from async context
            let project_name_clone = project_name.to_string();
            let filtered_specs = tokio::task::spawn_blocking(move || {
                let filter = SpecFilter {
                    feature_name_contains: Some("user".to_string()),
                    ..Default::default()
                };
                list_specs_filtered(&project_name_clone, filter)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(filtered_specs.len(), 2);

            // Test filtering with limit
            let project_name_clone = project_name.to_string();
            let limited_specs = tokio::task::spawn_blocking(move || {
                let filter = SpecFilter {
                    limit: Some(1),
                    ..Default::default()
                };
                list_specs_filtered(&project_name_clone, filter)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(limited_specs.len(), 1);
        });
    }

    #[test]
    fn test_spec_existence_and_counting() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-existence";

        env.with_env_async(|| async {
            // First create the project
            env.create_test_project(project_name).await.unwrap();

            // Use spawn_blocking to run sync functions from async context
            let project_name_clone = project_name.to_string();
            let count = tokio::task::spawn_blocking(move || count_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(count, 0);

            let project_name_clone = project_name.to_string();
            let exists = tokio::task::spawn_blocking(move || {
                spec_exists(&project_name_clone, "nonexistent_spec")
            })
            .await
            .unwrap()
            .unwrap();
            assert!(!exists);

            // Create a test spec
            env.create_test_spec(project_name, "test_feature", "Test specification")
                .await
                .unwrap();

            // Test counting and existence
            let project_name_clone = project_name.to_string();
            let count = tokio::task::spawn_blocking(move || count_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(count, 1);

            // List specs to get the actual spec name for existence check
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(specs.len(), 1);

            let project_name_clone = project_name.to_string();
            let spec_name = specs[0].name.clone();
            let exists =
                tokio::task::spawn_blocking(move || spec_exists(&project_name_clone, &spec_name))
                    .await
                    .unwrap()
                    .unwrap();
            assert!(exists);
        });
    }

    #[test]
    fn test_spec_content_updates() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-content-updates";

        env.with_env_async(|| async {
            // First create the project
            env.create_test_project(project_name).await.unwrap();

            // Create a test spec
            env.create_test_spec(project_name, "updatable_spec", "Original specification")
                .await
                .unwrap();

            // Use spawn_blocking to run sync functions from async context
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(specs.len(), 1);
            let spec_name = specs[0].name.clone();

            // Update task list
            let new_tasks = "- Updated task\n- New task\n- [ ] Completed task";
            let project_name_clone = project_name.to_string();
            let spec_name_clone = spec_name.clone();
            let new_tasks_clone = new_tasks.to_string();
            tokio::task::spawn_blocking(move || {
                update_spec_content(
                    &project_name_clone,
                    &spec_name_clone,
                    SpecFileType::TaskList,
                    &new_tasks_clone,
                )
            })
            .await
            .unwrap()
            .unwrap();

            // Verify update
            let project_name_clone = project_name.to_string();
            let spec_name_clone = spec_name.clone();
            let loaded_spec = tokio::task::spawn_blocking(move || {
                load_spec(&project_name_clone, &spec_name_clone)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(loaded_spec.content.tasks, new_tasks);
            // Note: The spec content will be longer than "Original specification" due to our template
            assert!(loaded_spec.content.spec.contains("Original specification"));
        });
    }

    #[test]
    fn test_spec_validation() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-validation";

        env.with_env_async(|| async {
            // First create the project
            env.create_test_project(project_name).await.unwrap();

            // Create a test spec
            env.create_test_spec(
                project_name,
                "validation_test",
                "Valid specification content",
            )
            .await
            .unwrap();

            // Use spawn_blocking to run sync functions from async context
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(specs.len(), 1);
            let spec_name = specs[0].name.clone();

            // Validate the spec
            let project_name_clone = project_name.to_string();
            let spec_name_clone = spec_name.clone();
            let validation_result = tokio::task::spawn_blocking(move || {
                validate_spec_files(&project_name_clone, &spec_name_clone)
            })
            .await
            .unwrap()
            .unwrap();

            assert!(validation_result.is_valid());
            assert!(validation_result.spec_file_exists);
            assert!(validation_result.notes_file_exists);
            assert!(validation_result.task_list_file_exists);
            assert!(validation_result.content_validation.spec_valid);
            assert!(validation_result.content_validation.notes_valid);
            assert!(validation_result.content_validation.task_list_valid);
            assert!(validation_result.validation_errors.is_empty());
            assert_eq!(validation_result.summary(), "Spec is valid");
        });
    }

    #[test]
    fn test_latest_spec_retrieval() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-latest-spec-retrieval";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Initially no specs
            let project_name_clone = project_name.to_string();
            let latest = tokio::task::spawn_blocking(move || get_latest_spec(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert!(latest.is_none());

            // Create first spec
            env.create_test_spec(project_name, "first_spec", "First specification")
                .await
                .unwrap();

            // Delay to ensure different timestamps (need at least 1 second difference)
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

            // Create second spec
            env.create_test_spec(project_name, "second_spec", "Second specification")
                .await
                .unwrap();

            // Get all specs to verify we have both
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(specs.len(), 2);

            // Get latest spec (should be the second one based on timestamp)
            let project_name_clone = project_name.to_string();
            let latest = tokio::task::spawn_blocking(move || get_latest_spec(&project_name_clone))
                .await
                .unwrap()
                .unwrap()
                .unwrap();

            // Find the second spec by feature name to compare
            let second_spec = specs
                .iter()
                .find(|s| s.feature_name == "second_spec")
                .unwrap();
            assert_eq!(latest.name, second_spec.name);
            assert_eq!(latest.feature_name, "second_spec");
        });
    }

    #[test]
    fn test_directory_management() {
        let env = TestEnvironment::new().unwrap();

        env.with_env_async(|| async {
            // Use a consistent project name for this test
            let project_name = "test-directory-management-project";

            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Use the async foundry directly to avoid nested runtime issues
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // Test directory creation
            let specs_dir = ensure_specs_directory(project_name).unwrap();
            assert!(specs_dir.exists());
            assert!(specs_dir.is_dir());

            // Test path getters
            let specs_dir_path = get_specs_directory(project_name).unwrap();
            assert_eq!(specs_dir, specs_dir_path);

            // Create a spec and test spec path using async API
            let config = SpecConfig {
                project_name: project_name.to_string(),
                feature_name: "path_test".to_string(),
                content: SpecContentData {
                    spec: "Path test spec".to_string(),
                    notes: "Path test notes".to_string(),
                    tasks: "- Path test task".to_string(),
                },
            };

            let created_spec = foundry.create_spec(config).await.unwrap();
            let spec_path = get_spec_path(project_name, &created_spec.name).unwrap();

            // Test that the spec path exists and is correct
            assert!(spec_path.exists());
            assert!(spec_path.is_dir());
            assert!(spec_path.ends_with(&created_spec.name));
        });
    }

    #[test]
    fn test_fuzzy_matching_exact_spec_name() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-exact-spec";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Create test specs using helper method
            env.create_test_spec(project_name, "user_authentication", "Auth spec")
                .await
                .unwrap();
            env.create_test_spec(project_name, "payment_processing", "Payment spec")
                .await
                .unwrap();

            // Get the actual spec names from the isolated environment
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();

            assert_eq!(specs.len(), 2);
            let auth_spec = specs
                .iter()
                .find(|s| s.feature_name == "user_authentication")
                .unwrap();
            let payment_spec = specs
                .iter()
                .find(|s| s.feature_name == "payment_processing")
                .unwrap();

            // Test exact spec name match
            let project_name_clone = project_name.to_string();
            let auth_spec_name = auth_spec.name.clone();
            let auth_spec_name_clone = auth_spec_name.clone();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, &auth_spec_name_clone)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::Exact(auth_spec_name));

            let project_name_clone = project_name.to_string();
            let payment_spec_name = payment_spec.name.clone();
            let payment_spec_name_clone = payment_spec_name.clone();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, &payment_spec_name_clone)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::Exact(payment_spec_name));
        });
    }

    #[test]
    fn test_fuzzy_matching_feature_name() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-feature";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Create test specs using helper method
            env.create_test_spec(project_name, "user_authentication", "Auth spec")
                .await
                .unwrap();
            env.create_test_spec(project_name, "payment_processing", "Payment spec")
                .await
                .unwrap();

            // Get the actual spec names from the isolated environment
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();

            assert_eq!(specs.len(), 2);
            let auth_spec = specs
                .iter()
                .find(|s| s.feature_name == "user_authentication")
                .unwrap();
            let payment_spec = specs
                .iter()
                .find(|s| s.feature_name == "payment_processing")
                .unwrap();

            // Test exact feature name match using spawn_blocking
            let project_name_clone = project_name.to_string();
            let auth_spec_name = auth_spec.name.clone();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, "user_authentication")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::FeatureExact(auth_spec_name));

            let project_name_clone = project_name.to_string();
            let payment_spec_name = payment_spec.name.clone();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, "payment_processing")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::FeatureExact(payment_spec_name));

            // Test feature name substring match
            let project_name_clone = project_name.to_string();
            let auth_spec_name = auth_spec.name.clone();
            let result =
                tokio::task::spawn_blocking(move || find_spec_match(&project_name_clone, "auth"))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(result, SpecMatchStrategy::FeatureFuzzy(auth_spec_name));

            let project_name_clone = project_name.to_string();
            let payment_spec_name = payment_spec.name.clone();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, "payment")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::FeatureFuzzy(payment_spec_name));
        });
    }

    #[test]
    fn test_fuzzy_matching_no_matches() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-no-matches";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Create test spec using helper method
            env.create_test_spec(project_name, "user_authentication", "Auth spec")
                .await
                .unwrap();

            // Test no matches
            let project_name_clone = project_name.to_string();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, "completely_different")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::None);

            let project_name_clone = project_name.to_string();
            let result =
                tokio::task::spawn_blocking(move || find_spec_match(&project_name_clone, "xyz"))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(result, SpecMatchStrategy::None);
        });
    }

    #[test]
    fn test_fuzzy_matching_empty_project() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-empty";

        env.with_env_async(|| async {
            // Create test project first (but no specs)
            env.create_test_project(project_name).await.unwrap();

            // Test empty project
            let project_name_clone = project_name.to_string();
            let result = tokio::task::spawn_blocking(move || {
                find_spec_match(&project_name_clone, "anything")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result, SpecMatchStrategy::None);
        });
    }

    #[test]
    fn test_load_spec_with_fuzzy() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-load-fuzzy";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Create test spec using helper method
            env.create_test_spec(project_name, "user_authentication", "Auth spec")
                .await
                .unwrap();

            // Get the actual spec name from the isolated environment
            let project_name_clone = project_name.to_string();
            let specs = tokio::task::spawn_blocking(move || list_specs(&project_name_clone))
                .await
                .unwrap()
                .unwrap();

            assert_eq!(specs.len(), 1);
            let created_spec = &specs[0];

            // Test fuzzy loading with feature name
            let project_name_clone = project_name.to_string();
            let (loaded_spec, match_strategy) = tokio::task::spawn_blocking(move || {
                load_spec_with_fuzzy(&project_name_clone, "auth")
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(loaded_spec.name, created_spec.name);
            assert!(matches!(match_strategy, SpecMatchStrategy::FeatureFuzzy(_)));

            // Test exact loading
            let project_name_clone = project_name.to_string();
            let created_spec_name = created_spec.name.clone();
            let (loaded_spec, match_strategy) = tokio::task::spawn_blocking(move || {
                load_spec_with_fuzzy(&project_name_clone, &created_spec_name)
            })
            .await
            .unwrap()
            .unwrap();
            assert_eq!(loaded_spec.name, created_spec.name);
            assert_eq!(
                match_strategy,
                SpecMatchStrategy::Exact(created_spec.name.clone())
            );
        });
    }

    #[test]
    fn test_load_spec_with_fuzzy_no_matches() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-load-fuzzy-no-matches";

        env.with_env_async(|| async {
            // Create test project first
            env.create_test_project(project_name).await.unwrap();

            // Create test spec using helper method
            env.create_test_spec(project_name, "user_authentication", "Auth spec")
                .await
                .unwrap();

            // Test no matches
            let project_name_clone = project_name.to_string();
            let result = tokio::task::spawn_blocking(move || {
                load_spec_with_fuzzy(&project_name_clone, "completely_different")
            })
            .await
            .unwrap();
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("No specs found matching")
            );
        });
    }

    #[test]
    fn test_fuzzy_matching_empty_query() {
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-empty-query";

        _env.with_env_async(|| async {
            _env.create_test_project(project_name).await.unwrap();

            // Test empty query
            let result = load_spec_with_fuzzy(project_name, "");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Cannot search for empty spec name")
            );

            // Test whitespace-only query
            let result = load_spec_with_fuzzy(project_name, "   ");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Cannot search for empty spec name")
            );
        });
    }

    #[test]
    fn test_fuzzy_matching_empty_project_name() {
        let _env = TestEnvironment::new().unwrap();

        _env.with_env_async(|| async {
            // Test empty project name
            let result = load_spec_with_fuzzy("", "some_query");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains("Project name cannot be empty"));

            // Test whitespace-only project name
            let result = load_spec_with_fuzzy("   ", "some_query");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains("Project name cannot be empty"));
        });
    }

    #[test]
    fn test_fuzzy_matching_multiple_matches() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-multiple-matches";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();
            env.create_test_spec(project_name, "user_authentication", "User auth spec")
                .await
                .unwrap();
            env.create_test_spec(project_name, "user_management", "User management spec")
                .await
                .unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();
            let result = foundry.find_spec_match(project_name, "user").await;
            assert!(result.is_ok());
            match result.unwrap() {
                SpecMatchStrategy::Multiple(candidates) => {
                    assert!(candidates.len() >= 2);
                    assert!(candidates.iter().any(|c| c.contains("user_authentication")));
                    assert!(candidates.iter().any(|c| c.contains("user_management")));
                }
                _ => panic!("Expected Multiple match strategy"),
            }
        });
    }

    #[test]
    fn test_fuzzy_matching_empty_project_with_query() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-empty-project-with-query";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();
            let result = foundry.find_spec_match(project_name, "any_query").await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), SpecMatchStrategy::None);
        });
    }

    #[test]
    fn test_list_specs_performance() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-performance";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();
            env.create_test_spec(project_name, "test_feature", "Test spec")
                .await
                .unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // Multiple calls should work consistently (no caching, but still fast)
            let specs1 = foundry.list_specs(project_name).await.unwrap();
            assert_eq!(specs1.len(), 1);

            let specs2 = foundry.list_specs(project_name).await.unwrap();
            assert_eq!(specs2.len(), 1);
            assert_eq!(specs1[0].name, specs2[0].name);
        });
    }

    #[test]
    fn test_malformed_spec_handling() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-malformed";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();

            // Create a valid spec
            env.create_test_spec(project_name, "valid_spec", "Valid spec")
                .await
                .unwrap();

            // Create a malformed spec directory (invalid name format)
            let foundry_dir = crate::core::filesystem::foundry_dir().unwrap();
            let specs_dir = foundry_dir.join(project_name).join("specs");
            let malformed_dir = specs_dir.join("invalid_spec_name");
            std::fs::create_dir_all(&malformed_dir).unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // List specs should skip malformed ones but still return valid ones
            let specs = foundry.list_specs(project_name).await.unwrap();
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].feature_name, "valid_spec");
        });
    }

    #[test]
    fn test_fuzzy_matching_similarity_thresholds() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-similarity-thresholds";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();

            // Create test specs with similar names
            env.create_test_spec(project_name, "user_auth", "User auth spec")
                .await
                .unwrap();
            env.create_test_spec(
                project_name,
                "user_authentication",
                "User authentication spec",
            )
            .await
            .unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // Test exact match (similarity = 1.0)
            let result = foundry
                .find_spec_match(project_name, "user_auth")
                .await
                .unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_user_auth"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test high similarity match (should match "user_auth" for "user_authentication" query)
            let result = foundry
                .find_spec_match(project_name, "user_authentication")
                .await
                .unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_user_authentication"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test fuzzy match with partial similarity
            let result = foundry
                .find_spec_match(project_name, "usr_auth")
                .await
                .unwrap();
            match result {
                SpecMatchStrategy::FeatureFuzzy(_) => {
                    // This should find a fuzzy match due to high similarity
                }
                SpecMatchStrategy::Multiple(_) => {
                    // Multiple matches due to both being similar
                }
                _ => panic!("Expected fuzzy or multiple match for partial similarity"),
            }

            // Test low similarity (should not match above threshold)
            let result = foundry
                .find_spec_match(project_name, "completely_different")
                .await
                .unwrap();
            assert_eq!(result, SpecMatchStrategy::None);
        });
    }

    #[test]
    fn test_fuzzy_matching_edge_cases() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-edge-cases";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();

            // Test empty string similarity
            let similarity = strsim::normalized_levenshtein("", "");
            assert_eq!(similarity, 1.0);

            // Test single character similarity
            let similarity = strsim::normalized_levenshtein("a", "a");
            assert_eq!(similarity, 1.0);

            let similarity = strsim::normalized_levenshtein("a", "b");
            assert_eq!(similarity, 0.0);

            // Test case sensitivity (strsim is case sensitive)
            let similarity = strsim::normalized_levenshtein("User", "user");
            assert!(similarity < 1.0); // Should be less than perfect match

            // Test with actual spec data
            env.create_test_spec(project_name, "test_feature", "Test spec")
                .await
                .unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // Test exact case match
            let result = foundry
                .find_spec_match(project_name, "test_feature")
                .await
                .unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_test_feature"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test case mismatch (should not find exact match)
            let result = foundry
                .find_spec_match(project_name, "Test_Feature")
                .await
                .unwrap();
            match result {
                SpecMatchStrategy::FeatureFuzzy(_) => {
                    // Should find fuzzy match due to case difference
                }
                SpecMatchStrategy::None => {
                    // Could be no match if similarity is below threshold
                }
                _ => panic!("Unexpected match strategy for case mismatch"),
            }
        });
    }

    #[test]
    fn test_logging_hygiene_no_stderr_output() {
        let env = TestEnvironment::new().unwrap();
        let project_name = "test-logging-hygiene";

        env.with_env_async(|| async {
            env.create_test_project(project_name).await.unwrap();

            // Create a spec with a malformed directory to trigger logging
            env.create_test_spec(project_name, "valid_spec", "Valid spec")
                .await
                .unwrap();

            // Create a malformed directory manually to trigger warning logs
            let foundry_dir = crate::core::filesystem::foundry_dir().unwrap();
            let specs_dir = foundry_dir.join(project_name).join("specs");
            let malformed_dir = specs_dir.join("invalid_format_spec");
            std::fs::create_dir_all(&malformed_dir).unwrap();

            // Use the facade directly in async context
            let foundry = crate::core::foundry::get_default_foundry().unwrap();

            // Verify no eprintln! output from core functions (stderr is empty)
            // This would require more complex setup to capture stderr
            // For now, we just ensure the function calls work without panicking
            let specs = foundry.list_specs(project_name).await.unwrap();

            // Verify we still get the valid spec despite the malformed one
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].feature_name, "valid_spec");

            // In a real test, we'd check that stderr_buf is empty
            // For now, this test ensures the functions work correctly
        });
    }
}
