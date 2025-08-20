//! Project Manager MCP CLI
//!
//! A CLI tool for project management with MCP server capabilities.
//! Provides project management commands and can run as an MCP server.

use clap::Parser;
use project_manager_mcp::cli::{Cli, Commands};
use project_manager_mcp::cli::commands::{serve, install, project};
use project_manager_mcp::cli::args::ServeArgs;
use project_manager_mcp::cli::config::CliConfig;
use project_manager_mcp::errors::Result;
use std::env;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration with precedence: CLI > env vars > config file > defaults
    let config = load_configuration(&cli)?;

    // Set up logging based on global options and configuration
    setup_logging(&cli, &config);

    // Handle the command or default to serve mode
    let result = match cli.command {
        Some(Commands::Serve(ref args)) => {
            // Log serve command configuration for visibility
            info!("Starting serve command with configuration:");
            info!("  - Log format preference: {}", args.log_format);
            serve::run_server(args.clone()).await
        }
        
        Some(Commands::Install(args)) => install::run_install(args).await,
        
        Some(Commands::CreateProject(args)) => project::create_project(args).await,
        
        Some(Commands::ListProjects) => project::list_projects().await,
        
        Some(Commands::ClearProjects { force, project, backup }) => {
            project::clear_projects(force, project, backup).await
        }
        
        Some(Commands::ShowProject { name, format }) => {
            project::show_project(name, format).await
        }
        
        Some(Commands::ListSpecs { project, detailed, json }) => {
            project::list_specs(project, detailed, json).await
        }
        
        Some(Commands::ShowSpec { project, spec_id, tasks_only, notes_only, format }) => {
            project::show_spec(project, spec_id, tasks_only, notes_only, format).await
        }
        
        Some(Commands::Export { project, spec, format, output_dir }) => {
            project::export_data(project, spec, format, output_dir).await
        }
        
        Some(Commands::Import { file, merge }) => {
            project::import_data(file, merge).await
        }
        
        Some(Commands::Config { action }) => {
            project::handle_config(action).await
        }
        
        Some(Commands::Status { verbose }) => {
            project::show_status(verbose).await
        }
        
        Some(Commands::Doctor { fix }) => {
            project::run_doctor(fix).await
        }
        
        Some(Commands::Completions { shell }) => {
            project::generate_completions(shell).await
        }
        
        // Default behavior: run serve mode when no subcommand is provided
        None => {
            info!("No subcommand provided, starting MCP server mode");
            serve::run_server(ServeArgs::default()).await
        }
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        if cli.verbose {
            error!("Debug info: {:?}", e);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn load_configuration(cli: &Cli) -> Result<CliConfig> {
    // Start with default configuration
    let mut config = CliConfig::default();

    // Try to load from config file (if it exists)
    if let Ok(file_config) = CliConfig::load() {
        config = file_config;
    }

    // Override with environment variables
    if let Ok(env_log_level) = env::var("LOG_LEVEL") {
        if let Err(e) = config.set("log_level", &env_log_level) {
            error!("Invalid LOG_LEVEL environment variable: {}", e);
        }
    }

    // Override with CLI arguments (highest priority)
    if let Some(ref log_level) = cli.log_level {
        if let Err(e) = config.set("log_level", log_level) {
            error!("Invalid log level from CLI: {}", e);
        }
    }

    // Handle config directory override
    if cli.config_dir.is_some() {
        info!("Custom config directory specified: {:?}", cli.config_dir);
        // Note: This would require modifying CliConfig to accept custom paths
        // For now, we'll just log it
    }

    Ok(config)
}

fn setup_logging(cli: &Cli, config: &CliConfig) {
    // Determine log level from configuration (with CLI overrides taking precedence)
    let log_level = if cli.verbose {
        Level::DEBUG
    } else if cli.quiet {
        Level::WARN
    } else {
        match config.log_level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    };

    // Set up logging subscriber
    FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(!cli.quiet)
        .with_thread_ids(cli.verbose)
        .with_thread_names(cli.verbose)
        .with_file(cli.verbose)
        .with_line_number(cli.verbose)
        .init();
}
