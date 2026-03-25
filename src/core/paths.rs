use crate::core::names::validate_single_path_segment;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolve `~/.foundry` without creating it.
pub fn foundry_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".foundry"))
}

/// Ensure `~/.foundry` exists.
pub fn ensure_foundry_dir() -> Result<PathBuf> {
    let dir = foundry_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create foundry directory: {}", dir.display()))?;
    }
    Ok(dir)
}

/// `~/.foundry/<name>/`
pub fn project_path(name: &str) -> Result<PathBuf> {
    Ok(foundry_dir()?.join(name))
}

/// `~/.foundry/<project>/specs/<id>/`
pub fn spec_dir_path(project: &str, spec_id: &str) -> Result<PathBuf> {
    validate_single_path_segment(spec_id)?;
    Ok(project_path(project)?.join("specs").join(spec_id))
}

/// `~/.foundry/<project>/completed/` — historical collapsed specs.
pub fn completed_root(project: &str) -> Result<PathBuf> {
    Ok(project_path(project)?.join("completed"))
}

/// `~/.foundry/<project>/completed/<spec_id>/`
pub fn completed_spec_dir(project: &str, spec_id: &str) -> Result<PathBuf> {
    validate_single_path_segment(spec_id)?;
    Ok(completed_root(project)?.join(spec_id))
}

/// Read a file if it exists; otherwise `Ok(None)`.
pub fn read_file_opt(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path).with_context(|| {
        format!("Failed to read {}", path.display())
    })?))
}
