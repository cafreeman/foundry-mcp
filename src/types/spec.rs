//! Spec-related type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::backends::ResourceLocator;

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
    pub path: PathBuf, // Keep for backward compatibility (deprecated)
    pub project_name: String,

    // New optional fields for backend abstraction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<ResourceLocator>,

    // Existing fields
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
#[serde(rename_all = "lowercase")]
pub enum SpecFileType {
    Spec,
    Notes,
    TaskList,
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
