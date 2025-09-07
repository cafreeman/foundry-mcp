//! CLI argument structures

use crate::impl_mcp_tool;
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
    pub summary: String,
}

// Generate MCP tool implementation for CreateProjectArgs
impl_mcp_tool! {
    name = "create_project",
    description = "Create new project structure with LLM-provided content. Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md",
    struct CreateProjectArgs {
        project_name: String {
            description = "Descriptive project name using kebab-case (e.g., 'my-awesome-app')"
        },
        vision: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: High-level product vision (2-4 paragraphs, 200+ chars) that will serve as the COMPLETE implementation context for future LLMs who have NO prior knowledge of this project. Must include comprehensive problem definition, target users, unique value proposition, and key roadmap priorities. This document will be loaded as the PRIMARY reference for all future development work. Apply 'Cold Start Test': Could a skilled developer understand the project purpose using only this document? Use markdown with ## headers, bullet points, and clear structure. Include specific examples and architectural context. Goes into vision.md",
            min_length = 200
        },
        tech_stack: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: Comprehensive technology decisions (150+ chars) that will serve as the COMPLETE technical architecture guide for future LLMs with NO prior project knowledge. Must include languages, frameworks, databases, deployment platforms, and detailed rationale for each choice. This document will be the PRIMARY reference for all technical implementation decisions. Include integration patterns, dependencies, constraints, team standards, and architectural context. Future implementers must understand the complete technical landscape from this document alone. Use markdown with ## headers for categories, bullet points for technologies, and comprehensive explanations. Goes into tech-stack.md",
            min_length = 150
        },
        summary: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: Concise summary (100+ chars) of vision and tech-stack for quick context loading by future LLMs. This will be the FIRST document loaded to provide immediate project understanding for implementers with NO prior knowledge. Should capture essential project essence, main value proposition, and primary technology in 2-3 sentences using clear, professional language. Must enable rapid context acquisition for future development sessions. Goes into summary.md",
            min_length = 100
        }
    }
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
    pub tech_stack: String,

    /// Concise summary (100+ characters) combining vision and tech stack from your analysis
    ///
    /// Should capture key insights from your codebase exploration for quick context loading
    /// Use this to understand the project essence before diving into implementation
    /// Goes into summary.md
    #[arg(long, required = true)]
    pub summary: String,
}

// Generate MCP tool implementation for AnalyzeProjectArgs
impl_mcp_tool! {
    name = "analyze_project",
    description = "Create project structure by analyzing existing codebase. You analyze codebase and provide vision, tech-stack, and summary content as arguments.",
    struct AnalyzeProjectArgs {
        project_name: String {
            description = "Descriptive project name using kebab-case (e.g., 'my-analyzed-project')"
        },
        vision: String {
            description = "Your analyzed product vision (200+ chars) based on codebase examination. Use Search/Grep/Read tools first. Structure with ## headers, bullet points, and specific examples from code. Cover problem solved, target users, and value proposition derived from actual functionality. Goes into vision.md",
            min_length = 200
        },
        tech_stack: String {
            description = "Your detected technology stack (150+ chars) based on codebase analysis. Examine package files, configs, and code patterns. Structure with ## headers for categories, list technologies with versions, include rationale from observed patterns. Reference specific files discovered. Goes into tech-stack.md",
            min_length = 150
        },
        summary: String {
            description = "Your created concise summary (100+ chars) of analyzed project combining vision and tech-stack insights for quick context loading. Goes into summary.md",
            min_length = 100
        }
    }
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
    pub tasks: String,
}

// Generate MCP tool implementation for CreateSpecArgs
impl_mcp_tool! {
    name = "create_spec",
    description = "Create timestamped specification for a feature. Creates YYYYMMDD_HHMMSS_FEATURE_NAME directory with spec.md, task-list.md, and notes.md. You provide complete specification content as arguments.",
    struct CreateSpecArgs {
        project_name: String {
            description = "Name of the existing project to create spec for"
        },
        feature_name: String {
            description = "Descriptive feature name using snake_case (e.g., 'user_authentication')"
        },
        spec: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: Detailed feature specification (200+ chars) that will serve as the COMPLETE implementation guide for future LLMs who have NO prior knowledge of this feature. Must include comprehensive requirements, architectural context, implementation approach with component interactions, dependencies, edge cases, and everything needed for successful implementation. This document will be the PRIMARY reference for feature development. Apply 'Cold Start Test': Could a skilled developer implement this feature using only this document? Use # for feature name, ## for major sections (Overview, Requirements, Implementation, Testing). Include code blocks, bullet points, tables, and detailed technical context. Goes into spec.md",
            min_length = 200
        },
        notes: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: Additional context and design decisions (50+ chars) that will provide COMPLETE implementation context for future LLMs with NO prior feature knowledge. Must include comprehensive design rationale, architectural tradeoffs, dependency analysis, implementation constraints, and future opportunities. This document will be loaded alongside the spec to provide full context for implementation decisions. Include business context, technical constraints, and decision history that future implementers need to understand. Use ## headers for categories, bullet points for considerations. Keep technical but conversational. Goes into notes.md",
            min_length = 50
        },
        tasks: String {
            description = "**CONTEXT FOR FUTURE IMPLEMENTATION**: Markdown checklist (100+ chars) of implementation steps that will guide future LLMs through COMPLETE feature implementation with NO prior knowledge. Must include comprehensive, actionable phases covering setup, development, testing, and deployment. This task list will be the PRIMARY implementation roadmap for future development sessions. Break feature into specific, measurable tasks that provide complete implementation guidance. Use ## headers for phases, - [ ] for uncompleted tasks, - [x] for completed. Include dependencies, prerequisites, and validation steps. Goes into task-list.md",
            min_length = 100
        }
    }
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

// Manual MCP tool implementation for LoadSpecArgs (has optional field)
impl crate::mcp::traits::McpToolDefinition for LoadSpecArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        let mut project_name_prop = serde_json::Map::new();
        project_name_prop.insert("type".to_string(), serde_json::json!("string"));
        project_name_prop.insert(
            "description".to_string(),
            serde_json::json!("Name of the project containing the spec"),
        );
        properties.insert("project_name".to_string(), project_name_prop);

        let mut spec_name_prop = serde_json::Map::new();
        spec_name_prop.insert("type".to_string(), serde_json::json!("string"));
        spec_name_prop.insert("description".to_string(), serde_json::json!("Optional: specific spec to load (YYYYMMDD_HHMMSS_feature_name format). If omitted, lists available specs"));
        properties.insert("spec_name".to_string(), spec_name_prop);

        rust_mcp_sdk::schema::Tool {
            name: "load_spec".to_string(),
            description: Some("Load specific specification content with project context. You can use this to review full specification details, task lists, and implementation notes. If spec_name is omitted, lists available specs.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec!["project_name".to_string()], // Only project_name is required
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            spec_name: params["spec_name"].as_str().map(|s| s.to_string()),
        })
    }
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

// Generate MCP tool implementation for LoadProjectArgs
impl_mcp_tool! {
    name = "load_project",
    description = "Load complete project context (vision, tech-stack, summary) for LLM sessions. Essential for resuming work on existing projects. You can use this to get full project context before creating specifications or continuing development work.",
    struct LoadProjectArgs {
        project_name: String {
            description = "Name of the existing project to load (must exist in ~/.foundry/)"
        }
    }
}

/// Arguments for list_projects command
#[derive(Args, Debug)]
pub struct ListProjectsArgs;

// Note: This command takes no arguments - it lists all projects in ~/.foundry/
// Returns: project names, creation dates, spec counts, validation status
// Use this to discover available projects before loading or creating specs

/// Arguments for list_specs command
#[derive(Args, Debug)]
pub struct ListSpecsArgs {
    /// Project name to list specs for
    ///
    /// Must be an existing project in ~/.foundry/
    /// Typically matches repository name - try this first before listing projects
    pub project_name: String,
}

// Generate MCP tool implementation for ListSpecsArgs
impl_mcp_tool! {
    name = "list_specs",
    description = "List available specifications for a project without loading full context. Returns lightweight spec metadata including names, feature names, and creation dates for efficient spec discovery.",
    struct ListSpecsArgs {
        project_name: String {
            description = "Name of the existing project to list specs for (must exist in ~/.foundry/)"
        }
    }
}

/// Arguments for get_foundry_help command
#[derive(Args, Debug)]
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
    pub topic: Option<String>,
}

// Manual MCP tool implementation for GetFoundryHelpArgs (has optional field)
impl crate::mcp::traits::McpToolDefinition for GetFoundryHelpArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        let mut topic_prop = serde_json::Map::new();
        topic_prop.insert("type".to_string(), serde_json::json!("string"));
        topic_prop.insert("description".to_string(), serde_json::json!("Optional: specific help topic (workflows, decision-points, content-examples, project-structure, parameter-guidance, tool-capabilities). If omitted, provides general guidance"));
        properties.insert("topic".to_string(), topic_prop);

        rust_mcp_sdk::schema::Tool {
            name: "get_foundry_help".to_string(),
            description: Some("Get comprehensive workflow guidance, content examples, and usage patterns. You can use this to understand foundry workflows and content standards. Essential for effective tool selection and workflow optimization.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec![], // No required fields
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            topic: params["topic"].as_str().map(|s| s.to_string()),
        })
    }
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

// Generate MCP tool implementation for ValidateContentArgs
impl_mcp_tool! {
    name = "validate_content",
    description = "Validate content against schema requirements with improvement suggestions. You can use this to ensure your content meets foundry standards before creating projects or specifications. Provides detailed feedback for content improvement.",
    struct ValidateContentArgs {
        content: String {
            description = "Content to validate against the specified type's requirements"
        },
        content_type: String {
            description = "Type of content to validate (vision, tech-stack, summary, spec, notes, tasks)"
        }
    }
}

/// Arguments for update_spec command
#[derive(Args, Debug)]
pub struct UpdateSpecArgs {
    /// Project name containing the spec to update
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    pub project_name: String,

    /// Spec name to update (YYYYMMDD_HHMMSS_feature_name format)
    ///
    /// Must be an existing spec within the project
    /// Use 'foundry load-project PROJECT_NAME' to see available specs
    pub spec_name: String,

    /// New content for spec.md (optional)
    ///
    /// **Complete Content Replacement (--operation replace)**:
    /// - Entirely replaces the existing spec.md content
    /// - Use for major requirement changes or complete rewrites
    /// - Existing content is lost - ensure you have backups if needed
    ///
    /// **Content Addition (--operation append)**:
    /// - Adds new content to the end of existing spec.md
    /// - Preserves existing requirements and specifications
    /// - Use for iterative spec development and additions
    ///
    /// **Markdown Formatting Guidelines:**
    /// - Use # Feature Name as main header
    /// - Use ## for major sections (## Overview, ## Requirements, ## Implementation)
    /// - Include functional requirements, acceptance criteria, technical approach
    /// - Use bullet points, numbered lists, and code blocks as needed
    #[arg(long)]
    pub spec: Option<String>,

    /// New content for task-list.md (optional)
    ///
    /// **Complete Content Replacement (--operation replace)**:
    /// - Entirely replaces the existing task-list.md content
    /// - Use when completely restructuring the implementation plan
    /// - Existing task history is lost
    ///
    /// **Content Addition (--operation append)**:
    /// - Adds new tasks to the end of existing task-list.md
    /// - Preserves existing task history and completion status
    /// - Use for adding new tasks or marking existing tasks as complete
    ///
    /// **Markdown Checklist Format:**
    /// - Use "## Phase Name" headers to group related tasks
    /// - Use "- [ ] Task description" for uncompleted tasks
    /// - Use "- [x] Task description" for completed tasks
    /// - Include implementation details and dependencies
    #[arg(long)]
    pub tasks: Option<String>,

    /// New content for notes.md (optional)
    ///
    /// **Complete Content Replacement (--operation replace)**:
    /// - Entirely replaces the existing notes.md content
    /// - Use when consolidating or restructuring design decisions
    /// - Existing notes and rationale are lost
    ///
    /// **Content Addition (--operation append)**:
    /// - Adds new notes to the end of existing notes.md
    /// - Preserves existing design decisions and implementation notes
    /// - Use for adding new insights, decisions, or implementation details
    ///
    /// **Markdown Formatting Guidelines:**
    /// - Use ## headers for different categories (## Design Decisions, ## Implementation Notes)
    /// - Document technical tradeoffs, constraints, and rationale
    /// - Include code snippets, external references, and future considerations
    /// - Keep notes conversational but technical
    #[arg(long)]
    pub notes: Option<String>,

    /// Content replacement strategy (REQUIRED)
    ///
    /// **replace**: Completely replaces the target file content with new content
    /// - Use when: Completely rewriting content, major changes, starting fresh
    /// - Risk: Existing content is lost permanently
    /// - Example: Major requirement changes, technical direction changes
    ///
    /// **append**: Adds new content to the end of existing file content
    /// - Use when: Adding new content, iterative development, preserving history
    /// - Risk: Low - existing content is preserved
    /// - Example: Adding new tasks, accumulating notes, marking items complete
    ///
    /// Applies to ALL files being updated in this command.
    #[arg(long, required = true)]
    pub operation: String,

    /// Context-based patch data (JSON string, optional)
    ///
    /// **Context-Based Patching (--operation context_patch)**:
    /// - Enables precise, targeted updates using surrounding text context
    /// - Avoids need for line number precision or full file replacement
    /// - Requires exact text matching with current content for reliability
    /// - Choose unique, specific context lines (avoid generic words like "TODO")
    /// - Example: {"file_type":"tasks","operation":"replace","section_context":"## Phase 1: Authentication","before_context":["## Phase 1: Authentication","- [ ] Implement OAuth2 integration with Google"],"after_context":["- [ ] Add password strength validation"],"content":"- [x] Implement OAuth2 integration with Google"}
    ///
    /// **Prerequisites for Success**:
    /// - Ensure you have current content in context (reload with load_spec if you've made previous edits)
    /// - Use exact text from current content for context selection
    /// - Choose 3-5 lines of unique, distinctive text unlikely to appear elsewhere
    /// - Use section_context for disambiguation when context appears in multiple sections
    ///
    /// **When to use context_patch**:
    /// - Mark tasks complete, add single requirements, fix specific content
    /// - Small targeted changes where you know unique surrounding context
    /// - Updates that need precise positioning within existing structure
    ///
    /// **Recovery when context matching fails**:
    /// - Reload current content to verify exact text
    /// - Choose more specific, unique context lines
    /// - Add section_context to disambiguate
    /// - As last resort, use replace for major changes or append for additions
    ///
    /// **JSON Schema Requirements**:
    /// - file_type: "spec", "tasks", or "notes"
    /// - operation: "insert", "replace", or "delete"
    /// - before_context: Array of strings (unique text from current content)
    /// - after_context: Array of strings (unique text from current content)
    /// - content: String content to insert/replace
    /// - section_context: Optional header for disambiguation (e.g., "## Requirements")
    #[arg(long)]
    pub context_patch: Option<String>,
}

// Manual MCP tool implementation for UpdateSpecArgs (has optional fields)
impl crate::mcp::traits::McpToolDefinition for UpdateSpecArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        let mut properties = std::collections::HashMap::new();

        let mut project_name_prop = serde_json::Map::new();
        project_name_prop.insert("type".to_string(), serde_json::json!("string"));
        project_name_prop.insert(
            "description".to_string(),
            serde_json::json!("Name of the existing project containing the spec"),
        );
        properties.insert("project_name".to_string(), project_name_prop);

        let mut spec_name_prop = serde_json::Map::new();
        spec_name_prop.insert("type".to_string(), serde_json::json!("string"));
        spec_name_prop.insert(
            "description".to_string(),
            serde_json::json!(
                "Name of the existing spec to update (YYYYMMDD_HHMMSS_feature_name format)"
            ),
        );
        properties.insert("spec_name".to_string(), spec_name_prop);

        let mut spec_prop = serde_json::Map::new();
        spec_prop.insert("type".to_string(), serde_json::json!("string"));
        spec_prop.insert("description".to_string(), serde_json::json!("New content for spec.md (optional). Use with --operation replace to completely rewrite the spec, or --operation append to add new requirements while preserving existing content. Include functional requirements, acceptance criteria, and implementation approach."));
        properties.insert("spec".to_string(), spec_prop);

        let mut tasks_prop = serde_json::Map::new();
        tasks_prop.insert("type".to_string(), serde_json::json!("string"));
        tasks_prop.insert("description".to_string(), serde_json::json!("New content for task-list.md (optional). Use with --operation append to add new tasks or mark existing tasks complete while preserving history. Use --operation replace to completely restructure the implementation plan."));
        properties.insert("tasks".to_string(), tasks_prop);

        let mut notes_prop = serde_json::Map::new();
        notes_prop.insert("type".to_string(), serde_json::json!("string"));
        notes_prop.insert("description".to_string(), serde_json::json!("New content for notes.md (optional). Use with --operation append to accumulate design decisions and implementation notes over time. Use --operation replace to restructure or consolidate notes."));
        properties.insert("notes".to_string(), notes_prop);

        let mut operation_prop = serde_json::Map::new();
        operation_prop.insert("type".to_string(), serde_json::json!("string"));
        operation_prop.insert("description".to_string(), serde_json::json!("REQUIRED: Content replacement strategy - 'replace' (completely overwrite), 'append' (add to existing content), or 'context_patch' (targeted updates using surrounding text context). Applies to all files being updated. Use 'replace' for major changes, 'append' for iterative development, 'context_patch' for precise targeted updates."));
        properties.insert("operation".to_string(), operation_prop);

        let mut context_patch_prop = serde_json::Map::new();
        context_patch_prop.insert("type".to_string(), serde_json::json!("string"));
        context_patch_prop.insert("description".to_string(), serde_json::json!("Context-based patch data (JSON string, optional). Used with --operation context_patch for precise, targeted updates using surrounding text context. Requires exact text from current content for reliable matching. Choose unique, specific context lines. Example: {\"file_type\":\"tasks\",\"operation\":\"replace\",\"section_context\":\"## Phase 1: Authentication\",\"before_context\":[\"- [ ] Implement OAuth2 integration\"],\"after_context\":[\"- [ ] Add password validation\"],\"content\":\"- [x] Implement OAuth2 integration\"}. Recovery: If matching fails, reload content and use more specific context."));
        properties.insert("context_patch".to_string(), context_patch_prop);

        rust_mcp_sdk::schema::Tool {
            name: "update_spec".to_string(),
            description: Some("Update multiple spec files in a single operation with explicit control over content replacement strategy. You can update spec.md, task-list.md, and/or notes.md with replace or append operations. Essential for iterative development and progress tracking.".to_string()),
            title: None,
            input_schema: rust_mcp_sdk::schema::ToolInputSchema::new(
                vec!["project_name".to_string(), "spec_name".to_string(), "operation".to_string()], // Required fields
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    fn from_mcp_params(params: &serde_json::Value) -> anyhow::Result<Self> {
        Ok(Self {
            project_name: params["project_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing project_name parameter"))?
                .to_string(),
            spec_name: params["spec_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing spec_name parameter"))?
                .to_string(),
            spec: params["spec"].as_str().map(|s| s.to_string()),
            tasks: params["tasks"].as_str().map(|s| s.to_string()),
            notes: params["notes"].as_str().map(|s| s.to_string()),
            operation: params["operation"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing operation parameter"))?
                .to_string(),
            context_patch: params["context_patch"].as_str().map(|s| s.to_string()),
        })
    }
}

/// Arguments for delete_spec command
#[derive(Args, Debug)]
pub struct DeleteSpecArgs {
    /// Project name containing the spec to delete
    ///
    /// Must be an existing project in ~/.foundry/
    /// Use 'foundry list-projects' to see available projects
    pub project_name: String,

    /// Spec name to delete (YYYYMMDD_HHMMSS_feature_name format)
    ///
    /// Must be an existing spec within the project
    /// Use 'foundry load-project PROJECT_NAME' to see available specs
    /// **Warning: This will permanently delete all spec files**
    pub spec_name: String,

    /// Confirmation flag - must be "true" to proceed
    ///
    /// This is a safety mechanism to prevent accidental deletions
    /// Must explicitly set to "true": --confirm true
    #[arg(long, required = true)]
    pub confirm: String,
}

// Generate MCP tool implementation for DeleteSpecArgs
impl_mcp_tool! {
    name = "delete_spec",
    description = "Delete an existing specification and all its files (spec.md, task-list.md, notes.md). You can use this to permanently remove specifications that are no longer needed. This action cannot be undone.",
    struct DeleteSpecArgs {
        project_name: String {
            description = "Name of the existing project containing the spec"
        },
        spec_name: String {
            description = "Name of the spec to delete (YYYYMMDD_HHMMSS_feature_name format). WARNING: This permanently deletes all spec files."
        },
        confirm: String {
            description = "Confirmation flag - must be set to 'true' to proceed with deletion (safety mechanism)"
        }
    }
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
    /// - claude-code: Install for Claude Code CLI environment (includes subagent template)
    /// - cursor: Install for Cursor IDE environment (includes rules template)
    ///
    /// Installation creates both MCP server configuration and AI assistant guidance templates.
    pub target: String,

    /// Custom path to foundry binary (optional)
    ///
    /// If not provided, will attempt to detect the current binary path
    /// Useful for installations where the binary is in a custom location
    #[arg(long)]
    pub binary_path: Option<String>,

    /// Output installation information in JSON format
    ///
    /// When enabled, outputs structured JSON data instead of
    /// human-readable formatted text with colors and status indicators
    #[arg(long)]
    pub json: bool,
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

    /// Output uninstallation information in JSON format
    ///
    /// When enabled, outputs structured JSON data instead of
    /// human-readable formatted text with colors and status indicators
    #[arg(long)]
    pub json: bool,
}

/// Arguments for status command
#[derive(Args, Debug, Clone)]
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

    /// Output status information in JSON format
    ///
    /// When enabled, outputs structured JSON data instead of
    /// human-readable formatted text with colors
    #[arg(long)]
    pub json: bool,
}

// MCP parameter conversion implementations
// All structs now use auto-generated McpTool implementation via derive macro
// Except ListProjectsArgs which is a unit struct and needs manual implementation

impl crate::mcp::traits::McpToolDefinition for ListProjectsArgs {
    fn tool_definition() -> rust_mcp_sdk::schema::Tool {
        rust_mcp_sdk::schema::Tool {
            name: "list_projects".to_string(),
            description: Some("List all available projects with metadata including creation dates, spec counts, and validation status. You can use this to discover available projects before loading or creating specifications.".to_string()),
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
