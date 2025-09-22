# Foundry MCP

**Deterministic project context management for AI coding assistants**

An MCP (Model Context Protocol) server that enables AI assistants like Claude and Cursor to maintain persistent project context, specifications, and task lists across development sessions. Stores all context in `~/.foundry/` to keep your codebase clean.

## Installation

### Install from crates.io

```bash
cargo install foundry-mcp
```

## Setup with AI Assistants

After installation, configure Foundry with your AI development environment:

```bash
# For Cursor
foundry install cursor

# For Claude Code
foundry install claude-code

# Verify installation
foundry status
```

That's it! Foundry MCP tools are now available in your AI assistant.

**Installation includes helpful templates:**

- **Cursor**: Gets `.cursor/rules/foundry.mdc` with edit_commands guidance and workflow examples
- **Claude Code**: Gets `~/.claude/agents/foundry-mcp-agent.md` with edit_commands usage and intelligent defaults

## Why Foundry?

- 🎯 **Persistent Context**: Never lose project context between AI sessions
- 🗂️ **Clean Separation**: Project specs stored outside your codebase in `~/.foundry/`
- 📋 **Structured Planning**: Organized specifications with task breakdowns
- 🔄 **Resume Work**: Pick up complex development exactly where you left off
- 🛠️ **Deterministic**: Reliable file operations instead of error-prone AI prompting

## Problem Statement

Current AI coding assistant workflows suffer from critical issues:

- **Inconsistent file management**: Prompt-driven systems create files in unpredictable locations
- **Project directory pollution**: Context management files clutter the actual codebase
- **Context loss**: No reliable way to pause/resume complex development tasks across sessions
- **Error-prone prompting**: Relying on natural language instructions for file system operations leads to inconsistent behavior

## Solution Overview

Foundry MCP provides a set of MCP tools that enable deterministic project and specification management through a centralized file system outside of project directories.

### Core Value Propositions

1. **Deterministic operations**: MCP tools eliminate prompt-driven file system errors
2. **Clean project separation**: Context files stored outside project directories
3. **Persistent context**: Natural pause/resume functionality through structured file storage
4. **Hierarchical organization**: Project-level context with individual specs and task lists
5. **Simplified installation**: One-command setup with PATH-based reliability and automatic configuration

## MCP Tools

Foundry provides 9 MCP tools that enable comprehensive project management for AI assistants:

### Project Management

- **`create_project`**: Create new project with vision, tech stack, and summary
- **`analyze_project`**: Create project structure by analyzing existing codebases
- **`load_project`**: Load complete project context for LLM sessions
- **`list_projects`**: List all available projects with metadata

### Specification Management

- **`create_spec`**: Create timestamped specification with task breakdown
- **`load_spec`**: Load specification content with project context
- **`update_spec`**: Edit spec files using intent-based edit commands with precise anchors and idempotent updates
- **`delete_spec`**: Delete existing specification and all its files

### Content & Workflow

- **`validate_content`**: Validate content against schema requirements
- **`get_foundry_help`**: Get comprehensive workflow guidance and examples

## How It Works

Foundry stores structured project context in `~/.foundry/`:

```
~/.foundry/my-project/
├── vision.md          # Product vision and goals
├── tech-stack.md      # Technology decisions
├── summary.md         # Quick context summary
└── specs/
    └── 20250826_143052_user_auth/
        ├── spec.md        # Feature requirements
        ├── task-list.md   # Implementation checklist
        └── notes.md       # Design decisions
```

**Benefits**: Clean codebase separation • Persistent across git operations • Chronological feature tracking

## Architecture: Backends

Foundry uses a façade plus pluggable backend design to keep domain logic independent of storage.

- Façade: `Foundry<B: FoundryBackend>` centralizes domain logic (spec naming/validation, fuzzy matching) and delegates I/O to a backend.
- Default backend: `FilesystemBackend` preserves the existing on-disk layout and atomic write semantics.
- Edit Engine: Uses `SpecContentStore` implemented by the façade for read/write operations.
- Resource locators: Types include optional `location_hint` and `locator` for UI/deeplink use. The legacy `path` field is retained for compatibility but considered deprecated.

See docs/backends.md for trait contracts, invariants, and a checklist for adding new backends.

### Linear Backend (Experimental)

Foundry includes an experimental Linear backend that maps Projects/Specs/Notes/Tasks to Linear entities via the GraphQL API.

Current capabilities:

- Projects: find-or-create, description upsert
- Project Documents: upsert "Vision" and "Tech Stack" with hidden project markers
- Specs: create Issue with humanized title and hidden spec marker; create Notes document and backlink from the Issue
- Robust transport: shared client with retry/backoff and 429 Retry-After handling

Prerequisites:

- Linear API token in environment: `LINEAR_API_TOKEN` (or `LINEAR_API_KEY`)
- Team selection (one of):
  - `LINEAR_TEAM_ID` (preferred)
  - `LINEAR_TEAM_KEY` (e.g., "ENG")
  - `LINEAR_TEAM_NAME` (exact match)

Quick setup (environment variables):

```bash
export LINEAR_API_TOKEN=...           # or LINEAR_API_KEY
export LINEAR_TEAM_ID=...             # or LINEAR_TEAM_KEY/LINEAR_TEAM_NAME
# optional overrides
export LINEAR_GRAPHQL_ENDPOINT=https://api.linear.app/graphql
export LINEAR_HTTP_TIMEOUT_SECS=30
```

Notes & limitations:

- The Linear backend is not the default runtime backend yet; it is being developed behind a façade to avoid breaking changes.
- Team resolution prefers `LINEAR_TEAM_ID` and falls back to `LINEAR_TEAM_KEY` then `LINEAR_TEAM_NAME`.
- Sub-issue task reconciliation, listing, loading, and deletion phases are planned next.

## AI Assistant Benefits

When you work with AI assistants like Claude or Cursor, Foundry provides:

### 🔄 **Session Continuity**

```
Day 1: "Build a task management app"
→ AI creates project with vision, tech stack, feature specs

Day 5: "Let's work on authentication"
→ AI loads complete context, knows exactly what you're building
```

### 📋 **Structured Development**

- **Project-level context**: Vision, tech decisions, and architecture choices persist
- **Feature specifications**: Detailed requirements with implementation task lists
- **Progress tracking**: Task lists update as AI completes work

### 🧠 **Enhanced AI Performance**

- **No repeated explanations**: AI loads full project context in seconds
- **Consistent decisions**: Technology choices and architecture preserved
- **Better code quality**: Specifications guide implementation details
- **Reduced hallucination**: Structured context prevents AI from making assumptions

### ✏️ **Deterministic Edit Commands**

- **Intent-based commands**: `set_task_status`, `upsert_task`, `append_to_section` only
- **Precise selectors**: `task_text` (exact checkbox text) and `section` (case-insensitive headers)
- **Idempotent updates**: Safe to re-run commands without duplication or side effects
- **Smart error recovery**: Candidate selector suggestions with exact match requirements

### 🤝 **Collaborative User Experience**

- **Option-based guidance**: All tools provide "You can..." suggestions instead of directive commands
- **Content creation acknowledgment**: Tools explicitly recognize AI assistant's role in content generation
- **Workflow efficiency**: Smart guidance for optimal tool selection and usage patterns
- **User decision-making control**: Preserves user agency while providing helpful guidance

## MCP Tools Available

Once installed, AI assistants have access to these tools:

- **`create_project`** - Create new project with vision, tech stack, and summary
- **`analyze_project`** - Create project from existing codebase analysis
- **`load_project`** - Load complete project context for AI sessions
- **`list_projects`** - List all available projects with metadata
- **`create_spec`** - Create timestamped specification with task breakdown
- **`load_spec`** - Load specification content with project context
- **`update_spec`** - Edit spec files using intent-based commands: `set_task_status`, `upsert_task`, `append_to_section`
- **`delete_spec`** - Delete existing specification and all its files
- **`validate_content`** - Validate content against schema requirements
- **`get_foundry_help`** - Get workflow guidance and examples

## Development

### Building

```bash
git clone https://github.com/cafreeman/foundry-mcp.git
cd foundry-mcp
cargo build --release
```

### Testing

```bash
cargo test
```

### Running the MCP Server

```bash
# Start MCP server
cargo run -- serve

# With verbose logging for debugging
cargo run -- serve --verbose
```

### CLI Testing (Optional)

Test MCP tools from the command line:

```bash
# Basic workflow test
cargo run -- mcp create-project test-app --vision "Test app" --tech-stack "Rust" --summary "Testing Foundry"
cargo run -- mcp list-projects
cargo run -- mcp load-project test-app
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` passes
5. Submit a pull request
