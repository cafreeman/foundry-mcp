//! Helper functions for improving context patch reliability

use crate::types::spec::ContextPatch;

/// Preprocess a context patch to handle common LLM omissions
/// This function adds empty lines to the context when they're likely missing
pub fn preprocess_context_patch(patch: &mut ContextPatch, content: &str) {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Check if the patch is missing empty lines between contexts
    if let Some(empty_lines) = find_missing_empty_lines(patch, &lines) {
        // Insert empty lines at the beginning of after_context
        let mut new_after = empty_lines;
        new_after.extend(patch.after_context.clone());
        patch.after_context = new_after;
    }
}

/// Find empty lines that should be added between before and after context
/// Returns Some(empty_lines) if empty lines are found between contexts, None otherwise
fn find_missing_empty_lines(patch: &ContextPatch, lines: &[String]) -> Option<Vec<String>> {
    // If either context is empty, don't modify
    if patch.before_context.is_empty() || patch.after_context.is_empty() {
        return None;
    }

    let last_before = patch.before_context.last()?;
    let first_after = patch.after_context.first()?;

    // Find the position of last_before in the document
    lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.trim() == last_before.trim())
        .and_then(|(before_idx, _)| {
            // Find the position of first_after after last_before
            lines
                .iter()
                .enumerate()
                .skip(before_idx + 1)
                .find(|(_, line)| line.trim() == first_after.trim())
                .and_then(|(after_idx, _)| {
                    // Extract lines between before_idx and after_idx
                    let between_lines = &lines[(before_idx + 1)..after_idx];

                    // Collect consecutive empty lines from the beginning
                    let empty_lines: Vec<String> = between_lines
                        .iter()
                        .take_while(|line| line.trim().is_empty())
                        .cloned()
                        .collect();

                    if empty_lines.is_empty() {
                        None
                    } else {
                        Some(empty_lines)
                    }
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::spec::{ContextOperation, SpecFileType};

    #[test]
    fn test_preprocess_adds_missing_empty_lines() {
        let content = "Line 1\nLine 2\n\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        preprocess_context_patch(&mut patch, content);

        // Should have added the empty line
        assert_eq!(patch.after_context.len(), 2);
        assert_eq!(patch.after_context[0], "");
        assert_eq!(patch.after_context[1], "Line 3");
    }

    #[test]
    fn test_preprocess_no_change_when_no_empty_lines() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        let original_after = patch.after_context.clone();
        preprocess_context_patch(&mut patch, content);

        // Should not have changed
        assert_eq!(patch.after_context, original_after);
    }

    #[test]
    fn test_preprocess_multiple_empty_lines() {
        let content = "Line 1\nLine 2\n\n\n\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        preprocess_context_patch(&mut patch, content);

        // Should have added multiple empty lines
        assert_eq!(patch.after_context.len(), 4);
        assert_eq!(patch.after_context[0], "");
        assert_eq!(patch.after_context[1], "");
        assert_eq!(patch.after_context[2], "");
        assert_eq!(patch.after_context[3], "Line 3");
    }

    #[test]
    fn test_preprocess_empty_before_context() {
        let content = "Line 1\nLine 2\n\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Insert,
            before_context: vec![],
            after_context: vec!["Line 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        let original_after = patch.after_context.clone();
        preprocess_context_patch(&mut patch, content);

        // Should not modify when before_context is empty
        assert_eq!(patch.after_context, original_after);
    }

    #[test]
    fn test_preprocess_empty_after_context() {
        let content = "Line 1\nLine 2\n\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 2".to_string()],
            after_context: vec![],
            content: "New Line".to_string(),
            section_context: None,
        };

        let original_after = patch.after_context.clone();
        preprocess_context_patch(&mut patch, content);

        // Should not modify when after_context is empty
        assert_eq!(patch.after_context, original_after);
    }

    #[test]
    fn test_preprocess_with_whitespace_variations() {
        let content = "  Line 1  \n  Line 2  \n\n  Line 3  \n  Line 4  ";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        preprocess_context_patch(&mut patch, content);

        // Should have added the empty line (handles trimmed comparison)
        assert_eq!(patch.after_context.len(), 2);
        assert_eq!(patch.after_context[0], "");
        assert_eq!(patch.after_context[1], "Line 3");
    }

    #[test]
    fn test_preprocess_markdown_headers() {
        let content = "## Header 1\n\nContent 1\n\n## Header 2\n\nContent 2";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Content 1".to_string()],
            after_context: vec!["## Header 2".to_string()],
            content: "Modified Content 1".to_string(),
            section_context: None,
        };

        preprocess_context_patch(&mut patch, content);

        // Should have added the empty line between content and header
        assert_eq!(patch.after_context.len(), 2);
        assert_eq!(patch.after_context[0], "");
        assert_eq!(patch.after_context[1], "## Header 2");
    }

    #[test]
    fn test_preprocess_task_list() {
        let content = "## Tasks\n\n- [x] Task 1\n- [ ] Task 2\n\n## Next Section";

        let mut patch = ContextPatch {
            file_type: SpecFileType::TaskList,
            operation: ContextOperation::Replace,
            before_context: vec!["- [x] Task 1".to_string(), "- [ ] Task 2".to_string()],
            after_context: vec!["## Next Section".to_string()],
            content: "- [x] Task 2".to_string(),
            section_context: None,
        };

        preprocess_context_patch(&mut patch, content);

        // Should have added the empty line
        assert_eq!(patch.after_context.len(), 2);
        assert_eq!(patch.after_context[0], "");
        assert_eq!(patch.after_context[1], "## Next Section");
    }

    #[test]
    fn test_preprocess_no_match_found() {
        let content = "Line 1\nLine 2\n\nLine 3\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Not Found 1".to_string(), "Not Found 2".to_string()],
            after_context: vec!["Not Found 3".to_string()],
            content: "New Line".to_string(),
            section_context: None,
        };

        let original_after = patch.after_context.clone();
        preprocess_context_patch(&mut patch, content);

        // Should not change if context not found in document
        assert_eq!(patch.after_context, original_after);
    }

    #[test]
    fn test_preprocess_partial_match() {
        let content = "Line 1\nLine 2\nLine 3\n\nLine 4";

        let mut patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 2".to_string()],
            after_context: vec!["Line 4".to_string()], // Line 3 is skipped
            content: "New Line".to_string(),
            section_context: None,
        };

        let original_after = patch.after_context.clone();
        preprocess_context_patch(&mut patch, content);

        // Should not add empty lines if there's non-empty content between
        assert_eq!(patch.after_context, original_after);
    }

    #[test]
    fn test_find_missing_empty_lines_found() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "".to_string(),
            "Line 3".to_string(),
        ];

        let patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New".to_string(),
            section_context: None,
        };

        let result = find_missing_empty_lines(&patch, &lines);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![""]);
    }

    #[test]
    fn test_find_missing_empty_lines_not_found() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];

        let patch = ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Replace,
            before_context: vec!["Line 1".to_string(), "Line 2".to_string()],
            after_context: vec!["Line 3".to_string()],
            content: "New".to_string(),
            section_context: None,
        };

        assert!(find_missing_empty_lines(&patch, &lines).is_none());
    }
}
