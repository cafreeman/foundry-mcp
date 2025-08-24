//! Spec-related type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    pub created_at: String,
    pub path: PathBuf,
    pub project_name: String,
    pub spec_content: String,
    pub notes: String,
    pub tasks: String,
}

/// Spec creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecConfig {
    pub project_name: String,
    pub feature_name: String,
    pub spec_content: String,
    pub notes: String,
    pub tasks: String,
}

/// Spec metadata for listing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub name: String,
    pub created_at: String,
    pub feature_name: String,
    pub project_name: String,
}
