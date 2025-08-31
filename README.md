# Foundry MCP

An MCP (Model Context Protocol) server that provides deterministic tools for AI coding assistants to manage project context, specifications, and task lists. Foundry solves the persistent problem of context management in long-term software development by providing centralized, structured storage outside of project directories.

## Core Value Proposition

Foundry MCP enables AI assistants like Claude to:

- **Maintain persistent context** across development sessions
- **Manage project specifications** with structured task lists
- **Store context outside codebases** to avoid directory pollution
- **Resume complex work** through deterministic file operations

**Primary Use**: MCP server integration with AI development environments
**Bonus Feature**: CLI interface available for testing and debugging MCP tools
**Installation**: Easy one-command setup for Claude Code and Cursor with PATH-based reliability

## Problem Statement

Current AI coding assistant workflows suffer from several critical issues:

- **Inconsistent file management**: Prompt-driven systems create files in unpredictable locations
- **Project directory pollution**: Context management files clutter the actual codebase
- **Context loss**: No reliable way to pause/resume complex development tasks across sessions
- **Error-prone prompting**: Relying on natural language instructions for file system operations leads to inconsistent behavior
- **Installation complexity**: Traditional MCP installation requires manual configuration and binary path management

## Solution Overview

Foundry MCP provides a set of MCP tools that enable deterministic project and specification management through a centralized file system outside of project directories.

### Core Value Propositions

1. **Deterministic operations**: MCP tools eliminate prompt-driven file system errors
2. **Clean project separation**: Context files stored outside project directories
3. **Persistent context**: Natural pause/resume functionality through structured file storage
4. **Hierarchical organization**: Project-level context with individual specs and task lists
5. **Simplified installation**: One-command setup with PATH-based reliability and automatic configuration

## MCP Tools

Foundry provides 8 MCP tools that enable comprehensive project management for AI assistants:

### Project Management

- **`create_project`**: Create new project with vision, tech stack, and summary
- **`analyze_project`**: Create project structure by analyzing existing codebases
- **`load_project`**: Load complete project context for LLM sessions
- **`list_projects`**: List all available projects with metadata

### Specification Management

- **`create_spec`**: Create timestamped specification with task breakdown
- **`load_spec`**: Load specification content with project context

### Content & Workflow

- **`validate_content`**: Validate content against schema requirements
- **`get_foundry_help`**: Get comprehensive workflow guidance and examples

## File System Structure

Foundry stores all project data in `~/.foundry/` to keep your actual codebase clean:

```
~/.foundry/
├── project-name/
│   ├── vision.md          # High-level product vision and goals
│   ├── tech-stack.md      # Technology choices and architecture
│   ├── summary.md         # Concise project summary for quick context
│   └── specs/
│       ├── 20250826_143052_user_auth/
│       │   ├── spec.md        # Feature specification and requirements
│       │   ├── task-list.md   # Implementation checklist (updated by agents)
│       │   └── notes.md       # Design decisions and additional context
│       └── 20250826_145230_payment_system/
│           ├── spec.md
│           ├── task-list.md
│           └── notes.md
```

**Key Benefits:**

- **Clean separation**: Project context never pollutes your actual codebase
- **Persistent context**: Survive git operations, branch switches, and deployments
- **Hierarchical organization**: Project-level vision with feature-specific specs
- **Timestamped specs**: Chronological tracking of feature development

## AI Assistant Workflow

Foundry MCP enables powerful workflows for AI assistants like Claude:

### Typical LLM Development Session

1. **Project Setup**: AI assistant creates project context

   ```
   User: "Help me build a task management web app"
   AI: Uses create_project to establish vision, tech stack, and summary
   ```

2. **Feature Planning**: Break down work into specifications

   ```
   AI: Uses create_spec to create "user_authentication" spec with task breakdown
   AI: Uses create_spec to create "task_crud_operations" spec with implementation plan
   ```

3. **Context Loading**: Resume work with full context

   ```
   User: "Let's work on authentication"
   AI: Uses load_spec to retrieve authentication specification + project context
   AI: Now has complete context to implement features correctly
   ```

4. **Iterative Development**: Maintain context across sessions
   ```
   AI: Updates task-list.md as work progresses
   AI: Uses validate_content to ensure specifications meet quality standards
   AI: Uses get_foundry_help for workflow guidance when needed
   ```

### Key LLM Benefits

- **No context loss**: Project details persist between conversations
- **Structured planning**: Specs and task lists guide implementation
- **Clean codebases**: Context stored outside project directories
- **Resumable work**: Load complete context instantly in any session
- **Reliable installation**: Simple, predictable MCP server setup with PATH-based commands

## MCP Server Setup

### Installation

Foundry MCP can be installed in two ways:

#### Option 1: Build from Source

```bash
# Clone and build
git clone <repository-url>
cd foundry-mcp
cargo build --release

# Start the MCP server
./target/release/foundry-mcp serve
```

#### Option 2: Install MCP Server for AI Development Environments

Foundry MCP provides easy installation commands for popular AI development environments:

```bash
# Install for Claude Code
foundry mcp install claude-code

# Install for Cursor
foundry mcp install cursor

# Check installation status
foundry mcp status

# Uninstall if needed
foundry mcp uninstall cursor
```

**Supported Environments:**

- **Claude Code**: Uses `claude mcp add` CLI commands for server registration
- **Cursor**: Manages `~/.cursor/mcp.json` configuration file automatically

**Installation Behavior:**

- **Always Overwrite**: Installations automatically replace existing configurations
- **PATH Integration**: Both environments use `foundry` command from system PATH
- **No Force Flags**: Simplified installation process with predictable behavior

**Installation Features:**

- ✅ **PATH-based commands** - Uses `foundry` command from system PATH for reliability
- ✅ **Configuration management** - Creates/updates config files without manual editing
- ✅ **Always overwrite** - Installations automatically overwrite existing configurations
- ✅ **Cross-platform support** - Works on macOS, Linux, and Windows
- ✅ **Status reporting** - Comprehensive installation status checking
- ✅ **Clean uninstallation** - Complete removal of configurations

### Integration with AI Clients

Configure your AI development environment to use Foundry MCP:

**Claude Desktop**: Add to your MCP settings
**VS Code**: Configure MCP client extension
**Cursor**: Set up as MCP server

_Note: Specific integration guides coming soon_

### Quick Start

Get Foundry MCP running in your AI development environment in under 2 minutes:

```bash
# 1. Build Foundry MCP
git clone <repository-url>
cd foundry-mcp
cargo build --release

# 2. Install for your preferred environment
./target/release/foundry-mcp mcp install cursor      # For Cursor
./target/release/foundry-mcp mcp install claude-code # For Claude Code

# 3. Verify installation
./target/release/foundry-mcp mcp status

# 4. Start using Foundry MCP tools in your AI environment!
```

**What happens during installation:**

- ✅ **Config creation**: Creates/updates configuration files automatically
- ✅ **Server registration**: Registers Foundry MCP server with your AI environment
- ✅ **PATH integration**: Uses `foundry` command from PATH (Claude Code and Cursor)
- ✅ **Always overwrite**: Existing configurations are automatically replaced
- ✅ **Validation**: Verifies installation was successful

## Development and Testing

For developers and advanced users, Foundry includes a CLI interface to test and debug the MCP tools:

### Running the Server

```bash
# Start MCP server for AI integration
foundry serve

# Start with verbose logging for debugging
foundry serve --verbose
```

### CLI Tool Testing

You can invoke the MCP tools directly from command line for testing:

```bash
# Test project creation
foundry mcp create-project my-test-project --vision "..." --tech-stack "..." --summary "..."

# List all projects
foundry mcp list-projects

# Load project context
foundry mcp load-project my-test-project

# Create a specification
foundry mcp create-spec my-test-project auth_system --spec "..." --notes "..." --tasks "..."

# Get help on workflow
foundry mcp get-foundry-help workflows

# Validate content before creating projects/specs
foundry mcp validate-content --content-type vision --content "..."
```

_Note: CLI usage is primarily for development and testing. The main value is in MCP server integration with AI assistants._

## CLI Commands Reference

Foundry MCP provides a comprehensive CLI interface for all functionality:

### Installation Commands

```bash
# Install MCP server for AI environments
foundry mcp install <target> [--binary-path <path>]

# Available targets: claude-code, cursor
foundry mcp install claude-code    # May require binary path
foundry mcp install cursor         # Uses foundry from PATH

# Installations always overwrite existing configurations

# Specify custom binary path (Claude Code only, Cursor uses PATH)
foundry mcp install claude-code --binary-path /usr/local/bin/foundry-mcp

# Uninstall MCP server
foundry mcp uninstall <target>
foundry mcp uninstall cursor

# Check installation status
foundry mcp status                    # Basic status
foundry mcp status --detailed         # Detailed status with troubleshooting
```

### Project Management Commands

```bash
# Create new project
foundry mcp create-project <name> --vision "<vision>" --tech-stack "<tech>" --summary "<summary>"

# Analyze existing codebase
foundry mcp analyze-project <name> --vision "<vision>" --tech-stack "<tech>" --summary "<summary>"

# List all projects
foundry mcp list-projects

# Load project context
foundry mcp load-project <name>
```

### Specification Management Commands

```bash
# Create new specification
foundry mcp create-spec <project> <feature> --spec "<spec>" --notes "<notes>" --tasks "<tasks>"

# Load specification
foundry mcp load-spec <project> <spec>

# Update specification
foundry mcp update-spec <project> <spec> --spec "<content>" --operation <replace|append>

# Delete specification
foundry mcp delete-spec <project> <spec>
```

### Utility Commands

```bash
# Validate content before creating
foundry mcp validate-content --content-type <type> --content "<content>"

# Get workflow help
foundry mcp get-foundry-help [topic]

# Available help topics: workflows, content-examples, project-structure, parameter-guidance
foundry mcp get-foundry-help workflows
```

### Command Examples

```bash
# Complete workflow example
foundry mcp create-project my-web-app \
  --vision "Build a modern task management web application" \
  --tech-stack "React frontend, Node.js backend, PostgreSQL database" \
  --summary "Full-stack task management app with user authentication"

foundry mcp create-spec my-web-app user-auth \
  --spec "Implement user authentication system with JWT tokens" \
  --notes "Use bcrypt for password hashing, JWT for session management" \
  --tasks "- [ ] Set up user model and database schema\n- [ ] Implement registration endpoint\n- [ ] Add login/logout functionality"

# Check what's installed
foundry mcp status --detailed

# Install for Cursor development
foundry mcp install cursor
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
# Start MCP server
cargo run -- serve

# Start with verbose logging for debugging
cargo run -- serve --verbose

# Test CLI commands for development
cargo run -- --help
cargo run -- mcp list-projects
cargo run -- mcp create-project test-proj --vision "..." --tech-stack "..." --summary "..."
```

## Architecture

Foundry MCP is built with a modular Rust architecture:

### Core Modules

- **`mcp/`**: MCP server implementation with tool definitions and handlers
- **`cli/`**: Command-line interface for development and testing
- **`core/`**: Business logic for projects, specifications, and validation
- **`types/`**: Data structures and response formats
- **`utils/`**: Timestamp handling, paths, and formatting utilities

### Key Design Principles

- **MCP-first architecture**: All functionality exposed as MCP tools
- **CLI reuses MCP tools**: Command-line interface calls the same tool implementations
- **Pure file management**: No content generation - LLMs provide all content
- **Structured storage**: Deterministic file organization in ~/.foundry/

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

### ✅ Production Ready

Foundry MCP is **feature-complete** and production-ready:

- **8 MCP Tools**: Full project and specification management
- **CLI Interface**: All MCP tools available via command line for testing
- **Installation System**: Easy MCP server installation for Claude Code and Cursor
- **Robust Architecture**: Comprehensive error handling and validation
- **Clean Codebase**: 135 tests passing, zero compiler warnings
- **Documentation**: Complete implementation and usage guides

### 🧪 Testing

Comprehensive test coverage: **166 tests passing**

- **121 Unit tests** for all core business logic
- **45 Integration tests** for complete end-to-end workflows
- **Installation testing** with full filesystem isolation
- **Cross-platform compatibility** (Unix/Windows)
- **MCP protocol compatibility** tests
- **File system operation** tests with perfect isolation

### 🚀 Recent Improvements

**Latest Updates (August 2025):**

- **Simplified Installation**: Removed `--force` flag complexity - installations now always overwrite existing configurations
- **Enhanced Cursor Support**: Cursor installation now uses PATH-based `"foundry"` command for better reliability
- **Improved Error Handling**: Cleaner error messages without force flag suggestions
- **Code Simplification**: Removed ~500 lines of complex conditional logic
- **Better Testing**: Comprehensive test coverage with 166 tests passing

**Breaking Changes:**

- `--force` flag no longer available on install/uninstall commands
- Cursor installation no longer accepts `--binary-path` parameter
- Installations always overwrite existing configurations

### 📋 Optional Enhancements

See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for optional production improvements:

- **Phase 14**: Enhanced server transport and configuration (optional)
- All core functionality is complete and ready for use
