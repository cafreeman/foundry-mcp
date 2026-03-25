use crate::core::git;
use crate::core::names::validate_kebab_case;
use crate::core::paths::{self, project_path, spec_dir_path};
use crate::core::spec_id::{resolve_spec_id, sort_spec_ids, timestamp_id};
use crate::types::Spec;
use anyhow::{Context, Result, bail};
use std::fs;

pub fn init(project: &str, feature: &str) -> Result<Spec> {
    validate_kebab_case(feature)?;
    let project_path = crate::core::project::assert_project_exists(project)?;
    let specs_root = project_path.join("specs");
    fs::create_dir_all(&specs_root)
        .with_context(|| format!("Failed to create {}", specs_root.display()))?;

    let id = timestamp_id(feature);
    let path = specs_root.join(&id);
    if path.exists() {
        bail!("Spec directory already exists: {}", path.display());
    }

    fs::create_dir_all(&path).with_context(|| format!("Failed to create {}", path.display()))?;

    for file in ["spec.md", "task-list.md", "notes.md"] {
        let p = path.join(file);
        fs::write(&p, "").with_context(|| format!("Failed to write {}", p.display()))?;
    }

    let foundry_root = paths::foundry_dir()?;
    git::auto_commit_scaffold(
        &foundry_root,
        format!("foundry: init spec {project}/{feature}"),
        std::slice::from_ref(&path),
    )?;

    Ok(Spec {
        id,
        project_name: project.to_string(),
        feature: feature.to_string(),
        path,
    })
}

pub fn load_print(project: &str, spec_partial: &str) -> Result<()> {
    let _ = crate::core::project::assert_project_exists(project)?;
    let specs_root = project_path(project)?.join("specs");
    if !specs_root.is_dir() {
        bail!("No specs directory for project {:?}", project);
    }

    let id = resolve_spec_id(&specs_root, spec_partial)?;
    let dir = spec_dir_path(project, &id)?;

    for (label, file) in [
        ("spec.md", dir.join("spec.md")),
        ("task-list.md", dir.join("task-list.md")),
        ("notes.md", dir.join("notes.md")),
    ] {
        println!("=== {label} ===");
        match paths::read_file_opt(&file)? {
            None => println!("(missing file)"),
            Some(s) if s.is_empty() => println!("(empty)"),
            Some(s) => print!("{s}"),
        }
        println!();
    }

    Ok(())
}

pub fn list_ids(project: &str) -> Result<Vec<String>> {
    let _ = crate::core::project::assert_project_exists(project)?;
    let specs_root = project_path(project)?.join("specs");
    if !specs_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();
    for entry in fs::read_dir(&specs_root)
        .with_context(|| format!("Failed to read {}", specs_root.display()))?
    {
        let entry =
            entry.with_context(|| format!("Failed to read entry in {}", specs_root.display()))?;
        let ty = entry
            .file_type()
            .with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        if ty.is_dir() {
            ids.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    Ok(sort_spec_ids(ids))
}

pub fn delete_confirmed(project: &str, spec_partial: &str) -> Result<()> {
    let _ = crate::core::project::assert_project_exists(project)?;
    let specs_root = project_path(project)?.join("specs");
    if !specs_root.is_dir() {
        bail!("No specs directory for project {:?}", project);
    }

    let id = resolve_spec_id(&specs_root, spec_partial)?;
    let dir = spec_dir_path(project, &id)?;
    if !dir.is_dir() {
        bail!("Spec {:?} not found", id);
    }

    println!("Deleting {}", dir.display());
    fs::remove_dir_all(&dir).with_context(|| format!("Failed to delete {}", dir.display()))?;
    Ok(())
}
