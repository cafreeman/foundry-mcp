//! # Foundry MCP Error Types
//!
//! This module defines comprehensive error types for the Foundry MCP server,
//! providing proper error categorization and conversion from various error sources.

use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde_json;

/// Comprehensive error type for Foundry MCP server operations
#[derive(Debug, thiserror::Error)]
pub enum FoundryMcpError {
    /// Parameter validation errors (invalid input parameters)
    #[error("Parameter validation failed: {message}")]
    InvalidParams { message: String },

    /// CLI command execution errors
    #[error("CLI command execution failed: {source}")]
    CliCommand {
        #[from]
        source: anyhow::Error,
    },

    /// JSON serialization/deserialization errors
    #[error("JSON serialization failed: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// File system operation errors
    #[error("File system operation failed: {source}")]
    Filesystem {
        #[from]
        source: std::io::Error,
    },

    /// Transport layer errors
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Internal server errors
    #[error("Internal server error: {message}")]
    Internal { message: String },
}

/// Custom error types that implement std::error::Error for MCP protocol
#[derive(Debug)]
pub struct InvalidParamsError(pub String);

impl std::fmt::Display for InvalidParamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parameter validation failed: {}", self.0)
    }
}

impl std::error::Error for InvalidParamsError {}

#[derive(Debug)]
pub struct InternalMcpError(pub String);

impl std::fmt::Display for InternalMcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Internal server error: {}", self.0)
    }
}

impl std::error::Error for InternalMcpError {}

/// Convert FoundryMcpError to CallToolError for MCP protocol compliance
impl From<FoundryMcpError> for CallToolError {
    fn from(err: FoundryMcpError) -> Self {
        match err {
            FoundryMcpError::InvalidParams { message } => {
                CallToolError::new(InvalidParamsError(message))
            }
            FoundryMcpError::CliCommand { source } => {
                CallToolError::new(InternalMcpError(format!("CLI command failed: {}", source)))
            }
            FoundryMcpError::Serialization { source } => CallToolError::new(InternalMcpError(
                format!("JSON serialization failed: {}", source),
            )),
            FoundryMcpError::Filesystem { source } => {
                CallToolError::new(InternalMcpError(format!("File system error: {}", source)))
            }
            FoundryMcpError::Transport { message } => {
                CallToolError::new(InternalMcpError(format!("Transport error: {}", message)))
            }
            FoundryMcpError::Internal { message } => CallToolError::new(InternalMcpError(message)),
        }
    }
}

/// Utility function to create parameter validation errors
impl FoundryMcpError {
    pub fn invalid_params<S: Into<String>>(message: S) -> Self {
        FoundryMcpError::InvalidParams {
            message: message.into(),
        }
    }

    pub fn transport_error<S: Into<String>>(message: S) -> Self {
        FoundryMcpError::Transport {
            message: message.into(),
        }
    }

    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        FoundryMcpError::Internal {
            message: message.into(),
        }
    }
}
