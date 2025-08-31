//! Test utilities for integration testing

use crate::cli::args::*;
use anyhow::Result;
use assert_fs::TempDir;
use std::path::PathBuf;

/// Test environment that sets up temporary directories with assert_fs for isolated testing
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub foundry_dir: PathBuf,
    pub cursor_config_dir: PathBuf,
    pub claude_config_dir: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment with isolated directories
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base = temp_dir.path();

        // Create directory structure
        let foundry_dir = base.join(".foundry");
        let cursor_config_dir = base.join(".cursor");
        let claude_config_dir = base.join(".claude");

        Ok(TestEnvironment {
            temp_dir,
            foundry_dir,
            cursor_config_dir,
            claude_config_dir,
        })
    }

    /// Execute an async test with temporary environment variables set
    pub async fn with_env_async<F, Fut, R>(&self, test_fn: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        // temp-env doesn't support async closures directly, so we need to use a blocking approach
        temp_env::with_vars(
            &[
                ("HOME", Some(self.temp_dir.path().to_str().unwrap())),
                (
                    "CURSOR_CONFIG_DIR",
                    Some(self.cursor_config_dir.to_str().unwrap()),
                ),
                (
                    "CLAUDE_CONFIG_DIR",
                    Some(self.claude_config_dir.to_str().unwrap()),
                ),
            ],
            || {
                // Create a new async runtime for this scope
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(test_fn())
            },
        )
    }

    /// Get the foundry directory path within the test environment
    pub fn foundry_dir(&self) -> &PathBuf {
        &self.foundry_dir
    }

    /// Create a mock binary file for testing
    pub fn create_mock_binary(&self, name: &str) -> Result<PathBuf> {
        let binary_path = self.temp_dir.path().join(name);
        std::fs::write(&binary_path, b"mock foundry binary")?;

        // Make it executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    /// Get the cursor config file path
    pub fn cursor_config_path(&self) -> PathBuf {
        self.cursor_config_dir.join("mcp.json")
    }

    /// Get the claude config directory (claude uses directory, not file)
    pub fn claude_config_dir(&self) -> &PathBuf {
        &self.claude_config_dir
    }

    /// Create a pre-configured cursor config for testing
    pub fn create_cursor_config(&self, server_configs: &[(&str, &str)]) -> Result<()> {
        std::fs::create_dir_all(&self.cursor_config_dir)?;

        let mut servers = serde_json::Map::new();
        for (name, command) in server_configs {
            let server_config = serde_json::json!({
                "command": command,
                "args": ["serve"]
            });
            servers.insert(name.to_string(), server_config);
        }

        let config = serde_json::json!({
            "mcpServers": servers
        });

        let config_content = serde_json::to_string_pretty(&config)?;
        std::fs::write(self.cursor_config_path(), config_content)?;

        Ok(())
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
