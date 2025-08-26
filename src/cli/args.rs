//! CLI argument structures

use crate::McpTool;
use clap::Args;

/// Arguments for create_project command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "create_project",
    description = "Create new project structure with LLM-provided content. Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md"
)]
pub struct CreateProjectArgs {
    /// Project name in kebab-case (e.g., my-awesome-project)
    ///
    /// Must contain only lowercase letters, numbers, and hyphens
    /// Cannot contain spaces, underscores, or special characters
    #[mcp(description = "Descriptive project name using kebab-case (e.g., 'my-awesome-app')")]
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs, 200+ characters)
    ///
    /// Should answer: What problem does this solve? Who is it for?
    /// What makes it unique? What are the key features and priorities?
    /// Goes into project/vision.md
    #[arg(long, required = true)]
    #[mcp(
        description = "High-level product vision (2-4 paragraphs, 200+ chars) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Goes into project/vision.md",
        min_length = 200
    )]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters)
    ///
    /// Include: languages, frameworks, databases, deployment platforms
    /// Also include: rationale, constraints, team preferences, standards
    /// Goes into project/tech-stack.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Comprehensive technology decisions (150+ chars) including languages, frameworks, databases, deployment platforms, and rationale. Include constraints, preferences, or team standards. Goes into project/tech-stack.md",
        min_length = 150
    )]
    pub tech_stack: String,

    /// Concise summary of vision and tech stack (100+ characters)
    ///
    /// Should capture essential project essence in 2-3 sentences
    /// Used for quick context loading in LLM sessions
    /// Goes into project/summary.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Concise summary (100+ chars) of vision and tech-stack for quick context loading. Should capture essential project essence in 2-3 sentences. Goes into project/summary.md",
        min_length = 100
    )]
    pub summary: String,
}

/// Arguments for analyze_project command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "analyze_project",
    description = "Create project structure by analyzing existing codebase. LLM analyzes codebase and provides vision, tech-stack, and summary content."
)]
pub struct AnalyzeProjectArgs {
    /// Project name to create with your analyzed content
    ///
    /// Must be in kebab-case format (lowercase letters, numbers, hyphens only)
    /// Cannot contain spaces, underscores, or special characters
    #[mcp(description = "Descriptive project name using kebab-case (e.g., 'my-analyzed-project')")]
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs, 200+ characters) based on your codebase analysis
    ///
    /// Use codebase_search, grep_search, and read_file to understand the project first
    /// Consider: existing code patterns, project scope, apparent goals, user-facing features
    /// Should answer: What problem does this solve? Who is it for? What makes it unique?
    /// Goes into project/vision.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-analyzed product vision (200+ chars) based on codebase examination covering: problem being solved, target users, unique value proposition from code analysis. Goes into project/vision.md",
        min_length = 200
    )]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters) based on your codebase exploration
    ///
    /// Use your analysis tools to detect languages/frameworks, build systems, deployment patterns, dependencies
    /// Include: rationale for existing choices, recommendations, constraints, team preferences
    /// Goes into project/tech-stack.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-detected technology stack (150+ chars) and architectural decisions based on codebase analysis including languages, frameworks, databases, deployment platforms, and rationale. Goes into project/tech-stack.md",
        min_length = 150
    )]
    pub tech_stack: String,

    /// Concise summary (100+ characters) combining vision and tech stack from your analysis
    ///
    /// Should capture key insights from your codebase exploration for quick context loading
    /// Use this to understand the project essence before diving into implementation
    /// Goes into project/summary.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-created concise summary (100+ chars) of analyzed project combining vision and tech-stack insights for quick context loading. Goes into project/summary.md",
        min_length = 100
    )]
    pub summary: String,
}

/// Arguments for create_spec command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "create_spec",
    description = "Create timestamped specification for a feature. Creates YYYYMMDD_HHMMSS_FEATURE_NAME directory with spec.md, task-list.md, and notes.md"
)]
pub struct CreateSpecArgs {
    /// Project name to create spec for
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    #[mcp(description = "Name of the existing project to create spec for")]
    pub project_name: String,

    /// Feature name in snake_case (e.g., user_authentication)
    ///
    /// Used to create timestamped directory: YYYYMMDD_HHMMSS_feature_name
    /// Should be descriptive and use underscores, not spaces or hyphens
    #[mcp(description = "Descriptive feature name using snake_case (e.g., 'user_authentication')")]
    pub feature_name: String,

    /// Detailed specification content
    ///
    /// Should include: overview, requirements, implementation approach, testing strategy
    /// Use markdown formatting for structure (headers, lists, code blocks)
    /// Goes into spec.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Detailed feature specification (200+ chars) covering requirements, acceptance criteria, implementation approach, and constraints. Goes into spec.md",
        min_length = 200
    )]
    pub spec: String,

    /// Implementation notes and considerations
    ///
    /// Include: design decisions, dependencies, tradeoffs, constraints
    /// Use markdown formatting for organization
    /// Goes into notes.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Additional context and design decisions (50+ chars) for the feature implementation. Goes into notes.md",
        min_length = 50
    )]
    pub notes: String,

    /// Task list content
    ///
    /// Implementation checklist in markdown format
    /// Use "- [ ] Task description" format for checkboxes
    /// Update this as work progresses
    /// Goes into task-list.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Markdown checklist (100+ chars) of implementation steps that agents can update as work progresses. Goes into task-list.md",
        min_length = 100
    )]
    pub tasks: String,
}

/// Arguments for load_spec command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "load_spec",
    description = "Load specific specification content with project context. If spec_name is omitted, lists available specs."
)]
pub struct LoadSpecArgs {
    /// Project name to load spec from
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    #[mcp(description = "Name of the project containing the spec")]
    pub project_name: String,

    /// Specific spec name (if not provided, lists available specs)
    ///
    /// Spec names are in format: YYYYMMDD_HHMMSS_feature_name
    /// If omitted, returns list of all available specs for the project
    /// Use 'foundry load-project PROJECT_NAME' to see project context first
    #[mcp(
        description = "Optional: specific spec to load (YYYYMMDD_HHMMSS_feature_name format). If omitted, lists available specs"
    )]
    pub spec_name: Option<String>,
}

/// Arguments for load_project command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "load_project",
    description = "Load complete project context (vision, tech-stack, summary) for LLM sessions. Essential for resuming work on existing projects."
)]
pub struct LoadProjectArgs {
    /// Project name to load context from (must exist in ~/.foundry/)
    ///
    /// Returns complete project context: vision, tech-stack, summary, and available specs
    /// Essential for resuming work on existing projects
    /// Use 'foundry list-projects' to see available project names
    #[mcp(description = "Name of the existing project to load (must exist in ~/.foundry/)")]
    pub project_name: String,
}

/// Arguments for list_projects command
#[derive(Args, Debug)]
pub struct ListProjectsArgs;

// Note: This command takes no arguments - it lists all projects in ~/.foundry/
// Returns: project names, creation dates, spec counts, validation status
// Use this to discover available projects before loading or creating specs

/// Arguments for get_foundry_help command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "get_foundry_help",
    description = "Get comprehensive workflow guidance, content examples, and usage patterns. Essential for understanding foundry workflows and content standards."
)]
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
    #[mcp(
        description = "Optional: specific help topic (workflows, content-examples, project-structure, parameter-guidance). If omitted, provides general guidance"
    )]
    pub topic: Option<String>,
}

/// Arguments for validate_content command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "validate_content",
    description = "Validate content against schema requirements with improvement suggestions. Helps ensure content meets foundry standards before creation."
)]
pub struct ValidateContentArgs {
    /// Content to validate
    ///
    /// The actual content string to check against schema requirements
    /// Validation includes: length, format, content quality, improvement suggestions
    /// Returns detailed feedback for content improvement
    #[arg(long, required = true)]
    #[mcp(description = "Content to validate against the specified type's requirements")]
    pub content: String,

    /// Content type to validate
    ///
    /// Must be one of: vision, tech-stack, summary, spec, notes, tasks
    /// Each type has specific length and quality requirements
    /// Use this to check content before creating projects/specs
    #[mcp(
        description = "Type of content to validate (vision, tech-stack, summary, spec, notes, tasks)"
    )]
    pub content_type: String,
}

// MCP parameter conversion implementations
// All structs now use auto-generated McpTool implementation via derive macro
// Except ListProjectsArgs which is a unit struct and needs manual implementation

impl crate::mcp::traits::McpToolDefinition for ListProjectsArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        rust_mcp_sdk::schema::Tool {
            name: "list_projects".to_string(),
            description: Some("List all available projects with metadata including creation dates, spec counts, and validation status.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec![],
                Some(std::collections::HashMap::new()),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(_params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self)
    }
}
