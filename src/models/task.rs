//! Task and note models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Blocked,
}

/// Priority level of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
