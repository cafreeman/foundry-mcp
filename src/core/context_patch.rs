//! Context-based patching engine for precise markdown file updates

use crate::types::spec::{ContextOperation, ContextPatch, ContextPatchResult, MatchingConfig};
use anyhow::Result;

/// Context matching engine for applying patches to markdown content
pub struct ContextMatcher {
    content: String,
    lines: Vec<String>,
}

impl ContextMatcher {
    /// Create a new context matcher with the given content
    pub fn new(content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self { content, lines }
    }

    /// Apply a context patch to the content
    pub fn apply_patch(&mut self, patch: &ContextPatch) -> Result<ContextPatchResult> {
        // Validate patch requirements
        self.validate_patch(patch)?;

        // Find the target location using context matching
        let match_result = self.find_context_match(patch)?;

        if let Some((position, confidence)) = match_result {
            // Apply the operation at the found position
            let lines_modified = self.apply_operation_at_position(patch, position)?;

            // Update the content string from modified lines
            self.content = self.lines.join("\n");

            Ok(ContextPatchResult {
                success: true,
                match_confidence: Some(confidence),
                lines_modified,
                patch_type: format!("{:?}", patch.operation),
                error_message: None,
                suggestions: vec![],
            })
        } else {
            // Context not found - provide helpful error and suggestions
            let suggestions = self.generate_match_suggestions(patch);
            Ok(ContextPatchResult {
                success: false,
                match_confidence: None,
                lines_modified: 0,
                patch_type: format!("{:?}", patch.operation),
                error_message: Some(format!(
                    "Context not found: Could not locate the specified before/after context{}",
                    if let Some(ref section) = patch.section_context {
                        format!(" in section '{}'", section)
                    } else {
                        String::new()
                    }
                )),
                suggestions,
            })
        }
    }

    /// Get the current content after patches have been applied
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Validate that the patch has the required information
    fn validate_patch(&self, patch: &ContextPatch) -> Result<()> {
        if patch.before_context.is_empty() && patch.after_context.is_empty() {
            anyhow::bail!("At least one of before_context or after_context must be provided");
        }

        match patch.operation {
            ContextOperation::Insert | ContextOperation::Replace => {
                if patch.content.is_empty() {
                    anyhow::bail!("Content cannot be empty for insert/replace operations");
                }
            }
            ContextOperation::Delete => {
                // Delete operations don't require content
            }
        }

        Ok(())
    }

    /// Find the position where the context matches
    /// Returns (line_position, confidence_score) if found
    fn find_context_match(&self, patch: &ContextPatch) -> Result<Option<(usize, f32)>> {
        // First try section-aware matching if section_context is provided
        if let Some(ref section) = patch.section_context {
            if let Some(result) = self.find_match_in_section(patch, section)? {
                return Ok(Some(result));
            }
        }

        // Fall back to full document matching
        self.find_match_in_document(patch)
    }

    /// Find context match within a specific section
    fn find_match_in_section(
        &self,
        patch: &ContextPatch,
        section: &str,
    ) -> Result<Option<(usize, f32)>> {
        // Find the section boundaries
        let (section_start, section_end) = self.find_section_boundaries(section)?;

        if section_start.is_none() {
            return Ok(None); // Section not found
        }

        let start_line = section_start.unwrap();
        let end_line = section_end.unwrap_or(self.lines.len());

        // Search within the section boundaries
        self.find_match_in_range(patch, start_line, end_line)
    }

    /// Find section boundaries (start line, end line)
    fn find_section_boundaries(
        &self,
        section_header: &str,
    ) -> Result<(Option<usize>, Option<usize>)> {
        let normalized_header = self.normalize_text(section_header, &patch_default_config());

        let mut section_start = None;
        let mut section_end = None;

        for (i, line) in self.lines.iter().enumerate() {
            let normalized_line = self.normalize_text(line, &patch_default_config());

            // Check if this line is the target section header
            if section_start.is_none() && normalized_line.contains(&normalized_header) {
                section_start = Some(i);
                continue;
            }

            // If we're in a section, check for the next section header
            if section_start.is_some() && section_end.is_none() {
                // Look for markdown headers (lines starting with #)
                if line.trim_start().starts_with('#') {
                    section_end = Some(i);
                    break;
                }
            }
        }

        Ok((section_start, section_end))
    }

    /// Find context match within the full document
    fn find_match_in_document(&self, patch: &ContextPatch) -> Result<Option<(usize, f32)>> {
        self.find_match_in_range(patch, 0, self.lines.len())
    }

    /// Find context match within a specific line range
    fn find_match_in_range(
        &self,
        patch: &ContextPatch,
        start: usize,
        end: usize,
    ) -> Result<Option<(usize, f32)>> {
        let mut best_match: Option<(usize, f32)> = None;

        // Try exact matching first
        if let Some(position) = self.find_exact_match(patch, start, end)? {
            return Ok(Some((position, 1.0))); // Perfect confidence
        }

        // Fall back to fuzzy matching
        if let Some((position, confidence)) = self.find_fuzzy_match(patch, start, end)? {
            best_match = Some((position, confidence));
        }

        Ok(best_match)
    }

    /// Find exact context match
    fn find_exact_match(
        &self,
        patch: &ContextPatch,
        start: usize,
        end: usize,
    ) -> Result<Option<usize>> {
        for i in start..end {
            if self.matches_context_at_position(patch, i, true)? {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// Find fuzzy context match
    fn find_fuzzy_match(
        &self,
        patch: &ContextPatch,
        start: usize,
        end: usize,
    ) -> Result<Option<(usize, f32)>> {
        let mut best_match: Option<(usize, f32)> = None;

        for i in start..end {
            if let Some(confidence) = self.fuzzy_match_at_position(patch, i)? {
                if confidence >= patch.match_config.similarity_threshold {
                    if best_match.is_none() || confidence > best_match.unwrap().1 {
                        best_match = Some((i, confidence));
                    }
                }
            }
        }

        Ok(best_match)
    }

    /// Check if context matches exactly at a given position
    /// Position represents the insertion point (where new content would go)
    fn matches_context_at_position(
        &self,
        patch: &ContextPatch,
        position: usize,
        exact: bool,
    ) -> Result<bool> {
        // Check before_context (should match lines immediately before the position)
        if !patch.before_context.is_empty() {
            let before_end = position; // Lines before the insertion point
            let before_start = if before_end >= patch.before_context.len() {
                before_end - patch.before_context.len()
            } else {
                return Ok(false); // Not enough lines before
            };

            for (i, context_line) in patch.before_context.iter().enumerate() {
                let line_idx = before_start + i;
                if line_idx >= self.lines.len() {
                    return Ok(false);
                }

                if !self.lines_match(
                    context_line,
                    &self.lines[line_idx],
                    &patch.match_config,
                    exact,
                ) {
                    return Ok(false);
                }
            }
        }

        // Check after_context (should match lines immediately after the position)
        if !patch.after_context.is_empty() {
            let after_start = position; // Lines after the insertion point

            for (i, context_line) in patch.after_context.iter().enumerate() {
                let line_idx = after_start + i;
                if line_idx >= self.lines.len() {
                    return Ok(false);
                }

                if !self.lines_match(
                    context_line,
                    &self.lines[line_idx],
                    &patch.match_config,
                    exact,
                ) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Calculate fuzzy match confidence at a given position
    fn fuzzy_match_at_position(
        &self,
        patch: &ContextPatch,
        position: usize,
    ) -> Result<Option<f32>> {
        let mut total_similarity = 0.0;
        let mut total_comparisons = 0;

        // Check before_context with fuzzy matching
        if !patch.before_context.is_empty() {
            let before_end = position;
            let before_start = if before_end >= patch.before_context.len() {
                before_end - patch.before_context.len()
            } else {
                return Ok(None); // Not enough lines before
            };

            for (i, context_line) in patch.before_context.iter().enumerate() {
                let line_idx = before_start + i;
                if line_idx >= self.lines.len() {
                    continue;
                }

                let similarity = self.calculate_line_similarity(
                    context_line,
                    &self.lines[line_idx],
                    &patch.match_config,
                );
                total_similarity += similarity;
                total_comparisons += 1;
            }
        }

        // Check after_context with fuzzy matching
        if !patch.after_context.is_empty() {
            let after_start = position;

            for (i, context_line) in patch.after_context.iter().enumerate() {
                let line_idx = after_start + i;
                if line_idx >= self.lines.len() {
                    continue;
                }

                let similarity = self.calculate_line_similarity(
                    context_line,
                    &self.lines[line_idx],
                    &patch.match_config,
                );
                total_similarity += similarity;
                total_comparisons += 1;
            }
        }

        if total_comparisons > 0 {
            Ok(Some(total_similarity / total_comparisons as f32))
        } else {
            Ok(None)
        }
    }

    /// Check if two lines match according to the matching configuration
    fn lines_match(&self, line1: &str, line2: &str, config: &MatchingConfig, exact: bool) -> bool {
        if exact {
            self.normalize_text(line1, config) == self.normalize_text(line2, config)
        } else {
            let similarity = self.calculate_line_similarity(line1, line2, config);
            similarity >= config.similarity_threshold
        }
    }

    /// Calculate similarity between two lines (0.0 to 1.0)
    fn calculate_line_similarity(&self, line1: &str, line2: &str, config: &MatchingConfig) -> f32 {
        let norm1 = self.normalize_text(line1, config);
        let norm2 = self.normalize_text(line2, config);

        if norm1 == norm2 {
            return 1.0;
        }

        // Use a simple similarity metric (can be enhanced with external crates)
        let max_len = norm1.len().max(norm2.len());
        if max_len == 0 {
            return 1.0;
        }

        let common_chars = norm1
            .chars()
            .zip(norm2.chars())
            .filter(|(c1, c2)| c1 == c2)
            .count();

        common_chars as f32 / max_len as f32
    }

    /// Normalize text according to matching configuration
    fn normalize_text(&self, text: &str, config: &MatchingConfig) -> String {
        let mut result = text.to_string();

        if config.ignore_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if config.case_insensitive_fallback {
            result = result.to_lowercase();
        }

        result
    }

    /// Apply the patch operation at the specified position
    fn apply_operation_at_position(
        &mut self,
        patch: &ContextPatch,
        position: usize,
    ) -> Result<usize> {
        match patch.operation {
            ContextOperation::Insert => {
                self.lines.insert(position, patch.content.clone());
                Ok(1)
            }
            ContextOperation::Replace => {
                if position < self.lines.len() {
                    self.lines[position] = patch.content.clone();
                    Ok(1)
                } else {
                    anyhow::bail!(
                        "Cannot replace line at position {}: out of bounds",
                        position
                    );
                }
            }
            ContextOperation::Delete => {
                if position < self.lines.len() {
                    self.lines.remove(position);
                    Ok(1)
                } else {
                    anyhow::bail!("Cannot delete line at position {}: out of bounds", position);
                }
            }
        }
    }

    /// Generate helpful suggestions when context matching fails
    fn generate_match_suggestions(&self, patch: &ContextPatch) -> Vec<String> {
        let mut suggestions = vec![
            "Check if content has changed since last load".to_string(),
            "Try broader context (fewer lines) or more specific context".to_string(),
            "Use load_spec to see current content".to_string(),
        ];

        if patch.section_context.is_some() {
            suggestions.push("Verify section header exists and is spelled correctly".to_string());
        } else {
            suggestions.push("Consider adding section_context to disambiguate".to_string());
        }

        if patch.before_context.len() + patch.after_context.len() < 3 {
            suggestions.push("Consider providing more context lines (3-5 recommended)".to_string());
        }

        suggestions
    }
}

/// Get default matching configuration
fn patch_default_config() -> MatchingConfig {
    MatchingConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::spec::{ContextOperation, MatchingConfig, SpecFileType};

    fn create_test_patch() -> ContextPatch {
        ContextPatch {
            file_type: SpecFileType::Spec,
            operation: ContextOperation::Insert,
            section_context: None,
            before_context: vec!["- User registration".to_string()],
            after_context: vec!["- Password hashing".to_string()],
            content: "- Email verification".to_string(),
            match_config: MatchingConfig::default(),
        }
    }

    #[test]
    fn test_exact_context_match() {
        let content =
            "## Requirements\n- User registration\n- Password hashing\n- Session management"
                .to_string();
        let mut matcher = ContextMatcher::new(content);
        let patch = create_test_patch();

        let result = matcher.apply_patch(&patch).unwrap();
        assert!(result.success);
        assert_eq!(result.match_confidence, Some(1.0));
        assert_eq!(result.lines_modified, 1);

        let updated_content = matcher.get_content();
        assert!(updated_content.contains("- Email verification"));
    }

    #[test]
    fn test_context_not_found() {
        let content = "## Requirements\n- Different content\n- Not matching".to_string();
        let mut matcher = ContextMatcher::new(content);
        let patch = create_test_patch();

        let result = matcher.apply_patch(&patch).unwrap();
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_section_aware_matching() {
        let content = "## Other Section\n- User registration\n- Password hashing\n## Requirements\n- User registration\n- Password hashing\n- Session management".to_string();
        let mut matcher = ContextMatcher::new(content);

        let mut patch = create_test_patch();
        patch.section_context = Some("## Requirements".to_string());

        let result = matcher.apply_patch(&patch).unwrap();
        assert!(result.success);

        let updated_content = matcher.get_content();
        // Should insert in the Requirements section, not the Other Section
        let lines: Vec<&str> = updated_content.lines().collect();
        let requirements_start = lines
            .iter()
            .position(|&line| line == "## Requirements")
            .unwrap();
        let email_verification_pos = lines
            .iter()
            .position(|&line| line == "- Email verification")
            .unwrap();
        assert!(email_verification_pos > requirements_start);
    }
}
