# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Foundry MCP is a CLI tool and MCP server for deterministic project management and AI coding assistant integration. It manages project specifications in `~/.foundry/` directory structure, storing project context outside of actual codebases to prevent directory pollution.

## Development Commands

### Build & Test Commands

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run all tests (118 tests currently passing)
cargo test

# Run with verbose test output
cargo test -- --nocapture

# Run integration tests specifically
cargo test --test integration_tests

# Run specific test
cargo test test_function_name

# Run Clippy (strict linting - all warnings denied)
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Running the CLI

```bash
# Run as MCP server (default behavior)
cargo run

# Run CLI commands during development
cargo run -- --help
cargo run -- create-project "test-project" "vision content" "tech stack content"
cargo run -- list-projects
cargo run -- load-spec "project-name" "spec-id"

# Run with custom foundry directory for testing
cargo run -- --config-dir ./test-config serve --verbose
```

## Architecture Overview

### Core Module Structure

- **`src/main.rs`** - CLI entry point that dispatches to command handlers
- **`src/lib.rs`** - Library exports and `foundry_dir()` utility function
- **`src/cli/`** - Command-line interface with arguments and command implementations
- **`src/core/`** - Core business logic for projects, specs, and validation  
- **`src/types/`** - Type definitions for requests, responses, and data structures
- **`src/utils/`** - Utility functions for paths, timestamps, and formatting

### File System Organization

All project data stored in `~/.foundry/` directory:

```
~/.foundry/PROJECT_NAME/
├── project/
│   ├── vision.md      # High-level product vision
│   ├── tech-stack.md  # Technology decisions  
│   └── summary.md     # Concise summary for context loading
└── specs/
    └── YYYYMMDD_HHMMSS_FEATURE_NAME/
        ├── spec.md        # Feature specification
        ├── task-list.md   # Implementation checklist
        └── notes.md       # Additional context
```

### CLI Command Pattern

All CLI commands follow the same async pattern:

```rust
pub async fn execute(args: CommandArgs) -> Result<CommandResponse> {
    validate_args(&args).context("Invalid arguments")?;
    let result = perform_operation(&args).context("Operation failed")?;
    Ok(build_response(result))
}
```

All responses include:
- Core data (project, spec, etc.)
- `next_steps: Vec<String>` - Workflow guidance for LLMs
- `validation_status: String` - "complete", "partial", or "failed"

## Development Guidelines

### Content Philosophy

**Critical**: The CLI never generates content automatically. LLMs must provide all content as arguments:

- ✅ `create-project "name" "vision content" "tech stack content"`  
- ❌ CLI generating summaries or content from other content

### Error Handling

Use `anyhow` with meaningful context:

```rust
use anyhow::{Context, Result};

std::fs::create_dir_all(&project_dir)
    .context(format!("Failed to create project directory: {}", project_dir.display()))?;
```

### Code Style

Functional programming strongly preferred:

```rust
// ✅ Preferred
let content = tasks.iter()
    .filter(|task| task.status == TaskStatus::Completed)
    .map(|task| format!("- {}\n", task.title))
    .collect::<String>();

// ❌ Avoid
let mut content = String::new();
for task in &tasks {
    if task.status == TaskStatus::Completed {
        content.push_str(&format!("- {}\n", task.title));
    }
}
```

### Testing

- All commands have unit tests in `#[cfg(test)] mod tests`
- Integration tests in `tests/` directory
- Use `TestFoundryEnv` from `src/test_utils.rs` for test isolation
- Tests use `tempfile::TempDir` for file system operations

## Project Context

### Current Status
- Core CLI functionality implemented and tested
- 8 main commands: create-project, analyze-project, create-spec, load-project, load-spec, list-projects, get-foundry-help, validate-content
- Rust 2024 edition with strict Clippy lints
- Designed for both CLI and future MCP server use

### Development Priorities
1. CLI functionality (current focus)
2. MCP server implementation (future)
3. Maintain identical functionality between CLI and MCP interfaces
4. JSON responses with workflow guidance for LLM integration