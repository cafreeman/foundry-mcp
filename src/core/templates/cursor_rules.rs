//! Cursor rules template for Foundry MCP usage guidance

use super::ClientTemplate;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Cursor rules template implementation
///
/// Provides the embedded foundry.mdc content that guides AI assistants
/// on how to effectively use Foundry MCP tools in Cursor.
pub struct CursorRulesTemplate;

impl ClientTemplate for CursorRulesTemplate {
    fn content() -> &'static str {
        "---
name: 'Foundry MCP Usage Guide'
description: 'Comprehensive guide for using Foundry MCP tools effectively in AI-assisted development'
alwaysApply: true
---

# Foundry MCP Usage Guide for AI Assistants

## Overview

Foundry MCP is a project management tool designed specifically for AI coding assistants. It provides structured project context, specification management, and workflow guidance through a standardized directory structure in `~/.foundry/`.

**Key Principles:**

- **Content Agnostic**: Foundry manages structure, not content opinions
- **LLM-Driven**: You provide complete content as arguments
- **User-Controlled**: Users control content, CLI manages structure
- **Workflow-Aware**: Tools provide next-steps guidance for efficient development

## Core Concepts

### Project Structure

Each Foundry project contains:

- **`vision.md`**: High-level product vision and goals (200+ characters, you provide)
- **`tech-stack.md`**: Technology decisions and architecture (150+ characters, you provide)
- **`summary.md`**: Concise context for quick LLM consumption (100+ characters, you provide)
- **`specs/`**: Directory containing timestamped feature specifications

### Specification Structure

Each spec contains:

- **`spec.md`**: Detailed feature requirements, acceptance criteria, implementation approach
- **`task-list.md`**: Implementation checklist with checkboxes (- [ ] task, - [x] completed)
- **`notes.md`**: Additional context, design decisions, and implementation notes

## Tool Usage Guidelines

### When to Use Each Tool

#### Project Management

- **`create-project`**: For NEW initiatives requiring full project context
- **`analyze-project`**: For EXISTING codebases you want to manage with Foundry
- **`load-project`**: ALWAYS do this first when working on existing projects

#### Specification Management

- **`create-spec`**: Breaking down features into detailed implementation plans
- **`load-spec`**: Reviewing specifications and checking progress
- **`update-spec`**: Updating multiple spec files in a single operation

#### Discovery & Validation

- **`list-projects`**: Discovering available projects
- **`validate-content`**: Checking content before creating projects/specs
- **`get-foundry-help`**: Getting workflow guidance and examples

## Content Requirements & Boundaries

### What You MUST Provide

**Always include these in your arguments:**

- **Vision** (200+ chars): Problem statement, target users, value proposition, roadmap priorities
- **Tech Stack** (150+ chars): Languages, frameworks, databases, deployment, rationale
- **Summary** (100+ chars): 2-3 sentences capturing project essence
- **Specifications**: Requirements, acceptance criteria, implementation approach, dependencies
- **Task Lists**: Specific, actionable implementation steps with checkboxes
- **Notes**: Design decisions, context, implementation details

### What CLI Handles

**Never include these in content:**

- File structure and organization
- Content validation (length, format only)
- Directory creation and timestamps
- Project/spec naming conventions
- Error handling and recovery suggestions

## Best Practices & Workflow Patterns

### 1. Always Load Context First

Load project context before working on any specifications.

### 2. Use Iterative Development

Use `append` operations for task progress updates and implementation notes.

### 3. Follow Next Steps Guidance

Every Foundry command returns `next_steps` and `workflow_hints`.

### 4. Validate Content Proactively

Use `validate-content` to check before creating projects or specs.

### 5. Use Appropriate Spec Granularity

- One spec per feature/story
- Use task-list.md for implementation steps
- Use notes.md for design decisions
- Update regularly with append operations

## Common Workflows

### New Project Setup

1. Create Project with complete vision, tech-stack, and summary
2. Create First Spec for initial feature
3. Follow next_steps for iterative development

### Feature Development Cycle

1. Load Project context
2. Create Spec for new feature
3. Update Progress with append operations
4. Add Notes documenting decisions
5. Review Status and get workflow guidance

### Existing Codebase Analysis

1. Explore Codebase structure
2. Analyze Project with analyzed content
3. Create Specs for development plans

## Error Handling & Troubleshooting

### Content Validation Errors

- Content too short: Provide more detailed content
- Content validation failed: Use validate-content to check before creating
- Solution: Expand content with specific details and examples

### Project/Spec Not Found

- Project not found: Use list-projects to see available options
- Spec not found: Load project first to see available specs
- Solution: Always load project context before working with specs

### File Operation Errors

- Permission denied: Check file permissions
- File locked: Close conflicting applications
- Solution: Verify permissions and close conflicting apps

### MCP vs CLI Tool Confusion

- MCP Tools: Available through AI assistant interface
- CLI Tools: Use terminal/command line for installation management
- Most tools work through both interfaces
- Use CLI for installation, status checking, deletion operations

## Tips for Effective Usage

1. Start with Context: Always load project before creating specs
2. Use Append for Updates: Build up task lists and notes incrementally
3. Follow Guidance: Pay attention to next_steps and workflow_hints
4. Validate First: Use validate-content to avoid rejection
5. Keep Specs Focused: One feature per spec, use task-list for breakdown
6. Document Decisions: Use notes.md for rationale and context
7. Update Regularly: Mark tasks complete, add implementation notes
8. Get Help: Use get-foundry-help for workflow guidance

## Quick Reference

### Most Common Commands

- Start working: `foundry mcp load-project my-project`
- Create spec: `foundry mcp create-spec my-project user-auth`
- Update progress: `foundry mcp update-spec my-project spec-name --tasks \"- [x] Completed\" --operation append`
- Get help: `foundry mcp get-foundry-help workflows`

### Content Length Requirements

- **Vision**: 200+ characters (2-4 paragraphs with examples)
- **Tech Stack**: 150+ characters (detailed technology choices with rationale)
- **Summary**: 100+ characters (2-3 sentences capturing essence)
- **Specifications**: Comprehensive but focused on requirements and approach
- **Task Lists**: Specific, actionable implementation steps
- **Notes**: Design decisions, context, implementation details

Remember: Foundry manages structure, you provide content. Focus on comprehensive, well-structured content that helps future development."
    }

    fn file_path(config_dir: &Path) -> Result<PathBuf> {
        Ok(config_dir.join("rules").join("foundry.mdc"))
    }

    fn should_create_dir() -> bool {
        true
    }
}
