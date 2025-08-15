//! Formatting utilities for rendering project content to markdown

use crate::models::{Project, Specification, Task, TaskList, TechStack, Vision};
use chrono::{DateTime, Utc};

/// Format a timestamp for display
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a task status for display
pub fn format_task_status(status: &crate::models::TaskStatus) -> String {
    match status {
        crate::models::TaskStatus::Todo => "⏳ Todo".to_string(),
        crate::models::TaskStatus::InProgress => "🔄 In Progress".to_string(),
        crate::models::TaskStatus::Completed => "✅ Completed".to_string(),
        crate::models::TaskStatus::Blocked => "🚫 Blocked".to_string(),
    }
}

/// Format a task priority for display
pub fn format_task_priority(priority: &crate::models::TaskPriority) -> String {
    match priority {
        crate::models::TaskPriority::Low => "🟢 Low".to_string(),
        crate::models::TaskPriority::Medium => "🟡 Medium".to_string(),
        crate::models::TaskPriority::High => "🟠 High".to_string(),
        crate::models::TaskPriority::Critical => "🔴 Critical".to_string(),
    }
}

/// Format a specification status for display
pub fn format_spec_status(status: &crate::models::SpecStatus) -> String {
    match status {
        crate::models::SpecStatus::Draft => "📝 Draft".to_string(),
        crate::models::SpecStatus::InProgress => "🔄 In Progress".to_string(),
        crate::models::SpecStatus::Completed => "✅ Completed".to_string(),
        crate::models::SpecStatus::OnHold => "⏸️ On Hold".to_string(),
    }
}

/// Render a tech stack to markdown
pub fn render_tech_stack(tech_stack: &TechStack) -> String {
    let sections = [
        ("Programming Languages", &tech_stack.languages),
        ("Frameworks & Libraries", &tech_stack.frameworks),
        ("Databases & Storage", &tech_stack.databases),
        ("Development Tools", &tech_stack.tools),
        ("Deployment & Infrastructure", &tech_stack.deployment),
    ];

    let content = sections
        .iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(title, items)| {
            let items_list = items
                .iter()
                .map(|item| format!("- {}\n", item))
                .collect::<String>();
            format!("## {}\n{}\n", title, items_list)
        })
        .collect::<String>();

    format!("# Technology Stack\n\n{}", content)
}

/// Render a vision document to markdown
pub fn render_vision(vision: &Vision) -> String {
    let goals_section = if vision.goals.is_empty() {
        String::new()
    } else {
        let goals_list = vision
            .goals
            .iter()
            .enumerate()
            .map(|(i, goal)| format!("{}. {}\n", i + 1, goal))
            .collect::<String>();
        format!("## Goals\n{}\n", goals_list)
    };

    let target_users_section = if vision.target_users.is_empty() {
        String::new()
    } else {
        let users_list = vision
            .target_users
            .iter()
            .map(|user| format!("- {}\n", user))
            .collect::<String>();
        format!("## Target Users\n{}\n", users_list)
    };

    let success_criteria_section = if vision.success_criteria.is_empty() {
        String::new()
    } else {
        let criteria_list = vision
            .success_criteria
            .iter()
            .enumerate()
            .map(|(i, criterion)| format!("{}. {}\n", i + 1, criterion))
            .collect::<String>();
        format!("## Success Criteria\n{}\n", criteria_list)
    };

    format!(
        "# Project Vision\n\n## Overview\n{}\n\n{}{}{}",
        vision.overview, goals_section, target_users_section, success_criteria_section
    )
}

/// Render a task list to markdown
pub fn render_task_list(task_list: &TaskList) -> String {
    if task_list.tasks.is_empty() {
        return "# Task List\n\nNo tasks defined yet.\n\n".to_string();
    }

    // Group tasks by status using functional approach
    let (todo_tasks, in_progress_tasks, completed_tasks, blocked_tasks) =
        task_list.tasks.iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut todo, mut in_progress, mut completed, mut blocked), task| {
                match task.status {
                    crate::models::TaskStatus::Todo => todo.push(task),
                    crate::models::TaskStatus::InProgress => in_progress.push(task),
                    crate::models::TaskStatus::Completed => completed.push(task),
                    crate::models::TaskStatus::Blocked => blocked.push(task),
                }
                (todo, in_progress, completed, blocked)
            },
        );

    let status_groups = [
        ("⏳ Todo", &todo_tasks),
        ("🔄 In Progress", &in_progress_tasks),
        ("🚫 Blocked", &blocked_tasks),
        ("✅ Completed", &completed_tasks),
    ];

    let tasks_content = status_groups
        .iter()
        .filter(|(_, tasks)| !tasks.is_empty())
        .map(|(status, tasks)| {
            let tasks_list = tasks
                .iter()
                .map(|task| render_task(task))
                .collect::<String>();
            format!("## {}\n{}", status, tasks_list)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "# Task List\n\n{}\n\n*Last updated: {}*\n",
        tasks_content,
        format_timestamp(&task_list.last_updated)
    )
}

/// Render a single task to markdown
fn render_task(task: &Task) -> String {
    let mut sections = Vec::new();

    sections.push(format!("### {}\n", task.title));
    sections.push(format!(
        "**Status:** {} | **Priority:** {}\n",
        format_task_status(&task.status),
        format_task_priority(&task.priority)
    ));

    if !task.description.is_empty() {
        sections.push(format!("**Description:** {}\n", task.description));
    }

    if !task.dependencies.is_empty() {
        sections.push(format!(
            "**Dependencies:** {}\n",
            task.dependencies.join(", ")
        ));
    }

    sections.push(format!(
        "**Created:** {} | **Updated:** {}\n",
        format_timestamp(&task.created_at),
        format_timestamp(&task.updated_at)
    ));

    sections.push('\n'.to_string());

    sections.join("")
}

/// Render notes to markdown
pub fn render_notes(notes: &[crate::models::Note]) -> String {
    if notes.is_empty() {
        return "# Notes\n\nNo notes yet.\n\n".to_string();
    }

    // Group notes by category using functional approach
    let notes_by_category: std::collections::HashMap<_, Vec<_>> =
        notes
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, note| {
                acc.entry(&note.category).or_default().push(note);
                acc
            });

    let notes_content = notes_by_category
        .iter()
        .map(|(category, category_notes)| {
            let category_notes_content = category_notes
                .iter()
                .map(|note| {
                    format!(
                        "### Note: {}\n{}\n\n*Created: {}*\n\n",
                        note.id,
                        note.content,
                        format_timestamp(&note.created_at)
                    )
                })
                .collect::<String>();

            format!(
                "## {}\n{}",
                format_note_category(category),
                category_notes_content
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("# Notes\n\n{}", notes_content)
}

/// Format a note category for display
fn format_note_category(category: &crate::models::NoteCategory) -> String {
    match category {
        crate::models::NoteCategory::Implementation => "💻 Implementation",
        crate::models::NoteCategory::Decision => "🤔 Decision",
        crate::models::NoteCategory::Question => "❓ Question",
        crate::models::NoteCategory::Bug => "🐛 Bug",
        crate::models::NoteCategory::Enhancement => "✨ Enhancement",
        crate::models::NoteCategory::Other => "📝 Other",
    }
    .to_string()
}

/// Render a complete specification context
pub fn render_spec_context(
    project: &Project,
    spec: &Specification,
    task_list: &TaskList,
    notes: &[crate::models::Note],
) -> String {
    let project_header = format!(
        "# Project: {}\n\n**Description:** {}\n\n**Created:** {} | **Updated:** {}\n\n",
        project.name,
        project.description,
        format_timestamp(&project.created_at),
        format_timestamp(&project.updated_at)
    );

    let spec_header = format!(
        "## Specification: {}\n**Status:** {} | **ID:** {}\n\n",
        spec.name,
        format_spec_status(&spec.status),
        spec.id
    );

    let spec_description = if spec.description.is_empty() {
        String::new()
    } else {
        format!("**Description:** {}\n\n", spec.description)
    };

    let spec_timestamps = format!(
        "**Created:** {} | **Updated:** {}\n\n",
        format_timestamp(&spec.created_at),
        format_timestamp(&spec.updated_at)
    );

    let spec_content = if spec.content.is_empty() {
        String::new()
    } else {
        format!("## Specification Content\n\n{}\n\n", spec.content)
    };

    let tech_summary = format!(
        "## Technology Stack Summary\n{}\n",
        render_tech_stack_summary(&project.tech_stack)
    );

    let vision_summary = format!(
        "## Project Vision Summary\n{}\n",
        render_vision_summary(&project.vision)
    );

    let task_content = format!("## Current Tasks\n{}\n", render_task_list(task_list));

    let notes_content = render_notes(notes);

    format!(
        "{}{}{}{}{}{}{}{}{}",
        project_header,
        spec_header,
        spec_description,
        spec_timestamps,
        spec_content,
        tech_summary,
        vision_summary,
        task_content,
        notes_content
    )
}

/// Render a condensed tech stack summary
fn render_tech_stack_summary(tech_stack: &TechStack) -> String {
    let sections = [
        ("Languages", &tech_stack.languages),
        ("Frameworks", &tech_stack.frameworks),
        ("Databases", &tech_stack.databases),
    ];

    sections
        .iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(title, items)| format!("**{}:** {}\n", title, items.join(", ")))
        .collect::<String>()
}

/// Render a condensed vision summary
fn render_vision_summary(vision: &Vision) -> String {
    let sections = [
        ("Goals", &vision.goals),
        ("Target Users", &vision.target_users),
    ];

    sections
        .iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(title, items)| format!("**{}:** {}\n", title, items.join(", ")))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, TaskList, TaskPriority, TaskStatus};

    #[test]
    fn test_format_timestamp() {
        let timestamp = Utc::now();
        let formatted = format_timestamp(&timestamp);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_format_task_status() {
        assert_eq!(format_task_status(&TaskStatus::Todo), "⏳ Todo");
        assert_eq!(format_task_status(&TaskStatus::Completed), "✅ Completed");
    }

    #[test]
    fn test_format_task_priority() {
        assert_eq!(format_task_priority(&TaskPriority::Low), "🟢 Low");
        assert_eq!(format_task_priority(&TaskPriority::Critical), "🔴 Critical");
    }

    #[test]
    fn test_render_tech_stack() {
        let tech_stack = TechStack {
            languages: vec!["Rust".to_string(), "Python".to_string()],
            frameworks: vec!["Actix".to_string()],
            databases: vec!["PostgreSQL".to_string()],
            tools: vec!["Cargo".to_string()],
            deployment: vec!["Docker".to_string()],
        };

        let rendered = render_tech_stack(&tech_stack);
        assert!(rendered.contains("Rust"));
        assert!(rendered.contains("Python"));
        assert!(rendered.contains("Actix"));
    }

    #[test]
    fn test_render_task_list() {
        let task = Task {
            id: "task_1".to_string(),
            title: "Test Task".to_string(),
            description: "A test task".to_string(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task_list = TaskList {
            tasks: vec![task],
            last_updated: Utc::now(),
        };

        let rendered = render_task_list(&task_list);
        assert!(rendered.contains("Test Task"));
        assert!(rendered.contains("⏳ Todo"));
    }
}
