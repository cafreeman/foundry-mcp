//! Test utilities for Foundry integration tests
//!
//! This module provides the TestEnvironment struct and helper functions
//! that can be used by the test crate.

use crate::cli::args::*;
use crate::types::responses::{InstallResponse, UninstallResponse};
use anyhow::Result;
use std::env;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

/// Global mutex to ensure tests don't interfere with each other when setting environment variables
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Environment variable guard that restores the original value when dropped
pub struct EnvVarGuard {
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
pub fn env_var_guard(key: &str, value: &str) -> EnvVarGuard {
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
/// Uses thread-safe environment variable manipulation following testing best practices
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
            spec: "# Feature Name\n\n## Overview\nThis specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations.\n\n## Requirements\nThe feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage.".to_string(),
            notes: "# Implementation Notes\n\n## Security Considerations\nImplementation notes include important considerations for security, performance, and maintainability.\n\n## Error Handling\nSpecial attention should be paid to error handling and edge cases.\n\n## Dependencies\nConsider using established libraries where appropriate and ensure compatibility with existing system components.".to_string(),
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
    ) -> UpdateSpecArgs {
        let content = match file_type {
            "spec" => "\n## Requirements\nUpdated content for testing that meets the minimum length requirements and provides comprehensive information for the specification update.\n".to_string(),
            _ => "Updated content for testing that meets the minimum length requirements and provides comprehensive information for the specification update.".to_string(),
        };

        let command = match file_type {
            "spec" => serde_json::json!({
                "target": "spec",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Requirements"},
                "content": content
            }),
            "task-list" | "tasks" => {
                if spec_name.contains("lifecycle_feature") || spec_name.contains("lifecycle") {
                    serde_json::json!([{
                        "target": "tasks",
                        "command": "upsert_task",
                        "selector": {"type": "task_text", "value": "Initial setup complete"},
                        "content": "- [x] Initial setup complete"
                    }])
                } else {
                    serde_json::json!([{
                        "target": "tasks",
                        "command": "upsert_task",
                        "selector": {"type": "task_text", "value": "Test task"},
                        "content": "- [ ] Test task"
                    }])
                }
            }
            "notes" => serde_json::json!({
                "target": "notes",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Security Considerations"},
                "content": content
            }),
            _ => panic!("Invalid file_type: {}", file_type),
        };

        // If tasks, we already created an array of commands; otherwise wrap single command in an array
        let commands_json = if file_type == "task-list" || file_type == "tasks" {
            command
        } else {
            serde_json::json!([command])
        };

        UpdateSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            commands: serde_json::to_string(&commands_json).unwrap(),
        }
    }

    /// Create test arguments for update_spec with multiple file updates
    pub fn update_spec_args_multi(
        &self,
        project_name: &str,
        spec_name: &str,
        spec_content: Option<&str>,
        tasks_content: Option<&str>,
        notes_content: Option<&str>,
    ) -> UpdateSpecArgs {
        let mut commands: Vec<serde_json::Value> = Vec::new();

        if let Some(spec) = spec_content {
            commands.push(serde_json::json!({
                "target": "spec",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Implementation"},
                "content": spec
            }));
        }

        if let Some(tasks) = tasks_content {
            commands.push(serde_json::json!({
                "target": "tasks",
                "command": "upsert_task",
                "selector": {"type": "task_text", "value": tasks},
                "content": format!("- [ ] {}", tasks)
            }));
        }

        if let Some(notes) = notes_content {
            commands.push(serde_json::json!({
                "target": "notes",
                "command": "append_to_section",
                "selector": {"type": "section", "value": "## Design Decisions"},
                "content": notes
            }));
        }

        UpdateSpecArgs {
            project_name: project_name.to_string(),
            spec_name: spec_name.to_string(),
            commands: serde_json::to_string(&commands).unwrap(),
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
            json: false,
        }
    }

    /// Create test arguments for install command with JSON output
    pub fn install_args_json(&self, target: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(self.mock_binary_path()),
            json: true,
        }
    }

    /// Create test arguments for install command with explicit binary path
    pub fn install_args_with_binary(&self, target: &str, binary_path: &str) -> InstallArgs {
        InstallArgs {
            target: target.to_string(),
            binary_path: Some(binary_path.to_string()),
            json: false,
        }
    }

    /// Create test arguments for uninstall command
    pub fn uninstall_args(&self, target: &str, remove_config: bool) -> UninstallArgs {
        UninstallArgs {
            target: target.to_string(),
            remove_config,
            json: false,
        }
    }

    /// Create test arguments for uninstall command with JSON output
    pub fn uninstall_args_json(&self, target: &str, remove_config: bool) -> UninstallArgs {
        UninstallArgs {
            target: target.to_string(),
            remove_config,
            json: true,
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

    /// Parse install response JSON for testing
    pub fn parse_install_response(&self, json_response: &str) -> anyhow::Result<InstallResponse> {
        Ok(serde_json::from_str(json_response)?)
    }

    /// Parse uninstall response JSON for testing
    pub fn parse_uninstall_response(
        &self,
        json_response: &str,
    ) -> anyhow::Result<UninstallResponse> {
        Ok(serde_json::from_str(json_response)?)
    }

    /// Execute install command and return parsed response for testing
    pub async fn install_and_parse(&self, target: &str) -> anyhow::Result<InstallResponse> {
        use crate::cli::commands::install;
        let args = self.install_args_json(target);
        let response_json = install::execute(args).await?;
        self.parse_install_response(&response_json)
    }

    /// Execute uninstall command and return parsed response for testing
    pub async fn uninstall_and_parse(
        &self,
        target: &str,
        remove_config: bool,
    ) -> anyhow::Result<UninstallResponse> {
        use crate::cli::commands::uninstall;
        let args = self.uninstall_args_json(target, remove_config);
        let response_json = uninstall::execute(args).await?;
        self.parse_uninstall_response(&response_json)
    }

    /// Execute install command with standard args and return parsed response for testing
    /// This is a compatibility helper for tests that use the old pattern
    pub async fn install_with_args(&self, args: InstallArgs) -> anyhow::Result<InstallResponse> {
        use crate::cli::commands::install;
        // Convert to JSON mode for parsing
        let json_args = InstallArgs {
            target: args.target,
            binary_path: args.binary_path,
            json: true,
        };
        let response_json = install::execute(json_args).await?;
        self.parse_install_response(&response_json)
    }

    /// Execute uninstall command with standard args and return parsed response for testing
    /// This is a compatibility helper for tests that use the old pattern
    pub async fn uninstall_with_args(
        &self,
        args: UninstallArgs,
    ) -> anyhow::Result<UninstallResponse> {
        use crate::cli::commands::uninstall;
        // Convert to JSON mode for parsing
        let json_args = UninstallArgs {
            target: args.target,
            remove_config: args.remove_config,
            json: true,
        };
        let response_json = uninstall::execute(json_args).await?;
        self.parse_uninstall_response(&response_json)
    }

    /// Create test arguments for status command with JSON output for testing
    pub fn status_args_json(&self, target: Option<&str>, detailed: bool) -> StatusArgs {
        StatusArgs {
            target: target.map(|s| s.to_string()),
            detailed,
            json: true,
        }
    }

    /// Execute install command and return human-readable text output for testing
    pub async fn install_text_output(&self, target: &str) -> anyhow::Result<String> {
        use crate::cli::commands::install;
        let args = self.install_args(target); // Uses json: false
        install::execute(args).await
    }

    /// Execute uninstall command and return human-readable text output for testing
    pub async fn uninstall_text_output(
        &self,
        target: &str,
        remove_config: bool,
    ) -> anyhow::Result<String> {
        use crate::cli::commands::uninstall;
        let args = self.uninstall_args(target, remove_config); // Uses json: false
        uninstall::execute(args).await
    }

    /// Execute status command and return parsed structured response for testing
    pub async fn get_status_response(
        &self,
        target: Option<&str>,
        detailed: bool,
    ) -> anyhow::Result<crate::types::responses::StatusResponse> {
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

    /// Get Claude Code agents directory path within test environment
    pub fn claude_agents_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".claude").join("agents")
    }

    /// Get Claude Code subagent file path within test environment
    pub fn claude_subagent_path(&self) -> std::path::PathBuf {
        self.claude_agents_dir().join("foundry-mcp-agent.md")
    }

    /// Get Cursor rules directory path within test environment
    pub fn cursor_rules_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".cursor").join("rules")
    }

    /// Get Cursor rules file path within test environment
    pub fn cursor_rules_path(&self) -> std::path::PathBuf {
        self.cursor_rules_dir().join("foundry.mdc")
    }

    /// Get Claude commands directory path within test environment
    pub fn claude_commands_dir(&self) -> std::path::PathBuf {
        self.temp_dir
            .path()
            .join(".claude")
            .join("commands")
            .join("foundry")
    }

    /// Get Cursor commands directory path within test environment
    pub fn cursor_commands_dir(&self) -> std::path::PathBuf {
        self.cursor_config_dir().join("commands")
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

    /// Verify that Cursor rules template was created with expected content
    pub fn verify_cursor_rules_template(&self) -> Result<()> {
        let rules_path = self.cursor_rules_path();
        if !rules_path.exists() {
            anyhow::bail!("Cursor rules file should exist after installation");
        }

        let rules_content = std::fs::read_to_string(&rules_path)?;

        // Verify essential content sections
        if !rules_content.contains("# Foundry MCP Usage Guide") {
            anyhow::bail!("Rules should contain usage guide header");
        }
        if !rules_content.contains("create_project") || !rules_content.contains("update_spec") {
            anyhow::bail!("Rules should reference Foundry MCP tools");
        }
        if !rules_content.contains("Content Agnostic") {
            anyhow::bail!("Rules should contain core principles");
        }

        Ok(())
    }

    /// Verify that Claude subagent template was created with expected content
    pub fn verify_claude_subagent_template(&self) -> Result<()> {
        let subagent_path = self.claude_subagent_path();
        if !subagent_path.exists() {
            anyhow::bail!("Claude subagent file should exist after installation");
        }

        let subagent_content = std::fs::read_to_string(&subagent_path)?;

        // Verify essential content sections
        if !subagent_content.contains("---") {
            anyhow::bail!("Subagent should contain YAML frontmatter");
        }
        if !subagent_content.contains("foundry-mcp-agent") {
            anyhow::bail!("Subagent should contain agent name");
        }
        if !subagent_content.contains("mcp_foundry_") {
            anyhow::bail!("Subagent should reference MCP tools");
        }
        if !subagent_content.contains("Content Agnostic") {
            anyhow::bail!("Subagent should contain core principles");
        }
        if !subagent_content.contains("IMPORTANT: Append only adds to the END") {
            anyhow::bail!("Subagent should contain critical append guidance");
        }
        if !subagent_content.contains("Content Creation Standards") {
            anyhow::bail!("Subagent should contain content formatting guidelines");
        }

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
            .expect("Failed to create tokio runtime for test - this should never happen");
        rt.block_on(f())
    }

    /// Execute async test logic with PATH environment variable isolation
    /// This is needed for tests that require mock binaries to be in PATH
    pub fn with_env_and_path_async<F, Fut, T>(&self, f: F) -> T
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

        // Create isolated PATH that includes our temp bin directory
        let temp_bin_dir = self
            .temp_dir
            .path()
            .join("bin")
            .to_string_lossy()
            .to_string();
        // Use a minimal PATH that includes our temp bin directory
        // This avoids reading the real PATH environment variable
        let isolated_path = format!("{}:/usr/local/bin:/usr/bin:/bin", temp_bin_dir);

        // Set environment variables for the test
        let _cursor_guard = env_var_guard("CURSOR_CONFIG_DIR", &cursor_config_dir);
        let _claude_guard = env_var_guard("CLAUDE_CONFIG_DIR", &claude_config_dir);
        let _path_guard = env_var_guard("PATH", &isolated_path);

        // Always create a new single-threaded runtime for simplicity and isolation
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for test - this should never happen");
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

    /// Create a mock claude command that handles the specific commands used by the installation process
    /// Returns the path to the created binary
    pub fn create_mock_claude_binary(&self) -> Result<PathBuf> {
        let binary_dir = self.temp_dir.path().join("bin");
        std::fs::create_dir_all(&binary_dir)?;

        let binary_path = binary_dir.join("claude");

        // Create a bash script that handles the specific claude commands
        let script_content = r#"#!/bin/bash
# Mock claude command for testing
case "$1" in
    "--version")
        echo "claude version 1.0.0"
        exit 0
        ;;
    "mcp")
        case "$2" in
            "add")
                # Mock successful MCP server registration
                echo "MCP server 'foundry' added successfully"
                exit 0
                ;;
            "remove")
                # Mock MCP server removal - fail if server doesn't exist
                echo "No MCP server found with name: 'foundry'" >&2
                exit 1
                ;;
            *)
                echo "Unknown mcp command: $2"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unknown command: $1"
        exit 1
        ;;
esac
"#;

        std::fs::write(&binary_path, script_content)?;

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

    /// Create a test project for testing spec functionality
    pub async fn create_test_project(&self, project_name: &str) -> Result<()> {
        use crate::core::ops::create_project;
        let args = self.create_project_args(project_name);
        create_project::run(create_project::Input {
            project_name: args.project_name,
            vision: args.vision,
            tech_stack: args.tech_stack,
            summary: args.summary,
        })
        .await?;
        Ok(())
    }

    /// Create a test spec for testing spec functionality
    pub async fn create_test_spec(
        &self,
        project_name: &str,
        feature_name: &str,
        spec_content: &str,
    ) -> Result<()> {
        use crate::core::ops::create_spec;
        let mut args = self.create_spec_args(project_name, feature_name);
        args.spec = spec_content.to_string();
        create_spec::run(create_spec::Input {
            project_name: args.project_name,
            feature_name: args.feature_name,
            spec: args.spec,
            notes: args.notes,
            tasks: args.tasks,
        })
        .await?;
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
