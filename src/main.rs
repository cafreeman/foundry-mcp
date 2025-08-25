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

The core workflow is: create → list → load → create spec → validate → get help → work

Examples:
  foundry create-project my-app --vision '...' --tech-stack '...' --summary '...'
  foundry load-project my-app                    # Load complete project context
  foundry create-spec my-app user-auth --spec '...' --notes '...' --tasks '...'
  foundry list-projects                          # Discover available projects
  foundry get-foundry-help workflows             # Get detailed workflow guidance

Common Usage Patterns:
  • New Project: create-project → create-spec → work
  • Existing Code: analyze-project → create-spec → work
  • Resume Work: list-projects → load-project → load-spec → work
  • Content Check: validate-content before creating projects/specs

For more help on any command, use: foundry <COMMAND> --help"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project with vision, tech stack, and summary
    ///
    /// Creates ~/.foundry/PROJECT_NAME/ with project/vision.md, tech-stack.md, and summary.md
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
