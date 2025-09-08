//! Integration tests for context patching with preprocessing

use foundry_mcp::core::context_patch::ContextMatcher;
use foundry_mcp::types::spec::{ContextPatch, ContextOperation, SpecFileType};

#[test]
fn test_llm_style_patch_missing_empty_lines() {
    // This is the exact scenario from the bug report
    let content = r#"# Task List

## Phase 1: Initial setup

- [x] Add isChatGptServer utility and unit tests
- [ ] Extend PromoBadge with SYSTEM type and styles (or reuse INTERNAL with label SYSTEM)

## Phase 2: ToolSelector enhancements

- [ ] Add new feature
- [ ] Test the feature"#;

    // LLM-style patch without empty lines
    let patch = ContextPatch {
        file_type: SpecFileType::TaskList,
        operation: ContextOperation::Replace,
        before_context: vec![
            "- [x] Add isChatGptServer utility and unit tests".to_string(),
            "- [ ] Extend PromoBadge with SYSTEM type and styles (or reuse INTERNAL with label SYSTEM)".to_string(),
        ],
        after_context: vec![
            "## Phase 2: ToolSelector enhancements".to_string(),
        ],
        content: "- [x] Extend PromoBadge with SYSTEM type and styles (or reuse INTERNAL with label SYSTEM)".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success, "Patch should succeed with preprocessing");
    assert_eq!(result.lines_modified, 1, "Should modify exactly one line");
    
    // Verify the content was replaced, not duplicated
    let updated = matcher.get_content();
    let count = updated.matches("Extend PromoBadge").count();
    assert_eq!(count, 1, "Should have exactly one 'Extend PromoBadge' line");
    
    // Verify the task was marked as complete
    assert!(updated.contains("- [x] Extend PromoBadge"), "Task should be marked complete");
}

#[test]
fn test_multiple_empty_lines_between_sections() {
    let content = r#"## Section 1

Content 1


## Section 2

Content 2"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec!["Content 1".to_string()],
        after_context: vec!["## Section 2".to_string()],
        content: "Updated Content 1".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Updated Content 1"));
    // Empty lines should be preserved
    assert!(updated.contains("\n\n\n## Section 2") || updated.contains("\n\n## Section 2"));
}

#[test]
fn test_patch_with_complete_context_still_works() {
    // Ensure backward compatibility - patches with empty lines should still work
    let content = r#"Line 1
Line 2

Line 3
Line 4"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
        ],
        after_context: vec![
            "".to_string(),  // Empty line included
            "Line 3".to_string(),
        ],
        content: "New Line 2".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    assert_eq!(result.match_confidence, Some(1.0)); // Perfect match
}

#[test]
fn test_complex_markdown_with_mixed_empty_lines() {
    let content = r#"# Main Title

## Introduction

This is the introduction paragraph.

### Subsection 1

- Item 1
- Item 2


### Subsection 2

More content here.

## Conclusion

Final thoughts."#;

    // Patch missing some empty lines
    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec![
            "- Item 1".to_string(),
            "- Item 2".to_string(),
        ],
        after_context: vec![
            "### Subsection 2".to_string(),
        ],
        content: "- Item 1 (updated)\n- Item 2 (updated)".to_string(),
        section_context: Some("### Subsection 1".to_string()),
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Item 1 (updated)"));
    assert!(updated.contains("Item 2 (updated)"));
}

#[test]
fn test_insert_operation_with_missing_empty_lines() {
    let content = r#"Line 1
Line 2

Line 3"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Insert,
        before_context: vec!["Line 2".to_string()],
        after_context: vec!["Line 3".to_string()],
        content: "Inserted Line".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Line 2\nInserted Line"));
}

#[test]
fn test_delete_operation_with_missing_empty_lines() {
    let content = r#"Line 1
Line 2
Line to delete

Line 3"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Delete,
        before_context: vec![
            "Line 2".to_string(),
            "Line to delete".to_string(),
        ],
        after_context: vec!["Line 3".to_string()],
        content: String::new(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(!updated.contains("Line to delete"));
}

#[test]
fn test_edge_case_empty_lines_at_document_boundaries() {
    let content = r#"

Line 1
Line 2

"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec!["Line 1".to_string()],
        after_context: vec!["Line 2".to_string()],
        content: "Updated Line 1".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Updated Line 1"));
}

#[test]
fn test_real_world_spec_update() {
    // Simulate a real spec file update scenario
    let content = r#"# Feature Specification

## Requirements

- User authentication required
- Data validation must be performed

## Implementation

### Phase 1
- Setup database
- Create models

### Phase 2
- Build API endpoints
- Add authentication

## Testing

- Unit tests for all functions
- Integration tests for API"#;

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec![
            "- Setup database".to_string(),
            "- Create models".to_string(),
        ],
        after_context: vec!["### Phase 2".to_string()],
        content: "- Setup database ✓\n- Create models ✓\n- Add migrations".to_string(),
        section_context: Some("### Phase 1".to_string()),
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Setup database ✓"));
    assert!(updated.contains("Add migrations"));
    
    // Verify structure is maintained
    assert!(updated.contains("### Phase 1"));
    assert!(updated.contains("### Phase 2"));
}

#[test]
fn test_windows_line_endings() {
    // Test with CRLF line endings
    let content = "Line 1\r\nLine 2\r\n\r\nLine 3\r\nLine 4";

    let patch = ContextPatch {
        file_type: SpecFileType::Spec,
        operation: ContextOperation::Replace,
        before_context: vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
        ],
        after_context: vec!["Line 3".to_string()],
        content: "Updated Line 2".to_string(),
        section_context: None,
    };

    let mut matcher = ContextMatcher::new(content.to_string());
    let result = matcher.apply_patch(&patch).unwrap();

    assert!(result.success);
    let updated = matcher.get_content();
    assert!(updated.contains("Updated Line 2"));
}