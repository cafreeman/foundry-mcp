# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Foundry MCP is a comprehensive CLI tool and Model Context Protocol (MCP) server that provides deterministic tools for AI coding assistants to manage project context, specifications, and task lists. It operates in two modes:

- **CLI Mode**: Direct command-line project management and client configuration
- **MCP Server Mode**: Traditional MCP server for AI assistant integration (default when no command provided)

## Common Commands

### Development Commands

Build the project:
```bash
cargo build
cargo build --release
```

Run tests:
```bash
cargo test
```

Run the application:
```bash
# Run as MCP server (default behavior)
cargo run

# Run CLI commands
cargo run -- --help
cargo run -- serve --verbose
cargo run -- install --client cursor
```

Run with development logging:
```bash
cargo run -- serve --log-level debug --log-format pretty
```

### CLI Testing

Test CLI functionality during development:
```bash
# Test with development binary
./target/debug/foundry-mcp --help
./target/debug/foundry-mcp serve --verbose

# Test local installation
cargo install --path .
~/.cargo/bin/foundry-mcp --help
```

### Linting and Code Quality

Currently no specific lint command is configured. Use standard Rust tooling:
```bash
cargo clippy
cargo fmt
```

## Architecture Overview

### Core Module Structure

- **`src/cli/`** - Command-line interface components
  - `args.rs` - CLI argument definitions using clap
  - `config.rs` - Configuration management with file and environment support  
  - `commands/` - Command implementations (serve, install, project management)

- **`src/models/`** - Core data structures
  - `base.rs` - Project and shared models
  - `specification.rs` - Specification-related models
  - `task.rs` - Task and task list models

- **`src/repository/`** - Data access layer
  - `project.rs` - Project data operations
  - `specification.rs` - Specification data operations

- **`src/filesystem/`** - File system operations
  - `manager.rs` - Safe file system operations and directory management

- **`src/handlers/`** - MCP tool implementations
  - `server.rs` - Main MCP server handler
  - `setup_project.rs`, `create_spec.rs`, `load_spec.rs`, `update_spec.rs` - MCP tool handlers

- **`src/utils/`** - Utility functions
  - `validation.rs` - Input validation
  - `formatting.rs` - Output formatting
  - `id_generation.rs` - ID generation utilities

### Key Design Patterns

1. **Entry Point Dispatch**: `main.rs` serves as a CLI dispatcher, defaulting to MCP server mode when no subcommand is provided

2. **Configuration Precedence**: CLI args > environment variables > config file > defaults

3. **Dual Mode Operation**: Full backward compatibility as MCP server while providing comprehensive CLI functionality

4. **File System Structure**: Projects stored in `~/.foundry/` with organized hierarchy:
   ```
   ~/.foundry/
   ├── project-name/
   │   ├── project/
   │   │   ├── metadata.json
   │   │   ├── tech-stack.md
   │   │   └── vision.md
   │   └── specs/
   │       └── YYYYMMDD_feature_name/
   │           ├── metadata.json
   │           ├── spec.md
   │           ├── task-list.md
   │           └── notes.md
   ```

### MCP Integration

The server provides these MCP tools:
- **setup_project** - Create project context documents
- **create_spec** - Create new specification with task breakdown  
- **load_spec** - Load specification with full project context
- **update_spec** - Update tasks and notes in specifications

The `execute_task` prompt provides guided task execution workflow.

## Development Guidelines

### Code Style

- Uses `#![deny(clippy::all)]` for strict linting
- Comprehensive error handling with custom `ProjectManagerError` type
- Async/await throughout with Tokio runtime
- Strong typing with serde serialization for data persistence

### Testing Strategy

The project has **118 tests passing** with coverage across:
- Unit tests for core modules
- Integration tests for MCP protocol  
- End-to-end workflow tests
- CLI integration tests (in development)

### Implementation Status

**Completed Phases:**
- Phase 1: Core CLI Infrastructure ✓
- Phase 2: Refactor Main Entry Point ✓  
- Phase 3: MCP Server Mode (Serve Command) ✓

**Current Development:**
- Phase 4: Install Command Implementation (Client configuration)
- Phase 5: Project Management Commands
- Phase 6: Specification Management Commands

See `CLI_TRANSFORMATION_TASKS.md` for detailed implementation progress.

### Configuration Management

Configuration sources in precedence order:
1. CLI arguments (`--log-level debug`)
2. Environment variables (`LOG_LEVEL=debug`)  
3. Configuration file (`~/.foundry/config.toml`)
4. Built-in defaults

### Error Handling

Uses `anyhow` for error handling with user-friendly messages. The `ProjectManagerError` type provides structured error information for different failure scenarios.

### Security Considerations

The `src/security.rs` module handles security aspects including input validation and safe file system operations.