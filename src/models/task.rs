//! Task and note models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a task
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Blocked,
}

/// Priority level of a task
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Represents a task in a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Category for organizing notes
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NoteCategory {
    Implementation,
    Decision,
    Question,
    Bug,
    Enhancement,
    Other,
}

/// Represents a note in a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub category: NoteCategory,
    pub created_at: DateTime<Utc>,
}

/// Represents a list of tasks for a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_sample_task() -> Task {
        Task {
            id: "task_001".to_string(),
            title: "Implement user authentication".to_string(),
            description: "Add JWT-based authentication system".to_string(),
            status: TaskStatus::Todo,
            priority: TaskPriority::High,
            dependencies: vec!["task_000".to_string()],
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        }
    }

    fn create_sample_note() -> Note {
        Note {
            id: "note_001".to_string(),
            content: "Consider using OAuth 2.0 for third-party authentication".to_string(),
            category: NoteCategory::Implementation,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
        }
    }

    fn create_sample_task_list() -> TaskList {
        TaskList {
            tasks: vec![create_sample_task()],
            last_updated: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn test_task_status_serialization() {
        let statuses = [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Blocked,
        ];

        for status in &statuses {
            let serialized = serde_json::to_string(status).expect("Failed to serialize TaskStatus");
            let deserialized: TaskStatus = serde_json::from_str(&serialized).expect("Failed to deserialize TaskStatus");
            assert_eq!(*status, deserialized);
        }
    }

    #[test]
    fn test_task_priority_serialization() {
        let priorities = [
            TaskPriority::Low,
            TaskPriority::Medium,
            TaskPriority::High,
            TaskPriority::Critical,
        ];

        for priority in &priorities {
            let serialized = serde_json::to_string(priority).expect("Failed to serialize TaskPriority");
            let deserialized: TaskPriority = serde_json::from_str(&serialized).expect("Failed to deserialize TaskPriority");
            assert_eq!(*priority, deserialized);
        }
    }

    #[test]
    fn test_note_category_serialization() {
        let categories = [
            NoteCategory::Implementation,
            NoteCategory::Decision,
            NoteCategory::Question,
            NoteCategory::Bug,
            NoteCategory::Enhancement,
            NoteCategory::Other,
        ];

        for category in &categories {
            let serialized = serde_json::to_string(category).expect("Failed to serialize NoteCategory");
            let deserialized: NoteCategory = serde_json::from_str(&serialized).expect("Failed to deserialize NoteCategory");
            assert_eq!(*category, deserialized);
        }
    }

    #[test]
    fn test_task_serialization() {
        let task = create_sample_task();
        let serialized = serde_json::to_string(&task).expect("Failed to serialize Task");
        let deserialized: Task = serde_json::from_str(&serialized).expect("Failed to deserialize Task");

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.title, deserialized.title);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.status, deserialized.status);
        assert_eq!(task.priority, deserialized.priority);
        assert_eq!(task.dependencies, deserialized.dependencies);
        assert_eq!(task.created_at, deserialized.created_at);
        assert_eq!(task.updated_at, deserialized.updated_at);
    }

    #[test]
    fn test_task_empty_dependencies() {
        let mut task = create_sample_task();
        task.dependencies = vec![];

        let serialized = serde_json::to_string(&task).expect("Failed to serialize Task with empty dependencies");
        let deserialized: Task = serde_json::from_str(&serialized).expect("Failed to deserialize Task with empty dependencies");

        assert!(deserialized.dependencies.is_empty());
    }

    #[test]
    fn test_task_multiple_dependencies() {
        let mut task = create_sample_task();
        task.dependencies = vec![
            "task_001".to_string(),
            "task_002".to_string(),
            "task_003".to_string(),
        ];

        let serialized = serde_json::to_string(&task).expect("Failed to serialize Task with multiple dependencies");
        let deserialized: Task = serde_json::from_str(&serialized).expect("Failed to deserialize Task with multiple dependencies");

        assert_eq!(deserialized.dependencies.len(), 3);
        assert_eq!(deserialized.dependencies[0], "task_001");
        assert_eq!(deserialized.dependencies[1], "task_002");
        assert_eq!(deserialized.dependencies[2], "task_003");
    }

    #[test]
    fn test_note_serialization() {
        let note = create_sample_note();
        let serialized = serde_json::to_string(&note).expect("Failed to serialize Note");
        let deserialized: Note = serde_json::from_str(&serialized).expect("Failed to deserialize Note");

        assert_eq!(note.id, deserialized.id);
        assert_eq!(note.content, deserialized.content);
        assert_eq!(note.category, deserialized.category);
        assert_eq!(note.created_at, deserialized.created_at);
    }

    #[test]
    fn test_note_long_content() {
        let long_content = "A".repeat(10000);
        let note = Note {
            id: "note_long".to_string(),
            content: long_content.clone(),
            category: NoteCategory::Other,
            created_at: Utc::now(),
        };

        let serialized = serde_json::to_string(&note).expect("Failed to serialize Note with long content");
        let deserialized: Note = serde_json::from_str(&serialized).expect("Failed to deserialize Note with long content");

        assert_eq!(note.content, deserialized.content);
        assert_eq!(deserialized.content.len(), 10000);
    }

    #[test]
    fn test_note_markdown_content() {
        let note = Note {
            id: "note_markdown".to_string(),
            content: r#"# Implementation Note

## Code Example
```rust
fn main() {
    println!("Hello, world!");
}
```

**Important**: This needs to be tested thoroughly.

- Item 1
- Item 2"#.to_string(),
            category: NoteCategory::Implementation,
            created_at: Utc::now(),
        };

        let serialized = serde_json::to_string(&note).expect("Failed to serialize Note with markdown");
        let deserialized: Note = serde_json::from_str(&serialized).expect("Failed to deserialize Note with markdown");

        assert_eq!(note.content, deserialized.content);
        assert!(deserialized.content.contains("# Implementation Note"));
        assert!(deserialized.content.contains("```rust"));
    }

    #[test]
    fn test_task_list_serialization() {
        let task_list = create_sample_task_list();
        let serialized = serde_json::to_string(&task_list).expect("Failed to serialize TaskList");
        let deserialized: TaskList = serde_json::from_str(&serialized).expect("Failed to deserialize TaskList");

        assert_eq!(task_list.tasks.len(), deserialized.tasks.len());
        assert_eq!(task_list.last_updated, deserialized.last_updated);
        assert_eq!(task_list.tasks[0].id, deserialized.tasks[0].id);
    }

    #[test]
    fn test_task_list_empty() {
        let task_list = TaskList {
            tasks: vec![],
            last_updated: Utc::now(),
        };

        let serialized = serde_json::to_string(&task_list).expect("Failed to serialize empty TaskList");
        let deserialized: TaskList = serde_json::from_str(&serialized).expect("Failed to deserialize empty TaskList");

        assert!(deserialized.tasks.is_empty());
    }

    #[test]
    fn test_task_list_multiple_tasks() {
        let tasks = vec![
            create_sample_task(),
            Task {
                id: "task_002".to_string(),
                title: "Database setup".to_string(),
                description: "Configure PostgreSQL database".to_string(),
                status: TaskStatus::InProgress,
                priority: TaskPriority::Medium,
                dependencies: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Task {
                id: "task_003".to_string(),
                title: "Frontend components".to_string(),
                description: "Create React components".to_string(),
                status: TaskStatus::Completed,
                priority: TaskPriority::Low,
                dependencies: vec!["task_002".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let task_list = TaskList {
            tasks,
            last_updated: Utc::now(),
        };

        let serialized = serde_json::to_string(&task_list).expect("Failed to serialize TaskList with multiple tasks");
        let deserialized: TaskList = serde_json::from_str(&serialized).expect("Failed to deserialize TaskList with multiple tasks");

        assert_eq!(deserialized.tasks.len(), 3);
        assert_eq!(deserialized.tasks[0].status, TaskStatus::Todo);
        assert_eq!(deserialized.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(deserialized.tasks[2].status, TaskStatus::Completed);
    }

    #[test]
    fn test_enum_hash_and_equality() {
        use std::collections::HashMap;

        let mut status_count = HashMap::new();
        status_count.insert(TaskStatus::Todo, 5);
        status_count.insert(TaskStatus::InProgress, 2);
        status_count.insert(TaskStatus::Completed, 10);

        assert_eq!(status_count.get(&TaskStatus::Todo), Some(&5));
        assert_eq!(status_count.get(&TaskStatus::Completed), Some(&10));

        let mut priority_count = HashMap::new();
        priority_count.insert(TaskPriority::High, 3);
        priority_count.insert(TaskPriority::Critical, 1);

        assert_eq!(priority_count.get(&TaskPriority::High), Some(&3));

        let mut category_count = HashMap::new();
        category_count.insert(NoteCategory::Bug, 2);
        category_count.insert(NoteCategory::Enhancement, 4);

        assert_eq!(category_count.get(&NoteCategory::Bug), Some(&2));
    }
}
