use crate::core::backends::SpecContentStore;
use crate::types::edit_commands::{
    EditCommand, EditCommandError, EditCommandName, EditCommandTarget, EditSelector,
    FileUpdateSummary, SelectorCandidate, TaskStatus,
};
use crate::types::spec::SpecFileType;
use anyhow::{Result, anyhow};

pub struct EditEngine;

pub struct EditCommandsResult {
    pub applied_count: usize,
    pub skipped_idempotent_count: usize,
    pub file_updates: Vec<FileUpdateSummary>,
    pub errors: Vec<EditCommandError>,
    pub next_steps: Vec<String>,
    pub workflow_hints: Vec<String>,
    pub preview_diff: Option<String>,
}

impl EditEngine {
    pub fn apply_edit_commands(
        project_name: &str,
        spec_name: &str,
        commands: &[EditCommand],
    ) -> Result<EditCommandsResult> {
        // Legacy method - keep for backward compatibility during transition
        // This will be deprecated once all callers use the new method
        use crate::core::{filesystem, spec};
        
        if commands.is_empty() {
            return Err(anyhow!("commands must be a non-empty array"));
        }

        // Load current contents using direct filesystem calls (legacy)
        let mut spec_content =
            read_file_or_empty(&spec::get_spec_file_path(project_name, spec_name)?)?
        ;
        let mut tasks_content =
            read_file_or_empty(&spec::get_task_list_file_path(project_name, spec_name)?)?
        ;
        let mut notes_content =
            read_file_or_empty(&spec::get_notes_file_path(project_name, spec_name)?)?
        ;

        let result = Self::process_edit_commands(commands, &mut spec_content, &mut tasks_content, &mut notes_content)?;

        // Write back only if modified using direct filesystem calls (legacy)
        if is_modified(
            &spec::get_spec_file_path(project_name, spec_name)?,
            &spec_content,
        )? {
            filesystem::write_file_atomic(
                &spec::get_spec_file_path(project_name, spec_name)?,
                &spec_content,
            )?;
        }
        if is_modified(
            &spec::get_task_list_file_path(project_name, spec_name)?,
            &tasks_content,
        )? {
            filesystem::write_file_atomic(
                &spec::get_task_list_file_path(project_name, spec_name)?,
                &tasks_content,
            )?;
        }
        if is_modified(
            &spec::get_notes_file_path(project_name, spec_name)?,
            &notes_content,
        )? {
            filesystem::write_file_atomic(
                &spec::get_notes_file_path(project_name, spec_name)?,
                &notes_content,
            )?;
        }

        Ok(result)
    }

    pub async fn apply_edit_commands_with_store<S: SpecContentStore>(
        project_name: &str,
        spec_name: &str,
        commands: &[EditCommand],
        store: &S,
    ) -> Result<EditCommandsResult> {
        if commands.is_empty() {
            return Err(anyhow!("commands must be a non-empty array"));
        }

        // Load current contents via SpecContentStore
        let mut spec_content = store
            .read_spec_file(project_name, spec_name, SpecFileType::Spec)
            .await
            .unwrap_or_else(|_| String::new());
        let mut tasks_content = store
            .read_spec_file(project_name, spec_name, SpecFileType::TaskList)
            .await
            .unwrap_or_else(|_| String::new());
        let mut notes_content = store
            .read_spec_file(project_name, spec_name, SpecFileType::Notes)
            .await
            .unwrap_or_else(|_| String::new());

        let result = Self::process_edit_commands(commands, &mut spec_content, &mut tasks_content, &mut notes_content)?;

        // Write back only if modified via SpecContentStore
        if store
            .is_file_modified(project_name, spec_name, SpecFileType::Spec, &spec_content)
            .await?
        {
            store
                .write_spec_file(project_name, spec_name, SpecFileType::Spec, &spec_content)
                .await?;
        }
        if store
            .is_file_modified(project_name, spec_name, SpecFileType::TaskList, &tasks_content)
            .await?
        {
            store
                .write_spec_file(project_name, spec_name, SpecFileType::TaskList, &tasks_content)
                .await?;
        }
        if store
            .is_file_modified(project_name, spec_name, SpecFileType::Notes, &notes_content)
            .await?
        {
            store
                .write_spec_file(project_name, spec_name, SpecFileType::Notes, &notes_content)
                .await?;
        }

        Ok(result)
    }

    fn process_edit_commands(
        commands: &[EditCommand],
        spec_content: &mut String,
        tasks_content: &mut String,
        notes_content: &mut String,
    ) -> Result<EditCommandsResult> {
        let mut applied_total = 0usize;
        let mut skipped_total = 0usize;
        let mut file_updates: Vec<FileUpdateSummary> = vec![
            FileUpdateSummary {
                target: EditCommandTarget::Spec,
                applied: 0,
                skipped_idempotent: 0,
                hints: None,
            },
            FileUpdateSummary {
                target: EditCommandTarget::Tasks,
                applied: 0,
                skipped_idempotent: 0,
                hints: None,
            },
            FileUpdateSummary {
                target: EditCommandTarget::Notes,
                applied: 0,
                skipped_idempotent: 0,
                hints: None,
            },
        ];
        let mut errors: Vec<EditCommandError> = Vec::new();

        for (idx, command) in commands.iter().enumerate() {
            match (&command.target, &command.command, &command.selector) {
                (
                    EditCommandTarget::Tasks,
                    EditCommandName::SetTaskStatus,
                    EditSelector::TaskText { value },
                ) => {
                    let status = command
                        .status
                        .clone()
                        .ok_or_else(|| anyhow!("status is required for set_task_status"))?;
                    match set_task_status(&tasks_content, value, status) {
                        Ok(EditOutcome {
                            content,
                            applied,
                            skipped,
                        }) => {
                            *tasks_content = content;
                            update_counts(
                                file_updates.as_mut_slice(),
                                EditCommandTarget::Tasks,
                                applied,
                                skipped,
                            );
                            applied_total += applied;
                            skipped_total += skipped;
                        }
                        Err(EditAmbiguity { candidates }) => errors.push(EditCommandError {
                            target: EditCommandTarget::Tasks,
                            command_index: idx,
                            message: "Ambiguous or no matching task_text selector".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Tasks,
                    EditCommandName::UpsertTask,
                    EditSelector::TaskText { value },
                ) => {
                    let content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for upsert_task"))?;
                    match upsert_task(&tasks_content, value, &content) {
                        Ok(EditOutcome {
                            content,
                            applied,
                            skipped,
                        }) => {
                            *tasks_content = content;
                            update_counts(
                                file_updates.as_mut_slice(),
                                EditCommandTarget::Tasks,
                                applied,
                                skipped,
                            );
                            applied_total += applied;
                            skipped_total += skipped;
                        }
                        Err(EditAmbiguity { candidates }) => errors.push(EditCommandError {
                            target: EditCommandTarget::Tasks,
                            command_index: idx,
                            message: "Ambiguous task_text selector".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::AppendToSection,
                    EditSelector::Section { value },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::AppendToSection,
                    EditSelector::Section { value },
                ) => {
                    let content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for append_to_section"))?;
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match append_to_section(current, value, &content) {
                        Ok(EditOutcome {
                            content: new_content,
                            applied,
                            skipped,
                        }) => {
                            if is_spec {
                                *spec_content = new_content;
                            } else {
                                *notes_content = new_content;
                            }
                            let target = if is_spec {
                                EditCommandTarget::Spec
                            } else {
                                EditCommandTarget::Notes
                            };
                            update_counts(file_updates.as_mut_slice(), target, applied, skipped);
                            applied_total += applied;
                            skipped_total += skipped;
                        }
                        Err(EditAmbiguity { candidates }) => errors.push(EditCommandError {
                            target: if is_spec {
                                EditCommandTarget::Spec
                            } else {
                                EditCommandTarget::Notes
                            },
                            command_index: idx,
                            message: "Section not found or ambiguous".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (EditCommandTarget::Tasks, EditCommandName::AppendToSection, _) => {
                    errors.push(EditCommandError {
                        target: EditCommandTarget::Tasks,
                        command_index: idx,
                        message: "append_to_section is invalid for tasks".to_string(),
                        candidates: None,
                    })
                }
                _ => errors.push(EditCommandError {
                    target: command.target.clone(),
                    command_index: idx,
                    message: "Unsupported command/selector combination".to_string(),
                    candidates: None,
                }),
            }
        }

        let active_file_updates: Vec<FileUpdateSummary> = file_updates
            .into_iter()
            .filter(|fu| fu.applied > 0 || fu.skipped_idempotent > 0)
            .collect();

        Ok(EditCommandsResult {
            applied_count: applied_total,
            skipped_idempotent_count: skipped_total,
            file_updates: active_file_updates,
            errors,
            next_steps: vec!["Load updated spec with load_spec to verify changes".to_string()],
            workflow_hints: vec![
                "Always copy exact task text and headers from load_spec before editing".to_string(),
            ],
            preview_diff: None,
        })
    }
}

struct EditOutcome {
    content: String,
    applied: usize,
    skipped: usize,
}

struct EditAmbiguity {
    candidates: Vec<SelectorCandidate>,
}

fn read_file_or_empty(path: &std::path::Path) -> Result<String> {
    use crate::core::filesystem;
    filesystem::read_file(path).or_else(|_| Ok(String::new()))
}

fn is_modified(path: &std::path::Path, new_content: &str) -> Result<bool> {
    use crate::core::filesystem;
    filesystem::read_file(path).map_or_else(
        |_| Ok(!new_content.is_empty()),
        |existing| Ok(existing != new_content),
    )
}

fn normalize_task_text(line: &str) -> String {
    let text = line.trim_start();
    let text = text
        .strip_prefix("- [ ] ")
        .or_else(|| text.strip_prefix("- [x] "))
        .unwrap_or(text)
        .trim();

    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized
        .strip_suffix('.')
        .unwrap_or(&normalized)
        .to_string()
}

fn set_task_status(
    current: &str,
    task_text: &str,
    status: TaskStatus,
) -> Result<EditOutcome, EditAmbiguity> {
    let desired_prefix = match status {
        TaskStatus::Done => "- [x] ",
        TaskStatus::Todo => "- [ ] ",
    };
    let wanted_norm = normalize_task_text(task_text);
    let mut lines: Vec<String> = current.lines().map(|l| l.to_string()).collect();
    let match_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if line.trim_start().starts_with("- [") && normalize_task_text(line) == wanted_norm {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    if match_indices.is_empty() {
        return Err(EditAmbiguity {
            candidates: task_candidates(current),
        });
    }
    if match_indices.len() > 1 {
        return Err(EditAmbiguity {
            candidates: task_candidates(current),
        });
    }
    let idx = match_indices[0];
    let already = lines[idx].trim_start().starts_with(desired_prefix);
    if already {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }
    let normalized = normalize_task_text(&lines[idx]);
    lines[idx] = format!("{}{}", desired_prefix, normalized);
    Ok(EditOutcome {
        content: lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn upsert_task(
    current: &str,
    task_text: &str,
    new_task_line: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted_norm = normalize_task_text(task_text);
    let matches = current
        .lines()
        .filter(|line| normalize_task_text(line) == wanted_norm)
        .count();
    if matches > 1 {
        return Err(EditAmbiguity {
            candidates: task_candidates(current),
        });
    }
    if matches == 1 {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }
    let mut content = current.to_string();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(new_task_line);
    Ok(EditOutcome {
        content,
        applied: 1,
        skipped: 0,
    })
}

fn append_to_section(
    current: &str,
    header: &str,
    content_to_append: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted = header.trim().to_lowercase();
    let lines: Vec<&str> = current.lines().collect();
    let header_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if is_header_line(l) && l.trim().to_lowercase() == wanted {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    if header_indices.is_empty() {
        return Err(EditAmbiguity {
            candidates: header_candidates(current),
        });
    }
    if header_indices.len() > 1 {
        return Err(EditAmbiguity {
            candidates: header_candidates(current),
        });
    }
    let start_idx = header_indices[0];
    let mut end_idx = lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find(|(_, line)| is_header_line(line))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());
    let section_body = lines[(start_idx + 1)..end_idx].join("\n");
    if section_body.contains(content_to_append) {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    if end_idx > 0 && !new_lines[end_idx - 1].is_empty() {
        new_lines.insert(end_idx, String::new());
        end_idx += 1;
    }
    new_lines.insert(end_idx, content_to_append.to_string());
    Ok(EditOutcome {
        content: new_lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn is_header_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

fn header_candidates(current: &str) -> Vec<SelectorCandidate> {
    current
        .lines()
        .enumerate()
        .filter(|(_, l)| is_header_line(l))
        .map(|(i, l)| SelectorCandidate {
            selector_suggestion: EditSelector::Section {
                value: l.trim().to_string(),
            },
            preview: preview_excerpt(current, i),
        })
        .collect()
}

fn task_candidates(current: &str) -> Vec<SelectorCandidate> {
    current
        .lines()
        .enumerate()
        .filter(|(_, l)| l.trim_start().starts_with("- ["))
        .map(|(i, l)| SelectorCandidate {
            selector_suggestion: EditSelector::TaskText {
                value: normalize_task_text(l),
            },
            preview: preview_excerpt(current, i),
        })
        .collect()
}

fn update_counts(
    file_updates: &mut [FileUpdateSummary],
    target: EditCommandTarget,
    applied: usize,
    skipped: usize,
) {
    if let Some(update) = file_updates
        .iter_mut()
        .find(|update| update.target == target)
    {
        update.applied += applied;
        update.skipped_idempotent += skipped;
    }
}

fn preview_excerpt(all: &str, idx: usize) -> String {
    let lines: Vec<&str> = all.lines().collect();
    let start = idx.saturating_sub(2);
    let end = (idx + 3).min(lines.len());
    lines[start..end].join("\n")
}
