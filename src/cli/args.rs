//! CLI argument structures

use clap::Args;

/// Arguments for create_project command
#[derive(Args, Debug)]
pub struct CreateProjectArgs {
    /// Project name in kebab-case (e.g., my-awesome-project)
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs)
    #[arg(long, required = true)]
    pub vision: String,

    /// Technology stack and architecture decisions
    #[arg(long, required = true)]
    pub tech_stack: String,

    /// Concise summary of vision and tech stack
    #[arg(long, required = true)]
    pub summary: String,
}

/// Arguments for analyze_project command
#[derive(Args, Debug)]
pub struct AnalyzeProjectArgs {
    /// Project name to analyze
    pub project_name: String,

    /// Path to codebase to analyze (defaults to current directory)
    pub codebase_path: Option<String>,

    /// High-level product vision content
    #[arg(long, required = true)]
    pub vision: String,

    /// Technology stack and architecture decisions
    #[arg(long, required = true)]
    pub tech_stack: String,

    /// Concise summary of vision and tech stack
    #[arg(long, required = true)]
    pub summary: String,
}

/// Arguments for create_spec command
#[derive(Args, Debug)]
pub struct CreateSpecArgs {
    /// Project name to create spec for
    pub project_name: String,

    /// Feature name in snake_case (e.g., user_authentication)
    pub feature_name: String,

    /// Detailed specification content
    #[arg(long, required = true)]
    pub spec: String,

    /// Implementation notes and considerations
    #[arg(long, required = true)]
    pub notes: String,

    /// Task list content
    #[arg(long, required = true)]
    pub tasks: String,
}

/// Arguments for load_spec command
#[derive(Args, Debug)]
pub struct LoadSpecArgs {
    /// Project name to load spec from
    pub project_name: String,

    /// Specific spec name (if not provided, lists available specs)
    pub spec_name: Option<String>,
}

/// Arguments for load_project command
#[derive(Args, Debug)]
pub struct LoadProjectArgs {
    /// Project name to load context from (must exist in ~/.foundry/)
    pub project_name: String,
}

/// Arguments for list_projects command
#[derive(Args, Debug)]
pub struct ListProjectsArgs;

/// Arguments for get_foundry_help command
#[derive(Args, Debug)]
pub struct GetFoundryHelpArgs {
    /// Help topic (workflows, content-examples, project-structure, parameter-guidance)
    pub topic: Option<String>,
}

/// Arguments for validate_content command
#[derive(Args, Debug)]
pub struct ValidateContentArgs {
    /// Content type to validate (vision, tech-stack, summary, spec, notes)
    pub content_type: String,

    /// Content to validate
    #[arg(long, required = true)]
    pub content: String,
}
