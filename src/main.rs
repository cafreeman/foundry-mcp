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
    about = "Foundry: central ~/.foundry specs, JSON workflow output for agents, optional git backup"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inventory (JSON): projects, active specs, or completed-work records
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
    Link { name: Option<String> },
    /// Git backup for ~/.foundry/
    Git {
        #[command(subcommand)]
        cmd: GitCmd,
    },
    /// Install bundled skills into the selected agent target
    Install { target: SkillInstallTarget },
    /// Update bundled skills in every supported installed target
    Update,
    /// Remove bundled skills from the selected agent target
    Uninstall { target: SkillInstallTarget },
    /// Store status (JSON: path, git, skills, cwd link)
    Status,
}

#[derive(Subcommand)]
enum ListCmd {
    /// List projects in ~/.foundry/ (JSON array of `{ name, path }`)
    Projects,
    /// List active specs for a project (JSON array)
    Specs { project: String },
    /// List collapsed completed-work records (JSON array)
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
    Delete {
        project: String,
        id: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
    /// Workflow status for one active spec (JSON)
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
    /// Next-step guidance for implementing tasks (JSON)
    Apply { project: String, id: String },
    /// Guidance for collapsing a finished spec (JSON)
    Collapse { project: String, id: String },
}

#[derive(Subcommand)]
enum SpecCollapseCmd {
    /// Create completed record dir and empty summary.md (JSON)
    Prepare { project: String, id: String },
    /// Move active spec files into archive/ after summary.md is written (JSON)
    Finalize {
        project: String,
        id: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum ProjectCmd {
    /// Create project (JSON: `name`, `path`)
    Init { name: String },
    /// Print vision / tech-stack / summary (plain text)
    Load { name: String },
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

fn list_projects() -> Result<()> {
    print_json(&project::list_project_entries()?)
}

fn list_specs_for_project(project: &str) -> Result<()> {
    print_json(&spec_store::list_active_entries(project)?)
}

fn list_completed_for_project(project: &str) -> Result<()> {
    print_json(&completed::list_completed_entries(project)?)
}

fn resolve_spec(project: &str, partial: &str) -> Result<String> {
    let specs_root = project::assert_project_exists(project)?.join("specs");
    if !specs_root.is_dir() {
        bail!("No specs directory for project {:?}", project);
    }
    spec_id::resolve_spec_id(&specs_root, partial)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { cmd } => match cmd {
            ListCmd::Projects => list_projects()?,
            ListCmd::Specs { project: p } => list_specs_for_project(&p)?,
            ListCmd::Completed { project: p } => list_completed_for_project(&p)?,
        },
        Commands::Project { cmd } => match cmd {
            ProjectCmd::Init { name } => {
                let proj = project::init(&name)?;
                print_json(&serde_json::json!({
                    "name": proj.name,
                    "path": proj.path.display().to_string(),
                }))?;
            }
            ProjectCmd::Load { name } => {
                project::load_print(&name)?;
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
                print_json(&serde_json::json!({
                    "id": s.id,
                    "project": s.project_name,
                    "feature": s.feature,
                    "path": s.path.display().to_string(),
                }))?;
            }
            SpecCmd::Load { project, id } => {
                spec_store::load_print(&project, &id)?;
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
                print_json(&st)?;
            }
            SpecCmd::Instructions { cmd } => match cmd {
                SpecInstructionsCmd::Apply { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let j = workflow::build_instructions_apply(&project, &resolved)?;
                    print_json(&j)?;
                }
                SpecInstructionsCmd::Collapse { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let j = workflow::build_instructions_collapse(&project, &resolved)?;
                    print_json(&j)?;
                }
            },
            SpecCmd::Collapse { cmd } => match cmd {
                SpecCollapseCmd::Prepare { project, id } => {
                    let resolved = resolve_spec(&project, &id)?;
                    let summary_path = completed::collapse_prepare(&project, &resolved)?;
                    print_json(&serde_json::json!({
                        "project": project,
                        "spec_id": resolved,
                        "summary_path": summary_path.display().to_string(),
                        "instruction": "Edit summary.md with the completed-work narrative, then run `foundry spec collapse finalize --confirm`.",
                    }))?;
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
                    print_json(&serde_json::json!({
                        "project": project,
                        "spec_id": resolved,
                        "completed_dir": done_dir.display().to_string(),
                    }))?;
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

            print_json(&StoreStatusJson {
                store: root.display().to_string(),
                projects: project::list_names()?.len(),
                git_initialized: git::is_git_repo(&root),
                skills: skill_install::install_status()?,
                cwd_link_project: link_project,
            })?;
        }
    }

    Ok(())
}
