use crate::cache::ProjectManagerCache;
use crate::cli::args::{ConfigAction, ProjectArgs};
use crate::filesystem::FileSystemManager;
use crate::handlers::{create_spec::CreateSpecHandler, setup_project::SetupProjectHandler};
use crate::repository::{ProjectRepository, SpecificationRepository};
use anyhow::Result;
use serde_json::json;

pub async fn create_project(args: ProjectArgs) -> Result<()> {
    // Initialize components
    let fs_manager = FileSystemManager::new()?;
    let cache = ProjectManagerCache::new();
    let project_repo = ProjectRepository::with_cache(fs_manager, cache);
    let handler = SetupProjectHandler::new(project_repo);

    // For CLI usage, parse tech stack and put all items in frameworks by default
    // Users can manually organize later or we can enhance this in the future
    let tech_stack_items: Vec<String> = args
        .tech_stack
        .as_ref()
        .map(|s| s.split(',').map(|item| item.trim().to_string()).collect())
        .unwrap_or_default();

    // Prepare arguments for setup_project handler
    let arguments = json!({
        "name": args.name,
        "description": args.description.unwrap_or_else(|| "No description provided".to_string()),
        "overview": args.vision.unwrap_or_else(|| "No vision provided".to_string()),
        "languages": [],  // CLI doesn't auto-detect languages - user can organize later
        "frameworks": tech_stack_items,  // Put all tech stack items here by default
        "databases": [],
        "tools": [],
        "deployment": [],
        "goals": ["Complete project development"],
        "target_users": ["Development team"],
        "success_criteria": ["Project goals achieved"]
    });

    // Create the project using the setup_project handler
    match handler.handle_setup_project(&arguments).await {
        Ok(result) => {
            println!("✓ {}", result);
            println!("📁 Project structure created in ~/.foundry/{}", args.name);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Error creating project: {}", e);
            Err(e.into())
        }
    }
}

pub async fn create_spec(
    project_name: String,
    spec_name: String,
    description: String,
) -> Result<()> {
    // Initialize components
    let fs_manager = FileSystemManager::new()?;
    let cache = ProjectManagerCache::new();
    let project_repo = ProjectRepository::with_cache(fs_manager.clone(), cache.clone());
    let spec_repo = SpecificationRepository::with_cache(fs_manager, cache);
    let handler = CreateSpecHandler::new(project_repo.clone(), spec_repo);

    // Check if project exists
    if !project_repo.project_exists(&project_name).await {
        eprintln!("❌ Project '{}' not found.", project_name);
        println!("Available projects:");
        let projects = project_repo.list_projects().await?;
        for p in projects {
            println!("  - {}", p.name);
        }
        return Ok(());
    }

    // Prepare arguments for create_spec handler
    let arguments = json!({
        "project_name": project_name,
        "spec_name": spec_name,
        "description": description
    });

    // Create the specification using the create_spec handler
    match handler.handle_create_spec(&arguments).await {
        Ok(result) => {
            println!("✓ {}", result);
            println!(
                "📄 Specification created in ~/.foundry/{}/specs/",
                project_name
            );
            println!("💡 View with: foundry-mcp list-specs {}", project_name);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error creating specification: {}", e);
            Err(e.into())
        }
    }
}

pub async fn list_projects() -> Result<()> {
    // Initialize components
    let fs_manager = FileSystemManager::new()?;
    let cache = ProjectManagerCache::new();
    let project_repo = ProjectRepository::with_cache(fs_manager.clone(), cache.clone());

    // Get list of projects
    match project_repo.list_projects().await {
        Ok(projects) => {
            if projects.is_empty() {
                println!("No projects found.");
                println!("Create a project with: foundry-mcp create-project <name>");
                return Ok(());
            }

            println!("📂 Projects:");
            println!();
            for project in projects {
                println!("  {} - {}", project.name, project.description);
                println!(
                    "    Created: {}",
                    project.created_at.format("%Y-%m-%d %H:%M")
                );

                // Count specifications
                let spec_repo =
                    SpecificationRepository::with_cache(fs_manager.clone(), cache.clone());
                let spec_count = spec_repo
                    .list_specs(&project.name)
                    .await
                    .unwrap_or_default()
                    .len();
                println!("    Specs: {}", spec_count);
                println!();
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error listing projects: {}", e);
            Err(e.into())
        }
    }
}

pub async fn clear_projects(_force: bool, _project: Option<String>, _backup: bool) -> Result<()> {
    // This will be implemented in Phase 5
    println!("Clear projects command - implementation coming in Phase 5");
    Ok(())
}

pub async fn show_project(_name: String, _format: String) -> Result<()> {
    // This will be implemented in Phase 6
    println!("Show project command - implementation coming in Phase 6");
    Ok(())
}

pub async fn list_specs(project: String, _detailed: bool, _json: bool) -> Result<()> {
    // Initialize components
    let fs_manager = FileSystemManager::new()?;
    let cache = ProjectManagerCache::new();
    let spec_repo = SpecificationRepository::with_cache(fs_manager.clone(), cache.clone());
    let project_repo = ProjectRepository::with_cache(fs_manager, cache);

    // Check if project exists
    if !project_repo.project_exists(&project).await {
        eprintln!("❌ Project '{}' not found.", project);
        println!("Available projects:");
        let projects = project_repo.list_projects().await?;
        for p in projects {
            println!("  - {}", p.name);
        }
        return Ok(());
    }

    // Get list of specifications
    match spec_repo.list_specs(&project).await {
        Ok(specs) => {
            if specs.is_empty() {
                println!("No specifications found for project '{}'.", project);
                println!(
                    "Create a spec with: foundry-mcp create-spec {} <spec-name>",
                    project
                );
                return Ok(());
            }

            if _json {
                let json_output = serde_json::to_string_pretty(&specs)?;
                println!("{}", json_output);
            } else {
                println!("📋 Specifications for '{}':", project);
                println!();
                for spec in specs {
                    println!("  {} - {}", spec.id, spec.name);
                    println!("    Created: {}", spec.created_at.format("%Y-%m-%d %H:%M"));
                    println!("    Status: {:?}", spec.status);
                    println!("    Description: {}", spec.description);
                    println!();
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error listing specifications: {}", e);
            Err(e.into())
        }
    }
}

pub async fn show_spec(
    _project: String,
    _spec_id: String,
    _tasks_only: bool,
    _notes_only: bool,
    _format: String,
) -> Result<()> {
    // This will be implemented in Phase 6
    println!("Show spec command - implementation coming in Phase 6");
    Ok(())
}

pub async fn export_data(
    _project: Option<String>,
    _spec: Option<String>,
    _format: String,
    _output_dir: Option<String>,
) -> Result<()> {
    // This will be implemented in Phase 6
    println!("Export command - implementation coming in Phase 6");
    Ok(())
}

pub async fn import_data(_file: String, _merge: bool) -> Result<()> {
    // This will be implemented in Phase 6
    println!("Import command - implementation coming in Phase 6");
    Ok(())
}

pub async fn handle_config(_action: ConfigAction) -> Result<()> {
    // This will be implemented in Phase 7
    println!("Config command - implementation coming in Phase 7");
    Ok(())
}

pub async fn show_status(_verbose: bool) -> Result<()> {
    // This will be implemented in Phase 7
    println!("Status command - implementation coming in Phase 7");
    Ok(())
}

pub async fn run_doctor(_fix: bool) -> Result<()> {
    // This will be implemented in Phase 7
    println!("Doctor command - implementation coming in Phase 7");
    Ok(())
}

pub async fn generate_completions(_shell: String) -> Result<()> {
    // This will be implemented in Phase 8
    println!("Completions command - implementation coming in Phase 8");
    Ok(())
}
