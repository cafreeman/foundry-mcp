//! # MCP Tool Definitions
//!
//! This module defines all MCP tools that map directly to CLI commands.
//! Each tool provides identical functionality to its corresponding CLI command
//! with the same parameter validation and JSON response format.

use rust_mcp_sdk::schema::{Tool as McpTool, ToolInputSchema};
use serde_json::json;
use std::collections::HashMap;

/// Tool definitions for all 8 foundry commands
pub struct FoundryTools;

/// Helper function to create a ToolInputSchema property
fn create_property(
    type_: &str,
    description: &str,
    min_length: Option<u32>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut property = serde_json::Map::new();
    property.insert("type".to_string(), json!(type_));
    property.insert("description".to_string(), json!(description));
    if let Some(min_len) = min_length {
        property.insert("minLength".to_string(), json!(min_len));
    }
    property
}

/// Helper function to create a simple tool with string properties
fn create_tool(
    name: &str,
    description: &str,
    properties_def: Vec<(&str, &str, Option<u32>)>,
    required: Vec<&str>,
) -> McpTool {
    let mut properties = HashMap::new();

    for (prop_name, prop_desc, min_len) in properties_def {
        properties.insert(
            prop_name.to_string(),
            create_property("string", prop_desc, min_len),
        );
    }

    McpTool {
        name: name.to_string(),
        description: Some(description.to_string()),
        title: None,
        input_schema: ToolInputSchema::new(
            required.iter().map(|s| s.to_string()).collect(),
            Some(properties),
        ),
        annotations: None,
        meta: None,
        output_schema: None,
    }
}

impl FoundryTools {
    /// Get all available tools
    pub fn all_tools() -> Vec<McpTool> {
        vec![
            Self::create_project_tool(),
            Self::analyze_project_tool(),
            Self::load_project_tool(),
            Self::create_spec_tool(),
            Self::load_spec_tool(),
            Self::list_projects_tool(),
            Self::validate_content_tool(),
            Self::get_foundry_help_tool(),
        ]
    }

    /// Create project tool - identical to CLI create_project command
    pub fn create_project_tool() -> McpTool {
        create_tool(
            "create_project",
            "Create new project structure with LLM-provided content. Creates ~/.foundry/PROJECT_NAME/ with vision.md, tech-stack.md, and summary.md",
            vec![
                (
                    "project_name",
                    "Descriptive project name using kebab-case (e.g., 'my-awesome-app')",
                    None,
                ),
                (
                    "vision",
                    "High-level product vision (2-4 paragraphs, 200+ chars) covering: problem being solved, target users, unique value proposition, and key roadmap priorities. Goes into project/vision.md",
                    Some(200),
                ),
                (
                    "tech_stack",
                    "Comprehensive technology decisions (150+ chars) including languages, frameworks, databases, deployment platforms, and rationale. Include constraints, preferences, or team standards. Goes into project/tech-stack.md",
                    Some(150),
                ),
                (
                    "summary",
                    "Concise summary (100+ chars) of vision and tech-stack for quick context loading. Should capture essential project essence in 2-3 sentences. Goes into project/summary.md",
                    Some(100),
                ),
            ],
            vec!["project_name", "vision", "tech_stack", "summary"],
        )
    }

    /// Analyze project tool - identical to CLI analyze_project command
    pub fn analyze_project_tool() -> McpTool {
        create_tool(
            "analyze_project",
            "Create project structure by analyzing existing codebase. LLM analyzes codebase and provides vision, tech-stack, and summary content.",
            vec![
                (
                    "project_name",
                    "Descriptive project name using kebab-case",
                    None,
                ),
                (
                    "vision",
                    "LLM-analyzed product vision based on codebase examination (200+ chars)",
                    Some(200),
                ),
                (
                    "tech_stack",
                    "LLM-detected technology stack and architectural decisions (150+ chars)",
                    Some(150),
                ),
                (
                    "summary",
                    "LLM-created concise summary of analyzed project (100+ chars)",
                    Some(100),
                ),
            ],
            vec!["project_name", "vision", "tech_stack", "summary"],
        )
    }

    /// Load project tool - identical to CLI load_project command
    pub fn load_project_tool() -> McpTool {
        create_tool(
            "load_project",
            "Load complete project context (vision, tech-stack, summary) for LLM sessions. Essential for resuming work on existing projects.",
            vec![("project_name", "Name of the project to load", None)],
            vec!["project_name"],
        )
    }

    /// Create spec tool - identical to CLI create_spec command
    pub fn create_spec_tool() -> McpTool {
        create_tool(
            "create_spec",
            "Create timestamped specification for a feature. Creates YYYYMMDD_HHMMSS_FEATURE_NAME directory with spec.md, task-list.md, and notes.md",
            vec![
                (
                    "project_name",
                    "Name of the project to create spec for",
                    None,
                ),
                (
                    "feature_name",
                    "Descriptive feature name using snake_case (e.g., 'user_authentication')",
                    None,
                ),
                (
                    "spec",
                    "Detailed feature specification (200+ chars) covering requirements, acceptance criteria, implementation approach, and constraints",
                    Some(200),
                ),
                (
                    "task_list",
                    "Markdown checklist (100+ chars) of implementation steps that agents can update as work progresses",
                    Some(100),
                ),
                (
                    "notes",
                    "Additional context and design decisions (50+ chars) for the feature implementation",
                    Some(50),
                ),
            ],
            vec!["project_name", "feature_name", "spec", "task_list", "notes"],
        )
    }

    /// Load spec tool - identical to CLI load_spec command
    pub fn load_spec_tool() -> McpTool {
        let mut properties = HashMap::new();
        properties.insert(
            "project_name".to_string(),
            create_property("string", "Name of the project containing the spec", None),
        );
        properties.insert("spec_name".to_string(), create_property("string", "Optional: specific spec to load (YYYYMMDD_HHMMSS_feature_name format). If omitted, lists available specs", None));

        McpTool {
            name: "load_spec".to_string(),
            description: Some("Load specific specification content with project context. If spec_name is omitted, lists available specs.".to_string()),
            title: None,
            input_schema: ToolInputSchema::new(
                vec!["project_name".to_string()],
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    /// List projects tool - identical to CLI list_projects command
    pub fn list_projects_tool() -> McpTool {
        McpTool {
            name: "list_projects".to_string(),
            description: Some("List all available projects with metadata including creation dates, spec counts, and validation status.".to_string()),
            title: None,
            input_schema: ToolInputSchema::new(
                vec![],
                Some(HashMap::new()),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    /// Validate content tool - identical to CLI validate_content command
    pub fn validate_content_tool() -> McpTool {
        let mut properties = HashMap::new();

        let mut content_type_property = serde_json::Map::new();
        content_type_property.insert("type".to_string(), json!("string"));
        content_type_property.insert(
            "enum".to_string(),
            json!(["vision", "tech-stack", "summary", "spec", "notes"]),
        );
        content_type_property.insert(
            "description".to_string(),
            json!("Type of content to validate against schema requirements"),
        );
        properties.insert("content_type".to_string(), content_type_property);

        properties.insert(
            "content".to_string(),
            create_property(
                "string",
                "Content to validate against the specified type's requirements",
                None,
            ),
        );

        McpTool {
            name: "validate_content".to_string(),
            description: Some("Validate content against schema requirements with improvement suggestions. Helps ensure content meets foundry standards before creation.".to_string()),
            title: None,
            input_schema: ToolInputSchema::new(
                vec!["content_type".to_string(), "content".to_string()],
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }

    /// Get foundry help tool - identical to CLI get_foundry_help command
    pub fn get_foundry_help_tool() -> McpTool {
        let mut properties = HashMap::new();

        let mut topic_property = serde_json::Map::new();
        topic_property.insert("type".to_string(), json!("string"));
        topic_property.insert(
            "enum".to_string(),
            json!([
                "workflows",
                "content-examples",
                "project-structure",
                "parameter-guidance"
            ]),
        );
        topic_property.insert(
            "description".to_string(),
            json!("Optional: specific help topic. If omitted, provides general workflow guidance"),
        );
        properties.insert("topic".to_string(), topic_property);

        McpTool {
            name: "get_foundry_help".to_string(),
            description: Some("Get comprehensive workflow guidance, content examples, and usage patterns. Essential for understanding foundry workflows and content standards.".to_string()),
            title: None,
            input_schema: ToolInputSchema::new(
                vec![],
                Some(properties),
            ),
            annotations: None,
            meta: None,
            output_schema: None,
        }
    }
}
