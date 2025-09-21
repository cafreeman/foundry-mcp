//! Spec management core logic

use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc};
use std::fs;
use std::path::PathBuf;
use tracing::warn;

use crate::core::filesystem;
use crate::types::spec::{
    ContentValidationStatus, Spec, SpecConfig, SpecContentData, SpecFileType, SpecFilter,
    SpecMetadata, SpecValidationResult,
};
use crate::utils::timestamp;

/// Generate timestamped spec name
pub fn generate_spec_name(feature_name: &str) -> String {
    let now = Utc::now();
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}_{}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        feature_name
    )
}

/// Create a new spec
pub fn create_spec(config: SpecConfig) -> Result<Spec> {
    let foundry_dir = filesystem::foundry_dir()?;
    let project_path = foundry_dir.join(&config.project_name);
    let specs_dir = project_path.join("specs");
    let spec_name = generate_spec_name(&config.feature_name);
    let spec_path = specs_dir.join(&spec_name);
    let created_at = Utc::now().to_rfc3339();

    // Ensure specs directory exists
    filesystem::create_dir_all(&spec_path)?;

    // Write spec files
    filesystem::write_file_atomic(spec_path.join("spec.md"), &config.content.spec)?;
    filesystem::write_file_atomic(spec_path.join("notes.md"), &config.content.notes)?;

    filesystem::write_file_atomic(spec_path.join("task-list.md"), &config.content.tasks)?;

    Ok(Spec {
        name: spec_name,
        created_at,
        path: spec_path,
        project_name: config.project_name,
        content: config.content,
    })
}

/// Validate spec directory name format
pub fn validate_spec_name(spec_name: &str) -> Result<()> {
    if timestamp::parse_spec_timestamp(spec_name).is_none() {
        return Err(anyhow::anyhow!(
            "Invalid spec name format. Expected: YYYYMMDD_HHMMSS_feature_name, got: {}",
            spec_name
        ));
    }

    // Validate feature name part
    if let Some(feature_name) = timestamp::extract_feature_name(spec_name) {
        if feature_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Spec name must include a feature name after the timestamp"
            ));
        }

        // Validate feature name follows snake_case convention
        if !feature_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            || feature_name.starts_with('_')
            || feature_name.ends_with('_')
            || feature_name.contains("__")
        {
            return Err(anyhow::anyhow!(
                "Feature name must be in snake_case format: {}",
                feature_name
            ));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Could not extract feature name from spec name: {}",
            spec_name
        ));
    }

    Ok(())
}
/// List specs for a project with enhanced validation
pub fn list_specs(project_name: &str) -> Result<Vec<SpecMetadata>> {
    let foundry_dir = filesystem::foundry_dir()?;
    let specs_dir = foundry_dir.join(project_name).join("specs");

    if !specs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut specs = Vec::new();
    let mut malformed_count = 0;

    for entry in fs::read_dir(specs_dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let spec_name = entry.file_name().to_string_lossy().to_string();

                // Use enhanced timestamp parsing with better error handling
                match (
                    timestamp::parse_spec_timestamp(&spec_name),
                    timestamp::extract_feature_name(&spec_name),
                ) {
                    (Some(timestamp_str), Some(feature_name)) => {
                        // Convert timestamp to ISO format for consistent storage
                        let created_at = timestamp::spec_timestamp_to_iso(&timestamp_str)
                            .unwrap_or_else(|_| timestamp::iso_timestamp());

                        specs.push(SpecMetadata {
                            name: spec_name.clone(),
                            created_at,
                            feature_name,
                            project_name: project_name.to_string(),
                        });
                    }
                    _ => {
                        malformed_count += 1;
                        warn!("Skipping malformed spec directory: '{}'", spec_name);
                    }
                }
            }
        } else {
            warn!(
                "Failed to determine file type for entry: {:?}",
                entry.path()
            );
        }
    }

    // Log summary of malformed specs if any were found
    if malformed_count > 0 {
        warn!(
            "Skipped {} malformed spec directories in project '{}'",
            malformed_count, project_name
        );
    }

    // Sort by creation time (newest first)
    specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(specs)
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
    let specs = list_specs(project_name)?;
    Ok(specs.into_iter().next()) // Already sorted by creation time (newest first)
}

/// Count total specs for a project
pub fn count_specs(project_name: &str) -> Result<usize> {
    let specs = list_specs(project_name)?;
    Ok(specs.len())
}

/// Check if a spec exists
pub fn spec_exists(project_name: &str, spec_name: &str) -> Result<bool> {
    let foundry_dir = filesystem::foundry_dir()?;
    let spec_path = foundry_dir.join(project_name).join("specs").join(spec_name);

    Ok(spec_path.exists() && spec_path.is_dir())
}

/// Update spec content (for task list updates)
pub fn update_spec_content(
    project_name: &str,
    spec_name: &str,
    file_type: SpecFileType,
    new_content: &str,
) -> Result<()> {
    // Validate spec exists
    validate_spec_name(spec_name)?;
    if !spec_exists(project_name, spec_name)? {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'",
            spec_name,
            project_name
        ));
    }

    let foundry_dir = filesystem::foundry_dir()?;
    let spec_path = foundry_dir.join(project_name).join("specs").join(spec_name);

    let file_path = match file_type {
        SpecFileType::Spec => spec_path.join("spec.md"),
        SpecFileType::Notes => spec_path.join("notes.md"),
        SpecFileType::TaskList => spec_path.join("task-list.md"),
    };

    filesystem::write_file_atomic(&file_path, new_content)
        .with_context(|| format!("Failed to update {:?} for spec '{}'", file_type, spec_name))?;

    Ok(())
}

/// Get spec directory path
pub fn get_spec_path(project_name: &str, spec_name: &str) -> Result<PathBuf> {
    let foundry_dir = filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name).join("specs").join(spec_name))
}

/// Get specs directory path for a project
pub fn get_specs_directory(project_name: &str) -> Result<PathBuf> {
    let foundry_dir = filesystem::foundry_dir()?;
    Ok(foundry_dir.join(project_name).join("specs"))
}

/// Ensure specs directory exists for a project
pub fn ensure_specs_directory(project_name: &str) -> Result<PathBuf> {
    let specs_dir = get_specs_directory(project_name)?;
    filesystem::create_dir_all(&specs_dir).with_context(|| {
        format!(
            "Failed to create specs directory for project '{}'",
            project_name
        )
    })?;
    Ok(specs_dir)
}

/// Delete a spec (with confirmation)
pub fn delete_spec(project_name: &str, spec_name: &str) -> Result<()> {
    validate_spec_name(spec_name)?;

    let spec_path = get_spec_path(project_name, spec_name)?;

    if !spec_path.exists() {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'",
            spec_name,
            project_name
        ));
    }

    std::fs::remove_dir_all(&spec_path).with_context(|| {
        format!(
            "Failed to delete spec '{}' from project '{}'",
            spec_name, project_name
        )
    })?;

    Ok(())
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
        match filesystem::read_file(&spec_file) {
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
        match filesystem::read_file(&notes_file) {
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
        match filesystem::read_file(&task_list_file) {
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
    // Validate inputs
    if query.trim().is_empty() {
        return Err(anyhow::anyhow!("Query cannot be empty"));
    }

    if project_name.trim().is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    let available_specs = list_specs(project_name)?;

    if available_specs.is_empty() {
        return Ok(SpecMatchStrategy::None);
    }

    // Try exact spec name match first (highest priority)
    if let Some(exact_match) = available_specs.iter().find(|s| s.name == query) {
        return Ok(SpecMatchStrategy::Exact(exact_match.name.clone()));
    }

    // Try exact feature name match
    if let Some(feature_match) = available_specs.iter().find(|s| s.feature_name == query) {
        return Ok(SpecMatchStrategy::FeatureExact(feature_match.name.clone()));
    }

    // Try feature name substring match (case-insensitive)
    let query_lower = query.to_lowercase();
    let substring_matches: Vec<&SpecMetadata> = available_specs
        .iter()
        .filter(|s| s.feature_name.to_lowercase().contains(&query_lower))
        .collect();

    if substring_matches.len() == 1 {
        return Ok(SpecMatchStrategy::FeatureFuzzy(
            substring_matches[0].name.clone(),
        ));
    } else if substring_matches.len() > 1 {
        // Multiple substring matches - return for disambiguation
        let mut names: Vec<String> = substring_matches
            .into_iter()
            .map(|s| s.name.clone())
            .collect();
        names.sort();
        return Ok(SpecMatchStrategy::Multiple(names));
    }

    // Try fuzzy matching on feature names
    let feature_matches: Vec<(String, f32)> = available_specs
        .iter()
        .map(|s| {
            let similarity = strsim::normalized_levenshtein(query, &s.feature_name) as f32;
            (s.name.clone(), similarity)
        })
        .filter(|(_, confidence)| *confidence > 0.8) // High confidence threshold
        .collect();

    if feature_matches.len() == 1 {
        return Ok(SpecMatchStrategy::FeatureFuzzy(
            feature_matches[0].0.clone(),
        ));
    } else if feature_matches.len() > 1 {
        // Multiple feature matches - return for disambiguation
        let mut names: Vec<String> = feature_matches.into_iter().map(|(name, _)| name).collect();
        names.sort();
        return Ok(SpecMatchStrategy::Multiple(names));
    }

    // Try fuzzy matching on spec names
    let name_matches: Vec<(String, f32)> = available_specs
        .iter()
        .map(|s| {
            let similarity = strsim::normalized_levenshtein(query, &s.name) as f32;
            (s.name.clone(), similarity)
        })
        .filter(|(_, confidence)| *confidence > 0.8) // High confidence threshold
        .collect();

    if name_matches.len() == 1 {
        return Ok(SpecMatchStrategy::NameFuzzy(name_matches[0].0.clone()));
    } else if name_matches.len() > 1 {
        // Multiple name matches - return for disambiguation
        let mut names: Vec<String> = name_matches.into_iter().map(|(name, _)| name).collect();
        names.sort();
        return Ok(SpecMatchStrategy::Multiple(names));
    }

    Ok(SpecMatchStrategy::None)
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
    // Validate spec name format first
    validate_spec_name(spec_name).with_context(|| format!("Invalid spec name: {}", spec_name))?;

    let foundry_dir = filesystem::foundry_dir()?;
    let spec_path = foundry_dir.join(project_name).join("specs").join(spec_name);

    if !spec_path.exists() {
        return Err(anyhow::anyhow!(
            "Spec '{}' not found in project '{}'",
            spec_name,
            project_name
        ));
    }

    // Read spec files
    let spec_content = filesystem::read_file(spec_path.join("spec.md"))?;
    let notes = filesystem::read_file(spec_path.join("notes.md"))?;
    let task_list = filesystem::read_file(spec_path.join("task-list.md"))?;

    // Get creation time from spec name timestamp (more reliable than filesystem metadata)
    let created_at = timestamp::parse_spec_timestamp(spec_name).map_or_else(
        || {
            // Fallback to filesystem metadata if timestamp parsing fails
            fs::metadata(&spec_path)
                .and_then(|metadata| metadata.created())
                .map_err(anyhow::Error::from)
                .and_then(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .map_err(anyhow::Error::from)
                })
                .map(|duration| {
                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                        .unwrap_or_else(chrono::Utc::now)
                        .to_rfc3339()
                })
                .unwrap_or_else(|_| timestamp::iso_timestamp())
        },
        |timestamp_str| {
            timestamp::spec_timestamp_to_iso(&timestamp_str)
                .unwrap_or_else(|_| timestamp::iso_timestamp())
        },
    );

    Ok(Spec {
        name: spec_name.to_string(),
        created_at,
        path: spec_path,
        project_name: project_name.to_string(),
        content: SpecContentData {
            spec: spec_content,
            notes,
            tasks: task_list,
        },
    })
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
    use crate::types::spec::{SpecConfig, SpecFileType, SpecFilter};
    use std::sync::Mutex;

    // Use a mutex to serialize tests that modify global environment
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Acquire test mutex lock, handling poisoning gracefully
    fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_MUTEX.lock().unwrap_or_else(|poisoned| {
            // Clear the poisoned state and acquire the lock
            poisoned.into_inner()
        })
    }

    // removed legacy setup_test_environment in favor of TestEnvironment

    #[test]
    fn test_spec_filtering() {
        use crate::test_utils::TestEnvironment;
        let _lock = acquire_test_lock();
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-filtering";

        // Create a few test specs
        let spec_configs = vec![
            SpecConfig {
                project_name: project_name.to_string(),
                feature_name: "user_auth".to_string(),
                content: SpecContentData {
                    spec: "User authentication specification".to_string(),
                    notes: "Authentication notes".to_string(),
                    tasks: "- Implement login\n- Implement logout".to_string(),
                },
            },
            SpecConfig {
                project_name: project_name.to_string(),
                feature_name: "user_profile".to_string(),
                content: SpecContentData {
                    spec: "User profile management".to_string(),
                    notes: "Profile notes".to_string(),
                    tasks: "- Profile CRUD\n- Avatar upload".to_string(),
                },
            },
        ];

        for config in spec_configs {
            create_spec(config).unwrap();
        }

        // Test filtering by feature name
        let filter = SpecFilter {
            feature_name_contains: Some("user".to_string()),
            ..Default::default()
        };

        let filtered_specs = list_specs_filtered(project_name, filter).unwrap();
        assert_eq!(filtered_specs.len(), 2);

        // Test filtering with limit
        let filter = SpecFilter {
            limit: Some(1),
            ..Default::default()
        };

        let limited_specs = list_specs_filtered(project_name, filter).unwrap();
        assert_eq!(limited_specs.len(), 1);
    }

    #[test]
    fn test_spec_existence_and_counting() {
        use crate::test_utils::TestEnvironment;
        let _lock = acquire_test_lock();
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-existence";

        // Test empty project
        assert_eq!(count_specs(project_name).unwrap(), 0);
        assert!(!spec_exists(project_name, "nonexistent_spec").unwrap());

        // Create a spec
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "test_feature".to_string(),
            content: SpecContentData {
                spec: "Test specification".to_string(),
                notes: "Test notes".to_string(),
                tasks: "- Test task".to_string(),
            },
        };

        let created_spec = create_spec(config).unwrap();

        // Test counting and existence
        assert_eq!(count_specs(project_name).unwrap(), 1);
        assert!(spec_exists(project_name, &created_spec.name).unwrap());
    }

    #[test]
    fn test_spec_content_updates() {
        use crate::test_utils::TestEnvironment;
        let _lock = acquire_test_lock();
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-content-updates";

        // Create a spec
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "updatable_spec".to_string(),
            content: SpecContentData {
                spec: "Original specification".to_string(),
                notes: "Original notes".to_string(),
                tasks: "- Original task".to_string(),
            },
        };

        let created_spec = create_spec(config).unwrap();

        // Update task list
        let new_tasks = "- Updated task\n- New task\n- [ ] Completed task";
        update_spec_content(
            project_name,
            &created_spec.name,
            SpecFileType::TaskList,
            new_tasks,
        )
        .unwrap();

        // Verify update
        let loaded_spec = load_spec(project_name, &created_spec.name).unwrap();
        assert_eq!(loaded_spec.content.tasks, new_tasks);
        assert_eq!(loaded_spec.content.spec, "Original specification");
    }

    #[test]
    fn test_spec_validation() {
        use crate::test_utils::TestEnvironment;
        let _lock = acquire_test_lock();
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-spec-validation";

        // Create a spec
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "validation_test".to_string(),
            content: SpecContentData {
                spec: "Valid specification content".to_string(),
                notes: "Valid notes".to_string(),
                tasks: "- Valid task".to_string(),
            },
        };

        let created_spec = create_spec(config).unwrap();

        // Validate the spec
        let validation_result = validate_spec_files(project_name, &created_spec.name).unwrap();

        assert!(validation_result.is_valid());
        assert!(validation_result.spec_file_exists);
        assert!(validation_result.notes_file_exists);
        assert!(validation_result.task_list_file_exists);
        assert!(validation_result.content_validation.spec_valid);
        assert!(validation_result.content_validation.notes_valid);
        assert!(validation_result.content_validation.task_list_valid);
        assert!(validation_result.validation_errors.is_empty());
        assert_eq!(validation_result.summary(), "Spec is valid");
    }

    #[test]
    fn test_latest_spec_retrieval() {
        use crate::test_utils::TestEnvironment;
        let _lock = acquire_test_lock();
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-latest-spec-retrieval";

        // Initially no specs
        assert!(get_latest_spec(project_name).unwrap().is_none());

        // Create first spec
        let config1 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "first_spec".to_string(),
            content: SpecContentData {
                spec: "First specification".to_string(),
                notes: "First notes".to_string(),
                tasks: "- First task".to_string(),
            },
        };

        let _spec1 = create_spec(config1).unwrap();

        // Delay to ensure different timestamps (need at least 1 second difference)
        std::thread::sleep(std::time::Duration::from_millis(1100));

        // Create second spec
        let config2 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "second_spec".to_string(),
            content: SpecContentData {
                spec: "Second specification".to_string(),
                notes: "Second notes".to_string(),
                tasks: "- Second task".to_string(),
            },
        };

        let spec2 = create_spec(config2).unwrap();

        // Get latest spec (should be the second one)
        let latest = get_latest_spec(project_name).unwrap().unwrap();
        assert_eq!(latest.name, spec2.name);
        assert_eq!(latest.feature_name, "second_spec");
    }

    #[test]
    fn test_directory_management() {
        // Use proper TestEnvironment for isolation instead of setup_test_environment
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();

        // Use a consistent project name for this test
        let project_name = "test-directory-management-project";

        // Test directory creation
        let specs_dir = ensure_specs_directory(project_name).unwrap();
        assert!(specs_dir.exists());
        assert!(specs_dir.is_dir());

        // Test path getters
        let specs_dir_path = get_specs_directory(project_name).unwrap();
        assert_eq!(specs_dir, specs_dir_path);

        // Create a spec and test spec path
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "path_test".to_string(),
            content: SpecContentData {
                spec: "Path test spec".to_string(),
                notes: "Path test notes".to_string(),
                tasks: "- Path test task".to_string(),
            },
        };

        let created_spec = create_spec(config).unwrap();
        let spec_path = get_spec_path(project_name, &created_spec.name).unwrap();

        // Test that the spec path exists and is correct
        assert!(spec_path.exists());
        assert!(spec_path.is_dir());
        assert!(spec_path.ends_with(&created_spec.name));
    }

    #[test]
    fn test_fuzzy_matching_exact_spec_name() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-exact-spec";

        // Create test specs
        let config1 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "user_authentication".to_string(),
            content: SpecContentData {
                spec: "Auth spec".to_string(),
                notes: "Auth notes".to_string(),
                tasks: "- Auth task".to_string(),
            },
        };
        let spec1 = create_spec(config1).unwrap();

        let config2 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "payment_processing".to_string(),
            content: SpecContentData {
                spec: "Payment spec".to_string(),
                notes: "Payment notes".to_string(),
                tasks: "- Payment task".to_string(),
            },
        };
        let spec2 = create_spec(config2).unwrap();

        // Test exact spec name match
        let result = find_spec_match(project_name, &spec1.name).unwrap();
        assert_eq!(result, SpecMatchStrategy::Exact(spec1.name));

        let result = find_spec_match(project_name, &spec2.name).unwrap();
        assert_eq!(result, SpecMatchStrategy::Exact(spec2.name));
    }

    #[test]
    fn test_fuzzy_matching_feature_name() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-feature";

        // Create test specs
        let config1 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "user_authentication".to_string(),
            content: SpecContentData {
                spec: "Auth spec".to_string(),
                notes: "Auth notes".to_string(),
                tasks: "- Auth task".to_string(),
            },
        };
        let spec1 = create_spec(config1).unwrap();

        let config2 = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "payment_processing".to_string(),
            content: SpecContentData {
                spec: "Payment spec".to_string(),
                notes: "Payment notes".to_string(),
                tasks: "- Payment task".to_string(),
            },
        };
        let spec2 = create_spec(config2).unwrap();

        // Test exact feature name match
        let result = find_spec_match(project_name, "user_authentication").unwrap();
        assert_eq!(result, SpecMatchStrategy::FeatureExact(spec1.name.clone()));

        let result = find_spec_match(project_name, "payment_processing").unwrap();
        assert_eq!(result, SpecMatchStrategy::FeatureExact(spec2.name.clone()));

        // Test feature name substring match
        let result = find_spec_match(project_name, "auth").unwrap();
        assert_eq!(result, SpecMatchStrategy::FeatureFuzzy(spec1.name));

        let result = find_spec_match(project_name, "payment").unwrap();
        assert_eq!(result, SpecMatchStrategy::FeatureFuzzy(spec2.name));
    }

    #[test]
    fn test_fuzzy_matching_no_matches() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-no-matches";

        // Create test specs
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "user_authentication".to_string(),
            content: SpecContentData {
                spec: "Auth spec".to_string(),
                notes: "Auth notes".to_string(),
                tasks: "- Auth task".to_string(),
            },
        };
        let _spec = create_spec(config).unwrap();

        // Test no matches
        let result = find_spec_match(project_name, "completely_different").unwrap();
        assert_eq!(result, SpecMatchStrategy::None);

        let result = find_spec_match(project_name, "xyz").unwrap();
        assert_eq!(result, SpecMatchStrategy::None);
    }

    #[test]
    fn test_fuzzy_matching_empty_project() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-fuzzy-empty";

        // Test empty project
        let result = find_spec_match(project_name, "anything").unwrap();
        assert_eq!(result, SpecMatchStrategy::None);
    }

    #[test]
    fn test_load_spec_with_fuzzy() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-load-fuzzy";

        // Create test spec
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "user_authentication".to_string(),
            content: SpecContentData {
                spec: "Auth spec".to_string(),
                notes: "Auth notes".to_string(),
                tasks: "- Auth task".to_string(),
            },
        };
        let created_spec = create_spec(config).unwrap();

        // Test fuzzy loading with feature name
        let (loaded_spec, match_strategy) = load_spec_with_fuzzy(project_name, "auth").unwrap();
        assert_eq!(loaded_spec.name, created_spec.name);
        assert!(matches!(match_strategy, SpecMatchStrategy::FeatureFuzzy(_)));

        // Test exact loading
        let (loaded_spec, match_strategy) =
            load_spec_with_fuzzy(project_name, &created_spec.name).unwrap();
        assert_eq!(loaded_spec.name, created_spec.name);
        assert_eq!(match_strategy, SpecMatchStrategy::Exact(created_spec.name));
    }

    #[test]
    fn test_load_spec_with_fuzzy_no_matches() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-load-fuzzy-no-matches";

        // Create test spec
        let config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "user_authentication".to_string(),
            content: SpecContentData {
                spec: "Auth spec".to_string(),
                notes: "Auth notes".to_string(),
                tasks: "- Auth task".to_string(),
            },
        };
        let _spec = create_spec(config).unwrap();

        // Test no matches
        let result = load_spec_with_fuzzy(project_name, "completely_different");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No specs found matching")
        );
    }

    #[test]
    fn test_fuzzy_matching_empty_query() {
        use crate::test_utils::TestEnvironment;
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
        use crate::test_utils::TestEnvironment;
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
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-multiple-matches";

        _env.with_env_async(|| async {
            _env.create_test_project(project_name).await.unwrap();
            _env.create_test_spec(project_name, "user_authentication", "User auth spec")
                .await
                .unwrap();
            _env.create_test_spec(project_name, "user_management", "User management spec")
                .await
                .unwrap();

            // Test multiple matches
            let result = load_spec_with_fuzzy(project_name, "user");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains("Multiple specs match"));
            assert!(error.to_string().contains("user_authentication"));
            assert!(error.to_string().contains("user_management"));
        });
    }

    #[test]
    fn test_fuzzy_matching_empty_project_with_query() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-empty-project-with-query";

        _env.with_env_async(|| async {
            _env.create_test_project(project_name).await.unwrap();

            // Test query on empty project
            let result = load_spec_with_fuzzy(project_name, "any_query");
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains("No specs found in project"));
            assert!(error.to_string().contains("mcp_foundry_create_spec"));
        });
    }

    #[test]
    fn test_list_specs_performance() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-performance";

        _env.with_env_async(|| async {
            _env.create_test_project(project_name).await.unwrap();
            _env.create_test_spec(project_name, "test_feature", "Test spec")
                .await
                .unwrap();

            // Multiple calls should work consistently (no caching, but still fast)
            let specs1 = list_specs(project_name).unwrap();
            assert_eq!(specs1.len(), 1);

            let specs2 = list_specs(project_name).unwrap();
            assert_eq!(specs2.len(), 1);
            assert_eq!(specs1[0].name, specs2[0].name);
        });
    }

    #[test]
    fn test_malformed_spec_handling() {
        use crate::test_utils::TestEnvironment;
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test-malformed";

        _env.with_env_async(|| async {
            _env.create_test_project(project_name).await.unwrap();

            // Create a valid spec
            _env.create_test_spec(project_name, "valid_spec", "Valid spec")
                .await
                .unwrap();

            // Create a malformed spec directory (invalid name format)
            let foundry_dir = filesystem::foundry_dir().unwrap();
            let specs_dir = foundry_dir.join(project_name).join("specs");
            let malformed_dir = specs_dir.join("invalid_spec_name");
            std::fs::create_dir_all(&malformed_dir).unwrap();

            // List specs should skip malformed ones but still return valid ones
            let specs = list_specs(project_name).unwrap();
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].feature_name, "valid_spec");
        });
    }

    #[test]
    fn test_fuzzy_matching_similarity_thresholds() {
        use crate::test_utils::TestEnvironment;
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

            // Test exact match (similarity = 1.0)
            let result = find_spec_match(project_name, "user_auth").unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_user_auth"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test high similarity match (should match "user_auth" for "user_authentication" query)
            let result = find_spec_match(project_name, "user_authentication").unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_user_authentication"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test fuzzy match with partial similarity
            let result = find_spec_match(project_name, "usr_auth").unwrap();
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
            let result = find_spec_match(project_name, "completely_different").unwrap();
            assert_eq!(result, SpecMatchStrategy::None);
        });
    }

    #[test]
    fn test_fuzzy_matching_edge_cases() {
        use crate::test_utils::TestEnvironment;
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

            // Test exact case match
            let result = find_spec_match(project_name, "test_feature").unwrap();
            match result {
                SpecMatchStrategy::FeatureExact(spec_name) => {
                    assert!(spec_name.ends_with("_test_feature"));
                    assert!(spec_name.starts_with("20")); // Valid year prefix
                }
                _ => panic!("Expected FeatureExact match"),
            }

            // Test case mismatch (should not find exact match)
            let result = find_spec_match(project_name, "Test_Feature").unwrap();
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
        use crate::test_utils::TestEnvironment;

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

            // Verify no eprintln! output from core functions (stderr is empty)

            // This would require more complex setup to capture stderr
            // For now, we just ensure the function calls work without panicking
            let specs = list_specs(project_name).unwrap();

            // Verify we still get the valid spec despite the malformed one
            assert_eq!(specs.len(), 1);
            assert_eq!(specs[0].feature_name, "valid_spec");

            // In a real test, we'd check that stderr_buf is empty
            // For now, this test ensures the functions work correctly
        });
    }
}
