use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::process;

use foundry_mcp::cli;

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
    let args = Args::parse();

    let result: Result<Value> = match args.command {
        Commands::CreateProject(args) => cli::commands::create_project::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::AnalyzeProject(args) => cli::commands::analyze_project::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::CreateSpec(args) => cli::commands::create_spec::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::LoadProject(args) => cli::commands::load_project::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::LoadSpec(args) => cli::commands::load_spec::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::ListProjects(args) => cli::commands::list_projects::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::GetFoundryHelp(args) => cli::commands::get_foundry_help::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        Commands::ValidateContent(args) => cli::commands::validate_content::execute(args)
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
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
