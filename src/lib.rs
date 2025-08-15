#![deny(clippy::all)]

//! Project Manager MCP - A Model Context Protocol server for project context management
//!
//! This library provides deterministic tools for AI coding assistants to manage project
//! context, specifications, and task lists through a centralized file system.

pub mod filesystem;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod utils;

// Re-export main types for convenience
pub use filesystem::FileSystemManager;
pub use handlers::ProjectManagerHandler;
pub use models::{Note, Project, Specification, Task, TaskList, TechStack, Vision};
pub use repository::{ProjectRepository, SpecificationRepository};

/// Result type for the project manager operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
