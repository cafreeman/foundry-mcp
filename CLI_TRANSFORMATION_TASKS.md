# CLI Transformation Task List

## Instructions for LLM Agents

This task list is designed for systematic implementation by LLM agents. Follow these guidelines:

### How to Use This Task List

1. **Work sequentially through phases** - Complete all tasks in a phase before moving to the next
2. **Mark tasks complete** - Check off boxes `[x]` as you complete each task
3. **Preserve existing functionality** - Ensure all current MCP server features continue working
4. **Test after each task** - Run `cargo build` and `cargo test` to verify changes
5. **Update this file** - Keep the checklist current to track progress
6. **Ask for guidance** - If a task is unclear, ask the user for clarification

### Implementation Standards

- **Backward Compatibility**: All existing MCP integrations must continue working
- **Error Handling**: Provide clear, actionable error messages
- **Cross-Platform**: Support macOS, Linux, and Windows
- **Testing**: Write tests for new functionality
- **Documentation**: Update help text and comments

### Dependencies Management

When adding new dependencies, prefer well-maintained crates:
- `clap` for CLI parsing
- `inquire` for interactive prompts
- `indicatif` for progress bars
- `home` for cross-platform paths

---

## Phase 1: Core CLI Infrastructure

### Task 1.1: Add CLI Dependencies
- [ ] Add `clap = { version = "4.4", features = ["derive"] }` to Cargo.toml
- [ ] Add `clap_complete = "4.4"` for shell completions
- [ ] Add `home = "0.5"` for cross-platform home directory detection
- [ ] Add `inquire = "0.7"` for interactive prompts
- [ ] Add `indicatif = "0.17"` for progress indicators
- [ ] Run `cargo update` to ensure compatibility

### Task 1.2: Create CLI Module Structure
- [ ] Create `src/cli/mod.rs` with module declarations
- [ ] Create `src/cli/args.rs` for CLI argument definitions
- [ ] Create `src/cli/config.rs` for configuration management
- [ ] Create `src/cli/commands/mod.rs` for command implementations
- [ ] Create `src/cli/commands/serve.rs` for MCP server mode
- [ ] Create `src/cli/commands/install.rs` for client installation
- [ ] Create `src/cli/commands/project.rs` for project management
- [ ] Update `src/lib.rs` to expose CLI module

### Task 1.3: Define CLI Command Structure
- [ ] Define main `Cli` struct with clap derive macros in `args.rs`
- [ ] Define `Commands` enum with all subcommands
- [ ] Define `ServeArgs` struct for MCP server options
- [ ] Define `InstallArgs` struct with client selection
- [ ] Define `ProjectArgs` struct for project management
- [ ] Add comprehensive help text and examples for each command
- [ ] Add version information using `clap::crate_version!()`

## Phase 2: Refactor Main Entry Point

### Task 2.1: Extract MCP Server Logic
- [ ] Move current `main.rs` MCP server logic to `src/cli/commands/serve.rs`
- [ ] Create `pub async fn run_server(args: ServeArgs) -> Result<()>` function
- [ ] Preserve all existing MCP functionality exactly as-is
- [ ] Add logging configuration from `ServeArgs`
- [ ] Test that MCP server still works identically

### Task 2.2: Implement CLI Dispatcher
- [ ] Replace `main.rs` with CLI argument parsing using clap
- [ ] Add command routing to appropriate handlers
- [ ] Implement default behavior: run serve mode when no subcommand provided
- [ ] Add global error handling for CLI parsing failures
- [ ] Add `--version` and `--help` support

### Task 2.3: Add Global Configuration Options
- [ ] Add global `--verbose`, `--quiet`, `--log-level` flags
- [ ] Add `--config-dir` option for custom base directory
- [ ] Implement configuration precedence: CLI > env vars > defaults
- [ ] Add `LOG_LEVEL` environment variable support
- [ ] Test logging configuration works correctly

## Phase 3: MCP Server Mode (Serve Command)

### Task 3.1: Implement Serve Command Handler
- [ ] Implement `serve.rs` with async `run()` function
- [ ] Accept all existing MCP server functionality via CLI args
- [ ] Add `--port` option for future HTTP transport support
- [ ] Add `--transport` option (stdio as default)
- [ ] Maintain stdio transport as default for backward compatibility

### Task 3.2: Add Server Configuration Options
- [ ] Add `--host` option for future HTTP mode
- [ ] Add `--max-connections` option with reasonable default
- [ ] Add `--timeout` option for tool execution timeouts
- [ ] Add `--backup-retention-days` option (default: 7)
- [ ] Ensure all options are properly validated

### Task 3.3: Enhance Server Logging and Startup
- [ ] Use CLI-configured log level in server mode
- [ ] Add `--log-format` option (json, pretty, compact)
- [ ] Display server startup information when in verbose mode
- [ ] Add graceful shutdown handling with SIGINT/SIGTERM
- [ ] Test server starts and stops cleanly

## Phase 4: Install Command Implementation

### Task 4.1: Create Install Command Structure
- [ ] Implement `InstallArgs` with required and optional fields
- [ ] Support `--client cursor` and `--client claude-desktop` options
- [ ] Add `--global` flag for global vs project-specific installation
- [ ] Add `--dry-run` flag to preview configuration changes
- [ ] Add `--force` flag to overwrite existing configurations

### Task 4.2: Implement Cursor Installation Logic
- [ ] Create function to detect Cursor installation and config locations
- [ ] Handle global config: `~/.cursor/mcp.json`
- [ ] Handle project config: `.cursor/mcp.json`
- [ ] Create/update configuration with proper JSON merging
- [ ] Validate existing configuration before modification
- [ ] Backup existing config files before changes

### Task 4.3: Implement Claude Desktop Installation Logic
- [ ] Handle macOS config: `~/Library/Application Support/Claude/claude_desktop_config.json`
- [ ] Handle Windows config: `%APPDATA%\Claude\claude_desktop_config.json`
- [ ] Handle Linux config: `~/.config/claude/claude_desktop_config.json`
- [ ] Create configuration directories if they don't exist
- [ ] Merge with existing configuration safely
- [ ] Test on multiple platforms if possible

### Task 4.4: Add Installation Verification and Testing
- [ ] Add `--verify` flag to test MCP server startup after installation
- [ ] Check that `project-manager-mcp` binary is accessible in PATH
- [ ] Validate generated configuration files are syntactically correct
- [ ] Provide troubleshooting suggestions on installation failure
- [ ] Display installation summary and next steps

## Phase 5: Project Management Commands

### Task 5.1: Implement List Projects Command
- [ ] Create `list-projects` subcommand handler
- [ ] Display projects in formatted table with:
  - Project name
  - Creation date  
  - Number of specifications
  - Last modified date
- [ ] Add `--json` flag for machine-readable output
- [ ] Add `--sort` option (name, date, specs)
- [ ] Handle case when no projects exist gracefully

### Task 5.2: Implement Clear Projects Command  
- [ ] Create `clear-projects` subcommand with confirmation prompt
- [ ] Add `--force` flag to skip confirmation
- [ ] Add `--project <name>` to clear specific project only
- [ ] Add `--backup` flag to create backup before clearing
- [ ] Show detailed summary of what will be deleted
- [ ] Implement safe deletion with error recovery

### Task 5.3: Implement Create Project Command
- [ ] Create `create-project <name>` subcommand
- [ ] Accept all `setup_project` tool parameters as CLI arguments
- [ ] Add `--interactive` mode for guided project creation
- [ ] Add `--template` option for common project types (web, cli, lib)
- [ ] Validate project name and all parameters before creation
- [ ] Display creation summary and next steps

### Task 5.4: Implement List Specs Command
- [ ] Create `list-specs <project>` subcommand
- [ ] Display specifications in formatted table:
  - Spec ID
  - Title
  - Status (if available)
  - Creation date
  - Task count
- [ ] Add `--json` flag for machine-readable output
- [ ] Add `--detailed` flag for expanded information
- [ ] Handle project not found errors gracefully

## Phase 6: Advanced Utility Commands

### Task 6.1: Implement Show Project Command
- [ ] Create `show-project <name>` subcommand  
- [ ] Display comprehensive project information:
  - Basic metadata
  - Technology stack
  - Vision and goals
  - Statistics (specs, tasks, etc.)
- [ ] Add `--format` option (table, json, yaml)
- [ ] Handle missing projects with helpful error messages

### Task 6.2: Implement Show Spec Command
- [ ] Create `show-spec <project> <spec-id>` subcommand
- [ ] Display full specification details including:
  - Specification content
  - Current tasks with status
  - Development notes
  - Metadata and timestamps
- [ ] Add `--tasks-only` and `--notes-only` filter flags
- [ ] Add `--format` option for output format

### Task 6.3: Implement Export Command
- [ ] Create `export` subcommand with flexible options
- [ ] Support `--project <name>` to export specific project
- [ ] Support `--spec <project> <spec-id>` to export specific spec
- [ ] Add `--format` option (json, yaml, markdown)
- [ ] Add `--output-dir` option with default to current directory
- [ ] Create organized export structure

### Task 6.4: Implement Import Command
- [ ] Create `import <file>` subcommand
- [ ] Support importing projects from JSON/YAML files
- [ ] Add `--merge` flag for existing projects (vs. error)
- [ ] Validate import data structure before processing
- [ ] Show import summary and any conflicts/errors
- [ ] Support batch import of multiple projects

## Phase 7: Configuration and Status Commands

### Task 7.1: Implement Config Command
- [ ] Create `config` subcommand with sub-operations:
  - `config get <key>` - Show configuration value
  - `config set <key> <value>` - Set configuration value  
  - `config list` - Show all configuration
  - `config reset` - Reset to defaults
- [ ] Manage global CLI configuration file
- [ ] Add configuration validation and type checking
- [ ] Support nested configuration keys with dot notation

### Task 7.2: Implement Status Command
- [ ] Create `status` subcommand for system information
- [ ] Show current configuration and data directory paths
- [ ] Check MCP client installations (Cursor, Claude Desktop)
- [ ] Display version information and build details
- [ ] Show storage usage and project counts
- [ ] Add `--verbose` flag for detailed information

### Task 7.3: Implement Doctor Command
- [ ] Create `doctor` subcommand for troubleshooting
- [ ] Check file permissions on data directory
- [ ] Validate directory structure integrity
- [ ] Test MCP client configurations if present
- [ ] Check for common issues and suggest fixes
- [ ] Add `--fix` flag to automatically resolve issues where possible

## Phase 8: Enhanced User Experience

### Task 8.1: Add Shell Completions
- [ ] Generate bash completions using `clap_complete`
- [ ] Add zsh completion support
- [ ] Add fish completion support
- [ ] Create `completions` subcommand to generate completion scripts
- [ ] Add installation instructions for each shell
- [ ] Test completions work correctly

### Task 8.2: Implement Interactive Mode Enhancements
- [ ] Add `--interactive` flag for guided workflows
- [ ] Create interactive project setup wizard using `inquire`
- [ ] Add interactive specification creation flow
- [ ] Implement confirmation prompts for destructive operations
- [ ] Add input validation with helpful error messages

### Task 8.3: Add Progress Indicators and User Feedback  
- [ ] Use `indicatif` for progress bars on long operations
- [ ] Add spinners for network or file operations
- [ ] Display operation summaries and results
- [ ] Add color coding for success/warning/error messages
- [ ] Implement `--quiet` mode to suppress non-essential output

## Phase 9: Testing and Validation

### Task 9.1: Add CLI Integration Tests
- [ ] Create integration tests for all CLI commands
- [ ] Test command parsing and validation
- [ ] Test help text and error message display
- [ ] Mock filesystem operations for safe testing
- [ ] Test configuration file creation and modification
- [ ] Add tests for edge cases and error conditions

### Task 9.2: Add Installation and Configuration Tests
- [ ] Test client installation on available platforms
- [ ] Verify configuration file generation and merging
- [ ] Test conflict resolution with existing configurations
- [ ] Test installation verification functionality
- [ ] Mock client installations for consistent testing

### Task 9.3: Add Command Functionality Tests
- [ ] Test all project management commands thoroughly
- [ ] Verify JSON/table output formats are correct
- [ ] Test file export/import functionality end-to-end
- [ ] Test interactive modes and user input handling
- [ ] Validate error reporting provides actionable feedback

## Phase 10: Documentation and Polish

### Task 10.1: Update Help Documentation
- [ ] Write comprehensive help text for all commands and options
- [ ] Add usage examples for complex operations
- [ ] Create detailed descriptions with context
- [ ] Ensure help text is consistent and well-formatted
- [ ] Test help display works correctly for all commands

### Task 10.2: Create User Documentation
- [ ] Update README.md with CLI usage instructions
- [ ] Create step-by-step installation guide
- [ ] Document client configuration process
- [ ] Create troubleshooting guide with common issues
- [ ] Add platform-specific installation notes

### Task 10.3: Create Command Reference and Examples
- [ ] Document all CLI commands with examples
- [ ] Create example workflows for common use cases
- [ ] Document CLI + MCP integration patterns
- [ ] Add scripting examples for automation
- [ ] Create demo scripts for key workflows

---

## Completion Criteria

### Functionality Checklist
- [ ] All existing MCP server functionality preserved
- [ ] CLI works as drop-in replacement for MCP server mode
- [ ] Install command successfully configures Cursor and Claude Desktop
- [ ] All project management commands work correctly
- [ ] Configuration and status commands provide useful information
- [ ] Error handling is comprehensive and user-friendly

### Quality Checklist  
- [ ] All tests pass (`cargo test`)
- [ ] Code builds without warnings (`cargo build`)
- [ ] Documentation is complete and accurate
- [ ] Help text is comprehensive and useful
- [ ] Cross-platform compatibility verified
- [ ] Performance is acceptable for typical usage

### User Experience Checklist
- [ ] Installation process is smooth and well-documented
- [ ] Commands respond quickly (< 1 second for most operations)
- [ ] Error messages are helpful and actionable
- [ ] Interactive modes are intuitive and guide users effectively
- [ ] Configuration management is straightforward

---

## Notes for Future Development

- Consider adding a web UI subcommand for browser-based project management
- Potential for plugin system to extend CLI functionality  
- Integration with git workflows (git hooks, branch-specific specs)
- Synchronization between multiple machines/users
- Integration with other development tools and IDEs

## Current Status

**Phase Progress**: Phase 0 - Planning Complete ✓

**Next Steps**: Begin Phase 1 - Core CLI Infrastructure