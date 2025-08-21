#!/usr/bin/env rust-script
//! A script to create the foundry-development project using foundry's own tools
//! 
//! ```cargo
//! [dependencies]
//! foundry-mcp = { path = "." }
//! tokio = { version = "1.0", features = ["full"] }
//! serde_json = "1.0"
//! anyhow = "1.0"
//! ```

use foundry_mcp::cache::ProjectManagerCache;
use foundry_mcp::filesystem::FileSystemManager;
use foundry_mcp::handlers::setup_project::SetupProjectHandler;
use foundry_mcp::repository::ProjectRepository;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Creating foundry-development project...");

    // Initialize components
    let fs_manager = FileSystemManager::new()?;
    let cache = ProjectManagerCache::new();
    let project_repo = ProjectRepository::with_cache(fs_manager, cache);
    let handler = SetupProjectHandler::new(project_repo);

    // Prepare project data
    let arguments = json!({
        "name": "foundry-development",
        "description": "Self-managed development of foundry CLI tool and MCP server capabilities",
        "overview": "Comprehensive CLI tool that serves as both a project management system and MCP server for AI-assisted development workflows. This project uses foundry to manage its own development lifecycle, specifications, and task tracking.",
        "languages": ["Rust"],
        "frameworks": ["tokio", "clap", "serde", "rust-mcp-sdk"],
        "databases": [],
        "tools": ["Cargo", "cargo-test", "cargo-clippy", "cargo-fmt"],
        "deployment": ["cargo-install", "crates.io"],
        "goals": [
            "Complete CLI implementation (Phases 4-11)",
            "Maintain backward compatibility as MCP server",
            "Provide comprehensive project management capabilities",
            "Enable self-hosted development workflow",
            "Publish stable release to crates.io"
        ],
        "target_users": [
            "AI developers using MCP protocol",
            "Rust developers needing project management",
            "Development teams using AI coding assistants",
            "Solo developers managing complex projects"
        ],
        "success_criteria": [
            "All CLI commands implemented and tested",
            "Full backward compatibility maintained",
            "Comprehensive test coverage (>90%)",
            "User documentation complete",
            "Successfully published to crates.io",
            "Self-hosting development workflow operational"
        ]
    });

    // Create the project
    match handler.handle_setup_project(&arguments).await {
        Ok(result) => {
            println!("Success: {}", result);
            println!("Project structure created in ~/.foundry/foundry-development");
        }
        Err(e) => {
            eprintln!("Error creating project: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}