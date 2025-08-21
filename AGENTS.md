# Agent Guidelines for foundry-mcp

## Build/Test Commands
- **Build**: `cargo build`
- **Test all**: `cargo test`
- **Test single**: `cargo test test_name`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Run**: `cargo run` (starts MCP server)

## Code Style (Rust 2024 Edition)
- Use `#![deny(clippy::all)]` in lib.rs/main.rs
- **STRONGLY PREFER** functional programming over imperative/OO
- Use iterator methods: `map()`, `filter()`, `collect()`, `fold()` over `for` loops
- Avoid `mut` variables unless absolutely necessary
- Use `Result<T, E>` for fallible operations, `?` operator for error handling
- Prefer `&` references over cloning, minimize allocations
- Use `match`, `if let` over `unwrap()`, `expect()`

## Naming Conventions
- Snake_case for functions, variables: `create_project()`, `spec_id`
- PascalCase for types: `ProjectRepository`, `TaskStatus`
- SCREAMING_SNAKE_CASE for constants
- Descriptive names: `generate_test_project_name()` not `gen_name()`

## Module Organization
- Keep functions focused on single responsibility
- Use meaningful imports: `use project_manager_mcp::models::*;`
- Organize in modules: `models/`, `repository/`, `handlers/`, `utils/`