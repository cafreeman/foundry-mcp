use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A CLI tool for project management with MCP server capabilities")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(long, global = true)]
    pub log_level: Option<String>,

    /// Custom configuration directory
    #[arg(long, global = true)]
    pub config_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCP server (default behavior when no subcommand is provided)
    Serve(ServeArgs),
    /// Install client configuration for Cursor or Claude Desktop
    Install(InstallArgs),
    /// Create a new project
    CreateProject(ProjectArgs),
    /// Create a new specification
    CreateSpec {
        /// Project name
        project: String,
        /// Specification name (snake_case)
        name: String,
        /// Specification description
        #[arg(long)]
        description: String,
    },
    /// List all projects
    ListProjects,
    /// Clear projects (with confirmation)
    ClearProjects {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
        /// Clear specific project only
        #[arg(long)]
        project: Option<String>,
        /// Create backup before clearing
        #[arg(long)]
        backup: bool,
    },
    /// Show detailed project information
    ShowProject {
        /// Project name to show
        name: String,
        /// Output format (table, json, yaml)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// List specifications for a project
    ListSpecs {
        /// Project name
        project: String,
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Show detailed specification information
    ShowSpec {
        /// Project name
        project: String,
        /// Specification ID
        spec_id: String,
        /// Show only tasks
        #[arg(long)]
        tasks_only: bool,
        /// Show only notes
        #[arg(long)]
        notes_only: bool,
        /// Output format (table, json, yaml)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Export project or specification data
    Export {
        /// Export specific project
        #[arg(long)]
        project: Option<String>,
        /// Export specific spec (requires project)
        #[arg(long)]
        spec: Option<String>,
        /// Output format (json, yaml, markdown)
        #[arg(long, default_value = "json")]
        format: String,
        /// Output directory
        #[arg(long)]
        output_dir: Option<String>,
    },
    /// Import project data from file
    Import {
        /// File to import from
        file: String,
        /// Merge with existing projects instead of erroring
        #[arg(long)]
        merge: bool,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Show system status and information
    Status {
        /// Show verbose information
        #[arg(long)]
        verbose: bool,
    },
    /// Run system diagnostics and health checks
    Doctor {
        /// Automatically fix issues where possible
        #[arg(long)]
        fix: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Get configuration value
    Get { key: String },
    /// Set configuration value
    Set { key: String, value: String },
    /// List all configuration
    List,
    /// Reset configuration to defaults
    Reset,
}

#[derive(Parser, Clone)]
pub struct ServeArgs {
    /// Port for HTTP transport (future use)
    #[arg(long, default_value = "3000")]
    pub port: u16,

    /// Transport mode (stdio is default and currently only supported)
    #[arg(long, default_value = "stdio")]
    pub transport: String,

    /// Host for HTTP mode (future use)
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Maximum number of connections
    #[arg(long, default_value = "10")]
    pub max_connections: u32,

    /// Tool execution timeout in seconds
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Backup retention in days
    #[arg(long, default_value = "7")]
    pub backup_retention_days: u32,

    /// Log format (json, pretty, compact)
    #[arg(long, default_value = "pretty")]
    pub log_format: String,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            port: 3000,
            transport: "stdio".to_string(),
            host: "localhost".to_string(),
            max_connections: 10,
            timeout: 300,
            backup_retention_days: 7,
            log_format: "pretty".to_string(),
        }
    }
}

#[derive(Parser)]
pub struct InstallArgs {
    /// Client to install for (cursor, claude-desktop)
    #[arg(long, value_parser = ["cursor", "claude-desktop"])]
    pub client: String,

    /// Install globally instead of project-specific
    #[arg(long)]
    pub global: bool,

    /// Preview configuration changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing configuration
    #[arg(long)]
    pub force: bool,

    /// Verify installation by testing MCP server startup
    #[arg(long)]
    pub verify: bool,

    /// Custom binary path (defaults to current binary)
    #[arg(long)]
    pub binary_path: Option<String>,
}

#[derive(Parser)]
pub struct ProjectArgs {
    /// Project name
    pub name: String,

    /// Project description
    #[arg(long)]
    pub description: Option<String>,

    /// Technology stack (comma-separated)
    #[arg(long)]
    pub tech_stack: Option<String>,

    /// Project vision/goals
    #[arg(long)]
    pub vision: Option<String>,

    /// Interactive mode for guided creation
    #[arg(long)]
    pub interactive: bool,

    /// Project template (web, cli, lib)
    #[arg(long)]
    pub template: Option<String>,
}
