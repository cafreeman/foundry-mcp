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

        let result = Self::process_edit_commands(
            commands,
            &mut spec_content,
            &mut tasks_content,
            &mut notes_content,
        )?;

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
            .is_file_modified(
                project_name,
                spec_name,
                SpecFileType::TaskList,
                &tasks_content,
            )
            .await?
        {
            store
                .write_spec_file(
                    project_name,
                    spec_name,
                    SpecFileType::TaskList,
                    &tasks_content,
                )
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
                    EditSelector::TaskText { value, .. },
                ) => {
                    let status = command
                        .status
                        .clone()
                        .ok_or_else(|| anyhow!("status is required for set_task_status"))?;
                    match set_task_status(tasks_content, value, status) {
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
                    EditSelector::TaskText { value, .. },
                ) => {
                    let content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for upsert_task"))?;
                    match upsert_task(tasks_content, value, &content) {
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
                (
                    EditCommandTarget::Tasks,
                    EditCommandName::RemoveListItem,
                    EditSelector::TaskText { value, .. },
                ) => match remove_list_item(tasks_content, value) {
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
                        message: "List item not found or ambiguous".to_string(),
                        candidates: Some(candidates),
                    }),
                },
                (
                    EditCommandTarget::Spec,
                    EditCommandName::RemoveListItem,
                    EditSelector::TaskText { value, .. },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::RemoveListItem,
                    EditSelector::TaskText { value, .. },
                ) => {
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match remove_list_item(current, value) {
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
                            message: "List item not found or ambiguous".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::RemoveFromSection,
                    EditSelector::Section { value },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::RemoveFromSection,
                    EditSelector::Section { value },
                ) => {
                    let content_to_remove = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for remove_from_section"))?;
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match remove_from_section(current, value, &content_to_remove) {
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
                            message: "Section not found or content not found in section"
                                .to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::RemoveSection,
                    EditSelector::Section { value },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::RemoveSection,
                    EditSelector::Section { value },
                ) => {
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match remove_section(current, value) {
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
                (
                    EditCommandTarget::Tasks,
                    EditCommandName::ReplaceListItem,
                    EditSelector::TaskText { value, .. },
                ) => {
                    let new_content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for replace_list_item"))?;
                    match replace_list_item(tasks_content, value, &new_content) {
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
                            message: "List item not found or ambiguous".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::ReplaceListItem,
                    EditSelector::TaskText { value, .. },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::ReplaceListItem,
                    EditSelector::TaskText { value, .. },
                ) => {
                    let new_content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for replace_list_item"))?;
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match replace_list_item(current, value, &new_content) {
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
                            message: "List item not found or ambiguous".to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::ReplaceInSection,
                    EditSelector::TextInSection { section, text },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::ReplaceInSection,
                    EditSelector::TextInSection { section, text },
                ) => {
                    let new_content = command
                        .content
                        .clone()
                        .ok_or_else(|| anyhow!("content is required for replace_in_section"))?;
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match replace_in_section(current, section, text, &new_content) {
                        Ok(EditOutcome {
                            content: updated_content,
                            applied,
                            skipped,
                        }) => {
                            if is_spec {
                                *spec_content = updated_content;
                            } else {
                                *notes_content = updated_content;
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
                            message: "Section not found or old text not found in section"
                                .to_string(),
                            candidates: Some(candidates),
                        }),
                    }
                }
                (
                    EditCommandTarget::Spec,
                    EditCommandName::ReplaceSectionContent,
                    EditSelector::Section { value },
                )
                | (
                    EditCommandTarget::Notes,
                    EditCommandName::ReplaceSectionContent,
                    EditSelector::Section { value },
                ) => {
                    let new_content = command.content.clone().ok_or_else(|| {
                        anyhow!("content is required for replace_section_content")
                    })?;
                    let is_spec = matches!(command.target, EditCommandTarget::Spec);
                    let current = if is_spec {
                        &spec_content
                    } else {
                        &notes_content
                    };
                    match replace_section_content(current, value, &new_content) {
                        Ok(EditOutcome {
                            content: updated_content,
                            applied,
                            skipped,
                        }) => {
                            if is_spec {
                                *spec_content = updated_content;
                            } else {
                                *notes_content = updated_content;
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

fn normalize_task_text(line: &str) -> String {
    let text = line.trim_start();
    let text = text
        .strip_prefix("- [ ] ")
        .or_else(|| text.strip_prefix("- [x] "))
        .unwrap_or(text)
        .trim();
    // Also normalize common list markers and numbered list prefixes so callers
    // can match without including them when the remainder is unique.
    let text = text.strip_prefix("- ").map_or_else(
        || {
            text.strip_prefix("* ")
                .map_or_else(|| text, |stripped| stripped.trim_start())
        },
        |stripped| stripped.trim_start(),
    );

    // Strip numbered list prefix like "1. ", "23. ", preserving the remainder
    // for matching purposes only. This improves selector ergonomics while
    // replacement functions still preserve original prefixes/styles.
    let text = {
        let mut chars = text.chars().peekable();
        let mut _idx = 0usize;
        let mut saw_digit = false;
        while let Some(c) = chars.peek() {
            if c.is_ascii_digit() {
                saw_digit = true;
                _idx += 1;
                chars.next();
            } else {
                break;
            }
        }
        if saw_digit {
            if let Some('.') = chars.peek() {
                // Consume the dot
                chars.next();
                // Consume a single following space if present
                if let Some(' ') = chars.peek() {
                    chars.next();
                }
                // Remainder
                chars.collect::<String>().trim_start().to_string()
            } else {
                // Not a numbered prefix, restore original
                text.to_string()
            }
        } else {
            text.to_string()
        }
    };

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
                section_context: None,
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

fn remove_list_item(current: &str, item_text: &str) -> Result<EditOutcome, EditAmbiguity> {
    let wanted_norm = normalize_task_text(item_text);
    let mut lines: Vec<String> = current.lines().map(|l| l.to_string()).collect();

    // Find matching list items (tasks or regular list items)
    let match_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let normalized = normalize_task_text(line);
            if normalized == wanted_norm && is_list_item(line) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if match_indices.is_empty() {
        return Err(EditAmbiguity {
            candidates: list_item_candidates(current),
        });
    }
    if match_indices.len() > 1 {
        return Err(EditAmbiguity {
            candidates: list_item_candidates(current),
        });
    }

    let idx = match_indices[0];
    lines.remove(idx);

    Ok(EditOutcome {
        content: lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn remove_from_section(
    current: &str,
    section_header: &str,
    content_to_remove: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted = section_header.trim().to_lowercase();
    let lines: Vec<&str> = current.lines().collect();

    // Find the target section
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
    let end_idx = lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find(|(_, line)| is_header_line(line))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    // Check if content exists in the section
    let section_content = lines[(start_idx + 1)..end_idx].join("\n");
    if !section_content.contains(content_to_remove) {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }

    // Remove the content from the section
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let section_lines = new_lines[(start_idx + 1)..end_idx].to_vec();
    let updated_section = section_lines
        .join("\n")
        .replace(content_to_remove, "")
        .trim()
        .to_string();

    // Replace section content
    new_lines.drain((start_idx + 1)..end_idx);
    if !updated_section.is_empty() {
        let replacement_lines: Vec<String> =
            updated_section.lines().map(|s| s.to_string()).collect();
        for (offset, line) in replacement_lines.iter().enumerate() {
            new_lines.insert(start_idx + 1 + offset, line.clone());
        }
    }

    Ok(EditOutcome {
        content: new_lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn remove_section(current: &str, section_header: &str) -> Result<EditOutcome, EditAmbiguity> {
    let wanted = section_header.trim().to_lowercase();
    let lines: Vec<&str> = current.lines().collect();

    // Find the target section
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
    let end_idx = lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find(|(_, line)| is_header_line(line))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    // Remove the entire section (header + content)
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    new_lines.drain(start_idx..end_idx);

    Ok(EditOutcome {
        content: new_lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
}

fn list_item_candidates(current: &str) -> Vec<SelectorCandidate> {
    current
        .lines()
        .enumerate()
        .filter(|(_, l)| is_list_item(l))
        .map(|(i, l)| SelectorCandidate {
            selector_suggestion: EditSelector::TaskText {
                value: normalize_task_text(l),
                section_context: None,
            },
            preview: preview_excerpt(current, i),
        })
        .collect()
}

fn replace_list_item(
    current: &str,
    old_item_text: &str,
    new_item_text: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted_norm = normalize_task_text(old_item_text);
    let mut lines: Vec<String> = current.lines().map(|l| l.to_string()).collect();

    // Find matching list items
    let match_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let normalized = normalize_task_text(line);
            if normalized == wanted_norm && is_list_item(line) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if match_indices.is_empty() {
        return Err(EditAmbiguity {
            candidates: list_item_candidates(current),
        });
    }
    if match_indices.len() > 1 {
        return Err(EditAmbiguity {
            candidates: list_item_candidates(current),
        });
    }

    let idx = match_indices[0];

    // Check if already matches (idempotent)
    let current_normalized = normalize_task_text(&lines[idx]);
    let new_normalized = normalize_task_text(new_item_text);
    if current_normalized == new_normalized {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }

    // Preserve the prefix (indentation and list marker style)
    let original_line = &lines[idx];
    let trimmed_start = original_line.trim_start();
    let indent = &original_line[..original_line.len() - trimmed_start.len()];

    // Determine the list marker style from the original
    let new_line = if trimmed_start.starts_with("- [x] ") || trimmed_start.starts_with("- [ ] ") {
        // Task list format - preserve completion status
        let status_marker = if trimmed_start.starts_with("- [x] ") {
            "- [x] "
        } else {
            "- [ ] "
        };
        format!(
            "{}{}{}",
            indent,
            status_marker,
            normalize_task_text(new_item_text)
        )
    } else if trimmed_start.starts_with("- ") {
        format!("{}{}{}", indent, "- ", new_item_text.trim())
    } else if trimmed_start.starts_with("* ") {
        format!("{}{}{}", indent, "* ", new_item_text.trim())
    } else if trimmed_start
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
    {
        // Numbered list - try to preserve numbering
        let num_part = trimmed_start
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>();
        format!("{}{} {}", indent, num_part, new_item_text.trim())
    } else {
        // Fallback
        format!("{}{}", indent, new_item_text.trim())
    };

    lines[idx] = new_line;

    Ok(EditOutcome {
        content: lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn replace_in_section(
    current: &str,
    section_header: &str,
    old_text: &str,
    new_text: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted = section_header.trim().to_lowercase();
    let lines: Vec<&str> = current.lines().collect();

    // Find the target section
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
    let end_idx = lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find(|(_, line)| is_header_line(line))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    // Check if old text exists in the section
    let section_content = lines[(start_idx + 1)..end_idx].join("\n");
    if !section_content.contains(old_text) {
        return Err(EditAmbiguity {
            candidates: header_candidates(current),
        });
    }

    // Check if already replaced (idempotent)
    if section_content.contains(new_text) && !section_content.contains(old_text) {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }

    // Replace the content in the section
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let updated_section = section_content.replace(old_text, new_text);

    // Replace section content
    new_lines.drain((start_idx + 1)..end_idx);
    let replacement_lines: Vec<String> = updated_section.lines().map(|s| s.to_string()).collect();
    for (offset, line) in replacement_lines.iter().enumerate() {
        new_lines.insert(start_idx + 1 + offset, line.clone());
    }

    Ok(EditOutcome {
        content: new_lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn replace_section_content(
    current: &str,
    section_header: &str,
    new_content: &str,
) -> Result<EditOutcome, EditAmbiguity> {
    let wanted = section_header.trim().to_lowercase();
    let lines: Vec<&str> = current.lines().collect();

    // Find the target section
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
    let end_idx = lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find(|(_, line)| is_header_line(line))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    // Check if already matches (idempotent)
    let current_section_content = lines[(start_idx + 1)..end_idx].join("\n");
    if current_section_content.trim() == new_content.trim() {
        return Ok(EditOutcome {
            content: current.to_string(),
            applied: 0,
            skipped: 1,
        });
    }

    // Replace all content within the section, keeping the header
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    // Remove old section content
    new_lines.drain((start_idx + 1)..end_idx);

    // Insert new content
    if !new_content.trim().is_empty() {
        let replacement_lines: Vec<String> = new_content.lines().map(|s| s.to_string()).collect();
        for (offset, line) in replacement_lines.iter().enumerate() {
            new_lines.insert(start_idx + 1 + offset, line.clone());
        }
    }

    Ok(EditOutcome {
        content: new_lines.join("\n"),
        applied: 1,
        skipped: 0,
    })
}

fn preview_excerpt(all: &str, idx: usize) -> String {
    let lines: Vec<&str> = all.lines().collect();
    let start = idx.saturating_sub(2);
    let end = (idx + 3).min(lines.len());
    lines[start..end].join("\n")
}
