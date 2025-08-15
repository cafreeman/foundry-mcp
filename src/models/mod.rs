//! Core data structures for the Project Manager MCP

pub mod base;
pub mod specification;
pub mod task;

pub use base::{Project, TechStack, Vision};
pub use specification::{SpecStatus, Specification};
pub use task::{Note, NoteCategory, Task, TaskList, TaskPriority, TaskStatus};
