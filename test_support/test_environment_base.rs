// Modern TestEnvironment for Isolated Test Execution
//
// This module provides the foundational TestEnvironment implementation used by both
// unit tests (via src/test_environment.rs) and integration tests (via tests/common/test_utils.rs).
//
// ## Key Features
//
// - **Complete Isolation**: Each test gets a unique temporary directory and isolated environment
// - **Modern Rust Testing**: Uses assert_fs and temp-env for reliable test isolation
// - **Cross-Platform**: Works on Unix and Windows with appropriate path handling
// - **Async Support**: Full async/await support with proper runtime management
// - **No Global State**: No mutexes or global locks - pure isolation via environment variables
//
// ## Usage Pattern
//
// All tests should follow this pattern:
//
// ```rust
// #[test]
// fn test_something() {
//     let env = TestEnvironment::new().unwrap();
//
//     let _ = env.with_env_async(|| async {
//         // Your test code here - fully isolated
//         env.create_test_project("my-project").await.unwrap();
//
//         // Use spawn_blocking for sync functions to avoid nested runtimes
//         let project_name_clone = "my-project".to_string();
//         let projects = tokio::task::spawn_blocking(move || {
//             list_projects(&project_name_clone)
//         }).await.unwrap().unwrap();
//         assert_eq!(projects.len(), 1);
//     });
// }
// ```
//
// This follows the testing patterns described in .cursor/rules/testing-patterns.mdc

// Imports are handled by the including file

// Base TestEnvironment implementation shared between unit and integration tests

/// Modern test environment using assert_fs + temp-env
/// Follows the established testing patterns without global mutexes
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    home: PathBuf,
    cursor_config: PathBuf,
    claude_config: PathBuf,
    bin: PathBuf,
}

impl TestEnvironment {
    /// Create a new test environment with isolated directory
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Create isolated directory structure
        let home = temp_dir.path().join("home");
        let cursor_config = temp_dir.path().join(".cursor");
        let claude_config = temp_dir.path().join(".claude");
        let bin = temp_dir.path().join("bin");

        // Ensure directories exist
        fs::create_dir_all(&home)?;
        fs::create_dir_all(&cursor_config)?;
        fs::create_dir_all(&claude_config)?;
        fs::create_dir_all(&bin)?;

        Ok(TestEnvironment {
            temp_dir,
            home,
            cursor_config,
            claude_config,
            bin,
        })
    }

    /// Get the foundry directory path within the test environment
    pub fn foundry_dir(&self) -> PathBuf {
        self.home.join(".foundry")
    }

    /// Get the root path of the test environment
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Join a relative path to the test environment root
    pub fn join(&self, rel: impl AsRef<Path>) -> PathBuf {
        self.temp_dir.path().join(rel)
    }

    /// Get the bin directory for mock executables
    pub fn bin_dir(&self) -> &Path {
        &self.bin
    }

    /// Write a file within the test environment
    pub fn write_file(&self, rel: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
        let path = self.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
        Ok(())
    }

    /// Read a file from within the test environment
    pub fn read_to_string(&self, rel: impl AsRef<Path>) -> Result<String> {
        let path = self.join(rel);
        fs::read_to_string(path).map_err(Into::into)
    }

    /// Create an executable file within the test environment
    pub fn make_executable(&self, rel: impl AsRef<Path>, contents: &str) -> Result<PathBuf> {
        let path = self.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }

        Ok(path)
    }

    /// Get base environment variables for isolation
    fn base_vars(&self) -> Vec<(OsString, Option<OsString>)> {
        let mut vars = Vec::new();

        // Cross-platform HOME
        #[cfg(windows)]
        vars.push((
            OsString::from("USERPROFILE"),
            Some(self.home.clone().into_os_string()),
        ));
        #[cfg(not(windows))]
        vars.push((
            OsString::from("HOME"),
            Some(self.home.clone().into_os_string()),
        ));

        // Cursor and Claude config dirs
        vars.push((
            OsString::from("CURSOR_CONFIG_DIR"),
            Some(self.cursor_config.clone().into_os_string()),
        ));
        vars.push((
            OsString::from("CLAUDE_CONFIG_DIR"),
            Some(self.claude_config.clone().into_os_string()),
        ));

        // PATH: bin first, then original PATH for tool discovery
        let orig_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = OsString::new();
        new_path.push(self.bin.clone().into_os_string());
        #[cfg(windows)]
        new_path.push(";");
        #[cfg(not(windows))]
        new_path.push(":");
        new_path.push(orig_path);
        vars.push((OsString::from("PATH"), Some(new_path)));

        vars
    }

    /// Execute sync code within isolated environment
    pub fn with_env<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        temp_env::with_vars(self.base_vars(), f)
    }

    /// Execute async code within isolated environment
    pub fn with_env_async<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.with_env(|| {
            // Create a new single-threaded runtime for simplicity and isolation
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for test");
            rt.block_on(f())
        })
    }

    /// Execute code with additional environment variables
    pub fn with_env_and_vars<F, T>(&self, extra: &[(OsString, Option<OsString>)], f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let mut vars = self.base_vars();
        vars.extend_from_slice(extra);
        temp_env::with_vars(vars, f)
    }

    /// Execute async code with additional environment variables
    pub fn with_env_and_vars_async<F, Fut, T>(
        &self,
        extra: &[(OsString, Option<OsString>)],
        f: F,
    ) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.with_env_and_vars(extra, || {
            // Create a new single-threaded runtime for simplicity and isolation
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for test");
            rt.block_on(f())
        })
    }

    /// Execute async code with PATH environment including bin directory
    pub fn with_env_and_path_async<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        // Get current PATH and prepend our bin directory
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut path_vec = vec![self.bin.clone().into_os_string()];

        if !current_path.is_empty() {
            #[cfg(windows)]
            path_vec.push(OsString::from(";"));
            #[cfg(not(windows))]
            path_vec.push(OsString::from(":"));
            path_vec.push(current_path);
        }

        let new_path = path_vec.into_iter().collect::<OsString>();
        let extra_vars = &[(OsString::from("PATH"), Some(new_path))];

        self.with_env_and_vars(extra_vars, || {
            // Create a new single-threaded runtime for simplicity and isolation
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for test");
            rt.block_on(f())
        })
    }

    // Helper methods for common test patterns

    // Helper methods are implemented in the including file

    /// Get cursor config path within test environment
    pub fn cursor_config_path(&self) -> PathBuf {
        self.cursor_config.join("mcp.json")
    }

    /// Get cursor config directory within test environment
    pub fn cursor_config_dir(&self) -> PathBuf {
        self.cursor_config.clone()
    }

    /// Get claude code config path within test environment
    pub fn claude_code_config_path(&self) -> PathBuf {
        self.home.join(".claude.json")
    }

    /// Get Claude Code config directory path within test environment
    pub fn claude_config_dir(&self) -> PathBuf {
        self.claude_config.clone()
    }

    /// Get Claude Code agents directory path within test environment
    pub fn claude_agents_dir(&self) -> PathBuf {
        self.claude_config.join("agents")
    }

    /// Get Claude Code subagent file path within test environment
    pub fn claude_subagent_path(&self) -> PathBuf {
        self.claude_agents_dir().join("foundry-mcp-agent.md")
    }

    /// Get Cursor rules directory path within test environment
    pub fn cursor_rules_dir(&self) -> PathBuf {
        self.cursor_config.join("rules")
    }

    /// Get Cursor rules file path within test environment
    pub fn cursor_rules_path(&self) -> PathBuf {
        self.cursor_rules_dir().join("foundry.mdc")
    }

    /// Get Claude commands directory path within test environment
    pub fn claude_commands_dir(&self) -> PathBuf {
        self.claude_config.join("commands").join("foundry")
    }

    /// Get Cursor commands directory path within test environment
    pub fn cursor_commands_dir(&self) -> PathBuf {
        self.cursor_config.join("commands")
    }

    /// Create a cursor MCP configuration with the given server entries
    pub fn create_cursor_config(&self, servers: &[(&str, &str)]) -> Result<()> {
        fs::create_dir_all(&self.cursor_config)?;

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
        fs::write(self.cursor_config_path(), config_content)?;

        Ok(())
    }

    /// Create an existing cursor config with custom content for testing conflict scenarios
    pub fn create_existing_cursor_config(&self, content: &str) -> Result<()> {
        fs::create_dir_all(&self.cursor_config)?;
        fs::write(self.cursor_config_path(), content)?;
        Ok(())
    }

    /// Create a mock binary file for testing
    pub fn create_mock_binary(&self, name: &str) -> Result<PathBuf> {
        fs::create_dir_all(&self.bin)?;

        let binary_path = self.bin.join(name);
        fs::write(&binary_path, "#!/bin/bash\necho 'Mock binary'")?;

        // Make it executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    /// Create a mock claude command that handles the specific commands used by the installation process
    pub fn create_mock_claude_binary(&self) -> Result<PathBuf> {
        fs::create_dir_all(&self.bin)?;

        let binary_path = self.bin.join("claude");

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

        fs::write(&binary_path, script_content)?;

        // Make it executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    /// Create an invalid binary path for error testing
    pub fn invalid_binary_path(&self) -> String {
        "/definitely/does/not/exist/foundry".to_string()
    }

    /// Create a binary path that exists but is not executable (for platforms that check)
    pub fn non_executable_binary_path(&self) -> String {
        let binary_path = self.temp_dir.path().join("non-executable");
        fs::write(&binary_path, b"not executable content").unwrap();
        binary_path.to_string_lossy().to_string()
    }

    /// Verify that Cursor rules template was created with expected content
    pub fn verify_cursor_rules_template(&self) -> Result<()> {
        let rules_path = self.cursor_rules_path();
        if !rules_path.exists() {
            anyhow::bail!("Cursor rules file should exist after installation");
        }

        let rules_content = fs::read_to_string(&rules_path)?;

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

        let subagent_content = fs::read_to_string(&subagent_path)?;

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
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new().expect("Failed to create test environment")
    }
}
