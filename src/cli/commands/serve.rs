use crate::cli::args::ServeArgs;
use crate::errors::ProjectManagerError;
use crate::handlers::ProjectManagerHandler;
use anyhow::Result;
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
    ServerCapabilitiesTools,
};
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    mcp_server::{ServerRuntime, server_runtime},
};
use tracing::{error, info};

pub async fn run_server(args: ServeArgs) -> Result<()> {
    // Note: Logging is already set up in main.rs, so we don't initialize it here
    info!("Starting Project Manager MCP Server...");
    info!("Configuration:");
    info!("  - Transport: {}", args.transport);
    info!("  - Host: {}", args.host);
    info!("  - Port: {}", args.port);
    info!("  - Max Connections: {}", args.max_connections);
    info!("  - Timeout: {}s", args.timeout);
    info!("  - Backup Retention: {} days", args.backup_retention_days);    info!("  - Log Format: {}", args.log_format);

    // Validate transport mode
    if args.transport != "stdio" {
        return Err(anyhow::anyhow!(
            "Only 'stdio' transport is currently supported. HTTP transport coming in future versions."
        ));
    }

    // STEP 1: Define server details and capabilities
    let server_details = InitializeResult {
        // server name and version
        server_info: Implementation {
            name: "Project Manager MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
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

    // Set up graceful shutdown handling
    let server_handle = tokio::spawn(async move {
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
        }
    });

    // Wait for SIGINT or SIGTERM
    tokio::select! {
        _ = server_handle => {
            info!("Server task completed");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT, shutting down gracefully...");
        }
    }

    info!("Shutting down Project Manager MCP Server");
    Ok(())
}

// Note: Logging setup functions removed since logging is handled in main.rs
