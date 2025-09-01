//! Test utilities for Foundry CLI integration tests
//!
//! This module provides the TestEnvironment struct and helper functions
//! that can be used by the test crate.

use crate::cli::args::*;
use anyhow::Result;
use std::env;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

/// Global mutex to ensure tests don't interfere with each other when setting environment variables
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Environment variable guard that restores the original value when dropped
struct EnvVarGuard {
    key: String,
    original_value: Option<String>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = &self.original_value {
                env::set_var(&self.key, value);
            } else {
                env::remove_var(&self.key);
            }
        }
    }
}

/// Helper function to temporarily set an environment variable with automatic cleanup
fn env_var_guard(key: &str, value: &str) -> EnvVarGuard {
    let original_value = env::var(key).ok();
    unsafe {
        env::set_var(key, value);
    }
    EnvVarGuard {
        key: key.to_string(),
        original_value,
    }
}

/// Test environment that sets up a temporary foundry directory with proper isolation
/// Uses thread-safe environment variable manipulation following CLI testing best practices
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub original_home: Option<String>,
    _lock: MutexGuard<'static, ()>, // Held for the lifetime of the test environment
}

impl TestEnvironment {
    /// Create a new test environment with isolated foundry directory
    /// Sets HOME environment variable in a thread-safe manner
    pub fn new() -> Result<Self> {
        // Acquire global lock to prevent parallel tests from interfering
        // Use expect instead of unwrap to handle poisoned mutex gracefully
        let lock = TEST_MUTEX.lock().unwrap_or_else(|poisoned| {
            // Clear the poisoned state and acquire the lock
            poisoned.into_inner()
        });

        let temp_dir = TempDir::new()?;
        let original_home = env::var("HOME").ok();

        // Set HOME to temp directory so foundry uses temp/.foundry
        unsafe {
            env::set_var("HOME", temp_dir.path());
        }

        Ok(TestEnvironment {
            temp_dir,
            original_home,
            _lock: lock,
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

    /// Create test arguments for update_spec with single file update
    pub fn update_spec_args_single(
        &self,
        project_name: &str,
        spec_name: &str,
        file_type: &str,
        operation: &str,
    ) -> UpdateSpecArgs {
        let content = "Updated content for testing that meets the minimum length requirements and provides comprehensive information for the specification update.".to_string();

        match file_type {
            "spec" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: Some(content),
                tasks: None,
                notes: None,
                operation: operation.to_string(),
            },
            "task-list" | "tasks" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: None,
                tasks: Some(content),
                notes: None,
                operation: operation.to_string(),
            },
            "notes" => UpdateSpecArgs {
                project_name: project_name.to_string(),
                spec_name: spec_name.to_string(),
                spec: None,
                tasks: None,
                notes: Some(content),
                operation: operation.to_string(),
            },
            _ => panic!("Invalid file_type: {}", file_type),
        }
    }

    /// Create test arguments for update_spec with multiple file updates
    pub fn update_spec_args_multi(
        &self,
        project_name: &str,
        spec_name: &str,
        operation: &str,
        spec_content: Option<&str>,
        tasks_content: Option<&str>,
        notes_content: Option<&str>,
    ) -> UpdateSpecArgs {
        UpdateSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            spec: spec_content.map(|s| s.to_string()),
            tasks: tasks_content.map(|s| s.to_string()),
            notes: notes_content.map(|s| s.to_string()),
            operation: operation.to_string(),
        }
    }

    /// Create test arguments for delete_spec
    pub fn delete_spec_args(&self, project_name: &str, spec_name: &str) -> DeleteSpecArgs {
        DeleteSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            confirm: "true".to_string(),
        }
    }

    /// Create test arguments for install command
    pub fn install_args(&self, target: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(self.mock_binary_path()),
        }
    }

    /// Create test arguments for install command with explicit binary path
    pub fn install_args_with_binary(&self, target: &str, binary_path: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(binary_path.to_string()),
        }
    }

    /// Create test arguments for uninstall command
    pub fn uninstall_args(&self, target: &str, remove_config: bool) -> UninstallArgs {
        UninstallArgs {
            target: target.to_string(),
            remove_config,
        }
    }

    /// Create test arguments for status command
    pub fn status_args(&self, target: Option<&str>, detailed: bool) -> StatusArgs {
        StatusArgs {
            target: target.map(|s| s.to_string()),
            detailed,
            json: false,
        }
    }

    /// Create test arguments for status command with JSON output for testing
    pub fn status_args_json(&self, target: Option<&str>, detailed: bool) -> StatusArgs {
        StatusArgs {
            target: target.map(|s| s.to_string()),
            detailed,
            json: true,
        }
    }

    /// Execute status command and return parsed structured response for testing
    pub async fn get_status_response(
        &self,
        target: Option<&str>,
        detailed: bool,
    ) -> anyhow::Result<
        crate::types::responses::FoundryResponse<crate::types::responses::StatusResponse>,
    > {
        use crate::cli::commands::status;

        let status_args = self.status_args_json(target, detailed);
        let json_output = status::execute(status_args).await?;
        let response = serde_json::from_str(&json_output)?;
        Ok(response)
    }

    /// Return a realistic binary path without creating actual file
    /// Uses current executable for realistic testing
    fn mock_binary_path(&self) -> String {
        // Use current foundry binary for realistic testing
        std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("/usr/local/bin/foundry"))
            .to_string_lossy()
            .to_string()
    }

    /// Get cursor config path within test environment
    /// Returns the path where ~/.cursor/mcp.json would be created in the isolated test environment
    pub fn cursor_config_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".cursor").join("mcp.json")
    }

    /// Get cursor config directory within test environment
    pub fn cursor_config_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".cursor")
    }

    /// Get claude code config path within test environment
    pub fn claude_code_config_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".claude.json")
    }

    /// Create an invalid binary path for error testing
    pub fn invalid_binary_path(&self) -> String {
        "/definitely/does/not/exist/foundry".to_string()
    }

    /// Create a binary path that exists but is not executable (for platforms that check)
    pub fn non_executable_binary_path(&self) -> String {
        let binary_path = self.temp_dir.path().join("non-executable");
        std::fs::write(&binary_path, b"not executable content").unwrap();
        // Note: We still create this file since some tests might check file existence
        // but execution permission validation happens at runtime, not install time
        binary_path.to_string_lossy().to_string()
    }

    /// Create an existing cursor config with custom content for testing conflict scenarios
    pub fn create_existing_cursor_config(&self, content: &str) -> Result<()> {
        let config_dir = self.cursor_config_dir();
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(self.cursor_config_path(), content)?;
        Ok(())
    }

    /// Execute async test logic with proper environment isolation
    /// Uses a simple approach with a dedicated runtime
    pub fn with_env_async<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        // Set additional environment variables for test isolation
        let cursor_config_dir = self.cursor_config_dir().to_string_lossy().to_string();
        let claude_config_dir = self
            .temp_dir
            .path()
            .join(".claude")
            .to_string_lossy()
            .to_string();

        // Set environment variables for the test
        let _cursor_guard = env_var_guard("CURSOR_CONFIG_DIR", &cursor_config_dir);
        let _claude_guard = env_var_guard("CLAUDE_CONFIG_DIR", &claude_config_dir);

        // Always create a new single-threaded runtime for simplicity and isolation
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for test");
        rt.block_on(f())
    }

    /// Create a mock binary file for testing
    /// Returns the path to the created binary
    pub fn create_mock_binary(&self, name: &str) -> Result<PathBuf> {
        let binary_dir = self.temp_dir.path().join("bin");
        std::fs::create_dir_all(&binary_dir)?;

        let binary_path = binary_dir.join(name);
        std::fs::write(&binary_path, "#!/bin/bash\necho 'Mock binary'")?;

        // Make it executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    /// Create a cursor MCP configuration with the given server entries
    /// Each entry is a tuple of (server_name, command_path)
    pub fn create_cursor_config(&self, servers: &[(&str, &str)]) -> Result<()> {
        let config_dir = self.cursor_config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let mut config = serde_json::Map::new();
        let mut servers_config = serde_json::Map::new();

        // Always include mcpServers field, even if empty
        for (name, command) in servers {
            let mut server_config = serde_json::Map::new();
            server_config.insert(
                "command".to_string(),
                serde_json::Value::String(command.to_string()),
            );
            server_config.insert("args".to_string(), serde_json::Value::Array(vec![]));

            servers_config.insert(name.to_string(), serde_json::Value::Object(server_config));
        }

        config.insert(
            "mcpServers".to_string(),
            serde_json::Value::Object(servers_config),
        );

        let config_content = serde_json::to_string_pretty(&config)?;
        std::fs::write(self.cursor_config_path(), config_content)?;

        Ok(())
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
        // Lock is automatically released when _lock is dropped
    }
}
