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
#[command(
    long_about = "Foundry helps LLMs maintain context about software projects through structured specifications.

Examples:
  foundry serve                                   # Start MCP server
  foundry mcp create-project my-app --vision '...' --tech-stack '...' --summary '...'
  foundry mcp load-project my-app                 # Load complete project context
  foundry mcp create-spec my-app user-auth --spec '...' --notes '...' --tasks '...'
  foundry mcp list-projects                       # Discover available projects
  foundry mcp get-foundry-help workflows          # Get detailed workflow guidance

The core LLM workflow is: create → list → load → create spec → validate → get help → work

For more help: foundry <COMMAND> --help or foundry mcp <TOOL> --help"
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

    /// Test MCP tools via command line interface
    ///
    /// Execute the same MCP tools that AI assistants use, directly from CLI.
    /// Useful for testing, debugging, and understanding MCP tool behavior.
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Create a new project with vision, tech stack, and summary
    ///
    /// Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md
    /// All content must be provided by LLMs - Foundry manages structure only
    CreateProject(cli::args::CreateProjectArgs),

    /// Analyze an existing codebase and create project structure
    ///
    /// Use this when you have existing code to analyze. LLMs provide vision, tech-stack,
    /// and summary based on codebase exploration
    AnalyzeProject(cli::args::AnalyzeProjectArgs),

    /// Create a timestamped specification for a feature
    ///
    /// Creates YYYYMMDD_HHMMSS_FEATURE_NAME/ with spec.md, notes.md, and task-list.md
    /// Task-list.md serves as implementation checklist for agents
    CreateSpec(cli::args::CreateSpecArgs),

    /// Load complete project context for LLM sessions
    ///
    /// Returns vision, tech-stack, summary, and available specs
    /// Essential for resuming work on existing projects
    LoadProject(cli::args::LoadProjectArgs),

    /// Load specification content with project context
    ///
    /// If spec_name is omitted, lists all available specs for the project
    /// Returns spec content, project summary, and workflow hints
    LoadSpec(cli::args::LoadSpecArgs),

    /// List all available projects with metadata
    ///
    /// Shows project names, creation dates, spec counts, and validation status
    /// Use this to discover existing projects
    ListProjects(cli::args::ListProjectsArgs),

    /// Get comprehensive workflow guidance and examples
    ///
    /// Topics: workflows, content-examples, project-structure, parameter-guidance
    /// Essential for understanding Foundry usage patterns
    GetFoundryHelp(cli::args::GetFoundryHelpArgs),

    /// Validate content against schema requirements
    ///
    /// Check content quality before creating projects/specs
    /// Provides improvement suggestions and next steps
    ValidateContent(cli::args::ValidateContentArgs),

    /// Update existing specification files with new content
    ///
    /// Supports both replace and append operations for iterative development
    /// Use to update spec.md, task-list.md, or notes.md during implementation
    UpdateSpec(cli::args::UpdateSpecArgs),

    /// Delete an existing specification permanently
    ///
    /// Removes spec directory and all associated files (spec.md, task-list.md, notes.md)
    /// Requires confirmation flag - this action cannot be undone
    DeleteSpec(cli::args::DeleteSpecArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments and run in CLI mode
    let args = Args::parse();

    let result: Result<Value> = match args.command {
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
        Commands::Mcp { command } => match command {
            McpCommands::CreateProject(args) => cli::commands::create_project::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::AnalyzeProject(args) => cli::commands::analyze_project::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::CreateSpec(args) => cli::commands::create_spec::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::LoadProject(args) => cli::commands::load_project::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::LoadSpec(args) => cli::commands::load_spec::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::ListProjects(args) => cli::commands::list_projects::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::GetFoundryHelp(args) => cli::commands::get_foundry_help::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::ValidateContent(args) => cli::commands::validate_content::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::UpdateSpec(args) => cli::commands::update_spec::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
            McpCommands::DeleteSpec(args) => cli::commands::delete_spec::execute(args)
                .await
                .and_then(|r| serde_json::to_value(r).map_err(anyhow::Error::from)),
        },
    };

    match result {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output)?);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);

            // Provide helpful error recovery suggestions
            let error_msg = e.to_string();
            if error_msg.contains("Project name must be in kebab-case") {
                eprintln!(
                    "\n💡 Try using kebab-case format: my-awesome-project (not 'my awesome project' or 'MyProject')"
                );
            } else if error_msg.contains("Content validation failed") {
                eprintln!(
                    "\n💡 Use 'foundry validate-content <type> --content <content>' to check content before creating"
                );
            } else if error_msg.contains("already exists") {
                eprintln!(
                    "\n💡 Use 'foundry list-projects' to see existing projects, or choose a different name"
                );
            } else if error_msg.contains("not found") || error_msg.contains("does not exist") {
                eprintln!("\n💡 Use 'foundry list-projects' to see available projects");
            }

            eprintln!("\n💡 For help: foundry --help");
            eprintln!("💡 For workflow guidance: foundry get-foundry-help workflows");
            process::exit(1);
        }
    }
}
