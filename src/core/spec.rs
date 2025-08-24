//! Spec management core logic

use anyhow::Result;
use chrono::{Datelike, Timelike, Utc};
use std::fs;

use crate::core::filesystem;
use crate::types::spec::{Spec, SpecConfig, SpecMetadata};

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

/// List specs for a project
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

            // Parse timestamp and feature name from spec name
            if let Some((timestamp_str, feature_name)) = spec_name.split_once('_') {
                if timestamp_str.len() >= 15 {
                    // YYYYMMDD_HHMMSS is 15 chars
                    specs.push(SpecMetadata {
                        name: spec_name.clone(),
                        created_at: timestamp_str.to_string(),
                        feature_name: feature_name.to_string(),
                        project_name: project_name.to_string(),
                    });
                }
            }
        }
    }

    // Sort by creation time (newest first)
    specs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(specs)
}

/// Load a specific spec
pub fn load_spec(project_name: &str, spec_name: &str) -> Result<Spec> {
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

    // Get creation time from directory metadata
    let created_at = fs::metadata(&spec_path)?
        .created()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();

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
