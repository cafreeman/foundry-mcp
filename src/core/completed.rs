//! Collapsed specs under `~/.foundry/<project>/completed/<spec_id>/`.

use crate::core::git;
use crate::core::paths::{self, completed_root, completed_spec_dir, project_path, spec_dir_path};
use crate::core::project::validate_project_name;
use crate::core::spec_id::sort_spec_ids;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

pub const SUMMARY_FILE: &str = "summary.md";
pub const ARCHIVE_DIR: &str = "archive";

/// Files moved from the active spec into `archive/` on finalize.
const ARCHIVE_FILES: [&str; 4] = [
    "spec.md",
    "task-list.md",
    "notes.md",
    crate::core::spec_meta::META_FILE,
];

#[derive(Debug, Clone, Serialize)]
pub struct CompletedSpecEntry {
    pub id: String,
    pub path: PathBuf,
    pub summary_exists: bool,
}

pub fn list_completed_entries(project: &str) -> Result<Vec<CompletedSpecEntry>> {
    list_completed_ids(project)?
        .into_iter()
        .map(|id| {
            let dir = completed_spec_dir(project, &id)?;
            Ok(CompletedSpecEntry {
                summary_exists: dir.join(SUMMARY_FILE).is_file(),
                id,
                path: dir,
            })
        })
        .collect()
}

pub fn list_completed_ids(project: &str) -> Result<Vec<String>> {
    validate_project_name(project)?;
    let root = completed_root(project)?;
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let ids = fs::read_dir(&root)
        .with_context(|| format!("Failed to read {}", root.display()))?
        .map(|entry| {
            let entry =
                entry.with_context(|| format!("Failed to read entry in {}", root.display()))?;
            let ty = entry.file_type().with_context(|| {
                format!("Failed to read file type for {}", entry.path().display())
            })?;
            Ok(ty
                .is_dir()
                .then(|| entry.file_name().to_string_lossy().into_owned()))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(sort_spec_ids(ids))
}

pub fn collapse_prepare(project: &str, spec_id: &str) -> Result<PathBuf> {
    let _ = crate::core::project::assert_project_exists(project)?;
    let active = spec_dir_path(project, spec_id)?;
    if !active.is_dir() {
        bail!("Active spec {:?} not found", spec_id);
    }

    let completed = completed_spec_dir(project, spec_id)?;
    if completed.exists() {
        bail!(
            "Completed record already exists at {}; remove it or choose a different spec",
            completed.display()
        );
    }

    fs::create_dir_all(&completed)
        .with_context(|| format!("Failed to create {}", completed.display()))?;

    let summary = completed.join(SUMMARY_FILE);
    if !summary.exists() {
        fs::write(&summary, "")
            .with_context(|| format!("Failed to create {}", summary.display()))?;
    }

    let foundry_root = paths::foundry_dir()?;
    git::auto_commit_scaffold(
        &foundry_root,
        format!("foundry: collapse prepare {project}/{spec_id}"),
        std::slice::from_ref(&completed),
    )?;

    Ok(summary)
}

pub fn collapse_finalize(project: &str, spec_id: &str) -> Result<PathBuf> {
    let _ = crate::core::project::assert_project_exists(project)?;
    let active = spec_dir_path(project, spec_id)?;
    if !active.is_dir() {
        bail!("Active spec {:?} not found", spec_id);
    }

    let completed = completed_spec_dir(project, spec_id)?;
    let summary = completed_summary_path(project, spec_id)?;
    if !summary.is_file() {
        bail!(
            "Run `foundry spec collapse prepare` first; missing {}",
            summary.display()
        );
    }
    let summary_text = fs::read_to_string(&summary)
        .with_context(|| format!("Failed to read {}", summary.display()))?;
    if summary_text.trim().is_empty() {
        bail!(
            "summary.md is empty; the agent must write the completed-work summary before finalize"
        );
    }

    let archive = completed.join(ARCHIVE_DIR);
    if archive.exists() {
        bail!("Archive directory already exists; refusing to finalize twice");
    }
    fs::create_dir_all(&archive)
        .with_context(|| format!("Failed to create {}", archive.display()))?;

    for name in ARCHIVE_FILES {
        let from = active.join(name);
        if from.is_file() {
            let to = archive.join(name);
            fs::rename(&from, &to).with_context(|| {
                format!("Failed to move {} to {}", from.display(), to.display())
            })?;
        }
    }

    let remaining_entries = fs::read_dir(&active)
        .with_context(|| format!("Failed to read {}", active.display()))?
        .map(|entry| {
            let entry =
                entry.with_context(|| format!("Failed to read entry in {}", active.display()))?;
            let ty = entry.file_type().with_context(|| {
                format!("Failed to read file type for {}", entry.path().display())
            })?;
            Ok((entry.path(), ty))
        })
        .collect::<Result<Vec<_>>>()?;

    let unexpected_files: Vec<String> = remaining_entries
        .iter()
        .filter(|(_, ty)| ty.is_file())
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !unexpected_files.is_empty() {
        bail!(
            "Unexpected files in active spec dir {}; move or delete them before finalize:\n{}",
            active.display(),
            unexpected_files.join("\n")
        );
    }

    let unexpected_subdirs: Vec<String> = remaining_entries
        .iter()
        .filter(|(_, ty)| ty.is_dir())
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !unexpected_subdirs.is_empty() {
        bail!(
            "Unexpected subdirectories in active spec dir {}; move or delete them before finalize:\n{}",
            active.display(),
            unexpected_subdirs.join("\n")
        );
    }

    let unexpected_entries: Vec<String> = remaining_entries
        .iter()
        .filter(|(_, ty)| !ty.is_file() && !ty.is_dir())
        .map(|(path, _)| path.display().to_string())
        .collect();
    if !unexpected_entries.is_empty() {
        bail!(
            "Unexpected entries in active spec dir {}; move or delete them before finalize:\n{}",
            active.display(),
            unexpected_entries.join("\n")
        );
    }

    fs::remove_dir(&active)
        .with_context(|| format!("Failed to remove active spec dir {}", active.display()))?;

    let foundry_root = paths::foundry_dir()?;
    let proj_dir = project_path(project)?;
    git::commit_project_subtree(
        &foundry_root,
        &proj_dir,
        &format!("foundry: collapse finalize {project}/{spec_id}"),
    )?;

    Ok(completed)
}

pub(crate) fn completed_summary_path(project: &str, spec_id: &str) -> Result<PathBuf> {
    Ok(completed_spec_dir(project, spec_id)?.join(SUMMARY_FILE))
}
