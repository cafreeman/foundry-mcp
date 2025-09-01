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

# Run all tests (166 tests currently passing)
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
├── vision.md      # High-level product vision
├── tech-stack.md  # Technology decisions
├── summary.md     # Concise summary for context loading
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

Functional programming **STRONGLY** preferred (from `.cursor/rules/rust-standards.mdc`):

```rust
// ✅ Preferred - Use functional iterators
let content = tasks.iter()
    .filter(|task| task.status == TaskStatus::Completed)
    .map(|task| format!("- {}\n", task.title))
    .collect::<String>();

// ❌ Avoid - Imperative loops with mutable state
let mut content = String::new();
for task in &tasks {
    if task.status == TaskStatus::Completed {
        content.push_str(&format!("- {}\n", task.title));
    }
}
```

### Clippy Configuration (STRICT)

All warnings are denied. Always run before committing:

```toml
[lints.clippy]
all = "deny"
redundant_clone = "deny"
map_flatten = "deny"
manual_ok_or = "deny"
option_if_let_else = "deny"
redundant_closure = "deny"
```

### Testing (Modern Pattern)

Foundry uses `assert_fs` + `temp-env` for perfect test isolation:

```rust
// ✅ Use this pattern for all filesystem tests
#[test]
fn test_name() {
    let env = TestEnvironment::new().unwrap();
    let _ = env.with_env_async(|| async {
        // Test logic with real filesystem operations
        // Perfect isolation - HOME, CURSOR_CONFIG_DIR, etc. are temporary
    });
}

// ❌ Avoid this old pattern
#[tokio::test]
async fn test_name() {
    // No isolation, potential interference
}
```

**Key Points:**
- Unit tests in `#[cfg(test)] mod tests`
- Integration tests in `tests/` directory  
- Use `TestEnvironment` from `src/test_utils.rs` for isolation
- Real filesystem operations, not mocking
- Automatic cleanup and cross-platform support

## Project Context

### Current Status

- **Production ready**: Core CLI functionality implemented and tested
- **11 main commands**: 
  - Project: create-project, analyze-project, load-project, list-projects
  - Spec: create-spec, load-spec, update-spec, delete-spec
  - Utility: validate-content, get-foundry-help
  - Installation: install, uninstall, status
- **166 tests passing** (121 unit + 45 integration)
- **Rust 2024 edition** with strict Clippy lints (all warnings denied)
- **Dual interface**: CLI commands and MCP server in single binary

### Architecture Principles

1. **MCP-first design**: All functionality exposed as MCP tools
2. **CLI reuses MCP tools**: Command-line interface calls same implementations
3. **Content agnostic**: Foundry manages structure, LLMs provide content
4. **Workflow guidance**: All responses include next_steps for AI assistants
5. **Clean separation**: Project context stored outside codebases (`~/.foundry/`)

## Installation Commands

Foundry includes installation management for AI development environments:

```bash
# Install MCP server for supported environments
cargo run -- install claude-code  # May require --binary-path
cargo run -- install cursor       # Uses foundry from PATH

# Check installation status (comprehensive troubleshooting)
cargo run -- status --detailed

# Uninstall configurations
cargo run -- uninstall cursor
```

**Key Features:**
- Always overwrites existing configurations (no --force flag needed)
- PATH-based reliability for Cursor integration
- Comprehensive status checking with troubleshooting guidance
- Cross-platform support (macOS, Linux, Windows)
