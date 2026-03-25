# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Foundry is a **CLI-only** Rust tool for scaffolding and reading project/spec markdown under `~/.foundry/`, plus optional git backup, a `.foundry` symlink bridge, **JSON stdout for inventory and spec workflow commands** (no flag), and installation of bundled skills. There is **no MCP server** (removed in 0.8.0). The crate uses **Rust 2024**; **`rust-version` in `Cargo.toml`** is the MSRV for building from source.

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

## Versioning and changelog

- **`Cargo.toml` `version`**: Must equal the **latest published** crates.io release between ships; unreleased work does not get a forward-looking version bump in-tree.
- **`CHANGELOG.md`**: Accumulate changes under **`## [Unreleased]`**; dated sections and version bumps are handled by **`cargo release`** per **`release.toml`** (see **`RELEASE.md`**).

## Architecture Overview

### Core Module Structure

- **`src/main.rs`** - `clap` CLI entry (`list`, `project`, `spec` workflow, `link`, `git`, skills, `status`); JSON vs text per subcommand (no global `--json`)
- **`src/lib.rs`** - `foundry_dir()` and module exports for tests/library use
- **`src/core.rs`** + **`src/core/`** - `paths`, `project`, `spec_store`, `spec_id`, `spec_meta`, `task_parse`, `workflow`, `completed`, `git`, `link`, `skill_install`, `fsutil`, `names`
- **`src/types.rs`** + **`src/types/`** - Small structs (`Project`, `Spec`)
- **`src/skills.rs`** - `include_str!` bundled skill markdown (four files)
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
        ├── meta.json      # Optional workflow hints
        ├── spec.md        # Feature specification
        ├── task-list.md   # Markdown checkboxes drive derived phase
        └── notes.md       # Additional context
└── completed/
    └── <same-spec-id>/
        ├── summary.md     # Agent-written shipped-work record (after collapse)
        └── archive/       # Former spec files moved on finalize
```

### CLI behavior

- Commands are synchronous; inventory/spec workflow commands print **JSON**; `project load`, `spec load`, `link`, `git`, and skill installs print **plain text**; exit codes signal success/failure.
- Foundry does **not** author spec/vision prose; it scaffolds empty files (except `meta.json` on spec init). Agents write markdown—including **`summary.md`** on collapse.
- Optional git integration shells out to the system `git` binary (`commit_project_subtree` for collapse).

## Development Guidelines

### Content Philosophy

**Critical**: The CLI does not generate spec or summary prose. LLMs edit markdown files (including completed-work `summary.md` after `spec collapse prepare`).

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

- **CLI + skills**: `list` / `status` / `spec status|instructions|collapse` / `project init` / `spec init` → JSON; `project load`, `spec load`, deletes, `link`, `git`, skill commands → text
- **Rust 2024** with strict Clippy lints (`cargo clippy --all-targets -- -D warnings`)
- **Content-agnostic**: scaffolds files and filesystem moves; agents author markdown (including completed-work summaries)

### Architecture Principles

1. **CLI-only surface**: No MCP protocol; structured **JSON stdout** for workflow commands, not an in-process tool RPC
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
