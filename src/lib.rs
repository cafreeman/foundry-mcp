#![deny(clippy::all)]

//! Project Manager MCP Server Library
//!
//! A Model Context Protocol (MCP) server for managing software projects with specifications,
//! tasks, and notes. This library provides AI coding assistants with structured project
//! context and deterministic state management for complex software development workflows.
//!
//! # Overview
//!
//! The Project Manager MCP server enables AI coding assistants to:
//!
//! * **Create and manage projects** with technology stacks and business vision
//! * **Write detailed specifications** for features and requirements  
//! * **Track implementation tasks** with dependencies and priorities
//! * **Capture development notes** and decisions throughout the process
//! * **Load contextual information** to maintain continuity across coding sessions
//!
//! # Architecture
//!
//! The library is organized into several key modules:
//!
//! * [`models`] - Core data structures (Project, Specification, Task, Note)
//! * [`repository`] - Data access layer for persistent storage
//! * [`filesystem`] - Safe file system operations and management
//! * [`handlers`] - MCP tool implementations for the protocol interface
//! * [`errors`] - Comprehensive error types and handling
//! * [`utils`] - Utility functions for validation and formatting
//!
//! # Usage
//!
//! ## As an MCP Server
//!
//! The primary use case is running as an MCP server that AI coding assistants can connect to:
//!
//! ```bash
//! # Run the MCP server
//! cargo run --bin project-manager-mcp
//! ```
//!
//! ## As a Library
//!
//! You can also use the library directly in Rust applications:
//!
//! ```rust
//! use project_manager_mcp::{
//!     ProjectRepository, SpecificationRepository, FileSystemManager,
//!     Project, TechStack, Vision, ProjectManagerError
//! };
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), ProjectManagerError> {
//! // Set up the file system manager
//! let fs_manager = FileSystemManager::new()?;
//!
//! // Create repositories
//! let project_repo = ProjectRepository::new(fs_manager.clone());
//! let spec_repo = SpecificationRepository::new(fs_manager);
//!
//! // Create a new project
//! let project = Project {
//!     name: "my-web-app".to_string(),
//!     description: "A modern web application".to_string(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//!     tech_stack: TechStack {
//!         languages: vec!["Rust".to_string(), "TypeScript".to_string()],
//!         frameworks: vec!["Actix-Web".to_string(), "React".to_string()],
//!         databases: vec!["PostgreSQL".to_string()],
//!         tools: vec!["Cargo".to_string(), "npm".to_string()],
//!         deployment: vec!["Docker".to_string()],
//!     },
//!     vision: Vision {
//!         overview: "Fast and reliable web platform".to_string(),
//!         goals: vec!["Handle 10k users".to_string()],
//!         target_users: vec!["Web developers".to_string()],
//!         success_criteria: vec!["< 100ms response time".to_string()],
//!     },
//! };
//!
//! // Save the project
//! project_repo.create_project(&project).await?;
//!
//! // Create a specification
//! let spec = spec_repo.create_spec(
//!     "my-web-app",
//!     "user_authentication", 
//!     "User Authentication System",
//!     "OAuth2 and JWT-based authentication"
//! ).await?;
//!
//! println!("Created project and specification: {}", spec.id);
//! # Ok(())
//! # }
//! ```
//!
//! # File System Structure
//!
//! Projects are stored in `~/.project-manager-mcp/` with the following structure:
//!
//! ```text
//! ~/.project-manager-mcp/
//! ├── project-name/
//! │   ├── project/
//! │   │   ├── metadata.json      # Project information
//! │   │   ├── tech-stack.md      # Technology stack
//! │   │   └── vision.md          # Business vision
//! │   └── specs/
//! │       └── 20240115_feature_name/
//! │           ├── metadata.json  # Specification metadata
//! │           ├── spec.md        # Specification content
//! │           ├── task-list.md   # Implementation tasks
//! │           └── notes.md       # Development notes
//! ```
//!
//! # MCP Tools
//!
//! The server provides these MCP tools for AI assistants:
//!
//! * **setup_project** - Create a new project with tech stack and vision
//! * **create_spec** - Create a new specification for a feature or requirement
//! * **load_spec** - Load complete context for a specification
//! * **update_spec** - Update tasks and notes in a specification
//!
//! # Error Handling
//!
//! The library uses a comprehensive error system with user-friendly messages:
//!
//! ```rust
//! use project_manager_mcp::{ProjectManagerError, Result};
//!
//! fn handle_error(result: Result<()>) {
//!     match result {
//!         Ok(()) => println!("Success!"),
//!         Err(ProjectManagerError::NotFound { resource_type, identifier, .. }) => {
//!             println!("{} '{}' not found", resource_type, identifier);
//!         }
//!         Err(ProjectManagerError::Validation { field, value, reason }) => {
//!             println!("Invalid {}: '{}' - {}", field, value, reason);
//!         }
//!         Err(err) => println!("Error: {}", err.user_message()),
//!     }
//! }
//! ```
//!
//! # Features
//!
//! * **Type-safe data models** with comprehensive validation
//! * **Atomic file operations** with backup and rollback support
//! * **Dependency tracking** for tasks and specifications
//! * **Rich metadata** with timestamps and status tracking
//! * **Markdown content** support for specifications and notes
//! * **Unicode support** for international development teams
//! * **Comprehensive error handling** with user-friendly messages

pub mod cache;
pub mod errors;
pub mod filesystem;
pub mod handlers;
pub mod lazy;
pub mod models;
pub mod profiling;
pub mod progress;
pub mod repository;
pub mod security;
pub mod tools;
pub mod utils;

// Re-export main types for convenience
pub use errors::{ProjectManagerError, Result};
pub use filesystem::FileSystemManager;
pub use handlers::ProjectManagerHandler;
pub use models::{Note, Project, Specification, Task, TaskList, TechStack, Vision};
pub use repository::{ProjectRepository, SpecificationRepository};
