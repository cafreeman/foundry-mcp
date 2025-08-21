//! File system operations and directory management
//!
//! This module provides safe, atomic file system operations for the Project Manager MCP server.
//! It handles project directory structure creation, file read/write operations with backup support,
//! and path management with proper validation.
//!
//! # Key Features
//!
//! * **Atomic file writes** - Uses temporary files and atomic moves to prevent corruption
//! * **Automatic backups** - Creates backups before overwriting existing files
//! * **Directory management** - Creates and manages the standard project structure
//! * **Path validation** - Prevents directory traversal and ensures safe file operations
//! * **Error handling** - Comprehensive error reporting for file system issues
//!
//! # Directory Structure
//!
//! The file system manager creates and maintains this directory structure:
//!
//! ```text
//! ~/.foundry/               # Base directory
//! ├── project-1/                        # Project directory
//! │   ├── project/                      # Project metadata
//! │   │   ├── metadata.json            # Project information
//! │   │   ├── tech-stack.md            # Technology stack
//! │   │   └── vision.md                # Business vision
//! │   └── specs/                        # Specifications directory
//! │       ├── 20240115_auth/           # Individual specification
//! │       │   ├── metadata.json        # Spec metadata
//! │       │   ├── spec.md              # Specification content
//! │       │   ├── task-list.md         # Implementation tasks
//! │       │   └── notes.md             # Development notes
//! │       └── 20240116_api/            # Another specification
//! │           └── ...
//! └── project-2/                        # Another project
//!     └── ...
//! ```
//!
//! # Usage
//!
//! ```rust
//! use project_manager_mcp::filesystem::FileSystemManager;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a file system manager
//! let fs_manager = FileSystemManager::new()?;
//!
//! // Create project structure
//! fs_manager.create_project_structure("my-project")?;
//!
//! // Write a file safely
//! let file_path = fs_manager.project_dir("my-project").join("README.md");
//! fs_manager.write_file_safe(&file_path, "# My Project\n\nProject description.")?;
//!
//! // Read the file
//! let content = fs_manager.read_file(&file_path)?;
//! println!("File content: {}", content);
//!
//! // Check if project exists
//! if fs_manager.project_exists("my-project") {
//!     println!("Project exists!");
//! }
//!
//! // List all projects
//! let projects = fs_manager.list_projects()?;
//! for project in projects {
//!     println!("Found project: {}", project);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Safety and Reliability
//!
//! The file system manager prioritizes data safety through:
//!
//! * **Atomic writes** - Files are written to temporary locations then moved atomically
//! * **Backup creation** - Existing files are backed up before being overwritten
//! * **Error recovery** - Failed operations can be rolled back using backups
//! * **Path validation** - All paths are validated to prevent directory traversal
//! * **Unicode support** - Handles international file names and content correctly

pub mod manager;

pub use manager::FileSystemManager;
