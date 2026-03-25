use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write `content` to `path` atomically (temp file + rename).
pub fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    let temp = path.with_extension({
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        if ext.is_empty() {
            "tmp".to_string()
        } else {
            format!("{ext}.tmp")
        }
    });

    fs::write(&temp, content)
        .with_context(|| format!("Failed to write temporary file: {}", temp.display()))?;
    fs::rename(&temp, path)
        .with_context(|| format!("Failed to finalize write (rename) to {}", path.display()))?;

    Ok(())
}
