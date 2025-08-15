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
    let mut content = String::new();
    content.push_str("# Technology Stack\n\n");
    
    if !tech_stack.languages.is_empty() {
        content.push_str("## Programming Languages\n");
        for language in &tech_stack.languages {
            content.push_str(&format!("- {}\n", language));
        }
        content.push_str("\n");
    }
    
    if !tech_stack.frameworks.is_empty() {
        content.push_str("## Frameworks & Libraries\n");
        for framework in &tech_stack.frameworks {
            content.push_str(&format!("- {}\n", framework));
        }
        content.push_str("\n");
    }
    
    if !tech_stack.databases.is_empty() {
        content.push_str("## Databases & Storage\n");
        for database in &tech_stack.databases {
            content.push_str(&format!("- {}\n", database));
        }
        content.push_str("\n");
    }
    
    if !tech_stack.tools.is_empty() {
        content.push_str("## Development Tools\n");
        for tool in &tech_stack.tools {
            content.push_str(&format!("- {}\n", tool));
        }
        content.push_str("\n");
    }
    
    if !tech_stack.deployment.is_empty() {
        content.push_str("## Deployment & Infrastructure\n");
        for deployment in &tech_stack.deployment {
            content.push_str(&format!("- {}\n", deployment));
        }
        content.push_str("\n");
    }
    
    content
}

/// Render a vision document to markdown
pub fn render_vision(vision: &Vision) -> String {
    let mut content = String::new();
    content.push_str("# Project Vision\n\n");
    
    content.push_str("## Overview\n");
    content.push_str(&vision.overview);
    content.push_str("\n\n");
    
    if !vision.goals.is_empty() {
        content.push_str("## Goals\n");
        for (i, goal) in vision.goals.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, goal));
        }
        content.push_str("\n");
    }
    
    if !vision.target_users.is_empty() {
        content.push_str("## Target Users\n");
        for user in &vision.target_users {
            content.push_str(&format!("- {}\n", user));
        }
        content.push_str("\n");
    }
    
    if !vision.success_criteria.is_empty() {
        content.push_str("## Success Criteria\n");
        for (i, criterion) in vision.success_criteria.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, criterion));
        }
        content.push_str("\n");
    }
    
    content
}

/// Render a task list to markdown
pub fn render_task_list(task_list: &TaskList) -> String {
    let mut content = String::new();
    content.push_str("# Task List\n\n");
    
    if task_list.tasks.is_empty() {
        content.push_str("No tasks defined yet.\n\n");
    } else {
        // Group tasks by status
        let mut todo_tasks = Vec::new();
        let mut in_progress_tasks = Vec::new();
        let mut completed_tasks = Vec::new();
        let mut blocked_tasks = Vec::new();
        
        for task in &task_list.tasks {
            match task.status {
                crate::models::TaskStatus::Todo => todo_tasks.push(task),
                crate::models::TaskStatus::InProgress => in_progress_tasks.push(task),
                crate::models::TaskStatus::Completed => completed_tasks.push(task),
                crate::models::TaskStatus::Blocked => blocked_tasks.push(task),
            }
        }
        
        // Render each group
        if !todo_tasks.is_empty() {
            content.push_str("## ⏳ Todo\n");
            for task in &todo_tasks {
                content.push_str(&render_task(task));
            }
            content.push_str("\n");
        }
        
        if !in_progress_tasks.is_empty() {
            content.push_str("## 🔄 In Progress\n");
            for task in &in_progress_tasks {
                content.push_str(&render_task(task));
            }
            content.push_str("\n");
        }
        
        if !blocked_tasks.is_empty() {
            content.push_str("## 🚫 Blocked\n");
            for task in &blocked_tasks {
                content.push_str(&render_task(task));
            }
            content.push_str("\n");
        }
        
        if !completed_tasks.is_empty() {
            content.push_str("## ✅ Completed\n");
            for task in &completed_tasks {
                content.push_str(&render_task(task));
            }
            content.push_str("\n");
        }
    }
    
    content.push_str(&format!("*Last updated: {}*\n", format_timestamp(&task_list.last_updated)));
    
    content
}

/// Render a single task to markdown
fn render_task(task: &Task) -> String {
    let mut content = String::new();
    
    content.push_str(&format!("### {}\n", task.title));
    content.push_str(&format!("**Status:** {} | **Priority:** {}\n", 
        format_task_status(&task.status), 
        format_task_priority(&task.priority)));
    
    if !task.description.is_empty() {
        content.push_str(&format!("**Description:** {}\n", task.description));
    }
    
    if !task.dependencies.is_empty() {
        content.push_str(&format!("**Dependencies:** {}\n", task.dependencies.join(", ")));
    }
    
    content.push_str(&format!("**Created:** {} | **Updated:** {}\n", 
        format_timestamp(&task.created_at), 
        format_timestamp(&task.updated_at)));
    
    content.push_str("\n");
    
    content
}

/// Render notes to markdown
pub fn render_notes(notes: &[crate::models::Note]) -> String {
    let mut content = String::new();
    content.push_str("# Notes\n\n");
    
    if notes.is_empty() {
        content.push_str("No notes yet.\n\n");
    } else {
        // Group notes by category
        let mut notes_by_category = std::collections::HashMap::new();
        
        for note in notes {
            notes_by_category
                .entry(&note.category)
                .or_insert_with(Vec::new)
                .push(note);
        }
        
        // Render each category
        for (category, category_notes) in notes_by_category.iter() {
            content.push_str(&format!("## {}\n", format_note_category(category)));
            
            for note in category_notes {
                content.push_str(&format!("### Note: {}\n", note.id));
                content.push_str(&note.content);
                content.push_str(&format!("\n\n*Created: {}*\n\n", format_timestamp(&note.created_at)));
            }
        }
    }
    
    content
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
    }.to_string()
}

/// Render a complete specification context
pub fn render_spec_context(project: &Project, spec: &Specification, task_list: &TaskList, notes: &[crate::models::Note]) -> String {
    let mut content = String::new();
    
    // Project header
    content.push_str(&format!("# Project: {}\n\n", project.name));
    content.push_str(&format!("**Description:** {}\n\n", project.description));
    content.push_str(&format!("**Created:** {} | **Updated:** {}\n\n", 
        format_timestamp(&project.created_at), 
        format_timestamp(&project.updated_at)));
    
    // Specification header
    content.push_str(&format!("## Specification: {}\n", spec.name));
    content.push_str(&format!("**Status:** {} | **ID:** {}\n\n", 
        format_spec_status(&spec.status), 
        spec.id));
    
    if !spec.description.is_empty() {
        content.push_str(&format!("**Description:** {}\n\n", spec.description));
    }
    
    content.push_str(&format!("**Created:** {} | **Updated:** {}\n\n", 
        format_timestamp(&spec.created_at), 
        format_timestamp(&spec.updated_at)));
    
    // Specification content
    if !spec.content.is_empty() {
        content.push_str("## Specification Content\n\n");
        content.push_str(&spec.content);
        content.push_str("\n\n");
    }
    
    // Tech stack summary
    content.push_str("## Technology Stack Summary\n");
    let tech_summary = render_tech_stack_summary(&project.tech_stack);
    content.push_str(&tech_summary);
    content.push_str("\n");
    
    // Vision summary
    content.push_str("## Project Vision Summary\n");
    let vision_summary = render_vision_summary(&project.vision);
    content.push_str(&vision_summary);
    content.push_str("\n");
    
    // Task list
    content.push_str("## Current Tasks\n");
    let task_content = render_task_list(task_list);
    content.push_str(&task_content);
    content.push_str("\n");
    
    // Notes
    let notes_content = render_notes(notes);
    content.push_str(&notes_content);
    
    content
}

/// Render a condensed tech stack summary
fn render_tech_stack_summary(tech_stack: &TechStack) -> String {
    let mut summary = String::new();
    
    if !tech_stack.languages.is_empty() {
        summary.push_str(&format!("**Languages:** {}\n", tech_stack.languages.join(", ")));
    }
    
    if !tech_stack.frameworks.is_empty() {
        summary.push_str(&format!("**Frameworks:** {}\n", tech_stack.frameworks.join(", ")));
    }
    
    if !tech_stack.databases.is_empty() {
        summary.push_str(&format!("**Databases:** {}\n", tech_stack.databases.join(", ")));
    }
    
    summary
}

/// Render a condensed vision summary
fn render_vision_summary(vision: &Vision) -> String {
    let mut summary = String::new();
    
    if !vision.goals.is_empty() {
        summary.push_str(&format!("**Goals:** {}\n", vision.goals.join(", ")));
    }
    
    if !vision.target_users.is_empty() {
        summary.push_str(&format!("**Target Users:** {}\n", vision.target_users.join(", ")));
    }
    
    summary
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
