//! Custom error types and error handling for the Project Manager MCP server

use std::fmt;
use std::path::{Path, PathBuf};

/// Custom error types for different failure categories
#[derive(Debug)]
pub enum ProjectManagerError {
    /// File system related errors
    FileSystem {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },

    /// JSON serialization/deserialization errors
    Serialization {
        operation: String,
        content: String,
        source: serde_json::Error,
    },

    /// Validation errors for user input
    Validation {
        field: String,
        value: String,
        reason: String,
    },

    /// Resource not found errors
    NotFound {
        resource_type: String,
        identifier: String,
        context: Option<String>,
    },

    /// Resource already exists errors
    AlreadyExists {
        resource_type: String,
        identifier: String,
        context: Option<String>,
    },

    /// MCP protocol errors
    McpProtocol {
        operation: String,
        details: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors
    Configuration {
        setting: String,
        value: String,
        reason: String,
    },

    /// Permission errors
    Permission {
        operation: String,
        path: PathBuf,
        details: String,
    },

    /// Internal errors (should not happen in normal operation)
    Internal {
        operation: String,
        details: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl ProjectManagerError {
    /// Create a file system error with context
    pub fn file_system_error(operation: &str, path: &Path, source: std::io::Error) -> Self {
        Self::FileSystem {
            operation: operation.to_string(),
            path: path.to_path_buf(),
            source,
        }
    }

    /// Create a serialization error with context
    pub fn serialization_error(operation: &str, content: &str, source: serde_json::Error) -> Self {
        Self::Serialization {
            operation: operation.to_string(),
            content: content.to_string(),
            source,
        }
    }

    /// Create a validation error
    pub fn validation_error(field: &str, value: &str, reason: &str) -> Self {
        Self::Validation {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource_type: &str, identifier: &str, context: Option<&str>) -> Self {
        Self::NotFound {
            resource_type: resource_type.to_string(),
            identifier: identifier.to_string(),
            context: context.map(|s| s.to_string()),
        }
    }

    /// Create an already exists error
    pub fn already_exists(resource_type: &str, identifier: &str, context: Option<&str>) -> Self {
        Self::AlreadyExists {
            resource_type: resource_type.to_string(),
            identifier: identifier.to_string(),
            context: context.map(|s| s.to_string()),
        }
    }

    /// Create an MCP protocol error
    pub fn mcp_protocol_error(
        operation: &str,
        details: &str,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::McpProtocol {
            operation: operation.to_string(),
            details: details.to_string(),
            source,
        }
    }

    /// Create a configuration error
    pub fn configuration_error(setting: &str, value: &str, reason: &str) -> Self {
        Self::Configuration {
            setting: setting.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a permission error
    pub fn permission_error(operation: &str, path: &Path, details: &str) -> Self {
        Self::Permission {
            operation: operation.to_string(),
            path: path.to_path_buf(),
            details: details.to_string(),
        }
    }

    /// Create an internal error
    pub fn internal_error(
        operation: &str,
        details: &str,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Internal {
            operation: operation.to_string(),
            details: details.to_string(),
            source,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::FileSystem {
                operation, path, ..
            } => {
                format!("Failed to {} at path '{}'", operation, path.display())
            }
            Self::Serialization { operation, .. } => {
                format!("Failed to {} data", operation)
            }
            Self::Validation {
                field,
                value,
                reason,
            } => {
                format!(
                    "Invalid value '{}' for field '{}': {}",
                    value, field, reason
                )
            }
            Self::NotFound {
                resource_type,
                identifier,
                context,
            } => {
                if let Some(ctx) = context {
                    format!("{} '{}' not found in {}", resource_type, identifier, ctx)
                } else {
                    format!("{} '{}' not found", resource_type, identifier)
                }
            }
            Self::AlreadyExists {
                resource_type,
                identifier,
                context,
            } => {
                if let Some(ctx) = context {
                    format!(
                        "{} '{}' already exists in {}",
                        resource_type, identifier, ctx
                    )
                } else {
                    format!("{} '{}' already exists", resource_type, identifier)
                }
            }
            Self::McpProtocol {
                operation, details, ..
            } => {
                format!("MCP protocol error during {}: {}", operation, details)
            }
            Self::Configuration {
                setting,
                value,
                reason,
            } => {
                format!(
                    "Configuration error for '{}' (value: '{}'): {}",
                    setting, value, reason
                )
            }
            Self::Permission {
                operation, path, ..
            } => {
                format!(
                    "Permission denied for {} at path '{}'",
                    operation,
                    path.display()
                )
            }
            Self::Internal {
                operation, details, ..
            } => {
                format!("Internal error during {}: {}", operation, details)
            }
        }
    }

    /// Get a detailed error message for developers
    pub fn debug_message(&self) -> String {
        match self {
            Self::FileSystem {
                operation,
                path,
                source,
            } => {
                format!("FileSystem error: {} at {:?} - {}", operation, path, source)
            }
            Self::Serialization {
                operation,
                content,
                source,
            } => {
                format!(
                    "Serialization error: {} - content: {} - {}",
                    operation, content, source
                )
            }
            Self::Validation {
                field,
                value,
                reason,
            } => {
                format!(
                    "Validation error: field '{}' with value '{}' - {}",
                    field, value, reason
                )
            }
            Self::NotFound {
                resource_type,
                identifier,
                context,
            } => {
                format!(
                    "NotFound error: {} '{}' in context '{:?}'",
                    resource_type, identifier, context
                )
            }
            Self::AlreadyExists {
                resource_type,
                identifier,
                context,
            } => {
                format!(
                    "AlreadyExists error: {} '{}' in context '{:?}'",
                    resource_type, identifier, context
                )
            }
            Self::McpProtocol {
                operation,
                details,
                source,
            } => {
                format!(
                    "MCP Protocol error: {} - {} - source: {:?}",
                    operation, details, source
                )
            }
            Self::Configuration {
                setting,
                value,
                reason,
            } => {
                format!(
                    "Configuration error: setting '{}' with value '{}' - {}",
                    setting, value, reason
                )
            }
            Self::Permission {
                operation,
                path,
                details,
            } => {
                format!(
                    "Permission error: {} at {:?} - {}",
                    operation, path, details
                )
            }
            Self::Internal {
                operation,
                details,
                source,
            } => {
                format!(
                    "Internal error: {} - {} - source: {:?}",
                    operation, details, source
                )
            }
        }
    }

    /// Get the error category for logging and monitoring
    pub fn category(&self) -> &'static str {
        match self {
            Self::FileSystem { .. } => "filesystem",
            Self::Serialization { .. } => "serialization",
            Self::Validation { .. } => "validation",
            Self::NotFound { .. } => "not_found",
            Self::AlreadyExists { .. } => "already_exists",
            Self::McpProtocol { .. } => "mcp_protocol",
            Self::Configuration { .. } => "configuration",
            Self::Permission { .. } => "permission",
            Self::Internal { .. } => "internal",
        }
    }

    /// Check if this is a user-facing error (vs internal error)
    pub fn is_user_facing(&self) -> bool {
        !matches!(self, Self::Internal { .. })
    }
}

impl fmt::Display for ProjectManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ProjectManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FileSystem { source, .. } => Some(source),
            Self::Serialization { source, .. } => Some(source),
            Self::McpProtocol { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
            Self::Internal { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
            _ => None,
        }
    }
}

// Error conversion traits for common error types
impl From<std::io::Error> for ProjectManagerError {
    fn from(err: std::io::Error) -> Self {
        Self::FileSystem {
            operation: "perform I/O operation".to_string(),
            path: PathBuf::new(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for ProjectManagerError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            operation: "parse JSON".to_string(),
            content: "unknown".to_string(),
            source: err,
        }
    }
}

// Result type alias for the project manager
pub type Result<T> = std::result::Result<T, ProjectManagerError>;

// Helper functions for common error patterns
pub mod helpers {
    use super::*;

    /// Create a validation error for invalid project names
    pub fn invalid_project_name(name: &str) -> ProjectManagerError {
        ProjectManagerError::validation_error(
            "project_name",
            name,
            "Project names cannot contain special characters or spaces",
        )
    }

    /// Create a validation error for invalid spec names
    pub fn invalid_spec_name(name: &str) -> ProjectManagerError {
        ProjectManagerError::validation_error(
            "spec_name",
            name,
            "Spec names must be in snake_case format (lowercase with underscores)",
        )
    }

    /// Create a validation error
    pub fn validation_error(field: &str, value: &str, reason: &str) -> ProjectManagerError {
        ProjectManagerError::validation_error(field, value, reason)
    }

    /// Create a not found error for projects
    pub fn project_not_found(name: &str) -> ProjectManagerError {
        ProjectManagerError::not_found("Project", name, None)
    }

    /// Create a not found error for specifications
    pub fn spec_not_found(spec_id: &str, project_name: &str) -> ProjectManagerError {
        ProjectManagerError::not_found("Specification", spec_id, Some(project_name))
    }

    /// Create an already exists error for projects
    pub fn project_already_exists(name: &str) -> ProjectManagerError {
        ProjectManagerError::already_exists("Project", name, None)
    }

    /// Create an already exists error for specifications
    pub fn spec_already_exists(spec_id: &str, project_name: &str) -> ProjectManagerError {
        ProjectManagerError::already_exists("Specification", spec_id, Some(project_name))
    }

    /// Create a file system error with operation context
    pub fn file_system_error(
        operation: &str,
        path: &Path,
        source: std::io::Error,
    ) -> ProjectManagerError {
        ProjectManagerError::file_system_error(operation, path, source)
    }

    /// Create a serialization error with operation context
    pub fn serialization_error(
        operation: &str,
        content: &str,
        source: serde_json::Error,
    ) -> ProjectManagerError {
        ProjectManagerError::serialization_error(operation, content, source)
    }
}
