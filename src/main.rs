#![deny(clippy::all)]

use project_manager_mcp::ProjectManagerHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Project Manager MCP server");

    // Create the server handler
    let _handler = ProjectManagerHandler::new();

    tracing::info!("Project Manager MCP server initialized");

    // TODO: Implement MCP server functionality
    // For now, just keep the process running
    tokio::signal::ctrl_c().await?;

    tracing::info!("Shutting down Project Manager MCP server");
    Ok(())
}
