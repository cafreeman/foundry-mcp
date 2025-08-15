//! Project Manager MCP Server
//!
//! A Model Context Protocol server for managing software projects with specifications,
//! tasks, and notes.

use project_manager_mcp::errors::{ProjectManagerError, Result};
use project_manager_mcp::handlers::ProjectManagerHandler;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
    ServerCapabilitiesTools,
};
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    mcp_server::{ServerRuntime, server_runtime},
};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting Project Manager MCP Server...");

    // STEP 1: Define server details and capabilities
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "Project Manager MCP Server".to_string(),
            version: "0.1.0".to_string(),
            title: Some("Project Manager MCP Server".to_string()),
        },
        capabilities: ServerCapabilities {
            // indicates that server support mcp tools
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            // indicates that server support mcp prompts
            prompts: Some(Default::default()),
            ..Default::default() // Using default values for other fields
        },
        meta: None,
        instructions: Some(
            "Project and specification management for AI coding assistants".to_string(),
        ),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // STEP 2: create a std transport with default options
    let transport = StdioTransport::new(TransportOptions::default()).map_err(|e| {
        ProjectManagerError::mcp_protocol_error("create transport", &e.to_string(), None)
    })?;

    // STEP 3: instantiate our custom handler for handling MCP messages
    let handler = ProjectManagerHandler::new()?;

    // STEP 4: create a MCP server
    let server: ServerRuntime = server_runtime::create_server(server_details, transport, handler);

    info!("Project Manager MCP Server initialized successfully");
    info!("Handler created with the following capabilities:");
    info!("  - setup_project: Create new projects with tech stack and vision");
    info!("  - create_spec: Create new specifications for projects");
    info!("  - load_spec: Load specifications with full context");
    info!("  - update_spec: Update specifications, tasks, and notes");
    info!("  - execute_task: Generate task execution prompts");

    // STEP 5: Start the server
    info!("Starting MCP server with stdio transport...");
    if let Err(start_error) = server.start().await {
        error!("Failed to start MCP server: {}", start_error);
        eprintln!(
            "{}",
            start_error
                .rpc_error_message()
                .unwrap_or(&start_error.to_string())
        );
    };

    info!("Shutting down Project Manager MCP Server");
    Ok(())
}
