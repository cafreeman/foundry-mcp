# Project Manager MCP - Product Requirements Document

## Overview

Project Manager MCP is a Model Context Protocol (MCP) server that provides deterministic tools for AI coding assistants to manage project context, specifications, and task lists. It solves the persistent problem of context management in long-term software development projects by providing centralized, structured storage outside of project directories.

## Problem Statement

Current AI coding assistant workflows suffer from several critical issues:

### Current Pain Points

- **Inconsistent file management**: Prompt-driven systems like Agent-OS create files in unpredictable locations (`~/.agent-os`, `<project-dir>/.agent-os`, or random markdown files in project root)
- **Project directory pollution**: Context management files clutter the actual codebase and may be accidentally committed
- **Context loss**: No reliable way to pause/resume complex development tasks across sessions
- **Error-prone prompting**: Relying on natural language instructions for file system operations leads to inconsistent behavior

### Target Users

- Cursor users building software with AI assistance
- Claude Code users managing complex development projects
- Developers who need persistent context management across AI coding sessions

## Solution Overview

Project Manager MCP provides a set of MCP tools that enable deterministic project and specification management through a centralized file system outside of project directories.

### Core Value Propositions

1. **Deterministic operations**: MCP tools eliminate prompt-driven file system errors
2. **Clean project separation**: Context files stored outside project directories
3. **Persistent context**: Natural pause/resume functionality through structured file storage
4. **Hierarchical organization**: Project-level context with individual specs and task lists

## User Stories

### Project Setup Workflow

```
As a developer working on any software project,
I want to create foundational project context documents,
So that I have persistent context for all future development work.
```

**Acceptance Criteria:**

- User can execute `@setup_project [project-name] [description/path]`
- Agent calls appropriate project setup tool
- System creates project directory with foundational documents
- Subsequent specs inherit project context

### Specification Management

```
As a developer implementing a feature,
I want to create and manage detailed specifications with task lists,
So that I can track progress and resume work across sessions.
```

**Acceptance Criteria:**

- User can execute `@create_spec [description]`
- Agent generates snake_case name and calls `create_spec` tool
- System creates timestamped spec directory with structured files
- Agent can load and update specs deterministically

### Specification Loading

```
As a developer working with specifications,
I want to load specific specs into context for review or questions,
So that I can examine and discuss implementation details.
```

**Acceptance Criteria:**

- User can execute `@load_spec [project-name] [spec-id]`
- Agent calls `load_spec` tool and presents context
- User can ask questions about the loaded specification
- Context remains available for subsequent operations

### Task Execution

```
As a developer working on a specification,
I want to execute specific tasks while maintaining updated context,
So that my progress is tracked and resumable.
```

**Acceptance Criteria:**

- User can execute `@execute_task [optional task description]`
- Agent checks for existing spec context before loading
- Agent identifies and works on appropriate task
- Task list updates reflect completed work

## Technical Specifications

### File System Structure

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

### MCP Tools

#### `setup_project`

**Purpose**: Create project context documents for any software project
**Parameters**:

- `project_name` (string): Unique project identifier
- `description` (string): Project description/requirements
- `project_path` (string, optional): Path to existing codebase for analysis
  **Behavior**:
- Error if project_name already exists
- If project_path provided: analyze codebase structure, dependencies, patterns
- Generate `project/tech-stack.md` and `project/vision.md`
- Create foundational project context regardless of project type
  **Returns**: Success confirmation and project summary

#### `create_spec`

**Purpose**: Create new specification with task breakdown
**Parameters**:

- `project_name` (string): Target project
- `spec_name` (string): Snake_case specification name
- `description` (string): Feature/specification description
  **Behavior**:
- Prepend timestamp: `YYYYMMDD_spec_name`
- Create spec directory structure
- Generate initial `spec.md`, `task-list.md`, `notes.md`
- Include project context references
  **Returns**: Spec ID and creation confirmation

#### `load_spec`

**Purpose**: Load specification with full project context
**Parameters**:

- `project_name` (string): Target project
- `spec_id` (string): Timestamped spec identifier
  **Behavior**:
- Return structured context including:
  - Project context summary
  - Complete spec content
  - Current task list
  - Accumulated notes
    **Returns**: Formatted context document

### MCP Prompts

#### `execute_task`

**Purpose**: Guide agents through task execution with proper context loading
**Trigger**: When user executes `@execute_task [optional task description]`
**Prompt Content**:

```
You are helping execute a task for a software development project. Follow these steps:

1. **Check Context**: Look through the current conversation to see if a project specification has already been loaded with the load_spec tool.

2. **Load Context If Needed**: If no spec context is available in this conversation, ask the user which project and spec they want to work on, then call load_spec to get the current specification and project context.

3. **Identify Task**:
   - If the user specified a particular task or feature, find the relevant item(s) in the task list
   - If no specific task was mentioned, work on the next incomplete task in the task list
   - If all tasks are complete, ask the user what they'd like to work on next

4. **Execute Work**:
   - Read and understand the full context (project vision, tech stack, spec details)
   - Work on the identified task following the project's established patterns and preferences
   - Make sure your implementation aligns with the spec and project vision

5. **Update Task List**:
   - Mark completed tasks as done
   - Add new subtasks if you discover additional work needed
   - Update task descriptions if scope changes
   - Keep the task list as an accurate reflection of remaining work

6. **Update Notes**:
   - Add any implementation decisions, code patterns, or user preferences to notes.md
   - Include anything that might be helpful for future work on this spec

Remember: The spec and task list should always represent the current state of work to enable natural pause/resume functionality.
```

### Integration Requirements

#### Cursor Rules Integration

The system will include cursor rules that prompt appropriate tool usage:

- `@setup_project` → triggers `setup_project` tool
- `@create_spec` → triggers `create_spec` tool
- `@load_spec` → triggers `load_spec` tool
- `@execute_task` → triggers `execute_task` prompt with smart context loading

#### Error Handling

- Duplicate project names: Return error with suggestion to use unique name
- Missing projects/specs: Clear error messages with available options
- File system permissions: Graceful degradation with user guidance

## Success Criteria

### Primary Metrics

- **Consistency**: 100% of context files created in correct locations
- **Persistence**: Specifications and task lists accurately reflect current state
- **Adoption**: Reduced user complaints about context management

### User Experience Goals

- Zero manual file system navigation for context management
- Natural pause/resume functionality without additional setup
- Clean project directories free of AI context files

### Technical Goals

- Deterministic file operations replacing error-prone prompting
- Reliable project context inheritance across all specifications
- Robust error handling for edge cases

## Future Considerations

### Potential Enhancements

- Cloud storage backend options
- Multi-user project sharing
- Integration with version control systems
- Template system for common project types
- Analytics on development patterns

### Scalability

- Support for large projects with hundreds of specs
- Performance optimization for file system operations
- Memory management for large context documents

---

_This PRD represents the initial scope for Project Manager MCP. Features and specifications may evolve based on user feedback and technical constraints._
