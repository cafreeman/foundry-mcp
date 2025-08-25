# Foundry CLI Implementation Plan

## Project Overview

Building a Rust CLI tool that manages project structure in `~/.foundry/` to help LLMs maintain context about software projects through structured specifications.

**Core Principle**: Foundry is a **pure file management tool** - LLMs provide ALL content as arguments, we just write it to the correct structured locations with rich parameter guidance.

## Current Status: CLI MVP COMPLETE ✅

**8/8 CLI Commands Implemented and Tested**

The core LLM workflow is complete: **create → list → load → create spec → validate → get help → work**

### ✅ Completed Implementation (Phases 1-5)

**Foundation & Architecture:**

- Complete module hierarchy (cli/, core/, types/, utils/) with proper separation of concerns
- All data structures defined with validation, atomic file operations, CLI framework with clap
- Required fields enforced (LLMs must provide content, Foundry manages structure)

**8 CLI Commands Completed:**

- `foundry create_project` - Project creation with LLM-provided content (vision, tech-stack, summary)
- `foundry list_projects` - Project discovery and metadata extraction
- `foundry load_project` - Load complete project context for LLM sessions
- `foundry create_spec` - Timestamped spec creation with LLM content
- `foundry load_spec` - Spec content retrieval with project context
- `foundry analyze_project` - Pure file management for LLM-analyzed existing projects
- `foundry validate_content` - Content validation with structured feedback and improvement suggestions
- `foundry get_foundry_help` - Comprehensive workflow guidance and content examples

**Core Infrastructure:**

- Spec management: validation, directory management, listing/filtering, content operations
- Timestamp utilities: ISO format, spec naming (YYYYMMDD_HHMMSS_feature_name)
- Content validation engine with type-specific rules and suggestions
- Comprehensive testing: 52 total tests (37 unit + 15 integration) - all passing
- JSON response format with next_steps and workflow_hints for LLM guidance

**Key Design Decisions:**

- Content-agnostic: No content generation or analysis by Foundry
- LLM-provided content as required parameters ensures pure file management
- Rich parameter schemas guide LLM behavior without enforcing content opinions

## Project Structure

```
~/.foundry/PROJECT_NAME/
├── project/
│   ├── vision.md      # High-level product vision (LLM-provided)
│   ├── tech-stack.md  # Technology decisions (LLM-provided)
│   └── summary.md     # Concise summary for context loading (LLM-provided)
└── specs/
    └── YYYYMMDD_HHMMSS_FEATURE_NAME/
        ├── spec.md        # Feature specification (LLM-provided)
        ├── task-list.md   # Implementation checklist (LLM-provided)
        └── notes.md       # Additional context (LLM-provided)
```

**Module Architecture:**

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── cli/                 # CLI commands and argument parsing
├── core/                # Business logic (project, spec, validation, filesystem)
├── types/               # Data structures and response formats
└── utils/               # Utilities (timestamps, paths, formatting)
```

## Outstanding Tasks (Phase 6: Polish)

While the CLI MVP is functionally complete, these polish items remain for production readiness:

### Error Handling and Validation

- [ ] Implement comprehensive error messages
- [ ] Add input validation for all commands
- [ ] Create user-friendly error formatting
- [ ] Handle edge cases (missing directories, permissions, etc.)
- [ ] Add validation for file paths and names

### Documentation and Examples

- [ ] Write comprehensive CLI help documentation
- [ ] Create usage examples for each command
- [ ] Add parameter schema documentation
- [ ] Write troubleshooting guide
- [ ] Create getting started tutorial

### Performance and Reliability

- [ ] Optimize file operations for performance
- [ ] Add proper file locking for concurrent access
- [ ] Implement atomic operations where needed
- [ ] Add progress indicators for long operations
- [ ] Optimize JSON response generation

**Note**: Testing is complete (52 tests passing), core functionality is robust, and all commands work as designed.

## Command Reference

### Core Commands (All Implemented ✅)

**`foundry create_project`** - Create new project with LLM-provided content

- Parameters: project_name, vision (200+ chars), tech_stack (150+ chars), summary (100+ chars)
- Creates: `~/.foundry/PROJECT_NAME/project/` with vision.md, tech-stack.md, summary.md

**`foundry list_projects`** - List all projects with metadata

- Returns: Project names, creation dates, spec counts, validation status

**`foundry load_project`** - Load complete project context for LLM sessions

- Parameters: project_name
- Returns: All project content, available specs, workflow guidance

**`foundry create_spec`** - Create timestamped specification

- Parameters: project_name, feature_name, spec, notes, task_list
- Creates: `~/.foundry/PROJECT/specs/YYYYMMDD_HHMMSS_FEATURE_NAME/`

**`foundry load_spec`** - Load specification content with project context

- Parameters: project_name, [spec_name] (lists if omitted)
- Returns: Spec content, project summary, workflow hints

**`foundry analyze_project`** - Pure file management for LLM-analyzed existing projects

- Parameters: project_name, vision, tech_stack, summary
- Creates: Project structure from LLM analysis results

**`foundry validate_content`** - Validate content against schemas

- Parameters: content_type (vision|tech-stack|summary|spec|notes), content
- Returns: Validation results, improvement suggestions, next steps

**`foundry get_foundry_help`** - Comprehensive workflow guidance

- Parameters: [topic] (workflows|content-examples|project-structure|parameter-guidance)
- Returns: Topic-specific help, examples, parameter schemas

## JSON Response Format

All commands return consistent JSON structure:

```json
{
  "data": {
    /* Command-specific data */
  },
  "next_steps": ["Suggested next actions for LLM"],
  "validation_status": "complete|incomplete|error",
  "workflow_hints": ["Optional workflow guidance"]
}
```

## MCP Server Implementation Phases

**Status**: Ready to implement - CLI foundation is complete and battle-tested.

**Implementation Strategy**: Direct CLI command mapping following PRD guidance: "MCP tools map directly to CLI commands"

### MCP Server Architecture Overview

**Core Design Principles (from PRD):**

- ✅ **Identical functionality** between CLI and MCP interfaces
- ✅ **Same JSON response format** for both CLI and MCP
- ✅ **Rich parameter schemas** with embedded behavioral guidance
- ✅ **LLMs provide content as arguments**, not file paths
- ✅ **Direct mapping**: MCP tools call existing CLI command functions

**Module Structure:**

```
src/
├── main.rs              # Entry point with CLI/MCP mode detection
├── lib.rs               # Library exports
├── cli/                 # CLI commands (existing)
├── mcp/                 # NEW: MCP server implementation
│   ├── mod.rs           # MCP server module exports
│   ├── server.rs        # MCP server startup and configuration
│   ├── tools.rs         # MCP tool definitions and registration
│   └── handlers.rs      # Request routing to CLI commands
├── core/                # Business logic (existing)
├── types/               # Data structures (existing)
└── utils/               # Utilities (existing)
```

### Phase 7: MCP Module Foundation

**Estimated Time**: Week 1

**`src/mcp/mod.rs` - Module exports and structure**

- [ ] Create MCP module with public exports
- [ ] Define MCP-specific types and interfaces
- [ ] Set up module integration with main.rs

**`src/mcp/server.rs` - MCP server startup**

- [ ] Implement MCP server initialization using rust-mcp-sdk
- [ ] Configure async tokio runtime for MCP requests
- [ ] Add server shutdown handling and cleanup
- [ ] Implement stdio transport for MCP communication

**Binary Mode Detection in `src/main.rs`:**

- [ ] Detect CLI vs MCP server mode (default: MCP server if no CLI args)
- [ ] Route to appropriate execution path (CLI commands vs MCP server)
- [ ] Maintain existing CLI functionality unchanged

### Phase 8: MCP Tool Registration

**Estimated Time**: Week 1

**`src/mcp/tools.rs` - Tool definitions using PRD parameter schemas**

- [ ] **`create_project` MCP tool** - Use existing CreateProjectArgs parameter structure
- [ ] **`analyze_project` MCP tool** - Use existing AnalyzeProjectArgs parameter structure
- [ ] **`load_project` MCP tool** - Use existing LoadProjectArgs parameter structure
- [ ] **`create_spec` MCP tool** - Use existing CreateSpecArgs parameter structure
- [ ] **`load_spec` MCP tool** - Use existing LoadSpecArgs parameter structure
- [ ] **`list_projects` MCP tool** - Use existing ListProjectsArgs parameter structure
- [ ] **`validate_content` MCP tool** - Use existing ValidateContentArgs parameter structure
- [ ] **`get_foundry_help` MCP tool** - Use existing GetFoundryHelpArgs parameter structure

**Parameter Schema Implementation:**

- [ ] **Rich MCP parameter schemas** - Embed behavioral guidance in descriptions (from PRD)
- [ ] **Validation constraints** - Use same validation as CLI (minLength, format requirements)
- [ ] **Type definitions** - Map CLI argument types to MCP parameter types

### Phase 9: Request Routing and Response Handling

**Estimated Time**: Week 1

**`src/mcp/handlers.rs` - Route MCP requests to CLI command functions**

- [ ] **Direct function calls** - Route MCP tool requests to existing CLI command execute() functions
- [ ] **Parameter conversion** - Convert MCP request parameters to CLI Args structs
- [ ] **Response formatting** - Return existing CLI JSON responses unchanged (per PRD requirement)
- [ ] **Error handling** - Convert CLI errors to appropriate MCP error responses

**Integration with Existing CLI Logic:**

- [ ] **Zero duplication** - Reuse all existing CLI command implementations
- [ ] **Identical validation** - Use same parameter validation logic
- [ ] **Same file operations** - Use same core business logic for all operations
- [ ] **Consistent responses** - Return identical JSON structure for CLI and MCP

### Phase 10: MCP Testing and Validation

**Estimated Time**: Week 1

**MCP Integration Tests:**

- [ ] **End-to-end MCP workflows** - Test complete LLM workflow through MCP tools
- [ ] **Parameter validation testing** - Ensure MCP parameter validation matches CLI
- [ ] **Response format verification** - Verify identical JSON responses between CLI and MCP
- [ ] **Error handling tests** - Test error scenarios and response formatting

**CLI Compatibility Verification:**

- [ ] **Regression testing** - Ensure existing CLI functionality unchanged
- [ ] **Binary mode switching** - Test CLI vs MCP server mode detection
- [ ] **Parameter consistency** - Verify identical behavior between CLI and MCP interfaces

### Phase 11: MCP Documentation and Deployment

**Estimated Time**: Week 1

**MCP Server Documentation:**

- [ ] **Setup instructions** - How to configure MCP server for Claude/Cursor integration
- [ ] **Tool documentation** - Complete parameter schemas and usage examples
- [ ] **Configuration guide** - MCP server configuration options
- [ ] **Troubleshooting** - Common issues and debugging guidance

**Production Readiness:**

- [ ] **Binary optimization** - Single binary supporting both CLI and MCP modes
- [ ] **Configuration management** - Environment-based configuration for MCP server
- [ ] **Logging and monitoring** - Structured logging for MCP server operations
- [ ] **Performance optimization** - Optimize for MCP request/response cycles

### Technical Implementation Details

**MCP Tool Definition Pattern (following PRD):**

```rust
// Example: create_project MCP tool
pub fn create_project_tool() -> Tool {
    Tool {
        name: "create_project".to_string(),
        description: "Create new project structure with LLM-provided content".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "project_name": {
                    "type": "string",
                    "description": "Descriptive project name using kebab-case"
                },
                "vision": {
                    "type": "string",
                    "description": "High-level product vision (2-4 paragraphs) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Goes into project/vision.md",
                    "minLength": 200
                },
                // ... identical to PRD parameter schemas
            },
            "required": ["project_name", "vision", "tech_stack", "summary"]
        })
    }
}
```

**Request Routing Pattern:**

```rust
// Route MCP requests to existing CLI command functions
async fn handle_create_project(params: Value) -> Result<Value> {
    let args = CreateProjectArgs::from_mcp_params(params)?;
    let response = cli::commands::create_project::execute(args).await?;
    Ok(serde_json::to_value(response)?)
}
```

### Dependencies (Already Available)

- ✅ `rust-mcp-schema = "0.7.2"` - MCP protocol schemas
- ✅ `rust-mcp-sdk = "0.5.1"` - MCP server development kit
- ✅ `tokio = "1.47.1"` - Async runtime for MCP server
- ✅ `serde_json = "1.0.142"` - JSON handling for MCP requests/responses
- ✅ All CLI infrastructure - Complete foundation to build upon

### Success Criteria

**Functional Requirements:**

- [ ] **8 MCP tools** working identically to CLI commands
- [ ] **Identical JSON responses** between CLI and MCP interfaces
- [ ] **Complete LLM workflow** supported: create → list → load → create spec → validate → get help
- [ ] **Binary mode switching** between CLI and MCP server modes

**Quality Requirements:**

- [ ] **Zero regression** - All existing CLI functionality preserved
- [ ] **Comprehensive testing** - MCP integration tests covering all workflows
- [ ] **Production ready** - Logging, error handling, configuration management
- [ ] **Documentation complete** - Setup, usage, and troubleshooting guides

**Timeline**: 5 phases (7-11) over 5 weeks for complete MCP server implementation with testing and documentation.

**Key Dependencies:** clap, serde_json, anyhow, chrono, dirs, rust-mcp-sdk, rust-mcp-schema, tokio
