# Project Manager MCP

A comprehensive CLI tool and Model Context Protocol (MCP) server that provides deterministic tools for AI coding assistants to manage project context, specifications, and task lists. It solves the persistent problem of context management in long-term software development projects by providing centralized, structured storage outside of project directories.

## 🆕 CLI Mode Available

As of recent updates, Project Manager MCP now includes a full-featured command-line interface while maintaining complete backward compatibility as an MCP server. You can use it both ways:

- **CLI Mode**: Direct command-line project management and client configuration
- **MCP Server Mode**: Traditional MCP server for AI assistant integration (default when no command provided)

## Problem Statement

Current AI coding assistant workflows suffer from several critical issues:

- **Inconsistent file management**: Prompt-driven systems create files in unpredictable locations
- **Project directory pollution**: Context management files clutter the actual codebase
- **Context loss**: No reliable way to pause/resume complex development tasks across sessions
- **Error-prone prompting**: Relying on natural language instructions for file system operations leads to inconsistent behavior

## Solution Overview

Project Manager MCP provides a set of MCP tools that enable deterministic project and specification management through a centralized file system outside of project directories.

### Core Value Propositions

1. **Deterministic operations**: MCP tools eliminate prompt-driven file system errors
2. **Clean project separation**: Context files stored outside project directories
3. **Persistent context**: Natural pause/resume functionality through structured file storage
4. **Hierarchical organization**: Project-level context with individual specs and task lists

## Features

### MCP Tools

- **`setup_project`**: Create project context documents for any software project
- **`create_spec`**: Create new specification with task breakdown
- **`load_spec`**: Load specification with full project context

### MCP Prompts

- **`execute_task`**: Guide agents through task execution with proper context loading

## File System Structure

```
~/.project-manager-mcp/
├── project-name/
│   ├── project/
│   │   ├── tech-stack.md
│   │   └── vision.md
│   └── specs/
│       ├── 20250815_feature_name/
│       │   ├── spec.md
│       │   ├── task-list.md
│       │   └── notes.md
│       └── 20250816_other_feature/
│           ├── spec.md
│           ├── task-list.md
│           └── notes.md
```

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd project-manager-mcp

# Build the project
cargo build --release

# The binary will be available at target/release/project-manager-mcp
```

## Quick Start

### CLI Usage

```bash
# Get help for all commands
project-manager-mcp --help

# Start MCP server (default behavior)
project-manager-mcp
# or explicitly:
project-manager-mcp serve

# Start with enhanced logging
project-manager-mcp serve --verbose --log-format json

# Install for Cursor or Claude Desktop
project-manager-mcp install --client cursor
project-manager-mcp install --client claude-desktop

# Project management commands (coming in Phase 5)
project-manager-mcp list-projects
project-manager-mcp create-project "My New Project"
```

### MCP Server Mode

When running as an MCP server (default behavior), the system includes cursor rules that prompt appropriate tool usage:

- `@setup_project` → triggers `setup_project` tool
- `@create_spec` → triggers `create_spec` tool
- `@load_spec` → triggers `load_spec` tool
- `@execute_task` → triggers `execute_task` prompt with smart context loading

### Example Workflow

1. **Setup Project**: Use `@setup_project [project-name] [description]` to create foundational project context
2. **Create Specification**: Use `@create_spec [description]` to create a new specification with task breakdown
3. **Load Context**: Use `@load_spec [project-name] [spec-id]` to load specification context
4. **Execute Tasks**: Use `@execute_task` to work on tasks while maintaining updated context

## CLI Commands Reference

### Global Options

Available with all commands:

```bash
--verbose              Enable verbose output with detailed logging
--quiet                Suppress non-essential output  
--log-level LEVEL      Set log level (trace, debug, info, warn, error)
--config-dir DIR       Use custom configuration directory
--help                 Show help information
--version              Show version information
```

Environment variable support:
```bash
export LOG_LEVEL=debug  # Set default log level
```

### Server Command

Start the MCP server (default when no subcommand provided):

```bash
project-manager-mcp serve [OPTIONS]
```

**Options:**
- `--port PORT` - Port for HTTP transport (default: 3000, future use)
- `--transport MODE` - Transport mode, currently only "stdio" (default: stdio) 
- `--host HOST` - Host for HTTP mode (default: localhost, future use)
- `--max-connections NUM` - Maximum concurrent connections (default: 10)
- `--timeout SECONDS` - Tool execution timeout (default: 300)
- `--backup-retention-days DAYS` - Backup file retention (default: 7)
- `--log-format FORMAT` - Log format: pretty, json, compact (default: pretty)

**Examples:**
```bash
# Default MCP server
project-manager-mcp

# Explicit serve with custom settings
project-manager-mcp serve --verbose --max-connections 20 --timeout 600

# JSON logging for production monitoring
project-manager-mcp serve --log-format json --log-level info

# Quiet mode with minimal output
project-manager-mcp serve --quiet --log-level warn
```

### Install Command

Configure AI clients to use this MCP server:

```bash
project-manager-mcp install --client CLIENT [OPTIONS]
```

**Supported Clients:**
- `cursor` - Configure Cursor IDE
- `claude-desktop` - Configure Claude Desktop app

**Options:**
- `--global` - Install globally vs project-specific (future use)
- `--dry-run` - Preview configuration changes without applying (future use)
- `--force` - Overwrite existing configurations (future use)

**Examples:**
```bash
# Install for Cursor
project-manager-mcp install --client cursor

# Install for Claude Desktop  
project-manager-mcp install --client claude-desktop
```

### Configuration

Configuration precedence (highest to lowest):
1. CLI arguments (`--log-level debug`)
2. Environment variables (`LOG_LEVEL=debug`)
3. Configuration file (`~/.project-manager-mcp/config.toml`)
4. Built-in defaults

### Backward Compatibility

**100% backward compatible** - all existing MCP server integrations continue working unchanged. When run without arguments, the tool starts in MCP server mode exactly as before.

## Troubleshooting

### Common Issues

**MCP Server Won't Start:**
```bash
# Check with verbose logging
project-manager-mcp serve --verbose

# Verify with minimal configuration
project-manager-mcp serve --log-level debug
```

**CLI Command Not Found:**
```bash
# Verify installation
cargo build --release
./target/release/project-manager-mcp --version

# Or run directly with cargo
cargo run -- --help
```

**Configuration Issues:**
```bash
# Check current configuration
project-manager-mcp config list  # (Future Phase 5)

# Use custom config directory
project-manager-mcp --config-dir ./test-config serve --verbose

# Reset environment
unset LOG_LEVEL
```

**Logging Problems:**
```bash
# Different log formats
project-manager-mcp serve --log-format json     # Machine readable
project-manager-mcp serve --log-format compact  # Minimal output
project-manager-mcp serve --log-format pretty   # Human readable (default)

# Adjust verbosity
project-manager-mcp serve --verbose      # Debug level
project-manager-mcp serve --quiet        # Warnings only
project-manager-mcp serve --log-level trace  # Maximum detail
```

### Getting Help

```bash
# General help
project-manager-mcp --help

# Command-specific help
project-manager-mcp serve --help
project-manager-mcp install --help

# Version information
project-manager-mcp --version
```

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
# Run as MCP server (default)
cargo run

# Run specific CLI commands  
cargo run -- --help
cargo run -- serve --verbose
cargo run -- install --client cursor

# Development with custom config
cargo run -- serve --config-dir ./test-config --log-level debug
```

## Architecture

The project is organized into the following modules:

- **`cli/`**: Command-line interface with args, config, and command handlers
- **`models/`**: Core data structures for projects, specifications, and tasks
- **`filesystem/`**: File system operations and directory management
- **`repository/`**: Data access layer for projects and specifications
- **`handlers/`**: MCP tool implementations and server handlers
- **`utils/`**: Utility functions and helpers

### CLI Architecture

- **`src/cli/args.rs`**: Command-line argument definitions using clap
- **`src/cli/config.rs`**: Configuration management with file and environment support
- **`src/cli/commands/`**: Command implementations
  - `serve.rs`: MCP server mode with enhanced CLI integration
  - `install.rs`: Client configuration for Cursor/Claude Desktop
  - `project.rs`: Project management commands (Phase 5+)
- **`src/main.rs`**: CLI dispatcher with configuration precedence handling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

[License information to be added]

## Implementation Status

### ✅ Completed Phases

- **Phase 1**: Core CLI Infrastructure
- **Phase 2**: Refactor Main Entry Point  
- **Phase 3**: MCP Server Mode (Serve Command)

### 🚧 Current Development

- **Phase 4**: Install Command Implementation (Client configuration)
- **Phase 5**: Project Management Commands
- **Phase 6**: Specification Management Commands

### 📋 Progress Tracking

- See [CLI_TRANSFORMATION_TASKS.md](CLI_TRANSFORMATION_TASKS.md) for detailed implementation progress
- See [task_list.md](task_list.md) for original project roadmap
- All existing MCP server functionality remains fully operational

### 🧪 Testing

Current test coverage: **118 tests passing**
- Unit tests for all core modules
- Integration tests for MCP protocol
- End-to-end workflow tests
- CLI integration tests (in development)
