//! # MCP Server Module
//!
//! This module provides the MCP (Model Context Protocol) server implementation
//! that exposes the same functionality as the CLI commands through MCP tools.
//!
//! The MCP server provides identical functionality to the CLI, following the PRD
//! requirement that "MCP tools map directly to CLI commands" with identical
//! JSON response formats.

pub mod handlers;
pub mod server;
pub mod tools;

pub use handlers::*;
pub use server::*;
pub use tools::*;
