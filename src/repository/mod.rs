//! Data access layer for projects and specifications
//!
//! This module provides the repository pattern implementation for managing project and
//! specification data. Repositories handle all data persistence operations, including
//! creating, reading, updating, and deleting projects and specifications.
//!
//! # Architecture
//!
//! The repository layer sits between the MCP handlers and the file system, providing:
//!
//! * **Abstraction** - High-level operations that hide file system complexity
//! * **Validation** - Data validation and business rule enforcement
//! * **Consistency** - Atomic operations that maintain data integrity
//! * **Rendering** - Generation of markdown files from structured data
//!
//! # Repositories
//!
//! ## ProjectRepository
//!
//! Manages project lifecycle operations:
//! - Create new projects with tech stack and vision
//! - Load existing projects and their metadata
//! - Update project information and settings
//! - Delete projects and cleanup associated data
//!
//! ## SpecificationRepository
//!
//! Manages specification and task operations:
//! - Create specifications with unique timestamped IDs
//! - Load specification content and metadata
//! - Update task lists and development notes
//! - Render markdown files from structured data
//!
//! # Usage
//!
//! ```rust
//! use foundry_mcp::{
//!     ProjectRepository, SpecificationRepository, FileSystemManager,
//!     Project, TechStack, Vision, Task, TaskStatus, TaskPriority, Note, NoteCategory
//! };
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Set up repositories
//! let fs_manager = FileSystemManager::new()?;
//! let project_repo = ProjectRepository::new(fs_manager.clone());
//! let spec_repo = SpecificationRepository::new(fs_manager);
//!
//! // Create a project
//! let project = Project {
//!     name: "my-app".to_string(),
//!     description: "A sample application".to_string(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//!     tech_stack: TechStack {
//!         languages: vec!["Rust".to_string()],
//!         frameworks: vec!["Actix-Web".to_string()],
//!         databases: vec!["PostgreSQL".to_string()],
//!         tools: vec!["Cargo".to_string()],
//!         deployment: vec!["Docker".to_string()],
//!     },
//!     vision: Vision {
//!         overview: "Fast web application".to_string(),
//!         goals: vec!["High performance".to_string()],
//!         target_users: vec!["Developers".to_string()],
//!         success_criteria: vec!["< 100ms response".to_string()],
//!     },
//! };
//!
//! project_repo.create_project(&project).await?;
//!
//! // Create a specification
//! let spec = spec_repo.create_spec(
//!     "my-app",
//!     "user_auth",
//!     "User Authentication",
//!     "JWT-based authentication system"
//! ).await?;
//!
//! // Add tasks to the specification
//! let tasks = vec![
//!     Task {
//!         id: "task_001".to_string(),
//!         title: "Implement JWT library".to_string(),
//!         description: "Add jsonwebtoken crate and basic functions".to_string(),
//!         status: TaskStatus::Todo,
//!         priority: TaskPriority::High,
//!         dependencies: vec![],
//!         created_at: Utc::now(),
//!         updated_at: Utc::now(),
//!     }
//! ];
//!
//! let notes = vec![
//!     Note {
//!         id: "note_001".to_string(),
//!         content: "Consider using RS256 for token signing".to_string(),
//!         category: NoteCategory::Implementation,
//!         created_at: Utc::now(),
//!     }
//! ];
//!
//! spec_repo.update_spec("my-app", &spec.id, tasks, notes).await?;
//!
//! // Load the complete specification context
//! let context = spec_repo.load_spec("my-app", &spec.id).await?;
//! println!("Loaded specification context: {}", context.content.len());
//! # Ok(())
//! # }
//! ```
//!
//! # File Format
//!
//! Repositories generate human-readable markdown files alongside JSON metadata:
//!
//! ## Project Files
//!
//! * `metadata.json` - Structured project data
//! * `tech-stack.md` - Technology stack in markdown format
//! * `vision.md` - Business vision and goals
//!
//! ## Specification Files
//!
//! * `metadata.json` - Specification metadata
//! * `spec.md` - Main specification content
//! * `task-list.md` - Implementation tasks with status
//! * `notes.md` - Development notes by category
//!
//! # Data Integrity
//!
//! The repository layer ensures data integrity through:
//!
//! * **Validation** - Input validation before persistence
//! * **Atomic operations** - Using file system manager's atomic writes
//! * **Consistency checks** - Verifying relationships between entities
//! * **Error handling** - Comprehensive error reporting and recovery

pub mod project;
pub mod specification;

pub use project::ProjectRepository;
pub use specification::SpecificationRepository;
