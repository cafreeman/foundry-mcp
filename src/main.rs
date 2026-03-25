use anyhow::{Result, bail};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use foundry_mcp::core::{git, link, paths, project, skill_install, spec_store};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Scaffold, load, and back up Foundry project context in ~/.foundry/")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Project storage commands
    Project {
        #[command(subcommand)]
        cmd: ProjectCmd,
    },
    /// Spec storage commands
    Spec {
        #[command(subcommand)]
        cmd: SpecCmd,
    },
    /// Create a `.foundry` symlink in the current directory
    Link {
        /// Project name in ~/.foundry (optional; auto-detected when omitted)
        name: Option<String>,
    },
    /// Optional git backup for ~/.foundry/
    Git {
        #[command(subcommand)]
        cmd: GitCmd,
    },
    /// Install bundled skills for an AI tool
    Install {
        /// `claude-code` or `cursor`
        target: SkillInstallTarget,
    },
    /// Update installed skill files to the versions bundled in this binary
    Update,
    /// Remove Foundry-installed skill files for a target
    Uninstall {
        /// `claude-code` or `cursor`
        target: SkillInstallTarget,
    },
    /// Show Foundry store and install status
    Status,
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

#[derive(Subcommand)]
enum ProjectCmd {
    /// Create a new project scaffold
    Init {
        /// Kebab-case project name
        name: String,
    },
    /// Print vision.md, tech-stack.md, and summary.md
    Load { name: String },
    /// List projects
    List,
    /// Delete a project
    Delete {
        name: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum SpecCmd {
    /// Create a new timestamped spec scaffold
    Init {
        project: String,
        /// Kebab-case feature name (used in the directory suffix)
        feature: String,
    },
    /// Print spec.md, task-list.md, and notes.md
    Load {
        project: String,
        /// Spec id or an unambiguous partial id
        id: String,
    },
    /// List spec directories for a project
    List { project: String },
    /// Delete a spec directory
    Delete {
        project: String,
        id: String,
        #[arg(long, action = ArgAction::SetTrue)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum GitCmd {
    /// Initialize ~/.foundry as a git repository
    Init,
    /// Set or update the `origin` remote URL
    Remote { url: String },
    /// Stage all changes, commit, and push (if configured)
    Sync {
        #[arg(trailing_var_arg = true, num_args = 1.., required = true)]
        message: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Project { cmd } => match cmd {
            ProjectCmd::Init { name } => {
                let p = project::init(&name)?;
                println!("{}", p.path.display());
            }
            ProjectCmd::Load { name } => project::load_print(&name)?,
            ProjectCmd::List => {
                let names = project::list_names()?;
                if names.is_empty() {
                    println!("No projects found");
                } else {
                    for n in names {
                        println!("{n}");
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
                println!("{}", s.path.display());
            }
            SpecCmd::Load { project, id } => {
                spec_store::load_print(&project, &id)?;
            }
            SpecCmd::List { project } => {
                let ids = spec_store::list_ids(&project)?;
                if ids.is_empty() {
                    println!("No specs found for {project}");
                } else {
                    for id in ids {
                        println!("{id}");
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
                spec_store::delete_confirmed(&project, &id)?;
            }
        },
        Commands::Link { name } => {
            let cwd = link::current_dir()?;
            let project = match name {
                Some(n) => n,
                None => link::detect_project_name(&cwd)?,
            };
            link::link(&cwd, &project)?;
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
            println!("store: {}", root.display());

            let projects = project::list_names()?.len();
            println!("projects: {projects}");

            let git = if git::is_git_repo(&root) {
                "initialized"
            } else {
                "not initialized"
            };
            println!("git: {git}");

            let targets = skill_install::installed_targets_summary()?;
            if targets.is_empty() {
                println!("skills: (none recorded)");
            } else {
                println!("skills: {targets}");
            }
        }
    }

    Ok(())
}
