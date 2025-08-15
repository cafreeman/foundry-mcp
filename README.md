# Project Manager MCP

A Model Context Protocol (MCP) server that provides deterministic tools for AI coding assistants to manage project context, specifications, and task lists. It solves the persistent problem of context management in long-term software development projects by providing centralized, structured storage outside of project directories.

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

## Usage

### Cursor Rules Integration

The system includes cursor rules that prompt appropriate tool usage:

- `@setup_project` → triggers `setup_project` tool
- `@create_spec` → triggers `create_spec` tool
- `@load_spec` → triggers `load_spec` tool
- `@execute_task` → triggers `execute_task` prompt with smart context loading

### Example Workflow

1. **Setup Project**: Use `@setup_project [project-name] [description]` to create foundational project context
2. **Create Specification**: Use `@create_spec [description]` to create a new specification with task breakdown
3. **Load Context**: Use `@load_spec [project-name] [spec-id]` to load specification context
4. **Execute Tasks**: Use `@execute_task` to work on tasks while maintaining updated context

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
cargo run
```

## Architecture

The project is organized into the following modules:

- **`models/`**: Core data structures for projects, specifications, and tasks
- **`filesystem/`**: File system operations and directory management
- **`repository/`**: Data access layer for projects and specifications
- **`handlers/`**: MCP tool implementations and server handlers
- **`utils/`**: Utility functions and helpers

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

[License information to be added]

## Status

This project is currently in active development. See the [task list](task_list.md) for current implementation status and roadmap.
