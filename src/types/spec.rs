//! Spec-related type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Spec content data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContentData {
    pub spec: String,
    pub notes: String,
    pub tasks: String,
}

/// Core specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    pub created_at: String,
    pub path: PathBuf,
    pub project_name: String,
    pub content: SpecContentData,
}

/// Spec creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecConfig {
    pub project_name: String,
    pub feature_name: String,
    pub content: SpecContentData,
}

/// Spec metadata for listing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub name: String,
    pub created_at: String,
    pub feature_name: String,
    pub project_name: String,
}

/// Spec filtering criteria for advanced queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecFilter {
    pub feature_name_contains: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<usize>,
}

/// Spec file types for content updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecFileType {
    Spec,
    Notes,
    TaskList,
}

/// Context-based patch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextOperation {
    Insert,
    Replace,
    Delete,
}

/// Configuration for context matching behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingConfig {
    /// Whether to ignore whitespace differences when matching
    pub ignore_whitespace: bool,
    /// Similarity threshold for fuzzy matching (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Whether to use case-insensitive matching as fallback
    pub case_insensitive_fallback: bool,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            ignore_whitespace: true,
            similarity_threshold: 0.8,
            case_insensitive_fallback: true,
        }
    }
}

/// Context-based patch for precise content updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPatch {
    /// Which file to update (spec.md, task-list.md, or notes.md)
    pub file_type: SpecFileType,
    /// What operation to perform
    pub operation: ContextOperation,
    /// Optional section context for disambiguation (e.g., "## Requirements")
    pub section_context: Option<String>,
    /// Lines of context that should appear before the target location
    pub before_context: Vec<String>,
    /// Lines of context that should appear after the target location
    pub after_context: Vec<String>,
    /// Content to insert, replace, or delete
    pub content: String,
    /// Configuration for matching behavior
    pub match_config: MatchingConfig,
}

/// Result of a context patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPatchResult {
    /// Whether the patch was successfully applied
    pub success: bool,
    /// Confidence score of the context match (0.0 to 1.0)
    pub match_confidence: Option<f32>,
    /// Number of lines modified
    pub lines_modified: usize,
    /// Type of patch applied
    pub patch_type: String,
    /// Error message if patch failed
    pub error_message: Option<String>,
    /// Suggestions for fixing failed patches
    pub suggestions: Vec<String>,
}

/// Content validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentValidationStatus {
    pub spec_valid: bool,
    pub notes_valid: bool,
    pub task_list_valid: bool,
}

/// Spec validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecValidationResult {
    pub spec_name: String,
    pub project_name: String,
    pub spec_file_exists: bool,
    pub notes_file_exists: bool,
    pub task_list_file_exists: bool,
    pub content_validation: ContentValidationStatus,
    pub validation_errors: Vec<String>,
}

impl SpecValidationResult {
    /// Check if the spec is completely valid
    pub fn is_valid(&self) -> bool {
        self.spec_file_exists
            && self.notes_file_exists
            && self.task_list_file_exists
            && self.content_validation.spec_valid
            && self.content_validation.notes_valid
            && self.content_validation.task_list_valid
            && self.validation_errors.is_empty()
    }

    /// Get summary of validation issues
    pub fn summary(&self) -> String {
        if self.is_valid() {
            "Spec is valid".to_string()
        } else {
            format!(
                "Spec validation failed: {} errors",
                self.validation_errors.len()
            )
        }
    }
}
