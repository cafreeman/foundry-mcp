//! Project-related type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub created_at: String,
    pub path: PathBuf,
    pub vision: Option<String>,
    pub tech_stack: Option<String>,
    pub summary: Option<String>,
}

/// Project creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
}

/// Project metadata for listing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub created_at: String,
    pub spec_count: usize,
    pub last_modified: String,
}
