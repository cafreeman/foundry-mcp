# Context Patching Improvement Plan

## Problem Summary

The context patching feature is failing because LLMs are not including empty lines and whitespace in their context specifications. When the LLM provides context like:

```json
{
  "before_context": [
    "- [x] Add isChatGptServer utility",
    "- [ ] Extend PromoBadge with SYSTEM type"
  ],
  "after_context": [
    "## Phase 2: ToolSelector enhancements"
  ]
}
```

But the actual document has an empty line between these sections, the match fails.

## Root Causes

1. **Whitespace Sensitivity**: The current implementation requires exact line-by-line matching, including empty lines
2. **LLM Behavior**: LLMs naturally skip empty lines when copying text, as they appear insignificant
3. **Insufficient Guidance**: The error messages don't clearly explain the whitespace requirement
4. **No Fuzzy Whitespace Matching**: The system doesn't have a mode that's tolerant of whitespace differences

## Proposed Solutions

### Solution 1: Smart Whitespace Handling (Recommended)
Implement a "smart" context matching mode that:
- Ignores empty lines when matching context
- Optionally collapses multiple whitespace lines into one
- Preserves original formatting when applying patches

**Pros:**
- Works with natural LLM behavior
- More user-friendly
- Reduces token usage (no need to include empty lines)

**Cons:**
- May introduce ambiguity in some edge cases
- Requires careful implementation to preserve formatting

### Solution 2: Enhanced Error Messages with Examples
Improve error messages to show:
- The exact context that was searched for
- The closest match found in the document
- A visual diff showing what's different (including whitespace)

**Pros:**
- Helps LLMs learn the correct format
- Educational for users
- No changes to matching logic

**Cons:**
- Doesn't solve the fundamental problem
- Increases response size

### Solution 3: Context Normalization
Normalize both the document and the patch context before matching:
- Strip leading/trailing whitespace from lines
- Optionally ignore empty lines
- Use normalized version for matching, but apply to original

**Pros:**
- Robust against whitespace variations
- Maintains exact content when applying changes

**Cons:**
- May match unintended locations if context is too generic

### Solution 4: Multi-Mode Matching
Offer different matching modes:
- `strict`: Current behavior (exact match including whitespace)
- `normal`: Ignore empty lines and normalize whitespace (default)
- `fuzzy`: Use similarity algorithms with configurable threshold

**Pros:**
- Flexible for different use cases
- Backwards compatible
- User can choose based on needs

**Cons:**
- More complex API
- Requires documentation

## Implementation Plan

### Phase 1: Quick Fix (Immediate)
1. Add a whitespace-tolerant matching mode that ignores empty lines
2. Make this the default behavior
3. Update documentation to explain the behavior

### Phase 2: Enhanced Diagnostics (Short-term)
1. Improve error messages to show what was searched vs what exists
2. Add suggestions for fixing common issues
3. Include examples in error messages

### Phase 3: Comprehensive Solution (Long-term)
1. Implement multi-mode matching system
2. Add configuration options for whitespace handling
3. Create extensive test suite for edge cases
4. Update LLM guidance in system prompts

## Specific Code Changes

### 1. Add Whitespace-Tolerant Matching

```rust
// In context_patch.rs
impl ContextMatcher {
    /// Check if context matches at position, ignoring empty lines
    fn matches_context_ignoring_empty_lines(
        &self,
        patch: &ContextPatch,
        position: usize,
    ) -> Result<bool> {
        // Filter out empty lines from both patch and document
        let before_non_empty: Vec<_> = patch.before_context
            .iter()
            .filter(|line| !line.trim().is_empty())
            .collect();
        
        let after_non_empty: Vec<_> = patch.after_context
            .iter()
            .filter(|line| !line.trim().is_empty())
            .collect();
        
        // Match against non-empty lines in document
        // Implementation details...
    }
}
```

### 2. Improve Error Messages

```rust
fn generate_enhanced_error_message(&self, patch: &ContextPatch) -> String {
    let closest_match = self.find_closest_match(patch);
    
    format!(
        "Context not found. Looking for:\n\
        Before context:\n{}\n\
        After context:\n{}\n\n\
        Closest match found at line {}:\n{}\n\n\
        Hint: Make sure to include empty lines if they exist between your context lines.",
        // Format the contexts and closest match...
    )
}
```

### 3. Add Configuration Options

```rust
pub struct ContextPatchConfig {
    /// How to handle whitespace when matching
    pub whitespace_mode: WhitespaceMode,
    /// Minimum similarity threshold for fuzzy matching
    pub similarity_threshold: f32,
}

pub enum WhitespaceMode {
    /// Exact matching including all whitespace
    Strict,
    /// Ignore empty lines when matching
    IgnoreEmptyLines,
    /// Normalize all whitespace
    Normalize,
}
```

## Testing Strategy

1. **Unit Tests**: Test each matching mode with various whitespace scenarios
2. **Integration Tests**: Test with real-world document examples
3. **LLM Tests**: Test with actual LLM-generated patches
4. **Edge Cases**: 
   - Documents with no empty lines
   - Documents with multiple consecutive empty lines
   - Mixed line endings (CRLF vs LF)
   - Unicode whitespace characters

## Success Metrics

1. **Reliability**: 95%+ success rate for LLM-generated patches
2. **Performance**: No significant performance degradation
3. **Usability**: Clear error messages that help users fix issues
4. **Compatibility**: Backwards compatible with existing patches

## Timeline

- **Week 1**: Implement quick fix (whitespace-tolerant mode)
- **Week 2**: Add enhanced error messages
- **Week 3**: Implement multi-mode matching
- **Week 4**: Testing and documentation

## Risks and Mitigations

1. **Risk**: Breaking existing functionality
   - **Mitigation**: Keep strict mode available, extensive testing

2. **Risk**: Ambiguous matches
   - **Mitigation**: Require minimum context length, use section_context

3. **Risk**: Performance impact
   - **Mitigation**: Optimize algorithms, add caching

## Conclusion

The primary issue is a mismatch between how LLMs naturally represent text (without explicit empty lines) and how the system expects exact line-by-line matching. The recommended solution is to implement a smart whitespace handling mode that ignores empty lines by default while preserving the original document formatting. This will make the system more robust and user-friendly while maintaining the benefits of context-based patching.