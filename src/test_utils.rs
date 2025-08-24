//! Test utilities for integration testing

use crate::cli::args::*;
use anyhow::Result;
use std::env;
use tempfile::TempDir;

/// Test environment that sets up a temporary foundry directory
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub original_home: Option<String>,
}

impl TestEnvironment {
    /// Create a new test environment with isolated foundry directory
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let original_home = env::var("HOME").ok();

        // Set HOME to temp directory so foundry uses temp/.foundry
        unsafe {
            env::set_var("HOME", temp_dir.path());
        }

        Ok(TestEnvironment {
            temp_dir,
            original_home,
        })
    }

    /// Get the foundry directory path within the test environment
    pub fn foundry_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".foundry")
    }

    /// Create valid test arguments for create_project
    pub fn create_project_args(&self, project_name: &str) -> CreateProjectArgs {
        CreateProjectArgs {
            project_name: project_name.to_string(),
            vision: "This is a comprehensive test vision that meets all validation requirements. It describes a revolutionary software project that aims to solve complex problems in the development workflow. The project targets developers and teams who need better tooling for managing project contexts and specifications. Our unique value proposition includes seamless AI integration and deterministic project management that enhances rather than replaces existing workflows.".to_string(),
            tech_stack: "This project leverages Rust as the primary programming language for its performance and safety guarantees. We use clap for CLI argument parsing, serde for JSON serialization, anyhow for error handling, and chrono for timestamp management. The architecture follows modular design principles with clear separation between CLI interfaces, core business logic, and utility functions. For deployment, we target cross-platform compatibility with distribution through cargo install.".to_string(),
            summary: "A comprehensive Rust-based project management CLI tool that creates structured contexts for AI-assisted software development with atomic file operations and rich JSON responses.".to_string(),
        }
    }

    /// Create valid test arguments for create_spec
    pub fn create_spec_args(&self, project_name: &str, feature_name: &str) -> CreateSpecArgs {
        CreateSpecArgs {
            project_name: project_name.to_string(),
            feature_name: feature_name.to_string(),
            spec: "This specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations. The feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage.".to_string(),
            notes: "Implementation notes include important considerations for security, performance, and maintainability. Special attention should be paid to error handling and edge cases. Consider using established libraries where appropriate and ensure compatibility with existing system components.".to_string(),
            tasks: "Create feature scaffolding and basic structure, Implement core functionality with proper error handling, Add comprehensive test coverage for all scenarios, Update documentation and user guides, Perform integration testing with existing features, Conduct code review and optimization".to_string(),
        }
    }

    /// Create test arguments for load_project
    pub fn load_project_args(&self, project_name: &str) -> LoadProjectArgs {
        LoadProjectArgs {
            project_name: project_name.to_string(),
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore original HOME environment variable
        unsafe {
            if let Some(original_home) = &self.original_home {
                env::set_var("HOME", original_home);
            } else {
                env::remove_var("HOME");
            }
        }
    }
}
