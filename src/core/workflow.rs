//! Derived workflow phase and instruction strings for agent-facing JSON.

use crate::core::paths::{self, read_file_opt};
use crate::core::spec_id::parse_spec_id;
use crate::core::spec_meta::{SpecMeta, SpecPhaseHint, read_meta};
use crate::core::task_parse::{TaskListSummary, parse_task_list};
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivedPhase {
    EmptyScaffold,
    Drafting,
    ReadyForImplementation,
    Implementing,
    CompletedPendingCollapse,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecFileContents {
    pub spec_md: Option<String>,
    pub task_list_md: Option<String>,
    pub notes_md: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecContextFiles {
    pub spec_md: PathBuf,
    pub task_list_md: PathBuf,
    pub notes_md: PathBuf,
    pub meta_json: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecWorkflowSnapshot {
    pub project: String,
    pub spec_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_prefix: Option<String>,
    pub path: PathBuf,
    pub meta: SpecMeta,
    pub derived_phase: DerivedPhase,
    pub task_list: TaskListSummary,
    pub files: SpecFileContents,
}

fn trim_or_empty(s: &Option<String>) -> &str {
    s.as_deref().map(str::trim).unwrap_or("")
}

fn derive_phase(meta: &SpecMeta, spec_body: &str, task_summary: &TaskListSummary) -> DerivedPhase {
    if let Some(h) = meta.phase_hint {
        return match h {
            SpecPhaseHint::EmptyScaffold => DerivedPhase::EmptyScaffold,
            SpecPhaseHint::Drafting => DerivedPhase::Drafting,
            SpecPhaseHint::ReadyForImplementation => DerivedPhase::ReadyForImplementation,
            SpecPhaseHint::Implementing => DerivedPhase::Implementing,
            SpecPhaseHint::CompletedPendingCollapse => DerivedPhase::CompletedPendingCollapse,
        };
    }

    if task_summary.total > 0 {
        return if task_summary.complete == task_summary.total {
            DerivedPhase::CompletedPendingCollapse
        } else {
            DerivedPhase::Implementing
        };
    }

    if spec_body.trim().is_empty() {
        DerivedPhase::EmptyScaffold
    } else {
        DerivedPhase::Drafting
    }
}

pub fn load_spec_snapshot(
    project: &str,
    spec_dir: &Path,
    spec_id: &str,
) -> Result<SpecWorkflowSnapshot> {
    let meta = read_meta(spec_dir)?;
    let spec_md = read_file_opt(&spec_dir.join("spec.md"))?;
    let task_list_md = read_file_opt(&spec_dir.join("task-list.md"))?;
    let notes_md = read_file_opt(&spec_dir.join("notes.md"))?;

    let spec_body = trim_or_empty(&spec_md);
    let task_src = task_list_md.as_deref().unwrap_or("");
    let task_summary = parse_task_list(task_src);
    let derived_phase = derive_phase(&meta, spec_body, &task_summary);

    let (timestamp_prefix, feature) = parse_spec_id(spec_id)
        .map(|(ts, f)| (Some(ts), Some(f)))
        .unwrap_or((None, None));

    Ok(SpecWorkflowSnapshot {
        project: project.to_string(),
        spec_id: spec_id.to_string(),
        feature,
        timestamp_prefix,
        path: spec_dir.to_path_buf(),
        meta,
        derived_phase,
        task_list: task_summary,
        files: SpecFileContents {
            spec_md,
            task_list_md,
            notes_md,
        },
    })
}

pub fn context_files_for_spec(spec_dir: &Path) -> SpecContextFiles {
    SpecContextFiles {
        spec_md: spec_dir.join("spec.md"),
        task_list_md: spec_dir.join("task-list.md"),
        notes_md: spec_dir.join("notes.md"),
        meta_json: spec_dir.join(crate::core::spec_meta::META_FILE),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecStatusJson {
    pub project: String,
    pub spec_id: String,
    pub path: PathBuf,
    pub derived_phase: DerivedPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_hint: Option<SpecPhaseHint>,
    pub task_list: TaskListSummary,
    pub context_files: SpecContextFiles,
    pub state: String,
    pub instruction: String,
}

pub fn build_spec_status(project: &str, spec_id: &str) -> Result<SpecStatusJson> {
    let dir = paths::spec_dir_path(project, spec_id)?;
    if !dir.is_dir() {
        anyhow::bail!("Spec {:?} not found for project {:?}", spec_id, project);
    }
    let snap = load_spec_snapshot(project, &dir, spec_id)?;
    let context_files = context_files_for_spec(&dir);
    let (state, instruction) = status_message(&snap);
    Ok(SpecStatusJson {
        project: project.to_string(),
        spec_id: spec_id.to_string(),
        path: dir,
        derived_phase: snap.derived_phase,
        phase_hint: snap.meta.phase_hint,
        task_list: snap.task_list,
        context_files,
        state: state.to_string(),
        instruction,
    })
}

fn status_message(snap: &SpecWorkflowSnapshot) -> (&'static str, String) {
    match snap.derived_phase {
        DerivedPhase::EmptyScaffold => (
            "empty_scaffold",
            "Fill spec.md with intent, then add tasks to task-list.md using markdown checkboxes.".to_string(),
        ),
        DerivedPhase::Drafting => (
            "drafting",
            "Continue drafting spec.md and break work into task-list.md checkboxes when ready.".to_string(),
        ),
        DerivedPhase::ReadyForImplementation => (
            "ready_for_implementation",
            "Begin implementation work; track progress with task-list.md checkboxes.".to_string(),
        ),
        DerivedPhase::Implementing => (
            "implementing",
            format!(
                "Implementation in progress: {} of {} tasks complete.",
                snap.task_list.complete, snap.task_list.total
            ),
        ),
        DerivedPhase::CompletedPendingCollapse => (
            "completed_pending_collapse",
            "All tasks are checked. Run `foundry spec collapse prepare` then write summary.md, then `foundry spec collapse finalize --confirm`.".to_string(),
        ),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecInstructionsApplyJson {
    pub change_name: String,
    pub schema_name: String,
    pub context_files: SpecContextFiles,
    pub progress: ProgressBlock,
    pub tasks: Vec<TaskJson>,
    pub state: String,
    pub instruction: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressBlock {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskJson {
    pub id: String,
    pub description: String,
    pub done: bool,
}

pub fn build_instructions_apply(project: &str, spec_id: &str) -> Result<SpecInstructionsApplyJson> {
    let dir = paths::spec_dir_path(project, spec_id)?;
    if !dir.is_dir() {
        anyhow::bail!("Spec {:?} not found for project {:?}", spec_id, project);
    }
    let snap = load_spec_snapshot(project, &dir, spec_id)?;
    let context_files = context_files_for_spec(&dir);
    let total = snap.task_list.total;
    let complete = snap.task_list.complete;
    let remaining = total.saturating_sub(complete);
    let tasks: Vec<TaskJson> = snap
        .task_list
        .items
        .iter()
        .enumerate()
        .map(|(i, t)| TaskJson {
            id: format!("{}", i + 1),
            description: t.description.clone(),
            done: t.done,
        })
        .collect();

    let (state, instruction) = apply_state_and_instruction(&snap, remaining);

    Ok(SpecInstructionsApplyJson {
        change_name: format!("{project}/{spec_id}"),
        schema_name: "foundry-spec".to_string(),
        context_files,
        progress: ProgressBlock {
            total,
            complete,
            remaining,
        },
        tasks,
        state: state.to_string(),
        instruction,
    })
}

fn apply_state_and_instruction(
    snap: &SpecWorkflowSnapshot,
    remaining: usize,
) -> (&'static str, String) {
    match snap.derived_phase {
        DerivedPhase::EmptyScaffold | DerivedPhase::Drafting | DerivedPhase::ReadyForImplementation => (
            "blocked",
            "Complete spec.md and add actionable tasks to task-list.md before implementation.".to_string(),
        ),
        DerivedPhase::Implementing => (
            "in_progress",
            format!(
                "Work the next unchecked task in task-list.md. Progress: {} complete, {} remaining.",
                snap.task_list.complete, remaining
            ),
        ),
        DerivedPhase::CompletedPendingCollapse => (
            "all_done",
            "All tasks complete. Use `foundry spec instructions collapse --json` and then collapse prepare/finalize."
                .to_string(),
        ),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecInstructionsCollapseJson {
    pub project: String,
    pub spec_id: String,
    pub state: String,
    pub needs_prepare: bool,
    pub summary_path: PathBuf,
    pub archive_dir: PathBuf,
    pub active_spec_dir: PathBuf,
    pub context_files: SpecContextFiles,
    pub instruction: String,
}

pub fn build_instructions_collapse(
    project: &str,
    spec_id: &str,
) -> Result<SpecInstructionsCollapseJson> {
    let active = paths::spec_dir_path(project, spec_id)?;
    if !active.is_dir() {
        anyhow::bail!(
            "Active spec {:?} not found for project {:?}",
            spec_id,
            project
        );
    }
    let snap = load_spec_snapshot(project, &active, spec_id)?;
    if snap.derived_phase != DerivedPhase::CompletedPendingCollapse {
        anyhow::bail!(
            "Spec is not ready to collapse (phase {:?}). Complete all task-list checkboxes first.",
            snap.derived_phase
        );
    }
    let completed = paths::completed_spec_dir(project, spec_id)?;
    let needs_prepare = !completed.is_dir();
    let summary_path = completed.join(crate::core::completed::SUMMARY_FILE);
    let archive_dir = completed.join(crate::core::completed::ARCHIVE_DIR);
    let context_files = context_files_for_spec(&active);

    let instruction = if needs_prepare {
        "Run `foundry spec collapse prepare` to create the completed record directory and an empty summary.md. Then edit summary.md with a concise completed-work narrative (what shipped, scope, key decisions). Finally run `foundry spec collapse finalize --confirm`."
            .to_string()
    } else {
        "Edit summary.md if needed (it must be non-empty), then run `foundry spec collapse finalize --confirm` to move active spec files into archive/ and remove the active spec directory."
            .to_string()
    };

    Ok(SpecInstructionsCollapseJson {
        project: project.to_string(),
        spec_id: spec_id.to_string(),
        state: if needs_prepare {
            "needs_prepare".to_string()
        } else {
            "ready_to_finalize".to_string()
        },
        needs_prepare,
        summary_path,
        archive_dir,
        active_spec_dir: active,
        context_files,
        instruction,
    })
}
