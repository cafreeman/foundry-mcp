# Foundry CLI Implementation Plan

## Project Overview

Building a Rust CLI tool that manages project structure in `~/.foundry/` to help LLMs maintain context about software projects through structured specifications.

**Core Principle**: Foundry is a **pure file management tool** - LLMs provide ALL content as arguments, we just write it to the correct structured locations with rich parameter guidance.

## MVP Scope: CLI Commands Only

Focus on the 7 core CLI commands identified in the PRD, saving the MCP server implementation for later.

## Current Status

### ✅ Phase 1: Foundation & Core Infrastructure - COMPLETED

- **Project Structure**: Complete module hierarchy (cli/, core/, types/, utils/)
- **Core Types**: All data structures defined with proper validation
- **File System**: Atomic file operations and foundry directory management
- **CLI Framework**: Full command structure with clap integration
- **Validation**: Required fields properly enforced (LLM content must be provided)

### 🔄 Phase 2: Core Commands Implementation - READY TO START

All 7 CLI commands are stubbed and ready for implementation:

- `foundry create_project` - Project creation with LLM content
- `foundry analyze_project` - Codebase analysis
- `foundry create_spec` - Timestamped spec creation
- `foundry load_spec` - Spec content retrieval
- `foundry list_projects` - Project discovery
- `foundry get_foundry_help` - Workflow guidance
- `foundry validate_content` - Content validation

**Key Architectural Decision**: All content fields that LLMs must provide are **required** fields, ensuring the "pure file management" principle is maintained.

## Implementation Architecture

### Core Project Structure

```
src/
├── main.rs              # CLI entry point and argument parsing
├── lib.rs               # Library exports and common types
├── cli/
│   ├── mod.rs           # CLI command definitions and routing
│   ├── commands/        # Individual command implementations
│   │   ├── mod.rs
│   │   ├── create_project.rs
│   │   ├── analyze_project.rs
│   │   ├── create_spec.rs
│   │   ├── load_spec.rs
│   │   ├── list_projects.rs
│   │   ├── get_foundry_help.rs
│   │   └── validate_content.rs
│   └── args.rs          # CLI argument structures
├── core/
│   ├── mod.rs           # Core business logic
│   ├── project.rs       # Project structure management
│   ├── spec.rs          # Spec management
│   ├── filesystem.rs    # File operations
│   └── validation.rs    # Content validation logic
├── types/
│   ├── mod.rs           # Shared type definitions
│   ├── project.rs       # Project-related types
│   ├── spec.rs          # Spec-related types
│   └── responses.rs     # JSON response structures
└── utils/
    ├── mod.rs           # Utility functions
    ├── timestamp.rs     # ISO timestamp generation
    └── paths.rs         # Path manipulation utilities
```

## Implementation Phases

### Phase 1: Foundation & Core Infrastructure

**Estimated Time**: Week 1

#### Setup and Project Structure

- [x] Set up basic Rust project structure
- [x] Configure Cargo.toml with required dependencies
- [x] Create module structure (cli/, core/, types/, utils/)
- [x] Set up main.rs with basic CLI framework using clap
- [x] Create lib.rs with public exports

#### Core Types and Data Structures

- [x] Define Project struct in `types/project.rs`
- [x] Define Spec struct in `types/spec.rs` (with required fields: spec_content, notes, tasks)
- [x] Create FoundryResponse<T> generic response type in `types/responses.rs`
- [x] Define CLI argument structures in `cli/args.rs`
- [x] Create error types using anyhow for consistent error handling

#### Basic File System Operations

- [x] Implement foundry directory detection/creation (`~/.foundry/`)
- [x] Create basic file writing utilities in `core/filesystem.rs`
- [x] Implement directory creation with proper error handling
- [x] Add file existence checking utilities
- [x] Create atomic file writing functions

#### CLI Framework Setup

- [x] Set up clap command structure in `cli/mod.rs`
- [x] Create basic command routing
- [x] Implement help system integration
- [x] Add common CLI utilities (output formatting, etc.)

### Phase 2: Core Commands Implementation

**Estimated Time**: Week 2

#### `create_project` Command

- [ ] Define rich parameter schema for LLM guidance
- [ ] Implement CLI argument parsing for project_name, vision, tech_stack, summary
- [ ] Add parameter validation (minimum lengths, format checks)
- [ ] Create project directory structure (`~/.foundry/{project_name}/project/`)
- [ ] Implement file writing for vision.md, tech-stack.md, summary.md
- [ ] Create empty specs/ directory
- [ ] Return structured JSON response with next_steps
- [ ] Add comprehensive error handling and user-friendly messages

#### `list_projects` Command

- [ ] Implement directory scanning of `~/.foundry/`
- [ ] Extract project metadata (creation date, spec count)
- [ ] Create JSON response format for project listing
- [ ] Handle empty foundry directory gracefully
- [ ] Add sorting by creation date/name options

#### Project Management Core Logic

- [ ] Implement project validation in `core/project.rs`
- [ ] Create project structure management utilities
- [ ] Add project existence checking functions
- [ ] Implement project metadata extraction

### Phase 3: Spec Management

**Estimated Time**: Week 2

#### Timestamp Utilities

- [ ] Implement ISO timestamp generation in `utils/timestamp.rs`
- [ ] Create spec name formatting (`YYYYMMDD_HHMMSS_feature_name`)
- [ ] Add timestamp parsing utilities for existing specs
- [ ] Implement spec directory name validation

#### `create_spec` Command

- [ ] Define parameter schema for spec content, notes, optional task-list
- [ ] Implement CLI argument parsing
- [ ] Add project existence validation
- [ ] Generate timestamped spec directory
- [ ] Write spec.md, notes.md files with LLM-provided content
- [ ] Create empty or LLM-provided task-list.md
- [ ] Return JSON response with spec details

#### `load_spec` Command

- [ ] Implement spec discovery (list available specs if none specified)
- [ ] Add spec directory parsing and validation
- [ ] Read all spec files (spec.md, task-list.md, notes.md)
- [ ] Load project summary for context
- [ ] Return comprehensive JSON with spec content and workflow hints
- [ ] Handle missing files gracefully

#### Spec Management Core Logic

- [ ] Implement spec validation in `core/spec.rs`
- [ ] Create spec directory management utilities
- [ ] Add spec listing and filtering functions
- [ ] Implement spec content reading/writing

### Phase 4: Analysis and Validation Tools

**Estimated Time**: Week 1

#### `analyze_project` Command

- [ ] Implement codebase scanning utilities
- [ ] Add technology stack detection (file extensions, config files)
- [ ] Create project structure analysis
- [ ] Generate analysis report for LLM consumption
- [ ] Require LLM-provided vision, tech_stack, summary parameters
- [ ] Write LLM content to project structure (no auto-generation)
- [ ] Return analysis data + confirmation of written files

#### `validate_content` Command

- [ ] Define validation rules for each content type (vision, tech-stack, spec, etc.)
- [ ] Implement content length validation
- [ ] Add format checking (markdown structure, etc.)
- [ ] Create validation result reporting
- [ ] Provide improvement suggestions for LLMs
- [ ] Support stdin input for content validation

#### Validation Core Logic

- [ ] Implement content validation in `core/validation.rs`
- [ ] Create validation rule engine
- [ ] Add schema compliance checking
- [ ] Implement suggestion generation system

### Phase 5: Help and Documentation System

**Estimated Time**: 3 days

#### `get_foundry_help` Command

- [ ] Create help topic system (workflows, content-examples, project-structure)
- [ ] Implement workflow guidance content
- [ ] Add content examples for each file type
- [ ] Create parameter guidance documentation
- [ ] Implement topic-based help routing
- [ ] Format help output for both human and LLM consumption

#### Help Content Creation

- [ ] Write workflow examples and best practices
- [ ] Create template content examples for vision, tech-stack, specs
- [ ] Document parameter schemas and expectations
- [ ] Add troubleshooting guides
- [ ] Create getting started examples

### Phase 6: Polish and Testing

**Estimated Time**: Week 1

#### Error Handling and Validation

- [ ] Implement comprehensive error messages
- [ ] Add input validation for all commands
- [ ] Create user-friendly error formatting
- [ ] Handle edge cases (missing directories, permissions, etc.)
- [ ] Add validation for file paths and names

#### Testing Implementation

- [ ] Create unit tests for core functions
- [ ] Implement integration tests for each command
- [ ] Add filesystem testing with tempdir
- [ ] Create JSON response validation tests
- [ ] Test error handling scenarios

#### Documentation and Examples

- [ ] Write comprehensive CLI help documentation
- [ ] Create usage examples for each command
- [ ] Add parameter schema documentation
- [ ] Write troubleshooting guide
- [ ] Create getting started tutorial

#### Performance and Reliability

- [ ] Optimize file operations for performance
- [ ] Add proper file locking for concurrent access
- [ ] Implement atomic operations where needed
- [ ] Add progress indicators for long operations
- [ ] Optimize JSON response generation

## Command Specifications with Parameter Schemas

### `foundry create_project`

**Purpose**: Write LLM-provided content to new project structure

**CLI Usage:**

```bash
foundry create_project <project_name> --vision <content> --tech-stack <content> --summary <content>
```

**Parameter Schema (for MCP integration):**

```json
{
  "project_name": {
    "type": "string",
    "description": "Descriptive project name using kebab-case"
  },
  "vision": {
    "type": "string",
    "description": "High-level product vision (2-4 paragraphs) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Goes into project/vision.md",
    "minLength": 200
  },
  "tech_stack": {
    "type": "string",
    "description": "Comprehensive technology decisions including languages, frameworks, databases, deployment platforms, and rationale. Include constraints, preferences, or team standards. Goes into project/tech-stack.md",
    "minLength": 150
  },
  "summary": {
    "type": "string",
    "description": "Concise summary of vision and tech-stack for context loading. Should be brief but capture key points for LLM context. Goes into project/summary.md",
    "minLength": 100
  }
}
```

**Implementation Checklist:**

- [ ] Parse and validate all required parameters
- [ ] Check project_name format (kebab-case)
- [ ] Validate content length requirements
- [ ] Create project directory structure
- [ ] Write vision.md with provided content
- [ ] Write tech-stack.md with provided content
- [ ] Write summary.md with provided content
- [ ] Create empty specs/ directory
- [ ] Return JSON response with success confirmation

### `foundry analyze_project`

**Purpose**: Provide codebase analysis to inform LLM, then write LLM-provided content

**CLI Usage:**

```bash
foundry analyze_project <project_name> [codebase_path] --vision <content> --tech-stack <content> --summary <content>
```

**Implementation Checklist:**

- [ ] Scan codebase directory (default to current dir)
- [ ] Detect technology stack from file extensions and config files
- [ ] Analyze project structure and dependencies
- [ ] Generate analysis report data
- [ ] Validate LLM-provided content parameters
- [ ] Write LLM content to project structure (no auto-generation)
- [ ] Return JSON with analysis data + written file confirmation

### `foundry create_spec`

**Purpose**: Write LLM-provided spec content to timestamped directory

**CLI Usage:**

```bash
foundry create_spec <project_name> <feature_name> --spec <content> --notes <content> [--task-list <content>]
```

**Implementation Checklist:**

- [ ] Validate project exists
- [ ] Generate ISO timestamp (YYYYMMDD_HHMMSS_feature_name)
- [ ] Create spec directory
- [ ] Write spec.md with provided content
- [ ] Write notes.md with provided content
- [ ] Write task-list.md (provided or empty template)
- [ ] Return JSON response with spec details

### `foundry load_spec`

**Purpose**: Return existing spec content for LLM context

**CLI Usage:**

```bash
foundry load_spec <project_name> [spec_name]
```

**Implementation Checklist:**

- [ ] Validate project exists
- [ ] List available specs if spec_name not provided
- [ ] Load specified spec directory
- [ ] Read spec.md, task-list.md, notes.md files
- [ ] Load project summary for context
- [ ] Return JSON with all content and workflow hints

### `foundry list_projects`

**Purpose**: Return available projects for LLM discovery

**CLI Usage:**

```bash
foundry list_projects
```

**Implementation Checklist:**

- [ ] Scan ~/.foundry/ directory
- [ ] Extract project metadata (creation date, spec count)
- [ ] Format project list for display
- [ ] Return JSON with project information
- [ ] Handle empty directory gracefully

### `foundry get_foundry_help`

**Purpose**: Provide workflow guidance and content examples to LLMs

**CLI Usage:**

```bash
foundry get_foundry_help [topic]
```

**Topics**: workflows, content-examples, project-structure, parameter-guidance

**Implementation Checklist:**

- [ ] Create help topic routing system
- [ ] Write workflow guidance content
- [ ] Create content examples for each file type
- [ ] Add parameter schema documentation
- [ ] Format help for both human and LLM consumption

### `foundry validate_content`

**Purpose**: Validate LLM-provided content against schema requirements

**CLI Usage:**

```bash
foundry validate_content <content_type> --content <content>
```

**Content Types**: vision, tech-stack, summary, spec, notes

**Implementation Checklist:**

- [ ] Define validation rules for each content type
- [ ] Check length requirements
- [ ] Validate format and structure
- [ ] Generate improvement suggestions
- [ ] Return validation results and recommendations

## JSON Response Format

All commands return consistent JSON structure:

```json
{
  "data": {
    // Command-specific data
  },
  "next_steps": ["Suggested next actions for LLM"],
  "validation_status": "complete|incomplete|error",
  "workflow_hints": ["Optional workflow guidance"]
}
```

## Success Criteria for MVP

### Phase 1 Foundation ✅ COMPLETED

- [x] Complete module structure and project organization
- [x] Core type definitions with proper validation
- [x] File system utilities and atomic operations
- [x] CLI framework with command routing
- [x] Required field validation for LLM content

### Phase 2 Commands 🔄 IN PROGRESS

- [ ] All 7 CLI commands implemented and functional
- [ ] Consistent JSON response format across all commands
- [ ] Proper file structure creation in `~/.foundry/`
- [ ] Robust error handling and validation
- [ ] Rich parameter schemas with embedded LLM guidance
- [ ] Clear help documentation and examples
- [ ] Comprehensive testing coverage
- [ ] Ready for MCP server wrapper implementation

## Technical Implementation Notes

### Key Dependencies

- `clap` - CLI argument parsing with derive features
- `serde_json` - JSON response formatting
- `anyhow` - Error handling and context
- `chrono` - Timestamp generation
- `dirs` - Home directory detection

### Error Handling Strategy

- Use `anyhow::Result<T>` for all fallible operations
- Provide rich error context for debugging
- Return user-friendly error messages
- Handle file system errors gracefully
- Validate all inputs before processing

### File System Design

- Atomic file operations where possible
- Proper error handling for permissions issues
- Cross-platform path handling
- UTF-8 content encoding
- Preserve file metadata when possible

This implementation plan provides a clear roadmap for building the Foundry CLI MVP with LLM-centric design principles, focusing on pure file management operations with rich parameter guidance for LLM consumption.
