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
    /// Project name to create with your analyzed content
    pub project_name: String,

    /// High-level product vision content (2-4 paragraphs) based on your codebase analysis.
    /// Use codebase_search, grep_search, and read_file to understand the project first.
    /// Consider: existing code patterns, project scope, apparent goals, user-facing features.
    /// Goes into project/vision.md
    #[arg(long, required = true)]
    pub vision: String,

    /// Technology stack and architecture decisions based on your codebase exploration.
    /// Use your analysis tools to detect languages/frameworks, build systems, deployment patterns, dependencies.
    /// Include rationale for existing choices and recommendations. Goes into project/tech-stack.md
    #[arg(long, required = true)]
    pub tech_stack: String,

    /// Concise summary combining vision and tech stack from your analysis.
    /// Should capture key insights from your codebase exploration for quick context loading.
    /// Goes into project/summary.md
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
