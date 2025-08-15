#![deny(clippy::all)]

//! Project Manager MCP Server Library
//!
//! A Model Context Protocol server for managing software projects with specifications,
//! tasks, and notes.

pub mod filesystem;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod tools;
pub mod utils;

// Re-export main types for convenience
pub use filesystem::FileSystemManager;
pub use handlers::ProjectManagerHandler;
pub use models::{Note, Project, Specification, Task, TaskList, TechStack, Vision};
pub use repository::{ProjectRepository, SpecificationRepository};

/// Result type for the project manager operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
