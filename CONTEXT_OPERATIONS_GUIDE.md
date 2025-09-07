# Foundry Context Operations Usage Guide

## Overview

Foundry's context-based patching system supports three precise operations for targeted updates to specification files. This guide explains when and how to use each operation type.

## Operation Types

### 1. Insert Operation

**Purpose**: Adds new content between existing landmarks without modifying existing content.

**When to Use**:
- Adding new requirements to a specific section
- Inserting new tasks in the middle of a task list
- Adding implementation notes between existing items

**How it Works**:
- Content is inserted between the last line of `before_context` and the first line of `after_context`
- Existing content remains unchanged
- The new content appears exactly where the context landmarks indicate

**Example**:
```json
{
  "name": "update_spec",
  "arguments": {
    "project_name": "my-project",
    "spec_name": "20240101_user_auth",
    "operation": "context_patch",
    "context_patch": "{\"file_type\":\"spec\",\"operation\":\"insert\",\"section_context\":\"## Requirements\",\"before_context\":[\"- Password hashing with bcrypt\"],\"after_context\":[\"- Session management\"],\"content\":\"- Two-factor authentication support\"}"
  }
}
```

**Result**: The new requirement "- Two-factor authentication support" is inserted between the password hashing and session management requirements.

### 2. Replace Operation

**Purpose**: Modifies existing content by replacing it with new content.

**When to Use**:
- Marking tasks as complete (`[ ]` → `[x]`)
- Updating existing requirements with new information
- Fixing typos or outdated content
- Changing implementation details

**How it Works**:
- The last line of `before_context` is replaced with the new `content`
- The replacement happens at the boundary of the context landmarks
- This is the most common operation for task completion

**Example**:
```json
{
  "name": "update_spec",
  "arguments": {
    "project_name": "my-project",
    "spec_name": "20240101_user_auth",
    "operation": "context_patch",
    "context_patch": "{\"file_type\":\"tasks\",\"operation\":\"replace\",\"section_context\":\"## Phase 1\",\"before_context\":[\"## Phase 1\",\"- [ ] Implement user authentication\"],\"after_context\":[\"- [ ] Add password hashing\"],\"content\":\"- [x] Implement user authentication\"}"
  }
}
```

**Result**: The task "- [ ] Implement user authentication" is replaced with "- [x] Implement user authentication" (marked as complete).

### 3. Delete Operation

**Purpose**: Removes existing content without adding new content.

**When to Use**:
- Removing outdated requirements
- Deleting completed tasks that are no longer relevant
- Cleaning up obsolete implementation notes

**How it Works**:
- The last line of `before_context` is removed entirely
- No new content is added (content field should be empty)
- The surrounding context remains intact

**Example**:
```json
{
  "name": "update_spec",
  "arguments": {
    "project_name": "my-project",
    "spec_name": "20240101_user_auth",
    "operation": "context_patch",
    "context_patch": "{\"file_type\":\"notes\",\"operation\":\"delete\",\"before_context\":[\"## Outdated Approach\",\"- Use legacy authentication system\"],\"after_context\":[\"## Current Implementation\"],\"content\":\"\"}"
  }
}
```

**Result**: The line "- Use legacy authentication system" is removed, leaving the section headers intact.

## Context Selection Best Practices

### Effective Context Patterns

**✅ Good Context Selection**:
- Use 3-5 lines of specific, unique text
- Include distinctive phrases unlikely to appear elsewhere
- Choose structurally stable elements (section headers, specific requirements)
- Use exact text from current content (case-sensitive, whitespace-sensitive)

**❌ Poor Context Selection**:
- Generic words that appear multiple times ("TODO", "implement", "add")
- Single-line context (insufficient uniqueness)
- Overly long lines that might have formatting variations
- Content that appears in multiple sections without disambiguation

### Context Requirements

1. **Minimum Context**: At least one of `before_context` or `after_context` must be provided
2. **Uniqueness**: Context should be unique enough to identify the exact location
3. **Stability**: Choose context that won't change frequently
4. **Precision**: Use exact text from the current file content

### Section Context Usage

Use `section_context` when:
- The same content appears in multiple sections
- You need to disambiguate between similar content
- The target is within a specific section header

Example:
```json
{
  "section_context": "## Phase 1: Authentication",
  "before_context": ["- [ ] Implement OAuth2"],
  "after_context": ["- [ ] Add password validation"]
}
```

## Operation Selection Guide

### Choose Insert When:
- Adding new content between existing items
- Expanding requirements or task lists
- Adding implementation notes in specific locations
- Building up specifications incrementally

### Choose Replace When:
- Marking tasks complete (`[ ]` → `[x]`)
- Updating existing content with new information
- Fixing typos or correcting content
- Changing implementation details

### Choose Delete When:
- Removing outdated or obsolete content
- Cleaning up completed tasks
- Removing requirements that are no longer needed
- Simplifying specifications

## Error Handling and Recovery

### Common Error Patterns

**"Context not found"**:
- Load current content to verify context still exists
- Try broader context (fewer lines) or more specific context
- Check for typos in before_context and after_context
- Use section_context to limit search scope

**"Multiple matches"**:
- Add section_context to disambiguate
- Use more specific before_context and after_context
- Include more lines of context to make match unique

**"Ambiguous match"**:
- Use longer, more distinctive context sequences
- Add section_context when content appears in multiple places
- Ensure exact whitespace and capitalization from current content

### Recovery Process

1. **Load Current Content**: Use `load_spec` to see actual current state
2. **Copy Exact Text**: Use precise text from loaded content
3. **Choose Better Context**: Select more unique, specific surrounding lines
4. **Add Section Context**: Use section_context when content appears in multiple places
5. **Retry with Precision**: Apply context_patch with refined, exact context

## Token Efficiency Benefits

Context patching provides significant token efficiency improvements:

- **70-90% token reduction** compared to full file replacement
- **Precise targeting** without line number precision
- **Minimal context overhead** for maximum efficiency
- **Preserves existing content** while making targeted changes

## Best Practices Summary

1. **Always load current content first** before context patching
2. **Use 3-5 lines of context** for reliable matching
3. **Choose unique, specific context** that won't appear elsewhere
4. **Use section_context** for disambiguation when needed
5. **Test with small changes first** to verify context selection
6. **Keep context stable** - avoid frequently changing text
7. **Use exact text matching** - copy from current content
8. **Prefer replace for task completion** - most common use case
9. **Use insert for adding content** - preserves existing structure
10. **Use delete sparingly** - only when content is truly obsolete

## Integration with MCP Tools

All context operations are accessed through the `update_spec` MCP tool:

```json
{
  "name": "update_spec",
  "arguments": {
    "project_name": "project-name",
    "spec_name": "spec-name", 
    "operation": "context_patch",
    "context_patch": "{\"file_type\":\"tasks\",\"operation\":\"replace\",\"before_context\":[\"- [ ] Task\"],\"after_context\":[\"- [ ] Next\"],\"content\":\"- [x] Task\"}"
  }
}
```

This provides a consistent interface for all three operation types while maintaining the efficiency and precision benefits of context-based patching.