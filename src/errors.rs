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

    /// Get a user-friendly error message with context
    pub fn user_message(&self) -> String {
        match self {
            Self::FileSystem {
                operation,
                path,
                source,
            } => {
                let base_msg = format!("Failed to {} at path '{}'", operation, path.display());
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    format!("{}\nReason: {}", base_msg, source)
                }
            }
            Self::Serialization { operation, .. } => {
                let base_msg = format!("Failed to {} data", operation);
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::Validation {
                field,
                value,
                reason,
            } => {
                let base_msg = format!(
                    "Invalid value '{}' for field '{}': {}",
                    value, field, reason
                );
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::NotFound {
                resource_type,
                identifier,
                context,
            } => {
                let base_msg = if let Some(ctx) = context {
                    format!("{} '{}' not found in {}", resource_type, identifier, ctx)
                } else {
                    format!("{} '{}' not found", resource_type, identifier)
                };
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::AlreadyExists {
                resource_type,
                identifier,
                context,
            } => {
                let base_msg = if let Some(ctx) = context {
                    format!(
                        "{} '{}' already exists in {}",
                        resource_type, identifier, ctx
                    )
                } else {
                    format!("{} '{}' already exists", resource_type, identifier)
                };
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::McpProtocol {
                operation, details, ..
            } => {
                let base_msg = format!("MCP protocol error during {}: {}", operation, details);
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::Configuration {
                setting,
                value,
                reason,
            } => {
                let base_msg = format!(
                    "Configuration error for '{}' (value: '{}'): {}",
                    setting, value, reason
                );
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::Permission {
                operation,
                path,
                details,
            } => {
                let base_msg = format!(
                    "Permission denied for {} at path '{}': {}",
                    operation,
                    path.display(),
                    details
                );
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
            Self::Internal {
                operation, details, ..
            } => {
                let base_msg = format!("Internal error during {}: {}", operation, details);
                if let Some(suggestion) = self.get_suggestion() {
                    format!("{}\n\nSuggestion: {}", base_msg, suggestion)
                } else {
                    base_msg
                }
            }
        }
    }

    /// Get actionable suggestions for resolving the error
    pub fn get_suggestion(&self) -> Option<String> {
        match self {
            Self::FileSystem { operation, path, source } => {
                match source.kind() {
                    std::io::ErrorKind::NotFound => {
                        if operation.contains("create") {
                            Some("Check that the parent directory exists and you have write permissions.".to_string())
                        } else if operation.contains("read") || operation.contains("open") {
                            Some(format!("Verify that the file '{}' exists and you have read permissions.", path.display()))
                        } else {
                            Some("Check that the file or directory exists and you have the necessary permissions.".to_string())
                        }
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        Some("Check file/directory permissions or run with appropriate privileges.".to_string())
                    }
                    std::io::ErrorKind::AlreadyExists => {
                        Some("Choose a different name or remove the existing file/directory first.".to_string())
                    }
                    std::io::ErrorKind::InvalidData => {
                        Some("The file may be corrupted or in an unexpected format. Try recreating it.".to_string())
                    }
                    _ => Some("Check file system permissions and available disk space.".to_string())
                }
            }
            Self::Serialization { operation, .. } => {
                if operation.contains("serialize") {
                    Some("Check that all required fields are properly set and contain valid data.".to_string())
                } else if operation.contains("parse") || operation.contains("deserialize") {
                    Some("The file may be corrupted or in an unexpected format. Try regenerating it or check for syntax errors.".to_string())
                } else {
                    Some("Verify that the data structure matches the expected format.".to_string())
                }
            }
            Self::Validation { field, .. } => {
                match field.as_str() {
                    "project_name" => Some("Project names should be lowercase with hyphens or underscores, e.g., 'my-project' or 'my_project'.".to_string()),
                    "spec_name" => Some("Specification names should be in snake_case format, e.g., 'user_authentication' or 'api_design'.".to_string()),
                    "task_id" => Some("Task IDs should be unique strings. Use the generated UUID or a meaningful identifier.".to_string()),
                    _ => Some("Check the documentation for the expected format and valid values for this field.".to_string())
                }
            }
            Self::NotFound { resource_type, identifier, .. } => {
                match resource_type.as_str() {
                    "Project" => Some(format!("Create the project '{}' first using the setup_project tool.", identifier)),
                    "Specification" => Some(format!("Create the specification '{}' first using the create_spec tool, or check if the ID is correct.", identifier)),
                    "Task" => Some("Check that the task ID is correct or create the task first.".to_string()),
                    _ => Some("Verify that the resource exists and the identifier is correct.".to_string())
                }
            }
            Self::AlreadyExists { resource_type, identifier, .. } => {
                match resource_type.as_str() {
                    "Project" => Some(format!("Choose a different project name or use the existing project '{}'.", identifier)),
                    "Specification" => Some(format!("Choose a different specification name or load the existing specification '{}'.", identifier)),
                    "Task" => Some("Use a different task ID or update the existing task instead.".to_string()),
                    _ => Some("Choose a different identifier or work with the existing resource.".to_string())
                }
            }
            Self::McpProtocol { operation, .. } => {
                match operation.as_str() {
                    "tool_call" => Some("Check that all required parameters are provided and have correct types.".to_string()),
                    "list_tools" => Some("Ensure the MCP server is properly initialized and running.".to_string()),
                    _ => Some("Check the MCP client configuration and network connectivity.".to_string())
                }
            }
            Self::Configuration { setting, .. } => {
                match setting.as_str() {
                    "home_directory" => Some("Set the HOME environment variable or run from a user directory.".to_string()),
                    "base_directory" => Some("Ensure the base directory path is valid and accessible.".to_string()),
                    _ => Some("Check the configuration documentation and verify all required settings.".to_string())
                }
            }
            Self::Permission { operation, path, .. } => {
                if operation.contains("write") || operation.contains("create") {
                    Some(format!("Ensure you have write permissions for '{}' or change the target directory.", path.display()))
                } else if operation.contains("read") {
                    Some(format!("Ensure you have read permissions for '{}'.", path.display()))
                } else {
                    Some("Check that you have the necessary permissions for this operation.".to_string())
                }
            }
            Self::Internal { .. } => {
                Some("This is an unexpected error. Please report this issue with the error details.".to_string())
            }
        }
    }

    /// Get detailed troubleshooting steps
    pub fn get_troubleshooting_steps(&self) -> Vec<String> {
        let mut steps = Vec::new();

        match self {
            Self::FileSystem {
                operation,
                path,
                source,
            } => {
                steps.push(format!("1. Check if path '{}' exists", path.display()));
                if operation.contains("write") || operation.contains("create") {
                    steps.push("2. Verify you have write permissions to the directory".to_string());
                    steps.push("3. Check available disk space".to_string());
                    steps.push("4. Ensure parent directories exist".to_string());
                } else if operation.contains("read") {
                    steps.push("2. Verify you have read permissions to the file".to_string());
                    steps.push(
                        "3. Check that the file is not locked by another process".to_string(),
                    );
                }
                steps.push(format!("5. System error: {}", source));
            }
            Self::NotFound {
                resource_type,
                identifier,
                context,
            } => match resource_type.as_str() {
                "Project" => {
                    steps.push("1. List available projects to verify the name".to_string());
                    steps.push(format!(
                        "2. Create project '{}' using setup_project tool",
                        identifier
                    ));
                    steps.push("3. Check for typos in the project name".to_string());
                }
                "Specification" => {
                    steps.push("1. List available specifications for the project".to_string());
                    steps.push(format!(
                        "2. Create specification '{}' using create_spec tool",
                        identifier
                    ));
                    steps.push("3. Verify the specification ID format (YYYYMMDD_name)".to_string());
                }
                _ => {
                    steps.push("1. Verify the resource identifier is correct".to_string());
                    steps.push(
                        "2. Check if the resource exists in the expected location".to_string(),
                    );
                    if let Some(ctx) = context {
                        steps.push(format!("3. Context: {}", ctx));
                    }
                }
            },
            Self::Validation {
                field,
                value,
                reason,
            } => {
                steps.push(format!("1. Current value '{}' is invalid", value));
                steps.push(format!("2. Reason: {}", reason));
                if let Some(suggestion) = self.get_suggestion() {
                    steps.push(format!("3. {}", suggestion));
                }
                steps.push(format!("4. Check the documentation for field '{}'", field));
            }
            _ => {
                if let Some(suggestion) = self.get_suggestion() {
                    steps.push(format!("1. {}", suggestion));
                }
                steps.push("2. Check the logs for more detailed error information".to_string());
                steps.push("3. Verify your environment and configuration".to_string());
            }
        }

        steps
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_system_error_creation() {
        let path = PathBuf::from("/test/path");
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = ProjectManagerError::file_system_error("read file", &path, io_error);

        match error {
            ProjectManagerError::FileSystem {
                operation,
                path: error_path,
                ..
            } => {
                assert_eq!(operation, "read file");
                assert_eq!(error_path, path);
            }
            _ => panic!("Expected FileSystem error"),
        }
    }

    #[test]
    fn test_serialization_error_creation() {
        // Create a real JSON error by trying to parse invalid JSON
        let json_result: std::result::Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("invalid json");
        let json_error = json_result.unwrap_err();
        let error =
            ProjectManagerError::serialization_error("parse JSON", "invalid json", json_error);

        match error {
            ProjectManagerError::Serialization {
                operation, content, ..
            } => {
                assert_eq!(operation, "parse JSON");
                assert_eq!(content, "invalid json");
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ProjectManagerError::validation_error(
            "project_name",
            "invalid name",
            "contains spaces",
        );

        match error {
            ProjectManagerError::Validation {
                field,
                value,
                reason,
            } => {
                assert_eq!(field, "project_name");
                assert_eq!(value, "invalid name");
                assert_eq!(reason, "contains spaces");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_not_found_error_creation() {
        let error = ProjectManagerError::not_found("Project", "test-project", Some("workspace"));

        match error {
            ProjectManagerError::NotFound {
                resource_type,
                identifier,
                context,
            } => {
                assert_eq!(resource_type, "Project");
                assert_eq!(identifier, "test-project");
                assert_eq!(context, Some("workspace".to_string()));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_already_exists_error_creation() {
        let error =
            ProjectManagerError::already_exists("Project", "duplicate-project", Some("workspace"));

        match error {
            ProjectManagerError::AlreadyExists {
                resource_type,
                identifier,
                context,
            } => {
                assert_eq!(resource_type, "Project");
                assert_eq!(identifier, "duplicate-project");
                assert_eq!(context, Some("workspace".to_string()));
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[test]
    fn test_user_message_formatting() {
        let path = PathBuf::from("/test/file.txt");
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = ProjectManagerError::file_system_error("read file", &path, io_error);

        let message = error.user_message();
        assert!(message.contains("Failed to read file"));
        assert!(message.contains("/test/file.txt"));
    }

    #[test]
    fn test_user_message_validation_error() {
        let error = ProjectManagerError::validation_error(
            "project_name",
            "bad name",
            "contains invalid characters",
        );
        let message = error.user_message();

        assert!(message.contains("Invalid value 'bad name'"));
        assert!(message.contains("for field 'project_name'"));
        assert!(message.contains("contains invalid characters"));
    }

    #[test]
    fn test_error_category() {
        let fs_error = ProjectManagerError::file_system_error(
            "test",
            &PathBuf::new(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
        );
        assert_eq!(fs_error.category(), "filesystem");

        let validation_error = ProjectManagerError::validation_error("field", "value", "reason");
        assert_eq!(validation_error.category(), "validation");
    }

    #[test]
    fn test_is_user_facing() {
        let validation_error = ProjectManagerError::validation_error("field", "value", "reason");
        assert!(validation_error.is_user_facing());

        let internal_error = ProjectManagerError::internal_error("op", "details", None);
        assert!(!internal_error.is_user_facing());
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_result: std::result::Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("invalid json");
        let json_error = json_result.unwrap_err();
        let pm_error: ProjectManagerError = json_error.into();

        match pm_error {
            ProjectManagerError::Serialization {
                operation, content, ..
            } => {
                assert_eq!(operation, "parse JSON");
                assert_eq!(content, "unknown");
            }
            _ => panic!("Expected Serialization error from serde_json::Error conversion"),
        }
    }

    #[test]
    fn test_helpers_invalid_project_name() {
        let error = helpers::invalid_project_name("bad project name");

        match error {
            ProjectManagerError::Validation {
                field,
                value,
                reason,
            } => {
                assert_eq!(field, "project_name");
                assert_eq!(value, "bad project name");
                assert!(reason.contains("special characters"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_helpers_serialization_error() {
        let json_result: std::result::Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("invalid json");
        let json_error = json_result.unwrap_err();
        let error = helpers::serialization_error("deserialize", "bad json", json_error);

        match error {
            ProjectManagerError::Serialization {
                operation, content, ..
            } => {
                assert_eq!(operation, "deserialize");
                assert_eq!(content, "bad json");
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_helpers_project_already_exists() {
        let error = helpers::project_already_exists("duplicate-project");

        match error {
            ProjectManagerError::AlreadyExists {
                resource_type,
                identifier,
                context,
            } => {
                assert_eq!(resource_type, "Project");
                assert_eq!(identifier, "duplicate-project");
                assert_eq!(context, None);
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }
}
