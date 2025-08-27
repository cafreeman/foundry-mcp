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
    /// **Content Structure:**
    /// - Problem statement and market need
    /// - Target users and their pain points
    /// - Unique value proposition and competitive advantages
    /// - Key features and roadmap priorities
    ///
    /// **Markdown Formatting Guidelines:**
    /// - Use ## headers for main sections (## Problem, ## Target Users, ## Value Proposition)
    /// - Structure with bullet points and clear paragraphs
    /// - Include specific examples and use cases
    /// - Write in present tense, professional tone
    /// - End with clear success metrics or goals
    ///
    /// Goes into vision.md
    #[arg(long, required = true)]
    #[mcp(
        description = "High-level product vision (2-4 paragraphs, 200+ chars) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Use markdown with ## headers, bullet points, and clear structure. Include specific examples. Goes into vision.md",
        min_length = 200
    )]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters)
    ///
    /// **Content Structure:**
    /// - Core languages and frameworks with versions
    /// - Databases, storage, and data architecture
    /// - Deployment platforms and infrastructure
    /// - Development tools and build systems
    /// - Rationale for each major technology choice
    ///
    /// **Markdown Formatting Guidelines:**
    /// - Use ## headers for categories (## Backend, ## Frontend, ## Database, ## Deployment)
    /// - List technologies with bullet points and brief rationale
    /// - Include version constraints where relevant
    /// - Add ### subsections for complex areas
    /// - Include links to documentation where helpful
    ///
    /// Goes into tech-stack.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Comprehensive technology decisions (150+ chars) including languages, frameworks, databases, deployment platforms, and rationale. Use markdown with ## headers for categories, bullet points for technologies, and brief explanations. Include constraints, preferences, or team standards. Goes into tech-stack.md",
        min_length = 150
    )]
    pub tech_stack: String,

    /// Concise summary of vision and tech stack (100+ characters)
    ///
    /// **Content Guidelines:**
    /// - 2-3 sentences capturing project essence
    /// - Combine key points from vision and tech stack
    /// - Focus on what makes this project unique
    /// - Written for quick LLM context loading
    ///
    /// **Format:**
    /// - Clear, professional language
    /// - Present tense, active voice
    /// - Include primary technology and main value proposition
    /// - No markdown formatting needed (plain text)
    ///
    /// Goes into summary.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Concise summary (100+ chars) of vision and tech-stack for quick context loading. Should capture essential project essence in 2-3 sentences using clear, professional language. Combine main value proposition with primary technology. Goes into summary.md",
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
    /// **Analysis Approach:**
    /// - Use Search, Grep, and Read tools to explore the codebase first
    /// - Examine package.json, README files, main entry points, API routes
    /// - Look for user-facing features, business logic, and data models
    /// - Review existing documentation and configuration files
    ///
    /// **Content Structure & Markdown:**
    /// - Use ## headers (## Problem Analysis, ## Target Users, ## Product Goals)
    /// - Base vision on actual code functionality discovered
    /// - Include specific examples from the codebase
    /// - Structure with bullet points and clear paragraphs
    /// - Write in present tense, referencing actual implementation
    ///
    /// Goes into vision.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-analyzed product vision (200+ chars) based on codebase examination. Use Search/Grep/Read tools first. Structure with ## headers, bullet points, and specific examples from code. Cover problem solved, target users, and value proposition derived from actual functionality. Goes into vision.md",
        min_length = 200
    )]
    pub vision: String,

    /// Technology stack and architecture decisions (150+ characters) based on your codebase exploration
    ///
    /// **Detection Strategy:**
    /// - Analyze package.json, requirements.txt, Cargo.toml, etc. for dependencies
    /// - Check build scripts, Docker files, and deployment configurations
    /// - Examine database connections, API integrations, and external services
    /// - Review folder structure and architectural patterns used
    ///
    /// **Content Structure & Markdown:**
    /// - Use ## headers (## Languages, ## Frameworks, ## Database, ## Deployment, ## Build Tools)
    /// - List detected technologies with versions where found
    /// - Include rationale based on code patterns observed
    /// - Add ### subsections for complex architectural decisions
    /// - Reference specific files or configurations discovered
    ///
    /// Goes into tech-stack.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-detected technology stack (150+ chars) based on codebase analysis. Examine package files, configs, and code patterns. Structure with ## headers for categories, list technologies with versions, include rationale from observed patterns. Reference specific files discovered. Goes into tech-stack.md",
        min_length = 150
    )]
    pub tech_stack: String,

    /// Concise summary (100+ characters) combining vision and tech stack from your analysis
    ///
    /// Should capture key insights from your codebase exploration for quick context loading
    /// Use this to understand the project essence before diving into implementation
    /// Goes into summary.md
    #[arg(long, required = true)]
    #[mcp(
        description = "LLM-created concise summary (100+ chars) of analyzed project combining vision and tech-stack insights for quick context loading. Goes into summary.md",
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
    /// **Required Sections:**
    /// - Feature overview and purpose
    /// - Functional requirements and acceptance criteria
    /// - Technical implementation approach
    /// - Testing strategy and edge cases
    /// - Dependencies and constraints
    ///
    /// **Markdown Structure:**
    /// - Use # Feature Name as main header
    /// - Use ## for major sections (## Overview, ## Requirements, ## Implementation)
    /// - Use ### for subsections (### API Design, ### Database Changes)
    /// - Include code blocks with ```language for examples
    /// - Use bullet points and numbered lists for clarity
    /// - Add tables for complex requirements or APIs
    ///
    /// Goes into spec.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Detailed feature specification (200+ chars) with comprehensive markdown structure. Use # for feature name, ## for major sections (Overview, Requirements, Implementation, Testing). Include code blocks, bullet points, and tables. Cover functional requirements, acceptance criteria, technical approach, and constraints. Goes into spec.md",
        min_length = 200
    )]
    pub spec: String,

    /// Implementation notes and considerations
    ///
    /// **Content Focus:**
    /// - Design decisions and rationale
    /// - Technical tradeoffs and alternatives considered
    /// - Dependencies on other features or systems
    /// - Implementation constraints and limitations
    /// - Future enhancement opportunities
    ///
    /// **Markdown Structure:**
    /// - Use ## headers for categories (## Design Decisions, ## Dependencies, ## Constraints)
    /// - Use bullet points for lists of considerations
    /// - Include code snippets for technical details
    /// - Reference external documentation with links
    /// - Keep it conversational but technical
    ///
    /// Goes into notes.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Additional context and design decisions (50+ chars) for feature implementation. Use ## headers for categories, bullet points for considerations. Include design rationale, tradeoffs, dependencies, constraints, and future opportunities. Keep technical but conversational. Goes into notes.md",
        min_length = 50
    )]
    pub notes: String,

    /// Task list content
    ///
    /// **Task Organization:**
    /// - Break feature into actionable, testable tasks
    /// - Group related tasks under logical phases or components
    /// - Include both implementation and validation tasks
    /// - Consider setup, development, testing, and deployment phases
    ///
    /// **Markdown Checklist Format:**
    /// - Use ## headers for phases (## Phase 1: Setup, ## Phase 2: Core Implementation)
    /// - Use `- [ ] Task description` for uncompleted tasks
    /// - Use `- [x] Task description` for completed tasks
    /// - Include estimated effort or complexity where helpful
    /// - Add sub-tasks with indented checkboxes when needed
    /// - Keep tasks specific and measurable
    ///
    /// Goes into task-list.md
    #[arg(long, required = true)]
    #[mcp(
        description = "Markdown checklist (100+ chars) of implementation steps organized by phases. Use ## headers for phases, - [ ] for uncompleted tasks, - [x] for completed. Break feature into actionable, testable tasks including setup, development, testing, and deployment. Keep tasks specific and measurable. Goes into task-list.md",
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
    /// - workflows: User-driven development patterns (not automated sequences)
    /// - decision-points: Guidance for choosing appropriate tools based on user intent
    /// - content-examples: Content templates and examples
    /// - project-structure: File organization and structure
    /// - parameter-guidance: Parameter requirements and best practices
    /// - tool-capabilities: When each tool is appropriate and what user input is required
    ///
    /// If omitted, provides overview and available topics
    #[mcp(
        description = "Optional: specific help topic (workflows, decision-points, content-examples, project-structure, parameter-guidance, tool-capabilities). If omitted, provides general guidance"
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

/// Arguments for update_spec command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "update_spec",
    description = "Update existing specification files (spec.md, task-list.md, or notes.md) with new content. Supports both replace and append operations for iterative development."
)]
pub struct UpdateSpecArgs {
    /// Project name containing the spec to update
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    #[mcp(description = "Name of the existing project containing the spec")]
    pub project_name: String,

    /// Spec name to update (YYYYMMDD_HHMMSS_feature_name format)
    ///
    /// Must be an existing spec within the project
    /// Use 'foundry load-project PROJECT_NAME' to see available specs
    #[mcp(
        description = "Name of the existing spec to update (YYYYMMDD_HHMMSS_feature_name format)"
    )]
    pub spec_name: String,

    /// File type to update within the spec
    ///
    /// Must be one of:
    /// - "spec" - Updates spec.md (main feature specification)
    /// - "task-list" or "tasks" - Updates task-list.md (implementation checklist)
    /// - "notes" - Updates notes.md (additional context and design decisions)
    #[mcp(
        description = "File type to update: 'spec' (spec.md), 'task-list' or 'tasks' (task-list.md), or 'notes' (notes.md)"
    )]
    pub file_type: String,

    /// Operation type: 'replace' or 'append'
    ///
    /// - "replace" - Completely replaces the file content
    /// - "append" - Adds new content to the end of the existing file
    ///
    /// Use 'append' for updating task lists, adding notes, or extending specifications
    /// Use 'replace' for complete rewrites or major restructuring
    #[mcp(
        description = "Operation type: 'replace' (overwrite content) or 'append' (add to existing content)"
    )]
    pub operation: String,

    /// New content to write or append
    ///
    /// **Markdown Formatting Guidelines:**
    /// - Use proper headers (## Features, ### Implementation Details)
    /// - Structure with lists, code blocks, and clear sections
    /// - For task-lists: Use "- [ ] Task description" format for checkboxes
    /// - For specs: Include Requirements, Acceptance Criteria, Implementation Approach
    /// - For notes: Use bullet points, decision rationale, and cross-references
    ///
    /// **Content should be:**
    /// - Well-structured with clear hierarchy
    /// - Comprehensive yet focused
    /// - Technical but accessible
    /// - Include examples where helpful
    #[arg(long, required = true)]
    #[mcp(
        description = "Content to write or append. Use markdown formatting with proper headers, lists, and structure. For task-lists use '- [ ]' checkbox format. Include comprehensive details with examples.",
        min_length = 20
    )]
    pub content: String,
}

/// Arguments for delete_spec command
#[derive(Args, Debug, McpTool)]
#[mcp(
    name = "delete_spec",
    description = "Delete an existing specification and all its files (spec.md, task-list.md, notes.md). This action cannot be undone."
)]
pub struct DeleteSpecArgs {
    /// Project name containing the spec to delete
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    #[mcp(description = "Name of the existing project containing the spec")]
    pub project_name: String,

    /// Spec name to delete (YYYYMMDD_HHMMSS_feature_name format)
    ///
    /// Must be an existing spec within the project
    /// Use 'foundry load-project PROJECT_NAME' to see available specs
    /// **Warning: This will permanently delete all spec files**
    #[mcp(
        description = "Name of the spec to delete (YYYYMMDD_HHMMSS_feature_name format). WARNING: This permanently deletes all spec files."
    )]
    pub spec_name: String,

    /// Confirmation flag - must be "true" to proceed
    ///
    /// This is a safety mechanism to prevent accidental deletions
    /// Must explicitly set to "true": --confirm true
    #[arg(long, required = true)]
    #[mcp(
        description = "Confirmation flag - must be set to 'true' to proceed with deletion (safety mechanism)"
    )]
    pub confirm: String,
}

/// Arguments for serve command
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Enable verbose logging
    #[arg(long, short)]
    pub verbose: bool,
}

/// Arguments for install command
#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Target environment to install for
    ///
    /// Supported targets:
    /// - claude-code: Install for Claude Code CLI environment
    /// - cursor: Install for Cursor IDE environment
    pub target: String,

    /// Custom path to foundry binary (optional)
    ///
    /// If not provided, will attempt to detect the current binary path
    /// Useful for installations where the binary is in a custom location
    #[arg(long)]
    pub binary_path: Option<String>,

    /// Force overwrite existing configuration
    ///
    /// If set, will overwrite existing MCP server configuration
    /// without prompting for confirmation
    #[arg(long)]
    pub force: bool,
}

/// Arguments for uninstall command
#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Target environment to uninstall from
    ///
    /// Supported targets:
    /// - claude-code: Uninstall from Claude Code CLI environment
    /// - cursor: Uninstall from Cursor IDE environment
    pub target: String,

    /// Also remove configuration files
    ///
    /// If set, will remove configuration files in addition to
    /// unregistering the MCP server
    #[arg(long)]
    pub remove_config: bool,

    /// Force uninstall without confirmation prompts
    #[arg(long)]
    pub force: bool,
}

/// Arguments for status command
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show detailed configuration information
    ///
    /// Includes full paths, configuration file contents,
    /// and detailed installation status
    #[arg(long)]
    pub detailed: bool,

    /// Check only specific target environment
    ///
    /// If provided, only show status for the specified environment
    /// instead of all supported environments
    #[arg(long)]
    pub target: Option<String>,
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

// Manual implementations for installation commands (McpTool derive not compatible with bool fields)
impl crate::mcp::traits::McpToolDefinition for InstallArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        // target property
        let mut target_prop = serde_json::Map::new();
        target_prop.insert("type".to_string(), serde_json::json!("string"));
        target_prop.insert(
            "description".to_string(),
            serde_json::json!("Target environment: 'claude-code' or 'cursor'"),
        );
        properties.insert("target".to_string(), target_prop);

        // binary_path property (optional)
        let mut binary_path_prop = serde_json::Map::new();
        binary_path_prop.insert("type".to_string(), serde_json::json!("string"));
        binary_path_prop.insert(
            "description".to_string(),
            serde_json::json!(
                "Custom path to foundry binary (optional, auto-detected if not provided)"
            ),
        );
        properties.insert("binary_path".to_string(), binary_path_prop);

        // force property
        let mut force_prop = serde_json::Map::new();
        force_prop.insert("type".to_string(), serde_json::json!("boolean"));
        force_prop.insert(
            "description".to_string(),
            serde_json::json!("Force overwrite existing configuration without prompting"),
        );
        properties.insert("force".to_string(), force_prop);

        rust_mcp_sdk::schema::Tool {
            name: "install".to_string(),
            description: Some("Install Foundry MCP server for AI development environments. Creates necessary configuration files and registers the MCP server.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec!["target".to_string()], // target is required
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        let target = params["target"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?
            .to_string();

        let binary_path = params["binary_path"].as_str().map(|s| s.to_string());

        let force = params["force"].as_bool().unwrap_or(false);

        Ok(Self {
            target,
            binary_path,
            force,
        })
    }
}

impl crate::mcp::traits::McpToolDefinition for UninstallArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        // target property
        let mut target_prop = serde_json::Map::new();
        target_prop.insert("type".to_string(), serde_json::json!("string"));
        target_prop.insert(
            "description".to_string(),
            serde_json::json!("Target environment: 'claude-code' or 'cursor'"),
        );
        properties.insert("target".to_string(), target_prop);

        // remove_config property
        let mut remove_config_prop = serde_json::Map::new();
        remove_config_prop.insert("type".to_string(), serde_json::json!("boolean"));
        remove_config_prop.insert(
            "description".to_string(),
            serde_json::json!("Also remove configuration files (not just unregister)"),
        );
        properties.insert("remove_config".to_string(), remove_config_prop);

        // force property
        let mut force_prop = serde_json::Map::new();
        force_prop.insert("type".to_string(), serde_json::json!("boolean"));
        force_prop.insert(
            "description".to_string(),
            serde_json::json!("Force uninstall without confirmation prompts"),
        );
        properties.insert("force".to_string(), force_prop);

        rust_mcp_sdk::schema::Tool {
            name: "uninstall".to_string(),
            description: Some("Uninstall Foundry MCP server from AI development environments. Removes MCP server configuration and optionally cleans up files.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec!["target".to_string()], // target is required
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        let target = params["target"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?
            .to_string();

        let remove_config = params["remove_config"].as_bool().unwrap_or(false);

        let force = params["force"].as_bool().unwrap_or(false);

        Ok(Self {
            target,
            remove_config,
            force,
        })
    }
}

impl crate::mcp::traits::McpToolDefinition for StatusArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        // detailed property
        let mut detailed_prop = serde_json::Map::new();
        detailed_prop.insert("type".to_string(), serde_json::json!("boolean"));
        detailed_prop.insert(
            "description".to_string(),
            serde_json::json!("Show detailed configuration information and file contents"),
        );
        properties.insert("detailed".to_string(), detailed_prop);

        // target property (optional)
        let mut target_prop = serde_json::Map::new();
        target_prop.insert("type".to_string(), serde_json::json!("string"));
        target_prop.insert(
            "description".to_string(),
            serde_json::json!("Check only specific target: 'claude-code' or 'cursor'"),
        );
        properties.insert("target".to_string(), target_prop);

        rust_mcp_sdk::schema::Tool {
            name: "status".to_string(),
            description: Some("Show MCP server installation status across all supported AI development environments.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec![], // no required parameters
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        let detailed = params["detailed"].as_bool().unwrap_or(false);

        let target = params["target"].as_str().map(|s| s.to_string());

        Ok(Self { detailed, target })
    }
}
