//! File system operations and utilities

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Ensure the foundry directory exists
pub fn ensure_foundry_dir() -> Result<PathBuf> {
    let foundry_dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".foundry");

    if !foundry_dir.exists() {
        fs::create_dir_all(&foundry_dir)
            .with_context(|| format!("Failed to create foundry directory: {:?}", foundry_dir))?;
    }

    Ok(foundry_dir)
}

/// Get the foundry directory path
pub fn foundry_dir() -> Result<PathBuf> {
    ensure_foundry_dir()
}

/// Write content to a file atomically
pub fn write_file_atomic<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
    }

    // Write to temporary file first
    let temp_path = path.with_extension(format!(
        "{}.tmp",
        path.extension().unwrap_or_default().to_string_lossy()
    ));
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temporary file: {:?}", temp_path))?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temporary file to: {:?}", path))?;

    Ok(())
}

/// Read file content
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Create a directory and all necessary parent directories
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).with_context(|| format!("Failed to create directory: {:?}", path))
}
