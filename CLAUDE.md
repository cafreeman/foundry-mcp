# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Foundry is a **CLI-only** Rust tool for scaffolding and reading project/spec markdown under `~/.foundry/`, plus optional git backup, a `.foundry` symlink bridge, and installation of bundled agent skills. There is **no MCP server** in current versions (removed in 0.8.0).

## Development Commands

### Build & Test Commands

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run all tests
cargo test

# Run with verbose test output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name

# Run Clippy (strict linting - all warnings denied)
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt
```

### Running the CLI

```bash
cargo run -- --help
cargo run -- project init my-project
cargo run -- project load my-project
cargo run -- spec init my-project my-feature
cargo run -- link my-project
```

Integration tests set a temporary `HOME` and invoke `CARGO_BIN_EXE_foundry`.

## Architecture Overview

### Core Module Structure

- **`src/main.rs`** - `clap` CLI entry (project, spec, link, git, install/update/uninstall, status)
- **`src/lib.rs`** - `foundry_dir()` and module exports for tests/library use
- **`src/core/`** - Paths, project/spec storage, git backup, symlink bridge, skill installer
- **`src/types/`** - Small structs (`Project`, `Spec`)
- **`src/skills/`** - `include_str!` bundled skill markdown
- **`assets/skills/`** - Source files for bundled skills

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

### CLI behavior

- Commands are synchronous; they print plain text to stdout/stderr and use exit codes.
- Foundry does **not** validate or generate markdown content; it creates empty files and reads them back for `load` commands.
- Optional git integration shells out to the system `git` binary.

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

### Testing

- Unit tests: `#[cfg(test)]` modules in library code (e.g. `core::names`, `core::spec_id`, bundled skills).
- Integration tests: `tests/*.rs` spawn the `foundry` binary with `HOME` pointing at a temp directory (`assert_fs::TempDir`).
- Prefer real filesystem operations; avoid touching the developer’s real `~/.foundry/`.

## Project Context

### Current Status

- **CLI + skills**: `project`, `spec`, `link`, `git`, `install`, `update`, `uninstall`, `status`
- **Rust 2024** with strict Clippy lints (`cargo clippy --all-targets -- -D warnings`)
- **Content-agnostic**: scaffolds empty files; agents edit markdown directly

### Architecture Principles

1. **CLI-only surface**: No MCP protocol or JSON tool schema in-process
2. **Structure, not authorship**: Foundry creates paths and empty files; agents own content
3. **Symlink bridge**: `foundry link` exposes `~/.foundry/<project>/` as `./.foundry`
4. **Optional git backup**: subprocess `git` on `~/.foundry/`
5. **Bundled skills**: `include_str!` assets installed to Claude/Cursor paths

## Skills installation

```bash
cargo run -- install claude-code
cargo run -- install cursor
cargo run -- update
cargo run -- uninstall claude-code
cargo run -- uninstall cursor
cargo run -- status
```

Skill files are overwritten on install/update. Uninstall removes only `foundry_*.md` in the target directories.
