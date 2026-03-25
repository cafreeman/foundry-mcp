use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use foundry_mcp::core::{
    completed, git, link, paths, project, skill_install, spec_id, spec_store, workflow,
};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "foundry")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "Foundry: central ~/.foundry specs, workflow JSON for agents, optional git backup"
)]
struct Cli {
    /// Emit machine-readable JSON for supported commands (skills and automation)
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inventory: projects, active specs, or completed-work records
    List {
        #[command(subcommand)]
        cmd: ListCmd,
    },
    /// Project storage
    Project {
        #[command(subcommand)]
        cmd: ProjectCmd,
    },
    /// Spec workflow and storage
    Spec {
        #[command(subcommand)]
        cmd: SpecCmd,
    },
    /// Create a `.foundry` symlink in the current directory
    Link {
        name: Option<String>,
    },
    /// Git backup for ~/.foundry/
    Git {
        #[command(subcommand)]
        cmd: GitCmd,
    },
    Install {
        target: SkillInstallTarget,
    },
    Update,
    Uninstall {
        target: SkillInstallTarget,
    },
    /// Store status (path, git, skills, cwd link)
    Status,
}

#[derive(Subcommand)]
enum ListCmd {
    /// List projects in ~/.foundry/
    Projects,
    /// List active specs for a project
    Specs { project: String },
    /// List collapsed completed-work records
    Completed { project: String },
}

#[derive(Subcommand)]
enum SpecCmd {
    Init {
        project: String,
        feature: String,
    },
    Load {
        project: String,
        id: String,
    },
    List {
        project: String,
    },
    Delete {
        project: String,
        id: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
    /// Workflow status for one active spec
    Status {
        project: String,
        id: String,
    },
    Instructions {
        #[command(subcommand)]
        cmd: SpecInstructionsCmd,
    },
    Collapse {
        #[command(subcommand)]
        cmd: SpecCollapseCmd,
    },
}

#[derive(Subcommand)]
enum SpecInstructionsCmd {
    /// Next-step guidance for implementing tasks (JSON-first for agents)
    Apply { project: String, id: String },
    /// Guidance for collapsing a finished spec into a completed-work record
    Collapse { project: String, id: String },
}

#[derive(Subcommand)]
enum SpecCollapseCmd {
    /// Create ~/.foundry/<project>/completed/<id>/ and empty summary.md
    Prepare { project: String, id: String },
    /// Move active spec files into archive/ after summary.md is written
    Finalize {
        project: String,
        id: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum ProjectCmd {
    Init {
        name: String,
    },
    Load {
        name: String,
    },
    List,
    Delete {
        name: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum GitCmd {
    Init,
    Remote {
        url: String,
    },
    Sync {
        #[arg(trailing_var_arg = true, num_args = 1.., required = true)]
        message: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum SkillInstallTarget {
    ClaudeCode,
    Cursor,
}

impl SkillInstallTarget {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
        }
    }
}

#[derive(Serialize)]
struct StoreStatusJson {
    store: String,
    projects: usize,
    git_initialized: bool,
    skills: skill_install::InstallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    cwd_link_project: Option<String>,
}

fn print_json<T: Serialize>(v: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn resolve_spec(project: &str, partial: &str) -> Result<String> {
    project::validate_project_name(project)?;
    let specs_root = paths::project_path(project)?.join("specs");
    if !specs_root.is_dir() {
        bail!("No specs directory for project {:?}", project);
    }
    spec_id::resolve_spec_id(&specs_root, partial)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { cmd } => match cmd {
            ListCmd::Projects => {
                if cli.json {
                    let entries = project::list_project_entries()?;
                    print_json(&entries)?;
                } else {
                    let names = project::list_names()?;
                    if names.is_empty() {
                        println!("No projects found");
                    } else {
                        for n in names {
                            println!("{n}");
                        }
                    }
                }
            }
            ListCmd::Specs { project: p } => {
                if cli.json {
                    let entries = spec_store::list_active_entries(&p)?;
                    print_json(&entries)?;
                } else {
                    let entries = spec_store::list_active_entries(&p)?;
                    if entries.is_empty() {
                        println!("No specs found for {p}");
                    } else {
                        for e in entries {
                            println!("{}", e.id);
                        }
                    }
                }
            }
            ListCmd::Completed { project: p } => {
                if cli.json {
                    let entries = completed::list_completed_entries(&p)?;
                    print_json(&entries)?;
                } else {
                    let ids = completed::list_completed_ids(&p)?;
                    if ids.is_empty() {
                        println!("No completed records for {p}");
                    } else {
                        for id in ids {
                            println!("{id}");
                        }
                    }
                }
            }
        },
        Commands::Project { cmd } => match cmd {
            ProjectCmd::Init { name } => {
                let proj = project::init(&name)?;
                if cli.json {
                    print_json(&serde_json::json!({
                        "name": proj.name,
                        "path": proj.path.display().to_string(),
                    }))?;
                } else {
                    println!("{}", proj.path.display());
                }
            }
            ProjectCmd::Load { name } => {
                if cli.json {
                    bail!(
                        "Use plain text `project load` for file dumps; JSON bundle not implemented here"
                    );
                }
                project::load_print(&name)?;
            }
            ProjectCmd::List => {
                if cli.json {
                    let entries = project::list_project_entries()?;
                    print_json(&entries)?;
                } else {
                    let names = project::list_names()?;
                    if names.is_empty() {
                        println!("No projects found");
                    } else {
                        for n in names {
                            println!("{n}");
                        }
                    }
                }
            }
            ProjectCmd::Delete { name, confirm } => {
                if !confirm {
                    bail!("Refusing to delete without `--confirm`");
                }
                project::delete_confirmed(&name)?;
            }
        },
        Commands::Spec { cmd } => match cmd {
            SpecCmd::Init { project, feature } => {
                let s = spec_store::init(&project, &feature)?;
                if cli.json {
                    print_json(&serde_json::json!({
                        "id": s.id,
                        "project": s.project_name,
                        "feature": s.feature,
                        "path": s.path.display().to_string(),
                    }))?;
                } else {
                    println!("{}", s.path.display());
                }
            }
            SpecCmd::Load { project, id } => {
                if cli.json {
                    bail!(
                        "Use `spec status` / `spec instructions apply --json` for structured context"
                    );
                }
                spec_store::load_print(&project, &id)?;
            }
            SpecCmd::List { project } => {
                if cli.json {
                    let entries = spec_store::list_active_entries(&project)?;
                    print_json(&entries)?;
                } else {
                    let ids = spec_store::list_ids(&project)?;
                    if ids.is_empty() {
                        println!("No specs found for {project}");
                    } else {
                        for id in ids {
                            println!("{id}");
                        }
                    }
                }
            }
            SpecCmd::Delete {
                project,
                id,
                confirm,
            } => {
                if !confirm {
                    bail!("Refusing to delete without `--confirm`");
                }
                let resolved = resolve_spec(&project, &id)?;
                spec_store::delete_confirmed(&project, &resolved)?;
            }
            SpecCmd::Status { project, id } => {
                let resolved = resolve_spec(&project, &id)?;
                let st = workflow::build_spec_status(&project, &resolved)?;
                if cli.json {
                    print_json(&st)?;
                } else {
                    println!("project: {}", st.project);
                    println!("spec_id: {}", st.spec_id);
                    println!("path: {}", st.path.display());
                    println!("phase: {:?}", st.derived_phase);
                    println!("state: {}", st.state);
                    println!("tasks: {} / {}", st.task_list.complete, st.task_list.total);
                    println!();
                    println!("{}", st.instruction);
                }
            }
            SpecCmd::Instructions { cmd } => match cmd {
                SpecInstructionsCmd::Apply { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let j = workflow::build_instructions_apply(&project, &resolved)?;
                    if cli.json {
                        print_json(&j)?;
                    } else {
                        println!("change: {}", j.change_name);
                        println!("state: {}", j.state);
                        println!(
                            "progress: {}/{} ({} remaining)",
                            j.progress.complete, j.progress.total, j.progress.remaining
                        );
                        println!();
                        println!("{}", j.instruction);
                    }
                }
                SpecInstructionsCmd::Collapse { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let j = workflow::build_instructions_collapse(&project, &resolved)?;
                    if !cli.json {
                        bail!("`spec instructions collapse` is intended for `--json`; pass --json");
                    }
                    print_json(&j)?;
                }
            },
            SpecCmd::Collapse { cmd } => match cmd {
                SpecCollapseCmd::Prepare { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let summary_path = completed::collapse_prepare(&project, &resolved)?;
                    if cli.json {
                        print_json(&serde_json::json!({
                            "project": project,
                            "spec_id": resolved,
                            "summary_path": summary_path.display().to_string(),
                            "instruction": "Edit summary.md with the completed-work narrative, then run `foundry spec collapse finalize --confirm`.",
                        }))?;
                    } else {
                        println!("{}", summary_path.display());
                    }
                }
                SpecCollapseCmd::Finalize {
                    project,
                    id,
                    confirm,
                } => {
                    if !confirm {
                        bail!("Refusing to finalize without `--confirm`");
                    }
                    let resolved = resolve_spec(&project, &id)?;
                    let done_dir = completed::collapse_finalize(&project, &resolved)?;
                    if cli.json {
                        print_json(&serde_json::json!({
                            "project": project,
                            "spec_id": resolved,
                            "completed_dir": done_dir.display().to_string(),
                        }))?;
                    } else {
                        println!("Collapsed into {}", done_dir.display());
                    }
                }
            },
        },
        Commands::Link { name } => {
            let cwd = link::current_dir()?;
            let proj = match name {
                Some(n) => n,
                None => link::detect_project_name(&cwd)?,
            };
            link::link(&cwd, &proj)?;
        }
        Commands::Git { cmd } => {
            let root = paths::ensure_foundry_dir()?;
            match cmd {
                GitCmd::Init => git::init_foundry_repo(&root)?,
                GitCmd::Remote { url } => git::set_origin_remote(&root, &url)?,
                GitCmd::Sync { message } => {
                    let msg = message.join(" ");
                    git::sync(&root, &msg)?;
                }
            }
        }
        Commands::Install { target } => {
            let cwd = link::current_dir()?;
            skill_install::install(target.as_str(), &cwd)?;
        }
        Commands::Update => skill_install::update()?,
        Commands::Uninstall { target } => {
            let cwd = link::current_dir()?;
            skill_install::uninstall(target.as_str(), &cwd)?;
        }
        Commands::Status => {
            let root = paths::foundry_dir()?;
            let link_project = if let Ok(cwd) = link::current_dir() {
                link::resolve_dot_foundry_project(&cwd)?
            } else {
                None
            };

            if cli.json {
                print_json(&StoreStatusJson {
                    store: root.display().to_string(),
                    projects: project::list_names()?.len(),
                    git_initialized: git::is_git_repo(&root),
                    skills: skill_install::install_status()?,
                    cwd_link_project: link_project,
                })?;
            } else {
                println!("store: {}", root.display());
                println!("projects: {}", project::list_names()?.len());
                println!(
                    "git: {}",
                    if git::is_git_repo(&root) {
                        "initialized"
                    } else {
                        "not initialized"
                    }
                );
                if let Some(p) = link_project {
                    println!("cwd link -> project: {p}");
                }
                let targets = skill_install::installed_targets_summary()?;
                if targets.is_empty() {
                    println!("skills: (none recorded)");
                } else {
                    println!("skills: {targets}");
                }
            }
        }
    }

    Ok(())
}
