use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::env;
use std::process;

use foundry_mcp::{cli, mcp};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "A CLI tool for deterministic project management and AI coding assistant integration"
)]
#[command(long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project with vision, tech stack, and summary
    CreateProject(cli::args::CreateProjectArgs),
    /// Analyze an existing project and write LLM-provided content
    AnalyzeProject(cli::args::AnalyzeProjectArgs),
    /// Create a new specification for a project
    CreateSpec(cli::args::CreateSpecArgs),
    /// Load a project's complete context (vision, tech-stack, summary)
    LoadProject(cli::args::LoadProjectArgs),
    /// Load an existing specification
    LoadSpec(cli::args::LoadSpecArgs),
    /// List all available projects
    ListProjects(cli::args::ListProjectsArgs),
    /// Get Foundry help and workflow guidance
    GetFoundryHelp(cli::args::GetFoundryHelpArgs),
    /// Validate content against schema requirements
    ValidateContent(cli::args::ValidateContentArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check if we should run in MCP server mode vs CLI mode
    // MCP server mode: no CLI arguments provided (or explicit --mcp flag)
    // CLI mode: CLI subcommands provided
    let cli_args: Vec<String> = env::args().collect();

    // If only the binary name is provided (no arguments), start MCP server
    if cli_args.len() == 1 {
        tracing::info!("No CLI arguments provided, starting in MCP server mode");
        return mcp::FoundryMcpServer::start().await;
    }

    // Check for explicit MCP server flag
    if cli_args.len() == 2 && (cli_args[1] == "--mcp" || cli_args[1] == "mcp") {
        tracing::info!("Explicit MCP mode requested, starting MCP server");
        return mcp::FoundryMcpServer::start().await;
    }

    // Otherwise, parse CLI arguments and run in CLI mode
    tracing::debug!("CLI arguments provided, running in CLI mode");
    let args = Args::parse();

    let result: Result<Value> = match args.command {
        Commands::CreateProject(args) => cli::commands::create_project::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::AnalyzeProject(args) => cli::commands::analyze_project::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::CreateSpec(args) => cli::commands::create_spec::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::LoadProject(args) => cli::commands::load_project::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::LoadSpec(args) => cli::commands::load_spec::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::ListProjects(args) => cli::commands::list_projects::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::GetFoundryHelp(args) => cli::commands::get_foundry_help::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        Commands::ValidateContent(args) => cli::commands::validate_content::execute(args)
            .await
            .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
    };

    match result {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output)?);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
