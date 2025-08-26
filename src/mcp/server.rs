//! # MCP Server Implementation
//!
//! This module provides the MCP server startup and configuration functionality.
//! The server uses stdio transport for communication with MCP clients like Claude.

use anyhow::Result;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::server_runtime::create_server,
    schema::{
        Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
        ServerCapabilitiesTools,
    },
};
use rust_mcp_transport::{StdioTransport, TransportOptions};

use crate::mcp::{error::FoundryMcpError, handlers::FoundryServerHandler};

/// MCP Server configuration and startup
pub struct FoundryMcpServer;

impl FoundryMcpServer {
    /// Start the MCP server with stdio transport
    ///
    /// This is the main entry point for MCP server mode, providing all 8 foundry
    /// commands as MCP tools with identical functionality to the CLI.
    pub async fn start() -> Result<(), FoundryMcpError> {
        tracing::info!("Starting Foundry MCP server");

        // Create server details following PRD specifications
        let server_details = InitializeResult {
            server_info: Implementation {
                name: "Foundry MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("Foundry Project Management MCP Server".to_string()),
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                ..Default::default()
            },
            meta: None,
            instructions: Some(
                "Foundry MCP Server provides project management tools for LLM-driven development. \
                Use create_project to start new projects, load_project to resume work, \
                create_spec for feature planning, and get_foundry_help for guidance."
                    .to_string(),
            ),
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        };

        // Create the server handler
        let handler = FoundryServerHandler::new();

        // Create stdio transport
        let transport_options = TransportOptions::default();
        let transport = StdioTransport::new(transport_options).map_err(|e| {
            FoundryMcpError::transport_error(format!("Failed to create stdio transport: {}", e))
        })?;

        // Create and start the server
        tracing::info!("Foundry MCP server started, listening on stdio");
        let server = create_server(server_details, transport, handler);
        server.start().await.map_err(|e| {
            FoundryMcpError::internal_error(format!("MCP server runtime error: {}", e))
        })?;

        Ok(())
    }
}
