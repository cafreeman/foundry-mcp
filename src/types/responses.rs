//! JSON response structures for CLI commands

use serde::{Deserialize, Serialize};

/// Generic response wrapper for all CLI commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryResponse<T> {
    /// Command-specific data payload
    pub data: T,
    /// Suggested next actions for LLM consumption
    pub next_steps: Vec<String>,
    /// Validation status of the operation
    pub validation_status: ValidationStatus,
    /// Optional workflow guidance hints
    pub workflow_hints: Vec<String>,
}

/// Validation status for operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// Operation completed successfully with all validations passing
    Complete,
    /// Operation completed but with some validation warnings
    Incomplete,
    /// Operation failed due to validation errors
    Error,
}

/// Response for create_project command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectResponse {
    pub project_name: String,
    pub created_at: String,
    pub project_path: String,
    pub files_created: Vec<String>,
}

/// Response for list_projects command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub created_at: String,
    pub spec_count: usize,
    pub path: String,
}

/// Response for load_project command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProjectResponse {
    pub project: ProjectContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub vision: String,
    pub tech_stack: String,
    pub summary: String,
    pub specs_available: Vec<String>,
    pub created_at: String,
}

/// Response for create_spec command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpecResponse {
    pub project_name: String,
    pub spec_name: String,
    pub created_at: String,
    pub spec_path: String,
    pub files_created: Vec<String>,
}

/// Response for load_spec command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSpecResponse {
    pub project_name: String,
    pub project_summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_content: Option<SpecContent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub available_specs: Vec<SpecInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInfo {
    pub name: String,
    pub feature_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContent {
    pub spec: String,
    pub notes: String,
    pub task_list: String,
}

/// Response for analyze_project command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeProjectResponse {
    pub project_name: String,
    pub files_created: Vec<String>,
}

/// Response for get_foundry_help command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFoundryHelpResponse {
    pub topic: String,
    pub content: HelpContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpContent {
    pub title: String,
    pub description: String,
    pub examples: Vec<String>,
    pub workflow_guide: Vec<String>,
}

/// Response for validate_content command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateContentResponse {
    pub content_type: String,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Response for update_spec command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpecResponse {
    pub project_name: String,
    pub spec_name: String,
    pub file_type: String,
    pub operation: String,
    pub file_path: String,
    pub content_length: usize,
}

/// Response for delete_spec command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSpecResponse {
    pub project_name: String,
    pub spec_name: String,
    pub spec_path: String,
    pub files_deleted: Vec<String>,
}

/// Response for install command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResponse {
    pub target: String,
    pub binary_path: String,
    pub config_path: String,
    pub installation_status: InstallationStatus,
    pub actions_taken: Vec<String>,
}

/// Response for uninstall command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResponse {
    pub target: String,
    pub config_path: String,
    pub uninstallation_status: InstallationStatus,
    pub actions_taken: Vec<String>,
    pub files_removed: Vec<String>,
}

/// Response for status command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub binary_path: String,
    pub binary_found: bool,
    pub environments: Vec<EnvironmentStatus>,
}

/// Installation/uninstallation status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallationStatus {
    /// Installation/uninstallation completed successfully
    Success,
    /// Installation/uninstallation partially completed with warnings
    Partial,
    /// Installation/uninstallation failed
    Failed,
}

/// Status information for a specific environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    pub name: String,
    pub installed: bool,
    pub config_path: String,
    pub config_exists: bool,
    pub binary_path: String,
    pub binary_accessible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_content: Option<String>,
    pub issues: Vec<String>,
}
