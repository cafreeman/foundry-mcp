//! Spec management core logic

use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc};
use std::fs;

use crate::core::filesystem;
use crate::types::spec::{Spec, SpecConfig, SpecMetadata};
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
    let project_path = foundry_dir.join(&config.project_name).join("project");
    let specs_dir = project_path.join("specs");
    let spec_name = generate_spec_name(&config.feature_name);
    let spec_path = specs_dir.join(&spec_name);
    let created_at = Utc::now().to_rfc3339();

    // Ensure specs directory exists
    filesystem::create_dir_all(&spec_path)?;

    // Write spec files
    filesystem::write_file_atomic(spec_path.join("spec.md"), &config.spec_content)?;
    filesystem::write_file_atomic(spec_path.join("notes.md"), &config.notes)?;

    filesystem::write_file_atomic(spec_path.join("task-list.md"), &config.tasks)?;

    Ok(Spec {
        name: spec_name,
        created_at,
        path: spec_path,
        project_name: config.project_name,
        spec_content: config.spec_content,
        notes: config.notes,
        tasks: config.tasks,
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
    let specs_dir = foundry_dir.join(project_name).join("project").join("specs");

    if !specs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut specs = Vec::new();

    for entry in fs::read_dir(specs_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let spec_name = entry.file_name().to_string_lossy().to_string();

            // Use enhanced timestamp parsing
            if let Some(timestamp_str) = timestamp::parse_spec_timestamp(&spec_name) {
                if let Some(feature_name) = timestamp::extract_feature_name(&spec_name) {
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
            }
            // Skip invalid spec directories (they'll be ignored but won't cause errors)
        }
    }

    // Sort by creation time (newest first)
    specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(specs)
}

/// Load a specific spec with validation
pub fn load_spec(project_name: &str, spec_name: &str) -> Result<Spec> {
    // Validate spec name format first
    validate_spec_name(spec_name).with_context(|| format!("Invalid spec name: {}", spec_name))?;

    let foundry_dir = filesystem::foundry_dir()?;
    let spec_path = foundry_dir
        .join(project_name)
        .join("project")
        .join("specs")
        .join(spec_name);

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
    let created_at = if let Some(timestamp_str) = timestamp::parse_spec_timestamp(spec_name) {
        timestamp::spec_timestamp_to_iso(&timestamp_str)
            .unwrap_or_else(|_| timestamp::iso_timestamp())
    } else {
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
    };

    Ok(Spec {
        name: spec_name.to_string(),
        created_at,
        path: spec_path,
        project_name: project_name.to_string(),
        spec_content,
        notes,
        tasks: task_list,
    })
}
