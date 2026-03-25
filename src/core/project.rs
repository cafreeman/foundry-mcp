use crate::core::git;
use crate::core::names::validate_kebab_case;
use crate::core::paths::{self, project_path};
use crate::types::Project;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Reject path-like segments (`..`, `/`, etc.): project args must be a single kebab-case store name.
pub fn validate_project_name(name: &str) -> Result<()> {
    validate_kebab_case(name)
}

pub fn init(name: &str) -> Result<Project> {
    validate_project_name(name)?;
    let root = paths::ensure_foundry_dir()?;
    let path = root.join(name);
    if path.exists() {
        bail!("Project {:?} already exists", name);
    }

    fs::create_dir_all(path.join("specs"))
        .with_context(|| format!("Failed to create project directory: {}", path.display()))?;

    for file in ["vision.md", "tech-stack.md", "summary.md"] {
        let p = path.join(file);
        fs::write(&p, "").with_context(|| format!("Failed to write {}", p.display()))?;
    }

    git::auto_commit_scaffold(
        &root,
        format!("foundry: init project {name}"),
        std::slice::from_ref(&path),
    )?;

    Ok(Project {
        name: name.to_string(),
        path,
    })
}

pub fn load_print(name: &str) -> Result<()> {
    validate_project_name(name)?;
    let path = project_path(name)?;
    if !path.is_dir() {
        bail!("Project {:?} not found", name);
    }

    for (label, file) in [
        ("vision.md", path.join("vision.md")),
        ("tech-stack.md", path.join("tech-stack.md")),
        ("summary.md", path.join("summary.md")),
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

pub fn list_names() -> Result<Vec<String>> {
    let root = paths::foundry_dir()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in
        fs::read_dir(&root).with_context(|| format!("Failed to read {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", root.display()))?;
        let ty = entry
            .file_type()
            .with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        if ty.is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    names.sort();
    Ok(names)
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
}

pub fn list_project_entries() -> Result<Vec<ProjectEntry>> {
    let names = list_names()?;
    names
        .into_iter()
        .map(|name| {
            let path = project_path(&name)?;
            Ok(ProjectEntry { name, path })
        })
        .collect()
}

pub fn delete_confirmed(name: &str) -> Result<()> {
    validate_project_name(name)?;
    let path = project_path(name)?;
    if !path.is_dir() {
        bail!("Project {:?} not found", name);
    }

    println!("Deleting {}", path.display());
    fs::remove_dir_all(&path).with_context(|| format!("Failed to delete {}", path.display()))?;
    Ok(())
}

pub fn assert_project_exists(name: &str) -> Result<PathBuf> {
    validate_project_name(name)?;
    let path = project_path(name)?;
    if !path.is_dir() {
        bail!("Project {:?} not found", name);
    }
    Ok(path)
}
