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
    let ids = list_completed_ids(project)?;
    let mut out = Vec::new();
    for id in ids {
        let dir = completed_spec_dir(project, &id)?;
        let summary = dir.join(SUMMARY_FILE);
        out.push(CompletedSpecEntry {
            id,
            path: dir,
            summary_exists: summary.is_file(),
        });
    }
    Ok(out)
}

pub fn list_completed_ids(project: &str) -> Result<Vec<String>> {
    validate_project_name(project)?;
    let root = completed_root(project)?;
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in
        fs::read_dir(&root).with_context(|| format!("Failed to read {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", root.display()))?;
        if entry.file_type()?.is_dir() {
            ids.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
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
    let summary = completed.join(SUMMARY_FILE);
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

    // Remove any other loose files in active spec dir (except if non-empty)
    for entry in
        fs::read_dir(&active).with_context(|| format!("Failed to read {}", active.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            bail!(
                "Unexpected file in active spec dir {}; move or delete it before finalize: {}",
                active.display(),
                path.display()
            );
        }
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

pub fn completed_summary_path(project: &str, spec_id: &str) -> Result<PathBuf> {
    Ok(completed_spec_dir(project, spec_id)?.join(SUMMARY_FILE))
}
