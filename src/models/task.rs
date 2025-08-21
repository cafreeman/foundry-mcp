//! Task and note models for project management and collaboration
//!
//! This module provides data structures for managing tasks and notes within project specifications.
//! Tasks represent actionable work items with dependencies and priorities, while notes capture
//! important information, decisions, and observations during development.
//!
//! # Examples
//!
//! ```rust
//! use foundry_mcp::models::task::{Task, TaskStatus, TaskPriority, Note, NoteCategory, TaskList};
//! use chrono::Utc;
//!
//! // Create a high-priority task
//! let task = Task {
//!     id: "task_001".to_string(),
//!     title: "Implement user authentication".to_string(),
//!     description: "Add JWT-based authentication with OAuth2 support".to_string(),
//!     status: TaskStatus::Todo,
//!     priority: TaskPriority::High,
//!     dependencies: vec!["task_database_setup".to_string()],
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//! };
//!
//! // Create an implementation note
//! let note = Note {
//!     id: "note_001".to_string(),
//!     content: "Consider using bcrypt for password hashing".to_string(),
//!     category: NoteCategory::Implementation,
//!     created_at: Utc::now(),
//! };
//!
//! // Create a task list
//! let task_list = TaskList {
//!     tasks: vec![task],
//!     last_updated: Utc::now(),
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a task throughout its lifecycle.
///
/// Tasks progress through different states as they are worked on. This enum tracks
/// the current state to help teams coordinate work and understand progress.
///
/// # States
///
/// * `Todo` - Task is ready to be started but not yet begun
/// * `InProgress` - Task is actively being worked on
/// * `Completed` - Task has been finished successfully
/// * `Blocked` - Task cannot proceed due to dependencies or external factors
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::TaskStatus;
///
/// // New tasks start as Todo
/// let status = TaskStatus::Todo;
///
/// // Mark as in progress when work begins
/// let status = TaskStatus::InProgress;
///
/// // Complete when finished
/// let status = TaskStatus::Completed;
///
/// // Block if dependencies are missing
/// let status = TaskStatus::Blocked;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TaskStatus {
    /// Task is ready to be started but not yet begun
    Todo,
    /// Task is actively being worked on
    InProgress,
    /// Task has been finished successfully
    Completed,
    /// Task cannot proceed due to dependencies or external factors
    Blocked,
}

/// Priority level of a task for work scheduling and resource allocation.
///
/// Task priorities help teams decide which work to focus on first and how to
/// allocate resources effectively. Higher priority tasks should generally be
/// completed before lower priority ones.
///
/// # Priority Levels
///
/// * `Low` - Non-urgent tasks that can be done when time permits
/// * `Medium` - Standard priority for regular development work
/// * `High` - Important tasks that should be prioritized over medium/low
/// * `Critical` - Urgent tasks that need immediate attention (bugs, blockers)
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::TaskPriority;
///
/// // Regular feature work
/// let priority = TaskPriority::Medium;
///
/// // Important customer request
/// let priority = TaskPriority::High;
///
/// // Production bug fix
/// let priority = TaskPriority::Critical;
///
/// // Nice-to-have improvement
/// let priority = TaskPriority::Low;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TaskPriority {
    /// Non-urgent tasks that can be done when time permits
    Low,
    /// Standard priority for regular development work
    Medium,
    /// Important tasks that should be prioritized over medium/low
    High,
    /// Urgent tasks that need immediate attention (bugs, blockers)
    Critical,
}

/// Represents a task in a specification with dependencies and tracking information.
///
/// A `Task` represents a specific piece of work that needs to be completed as part
/// of implementing a specification. Tasks can depend on other tasks and are tracked
/// through various status states with assigned priorities.
///
/// # Task Dependencies
///
/// Tasks can depend on other tasks being completed first. The `dependencies` field
/// contains a list of task IDs that must be completed before this task can begin.
/// This enables proper work sequencing and prevents conflicts.
///
/// # Fields
///
/// * `id` - Unique identifier for the task (e.g., "task_001", "auth_implementation")
/// * `title` - Short, descriptive name for the task
/// * `description` - Detailed explanation of what needs to be done
/// * `status` - Current progress state (Todo, InProgress, Completed, Blocked)
/// * `priority` - Importance level for scheduling (Low, Medium, High, Critical)
/// * `dependencies` - List of task IDs that must be completed first
/// * `created_at` - Timestamp when the task was created
/// * `updated_at` - Timestamp of the last modification
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::{Task, TaskStatus, TaskPriority};
/// use chrono::Utc;
///
/// // Simple independent task
/// let setup_task = Task {
///     id: "setup_database".to_string(),
///     title: "Set up PostgreSQL database".to_string(),
///     description: "Install and configure PostgreSQL with initial schema".to_string(),
///     status: TaskStatus::Todo,
///     priority: TaskPriority::High,
///     dependencies: vec![], // No dependencies
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
///
/// // Task with dependencies
/// let api_task = Task {
///     id: "implement_api".to_string(),
///     title: "Implement REST API endpoints".to_string(),
///     description: "Create CRUD endpoints for user management".to_string(),
///     status: TaskStatus::Todo,
///     priority: TaskPriority::Medium,
///     dependencies: vec!["setup_database".to_string()], // Depends on database
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
///
/// // Critical bug fix
/// let bug_task = Task {
///     id: "fix_memory_leak".to_string(),
///     title: "Fix memory leak in user session handling".to_string(),
///     description: "Investigate and fix memory leak causing OOM errors".to_string(),
///     status: TaskStatus::InProgress,
///     priority: TaskPriority::Critical,
///     dependencies: vec![],
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,
    /// Short, descriptive name for the task
    pub title: String,
    /// Detailed explanation of what needs to be done
    pub description: String,
    /// Current progress state of the task
    pub status: TaskStatus,
    /// Importance level for scheduling work
    pub priority: TaskPriority,
    /// List of task IDs that must be completed first
    pub dependencies: Vec<String>,
    /// Timestamp when the task was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last modification
    pub updated_at: DateTime<Utc>,
}

/// Category for organizing notes by type and purpose.
///
/// Note categories help organize and filter notes based on their content and purpose.
/// This makes it easier to find relevant information when working on specific aspects
/// of a project.
///
/// # Categories
///
/// * `Implementation` - Technical implementation details, code notes, and how-to information
/// * `Decision` - Architectural decisions, trade-offs, and rationale for choices made
/// * `Question` - Open questions, uncertainties, and items needing clarification
/// * `Bug` - Bug reports, issues found, and troubleshooting information
/// * `Enhancement` - Ideas for improvements, feature requests, and optimizations
/// * `Other` - General notes that don't fit into specific categories
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::NoteCategory;
///
/// // Technical implementation note
/// let category = NoteCategory::Implementation;
///
/// // Architectural decision
/// let category = NoteCategory::Decision;
///
/// // Bug report
/// let category = NoteCategory::Bug;
///
/// // Feature request
/// let category = NoteCategory::Enhancement;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NoteCategory {
    /// Technical implementation details, code notes, and how-to information
    Implementation,
    /// Architectural decisions, trade-offs, and rationale for choices made
    Decision,
    /// Open questions, uncertainties, and items needing clarification
    Question,
    /// Bug reports, issues found, and troubleshooting information
    Bug,
    /// Ideas for improvements, feature requests, and optimizations
    Enhancement,
    /// General notes that don't fit into specific categories
    Other,
}

/// Represents a note in a specification for capturing important information.
///
/// A `Note` captures important information, decisions, observations, or questions
/// that arise during the development of a specification. Notes are categorized
/// to help organize and filter information effectively.
///
/// # Use Cases
///
/// * Document implementation decisions and rationale
/// * Record questions and uncertainties for later resolution
/// * Track bugs and issues discovered during development
/// * Capture ideas for future enhancements
/// * Store important technical details and observations
///
/// # Fields
///
/// * `id` - Unique identifier for the note
/// * `content` - The note content (supports Markdown formatting)
/// * `category` - Type of note for organization and filtering
/// * `created_at` - Timestamp when the note was created
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::{Note, NoteCategory};
/// use chrono::Utc;
///
/// // Implementation note with code example
/// let impl_note = Note {
///     id: "note_auth_impl".to_string(),
///     content: "## JWT Implementation\n\nUsing the `jsonwebtoken` crate for JWT handling:\n\n```rust\nuse jsonwebtoken::{encode, decode, Header, Validation};\n\nlet token = encode(&Header::default(), &claims, &encoding_key)?;\n```\n\n**Note**: Remember to set appropriate expiration times.".to_string(),
///     category: NoteCategory::Implementation,
///     created_at: Utc::now(),
/// };
///
/// // Decision note
/// let decision_note = Note {
///     id: "note_db_choice".to_string(),
///     content: "Chose PostgreSQL over MongoDB for ACID compliance and complex queries".to_string(),
///     category: NoteCategory::Decision,
///     created_at: Utc::now(),
/// };
///
/// // Question note
/// let question_note = Note {
///     id: "note_scaling_question".to_string(),
///     content: "Should we implement horizontal scaling now or wait for user growth?".to_string(),
///     category: NoteCategory::Question,
///     created_at: Utc::now(),
/// };
///
/// // Bug report note
/// let bug_note = Note {
///     id: "note_session_bug".to_string(),
///     content: "Session tokens not being invalidated on logout - security risk!".to_string(),
///     category: NoteCategory::Bug,
///     created_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note
    pub id: String,
    /// The note content (supports Markdown formatting)
    pub content: String,
    /// Type of note for organization and filtering
    pub category: NoteCategory,
    /// Timestamp when the note was created
    pub created_at: DateTime<Utc>,
}

/// Represents a list of tasks for a specification with tracking metadata.
///
/// A `TaskList` contains all tasks associated with a particular specification
/// along with metadata about when the list was last updated. This provides
/// a complete view of work items and their current state.
///
/// # Fields
///
/// * `tasks` - Vector of all tasks in the specification
/// * `last_updated` - Timestamp of the most recent modification to any task
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::task::{TaskList, Task, TaskStatus, TaskPriority};
/// use chrono::Utc;
///
/// // Create a task list for a feature specification
/// let task_list = TaskList {
///     tasks: vec![
///         Task {
///             id: "task_001".to_string(),
///             title: "Design user interface".to_string(),
///             description: "Create mockups and wireframes".to_string(),
///             status: TaskStatus::Completed,
///             priority: TaskPriority::Medium,
///             dependencies: vec![],
///             created_at: Utc::now(),
///             updated_at: Utc::now(),
///         },
///         Task {
///             id: "task_002".to_string(),
///             title: "Implement API endpoints".to_string(),
///             description: "Create REST API for user management".to_string(),
///             status: TaskStatus::InProgress,
///             priority: TaskPriority::High,
///             dependencies: vec!["task_001".to_string()],
///             created_at: Utc::now(),
///             updated_at: Utc::now(),
///         },
///     ],
///     last_updated: Utc::now(),
/// };
///
/// // Check task progress
/// let total_tasks = task_list.tasks.len();
/// let completed_tasks = task_list.tasks.iter()
///     .filter(|t| t.status == TaskStatus::Completed)
///     .count();
/// println!("Progress: {}/{} tasks completed", completed_tasks, total_tasks);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    /// Vector of all tasks in the specification
    pub tasks: Vec<Task>,
    /// Timestamp of the most recent modification to any task
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
            let deserialized: TaskStatus =
                serde_json::from_str(&serialized).expect("Failed to deserialize TaskStatus");
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
            let serialized =
                serde_json::to_string(priority).expect("Failed to serialize TaskPriority");
            let deserialized: TaskPriority =
                serde_json::from_str(&serialized).expect("Failed to deserialize TaskPriority");
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
            let serialized =
                serde_json::to_string(category).expect("Failed to serialize NoteCategory");
            let deserialized: NoteCategory =
                serde_json::from_str(&serialized).expect("Failed to deserialize NoteCategory");
            assert_eq!(*category, deserialized);
        }
    }

    #[test]
    fn test_task_serialization() {
        let task = create_sample_task();
        let serialized = serde_json::to_string(&task).expect("Failed to serialize Task");
        let deserialized: Task =
            serde_json::from_str(&serialized).expect("Failed to deserialize Task");

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

        let serialized =
            serde_json::to_string(&task).expect("Failed to serialize Task with empty dependencies");
        let deserialized: Task = serde_json::from_str(&serialized)
            .expect("Failed to deserialize Task with empty dependencies");

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

        let serialized = serde_json::to_string(&task)
            .expect("Failed to serialize Task with multiple dependencies");
        let deserialized: Task = serde_json::from_str(&serialized)
            .expect("Failed to deserialize Task with multiple dependencies");

        assert_eq!(deserialized.dependencies.len(), 3);
        assert_eq!(deserialized.dependencies[0], "task_001");
        assert_eq!(deserialized.dependencies[1], "task_002");
        assert_eq!(deserialized.dependencies[2], "task_003");
    }

    #[test]
    fn test_note_serialization() {
        let note = create_sample_note();
        let serialized = serde_json::to_string(&note).expect("Failed to serialize Note");
        let deserialized: Note =
            serde_json::from_str(&serialized).expect("Failed to deserialize Note");

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

        let serialized =
            serde_json::to_string(&note).expect("Failed to serialize Note with long content");
        let deserialized: Note = serde_json::from_str(&serialized)
            .expect("Failed to deserialize Note with long content");

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
- Item 2"#
                .to_string(),
            category: NoteCategory::Implementation,
            created_at: Utc::now(),
        };

        let serialized =
            serde_json::to_string(&note).expect("Failed to serialize Note with markdown");
        let deserialized: Note =
            serde_json::from_str(&serialized).expect("Failed to deserialize Note with markdown");

        assert_eq!(note.content, deserialized.content);
        assert!(deserialized.content.contains("# Implementation Note"));
        assert!(deserialized.content.contains("```rust"));
    }

    #[test]
    fn test_task_list_serialization() {
        let task_list = create_sample_task_list();
        let serialized = serde_json::to_string(&task_list).expect("Failed to serialize TaskList");
        let deserialized: TaskList =
            serde_json::from_str(&serialized).expect("Failed to deserialize TaskList");

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

        let serialized =
            serde_json::to_string(&task_list).expect("Failed to serialize empty TaskList");
        let deserialized: TaskList =
            serde_json::from_str(&serialized).expect("Failed to deserialize empty TaskList");

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

        let serialized = serde_json::to_string(&task_list)
            .expect("Failed to serialize TaskList with multiple tasks");
        let deserialized: TaskList = serde_json::from_str(&serialized)
            .expect("Failed to deserialize TaskList with multiple tasks");

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
