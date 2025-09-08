//! Helper functions for improving context patch reliability

use crate::types::spec::ContextPatch;

/// Preprocess a context patch to handle common LLM omissions
/// This function adds empty lines to the context when they're likely missing
pub fn preprocess_context_patch(patch: &mut ContextPatch, content: &str) {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    
    // Check if the patch is missing empty lines between contexts
    if should_add_empty_lines_between_contexts(patch, &lines) {
        add_missing_empty_lines(patch, &lines);
    }
}

/// Check if we should add empty lines between before and after context
fn should_add_empty_lines_between_contexts(patch: &ContextPatch, lines: &[String]) -> bool {
    // If either context is empty, don't modify
    if patch.before_context.is_empty() || patch.after_context.is_empty() {
        return false;
    }
    
    // Get the last line of before_context and first line of after_context
    let last_before = patch.before_context.last().unwrap();
    let first_after = patch.after_context.first().unwrap();
    
    // Check if these appear in the document with empty lines between them
    for i in 0..lines.len().saturating_sub(1) {
        if lines[i].trim() == last_before.trim() {
            // Look ahead for the first_after line
            for j in (i + 1)..lines.len() {
                if lines[j].trim() == first_after.trim() {
                    // Check if there are empty lines between them
                    for k in (i + 1)..j {
                        if lines[k].trim().is_empty() {
                            return true; // Found empty lines between the contexts
                        }
                    }
                    break;
                }
            }
        }
    }
    
    false
}

/// Add missing empty lines to the patch context
fn add_missing_empty_lines(patch: &mut ContextPatch, lines: &[String]) {
    // Find where the contexts appear in the document
    let last_before = patch.before_context.last().unwrap();
    let first_after = patch.after_context.first().unwrap();
    
    for i in 0..lines.len().saturating_sub(1) {
        if lines[i].trim() == last_before.trim() {
            // Look ahead for the first_after line
            for j in (i + 1)..lines.len() {
                if lines[j].trim() == first_after.trim() {
                    // Collect empty lines between them
                    let mut empty_lines = Vec::new();
                    for k in (i + 1)..j {
                        if lines[k].trim().is_empty() {
                            empty_lines.push(lines[k].clone());
                        } else {
                            // If we encounter a non-empty line that's not first_after, stop
                            break;
                        }
                    }
                    
                    // If we found empty lines, add them to after_context
                    if !empty_lines.is_empty() {
                        // Insert empty lines at the beginning of after_context
                        let mut new_after = empty_lines;
                        new_after.extend(patch.after_context.clone());
                        patch.after_context = new_after;
                    }
                    return;
                }
            }
        }
    }
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
}