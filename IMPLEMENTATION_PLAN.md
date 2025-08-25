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

### ✅ Phase 2: Core Commands Implementation - COMPLETED

**CRITICAL DISCOVERY RESOLVED**: The fundamental LLM workflow gap has been closed!

**Completed Commands:**

- ✅ `foundry create_project` - Project creation with LLM content (DONE)
- ✅ `foundry list_projects` - Project discovery (DONE)
- ✅ `foundry load_project` - Load project context for LLM sessions (DONE)
- ✅ `foundry create_spec` - Timestamped spec creation (DONE)
- ✅ `foundry load_spec` - Spec content retrieval (COMPLETED)
- ✅ `foundry analyze_project` - Pure file management for LLM-analyzed projects (COMPLETED)
- ✅ `foundry validate_content` - Content validation with structured feedback (COMPLETED)

**Core Workflow Complete**: create → list → load → create spec → load project → validate → work

**Remaining Commands:**

- `foundry get_foundry_help` - Workflow guidance

**✅ Spec Management Core Logic COMPLETED**: All spec validation, directory management, listing/filtering, and content operations are now fully implemented and tested.

**✅ Gap Resolved**: `load_project` has been implemented and tested! LLMs can now create projects and load context back to continue work. The fundamental workflow is complete: create → list → **LOAD** → work.

**Current Priority**: Continue with remaining commands to complete full spec management workflow.

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

#### `create_project` Command ✅ COMPLETED

- [x] Define rich parameter schema for LLM guidance
- [x] Implement CLI argument parsing for project_name, vision, tech_stack, summary
- [x] Add parameter validation (minimum lengths, format checks)
- [x] Create project directory structure (`~/.foundry/{project_name}/project/`)
- [x] Implement file writing for vision.md, tech-stack.md, summary.md
- [x] Create empty specs/ directory
- [x] Return structured JSON response with next_steps
- [x] Add comprehensive error handling and user-friendly messages

#### `list_projects` Command ✅ COMPLETED

- [x] Implement directory scanning of `~/.foundry/`
- [x] Extract project metadata (creation date, spec count)
- [x] Create JSON response format for project listing
- [x] Handle empty foundry directory gracefully
- [x] Add sorting by creation date/name options

#### `load_project` Command ✅ COMPLETED

- [x] Implement CLI argument parsing for project_name
- [x] Add project existence validation
- [x] Read project/vision.md content
- [x] Read project/tech-stack.md content
- [x] Read project/summary.md content
- [x] Scan specs/ directory for available specifications
- [x] Create comprehensive JSON response with all project context
- [x] Handle missing files gracefully (return empty strings)
- [x] Add workflow guidance for next steps
- [x] Enhanced validation status based on specs availability
- [x] Comprehensive testing with integration tests

#### Project Management Core Logic ✅ COMPLETED

- [x] Implement project validation in `core/project.rs`
- [x] Create project structure management utilities
- [x] Add project existence checking functions
- [x] Implement project metadata extraction

### Phase 3: Spec Management ✅ COMPLETED

**Estimated Time**: Week 2 → **COMPLETED EARLY**

#### Timestamp Utilities ✅ COMPLETED

- [x] Implement ISO timestamp generation in `utils/timestamp.rs`
- [x] Create spec name formatting (`YYYYMMDD_HHMMSS_feature_name`)
- [x] Add timestamp parsing utilities for existing specs with validation
- [x] Implement spec directory name validation with enhanced error handling
- [x] Add timestamp format conversion between ISO, spec format, and display format
- [x] Enhanced feature name extraction and validation
- [x] Comprehensive test coverage for all timestamp utilities

#### `create_spec` Command ✅ COMPLETED

- [x] Define parameter schema for spec content, notes, task-list
- [x] Implement CLI argument parsing
- [x] Add project existence validation
- [x] Generate timestamped spec directory
- [x] Write spec.md, notes.md files with LLM-provided content
- [x] Create task-list.md with LLM-provided content
- [x] Return JSON response with spec details
- [x] Enhanced validation with snake_case feature name requirements
- [x] Comprehensive content validation and error handling

#### `load_spec` Command ✅ COMPLETED

- [x] Implement spec discovery (list available specs if none specified)
- [x] Add spec directory parsing and validation
- [x] Read all spec files (spec.md, task-list.md, notes.md)
- [x] Load project summary for context
- [x] Return comprehensive JSON with spec content and workflow hints
- [x] Handle missing files gracefully
- [x] PRD-compliant response format with task_list field naming
- [x] Comprehensive testing with 7 integration tests covering all scenarios

#### Spec Management Core Logic ✅ COMPLETED

- [x] **Implement spec validation in `core/spec.rs`** - COMPLETED
  - Enhanced spec name validation with timestamp and snake_case enforcement
  - Comprehensive file validation with `validate_spec_files()` function
  - Detailed validation reporting via `SpecValidationResult` type
- [x] **Create spec directory management utilities** - COMPLETED
  - Path management: `get_spec_path()`, `get_specs_directory()`, `ensure_specs_directory()`
  - Existence checking: `spec_exists()` function
  - Safe deletion: `delete_spec()` with validation
- [x] **Add spec listing and filtering functions** - COMPLETED
  - Advanced filtering: `list_specs_filtered()` with `SpecFilter` type
  - Utility functions: `get_latest_spec()`, `count_specs()`
  - Support for feature name matching, date ranges, and result limiting
- [x] **Implement spec content reading/writing** - COMPLETED
  - Content updates: `update_spec_content()` for individual files
  - Atomic file operations with proper validation
  - Support for spec.md, notes.md, and task-list.md updates

### ✅ Phase 4: Analysis and Validation Tools - COMPLETED

**Estimated Time**: Week 1 → **COMPLETED**

**CRITICAL DESIGN CORRECTION**: During implementation, we discovered that providing scanning utilities violates Foundry's core principle of being content-agnostic. LLMs already have superior analysis tools (codebase_search, grep_search, read_file), so we pivoted to pure file management following the "LLMs provide content, Foundry manages files" principle.

#### ✅ `analyze_project` Command - COMPLETED

- [x] **Pure file management implementation** - No scanning, follows core principles
- [x] **Enhanced input validation** - Project name validation, content size limits
- [x] **LLM-provided content acceptance** - Requires vision, tech_stack, summary parameters
- [x] **Project structure creation** - Write LLM content to structured format
- [x] **Enhanced error handling** - Detailed error messages with actionable guidance
- [x] **CLI parameter guidance** - Rich descriptions guide LLM behavior
- [x] **Workflow hints** - Directs LLMs to use their superior analysis tools

#### ✅ `validate_content` Command - COMPLETED

- [x] **Content validation rules** - All content types (vision, tech-stack, spec, notes, tasks)
- [x] **Length validation** - Minimum/maximum size checking with clear error messages
- [x] **Content quality suggestions** - Improvement recommendations for each type
- [x] **Enhanced error reporting** - Structured validation results with counts
- [x] **Content-type specific guidance** - Tailored hints for each content type
- [x] **Input validation** - Size limits, binary detection, empty content handling
- [x] **Enhanced user experience** - Dynamic next steps based on validation results

#### ✅ Validation Core Logic - COMPLETED

- [x] **Comprehensive validation in `core/validation.rs`** - All content types supported
- [x] **Validation rule engine** - Flexible rules with structured feedback
- [x] **Error and suggestion system** - Clear separation of errors vs improvements
- [x] **Content type parsing** - Robust parsing with enhanced error messages

#### ✅ Architecture Cleanup - COMPLETED

- [x] **Removed obsolete `analysis.rs`** - 363 lines of dead code eliminated
- [x] **Clean module structure** - No redundant scanning functionality
- [x] **Aligned with core principles** - Pure file management, no content generation

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

### ✅ Phase 6: Polish and Testing - SIGNIFICANTLY ADVANCED

**Estimated Time**: Week 1 → **COMPLETED EARLY**

#### Error Handling and Validation

- [ ] Implement comprehensive error messages
- [ ] Add input validation for all commands
- [ ] Create user-friendly error formatting
- [ ] Handle edge cases (missing directories, permissions, etc.)
- [ ] Add validation for file paths and names

#### Testing Implementation ✅ SIGNIFICANTLY ENHANCED

- [x] **Comprehensive Integration Tests**: 8 integration tests covering full command workflows
- [x] **Real Filesystem Operations**: Tests use actual file creation/validation with temporary directories
- [x] **Test Environment Isolation**: Thread-safe test environment with proper cleanup
- [x] **End-to-End Workflows**: Complete test coverage from create → load → create spec → load
- [x] **Error Scenario Testing**: Real error conditions with proper error handling verification
- [x] **CLI Testing Best Practices**: Following industry standards for CLI application testing
- [x] Refactored unit tests to focus on business logic rather than trivial validation
- [x] 52 total tests (37 unit + 15 integration) - all passing ✅

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

### `foundry load_project` - **CRITICAL MISSING COMMAND**

**Purpose**: Load complete project context for LLM sessions - **ESSENTIAL for workflow completion**

**CLI Usage:**

```bash
foundry load_project <project_name>
```

**Parameter Schema (for MCP integration):**

```json
{
  "project_name": {
    "type": "string",
    "description": "Project name to load context from (must exist in ~/.foundry/)"
  }
}
```

**Expected Response Format:**

```json
{
  "data": {
    "project": {
      "name": "foundry-development",
      "vision": "<full content of project/vision.md>",
      "tech_stack": "<full content of project/tech-stack.md>",
      "summary": "<full content of project/summary.md>",
      "specs_available": ["20240824_120000_phase3_implementation"],
      "created_at": "2025-08-24T03:38:11Z"
    }
  },
  "next_steps": [
    "Load a specific spec with: foundry load_spec",
    "Create new specs with: foundry create_spec"
  ],
  "workflow_hints": [
    "Use project summary for quick context",
    "Full vision provides comprehensive background"
  ],
  "validation_status": "complete"
}
```

**Implementation Checklist:**

- [x] Parse and validate project_name parameter
- [x] Check if project exists in ~/.foundry/
- [x] Read project/vision.md content
- [x] Read project/tech-stack.md content
- [x] Read project/summary.md content
- [x] Scan specs/ directory for available specifications
- [x] Return comprehensive JSON response with all project context
- [x] Handle missing files gracefully (empty strings for missing content)
- [x] Provide workflow guidance for next steps
- [x] Enhanced validation status based on specs availability

**✅ Critical Gap Resolved**: This command completes the basic LLM workflow: create → list → **load** → work. LLMs can now maintain full project context across sessions.

### ✅ `foundry analyze_project` - COMPLETED

**Purpose**: Pure file management for LLM-analyzed projects (follows core principle: LLMs provide content, Foundry manages files)

**CLI Usage:**

```bash
foundry analyze-project <project_name> --vision <content> --tech-stack <content> --summary <content>
```

**Implementation Checklist:**

- [x] **Enhanced input validation** - Project name validation, content size limits
- [x] **LLM-provided content acceptance** - Vision, tech-stack, summary as required parameters
- [x] **Project structure creation** - Create ~/.foundry/PROJECT/project/ and specs/ directories
- [x] **File management** - Write LLM content to vision.md, tech-stack.md, summary.md
- [x] **Enhanced error handling** - Detailed error messages with actionable guidance
- [x] **CLI parameter guidance** - Rich descriptions guide LLM on expected content
- [x] **Workflow hints** - Direct LLMs to use their superior analysis tools (codebase_search, etc.)
- [x] **Return file confirmation** - JSON response with created files and next steps

**Key Design Decision**: No scanning or analysis performed by Foundry. LLMs use their existing superior tools (codebase_search, grep_search, read_file) for analysis, then provide Foundry with the resulting content for structured file management.

### `foundry create_spec`

**Purpose**: Write LLM-provided spec content to timestamped directory

**CLI Usage:**

```bash
foundry create_spec <project_name> <feature_name> --spec <content> --notes <content> [--task-list <content>]
```

**Implementation Checklist:**

- [x] Validate project exists
- [x] Generate ISO timestamp (YYYYMMDD_HHMMSS_feature_name)
- [x] Create spec directory
- [x] Write spec.md with provided content
- [x] Write notes.md with provided content
- [x] Write task-list.md with provided content
- [x] Return JSON response with spec details
- [x] Enhanced feature name validation (snake_case enforcement)
- [x] Comprehensive content validation and error handling
- [x] Integration with project loading workflow

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

### Phase 2 Commands ✅ COMPLETED (7/7 commands done)

- [x] **7/7 CLI commands implemented and functional** (create_project, list_projects, load_project, create_spec, load_spec, analyze_project, validate_content)
- [x] Consistent JSON response format across all commands
- [x] Proper file structure creation in `~/.foundry/`
- [x] **Enhanced error handling and validation** - Detailed error messages with actionable guidance
- [x] Rich parameter schemas with embedded LLM guidance
- [x] **Comprehensive testing coverage** (15 integration tests + 43 unit tests)
- [x] **Core spec management logic fully implemented** - All validation, directory management, filtering, and content operations
- [x] **Phase 4 commands completed** - analyze_project and validate_content with enhanced error handling
- [ ] Clear help documentation and examples (1 command remaining: get_foundry_help)
- [ ] Ready for MCP server wrapper implementation (after help command)

**Status**: **All core workflow commands complete** - LLMs can create projects, load context, manage specs, analyze codebases, and validate content. Only help documentation remains.

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
