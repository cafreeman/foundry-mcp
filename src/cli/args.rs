//! CLI argument structures

use clap::Args;

/// Arguments for create_project command
#[derive(Args, Debug)]
pub struct CreateProjectArgs {
    /// Project name in kebab-case (e.g., my-awesome-project)
    ///
    /// Must contain only lowercase letters, numbers, and hyphens
    /// Cannot contain spaces, underscores, or special characters
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs, 200+ characters)
    ///
    /// Should answer: What problem does this solve? Who is it for?
    /// What makes it unique? What are the key features and priorities?
    /// Goes into project/vision.md
    #[arg(long, required = true)]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters)
    ///
    /// Include: languages, frameworks, databases, deployment platforms
    /// Also include: rationale, constraints, team preferences, standards
    /// Goes into project/tech-stack.md
    #[arg(long, required = true)]
    pub tech_stack: String,

    /// Concise summary of vision and tech stack (100+ characters)
    ///
    /// Should capture essential project essence in 2-3 sentences
    /// Used for quick context loading in LLM sessions
    /// Goes into project/summary.md
    #[arg(long, required = true)]
    pub summary: String,
}

/// Arguments for analyze_project command
#[derive(Args, Debug)]
pub struct AnalyzeProjectArgs {
    /// Project name to create with your analyzed content
    ///
    /// Must be in kebab-case format (lowercase letters, numbers, hyphens only)
    /// Cannot contain spaces, underscores, or special characters
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs, 200+ characters) based on your codebase analysis
    ///
    /// Use codebase_search, grep_search, and read_file to understand the project first
    /// Consider: existing code patterns, project scope, apparent goals, user-facing features
    /// Should answer: What problem does this solve? Who is it for? What makes it unique?
    /// Goes into project/vision.md
    #[arg(long, required = true)]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters) based on your codebase exploration
    ///
    /// Use your analysis tools to detect languages/frameworks, build systems, deployment patterns, dependencies
    /// Include: rationale for existing choices, recommendations, constraints, team preferences
    /// Goes into project/tech-stack.md
    #[arg(long, required = true)]
    pub tech_stack: String,

    /// Concise summary (100+ characters) combining vision and tech stack from your analysis
    ///
    /// Should capture key insights from your codebase exploration for quick context loading
    /// Use this to understand the project essence before diving into implementation
    /// Goes into project/summary.md
    #[arg(long, required = true)]
    pub summary: String,
}

/// Arguments for create_spec command
#[derive(Args, Debug)]
pub struct CreateSpecArgs {
    /// Project name to create spec for
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    pub project_name: String,

    /// Feature name in snake_case (e.g., user_authentication)
    ///
    /// Used to create timestamped directory: YYYYMMDD_HHMMSS_feature_name
    /// Should be descriptive and use underscores, not spaces or hyphens
    pub feature_name: String,

    /// Detailed specification content
    ///
    /// Should include: overview, requirements, implementation approach, testing strategy
    /// Use markdown formatting for structure (headers, lists, code blocks)
    /// Goes into spec.md
    #[arg(long, required = true)]
    pub spec: String,

    /// Implementation notes and considerations
    ///
    /// Include: design decisions, dependencies, tradeoffs, constraints
    /// Use markdown formatting for organization
    /// Goes into notes.md
    #[arg(long, required = true)]
    pub notes: String,

    /// Task list content
    ///
    /// Implementation checklist in markdown format
    /// Use "- [ ] Task description" format for checkboxes
    /// Update this as work progresses
    /// Goes into task-list.md
    #[arg(long, required = true)]
    pub tasks: String,
}

/// Arguments for load_spec command
#[derive(Args, Debug)]
pub struct LoadSpecArgs {
    /// Project name to load spec from
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    pub project_name: String,

    /// Specific spec name (if not provided, lists available specs)
    ///
    /// Spec names are in format: YYYYMMDD_HHMMSS_feature_name
    /// If omitted, returns list of all available specs for the project
    /// Use 'foundry load-project PROJECT_NAME' to see project context first
    pub spec_name: Option<String>,
}

/// Arguments for load_project command
#[derive(Args, Debug)]
pub struct LoadProjectArgs {
    /// Project name to load context from (must exist in ~/.foundry/)
    ///
    /// Returns complete project context: vision, tech-stack, summary, and available specs
    /// Essential for resuming work on existing projects
    /// Use 'foundry list-projects' to see available project names
    pub project_name: String,
}

/// Arguments for list_projects command
#[derive(Args, Debug)]
pub struct ListProjectsArgs;

// Note: This command takes no arguments - it lists all projects in ~/.foundry/
// Returns: project names, creation dates, spec counts, validation status
// Use this to discover available projects before loading or creating specs

/// Arguments for get_foundry_help command
#[derive(Args, Debug)]
pub struct GetFoundryHelpArgs {
    /// Help topic for detailed guidance
    ///
    /// Available topics:
    /// - workflows: Step-by-step development workflows
    /// - content-examples: Content templates and examples
    /// - project-structure: File organization and structure
    /// - parameter-guidance: Parameter requirements and best practices
    ///
    /// If omitted, provides overview and available topics
    pub topic: Option<String>,
}

/// Arguments for validate_content command
#[derive(Args, Debug)]
pub struct ValidateContentArgs {
    /// Content to validate
    ///
    /// The actual content string to check against schema requirements
    /// Validation includes: length, format, content quality, improvement suggestions
    /// Returns detailed feedback for content improvement
    #[arg(long, required = true)]
    pub content: String,

    /// Content type to validate
    ///
    /// Must be one of: vision, tech-stack, summary, spec, notes, tasks
    /// Each type has specific length and quality requirements
    /// Use this to check content before creating projects/specs
    pub content_type: String,
}

// MCP parameter conversion implementations
impl CreateProjectArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            vision: params["vision"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing vision parameter"))?
                .to_string(),
            tech_stack: params["tech_stack"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing tech_stack parameter"))?
                .to_string(),
            summary: params["summary"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing summary parameter"))?
                .to_string(),
        })
    }
}

impl AnalyzeProjectArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            vision: params["vision"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing vision parameter"))?
                .to_string(),
            tech_stack: params["tech_stack"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing tech_stack parameter"))?
                .to_string(),
            summary: params["summary"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing summary parameter"))?
                .to_string(),
        })
    }
}

impl CreateSpecArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            feature_name: params["feature_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing feature_name parameter"))?
                .to_string(),
            spec: params["spec"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing spec parameter"))?
                .to_string(),
            notes: params["notes"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing notes parameter"))?
                .to_string(),
            tasks: params["task_list"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing task_list parameter"))?
                .to_string(),
        })
    }
}

impl LoadSpecArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            spec_name: params["spec_name"].as_str().map(|s| s.to_string()),
        })
    }
}

impl LoadProjectArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
        })
    }
}

impl ListProjectsArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(_params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl GetFoundryHelpArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            topic: params["topic"].as_str().map(|s| s.to_string()),
        })
    }
}

impl ValidateContentArgs {
    /// Convert MCP parameters to CLI arguments
    pub fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            content_type: params["content_type"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing content_type parameter"))?
                .to_string(),
            content: params["content"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?
                .to_string(),
        })
    }
}
