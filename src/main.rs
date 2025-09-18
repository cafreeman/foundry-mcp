use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

use foundry_mcp::{cli, mcp};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "A CLI tool for deterministic project management and AI coding assistant integration"
)]
#[command(
    long_about = "Foundry helps LLMs maintain context about software projects through structured specifications.

Examples:
  foundry serve                                   # Start MCP server
  foundry install claude-code                     # Install MCP server for Claude Code
  foundry install cursor                          # Install MCP server for Cursor IDE
  foundry status                                  # Check installation status
  foundry uninstall claude-code                   # Remove MCP server from Claude Code

For project/spec operations, use Foundry MCP tools from your IDE/agent."
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server
    ///
    /// Runs Foundry as an MCP (Model Context Protocol) server for integration
    /// with AI development environments like Claude Desktop or VS Code
    Serve(cli::args::ServeArgs),

    /// Install Foundry MCP server for AI development environments
    ///
    /// Supports installation for claude-code and cursor environments
    /// Creates necessary configuration files and registers the MCP server
    Install(cli::args::InstallArgs),

    /// Uninstall Foundry MCP server from AI development environments
    ///
    /// Removes MCP server configuration from claude-code and cursor environments
    /// Optionally cleans up configuration files
    Uninstall(cli::args::UninstallArgs),

    /// Show MCP server installation status across all supported tools
    ///
    /// Displays installation status, binary paths, and configuration details
    /// for all supported AI development environments
    Status(cli::args::StatusArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments and run in CLI mode
    let args = Args::parse();

    match args.command {
        Commands::Serve(args) => {
            if args.verbose {
                tracing::info!("Starting MCP server in verbose mode");
            } else {
                tracing::info!("Starting MCP server");
            }
            return mcp::FoundryMcpServer::start().await.map_err(|e| {
                eprintln!("MCP server error: {}", e);
                std::process::exit(1);
            });
        }
        Commands::Install(args) => {
            let output = cli::commands::install::execute(args).await?;
            println!("{}", output);
            return Ok(());
        }
        Commands::Uninstall(args) => {
            let output = cli::commands::uninstall::execute(args).await?;
            println!("{}", output);
            return Ok(());
        }
        Commands::Status(args) => {
            let output = cli::commands::status::execute(args).await?;
            println!("{}", output);
            return Ok(());
        }
    }
}
