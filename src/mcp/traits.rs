//! # MCP Tool Definition Traits
//!
//! This module defines traits that enable automatic MCP tool generation
//! from CLI argument structs using procedural macros.

use anyhow::Result;
use rust_mcp_sdk::schema::Tool as McpTool;
use serde_json::Value;

/// Trait for CLI argument structs that can be automatically converted to MCP tools
pub trait McpToolDefinition {
    /// Generate the MCP tool definition from the struct
    fn tool_definition() -> McpTool;

    /// Convert MCP parameters to the CLI argument struct
    fn from_mcp_params(params: &Value) -> Result<Self>
    where
        Self: Sized;
}
