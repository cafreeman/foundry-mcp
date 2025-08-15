//! MCP tool definitions using the rust-mcp-sdk macros

use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    tool_box,
};
use serde::{Deserialize, Serialize};

//****************//
//  SetupProjectTool  //
//****************//
#[mcp_tool(
    name = "setup_project",
    description = "Create project context documents for any software project. The LLM should research and analyze the project to provide comprehensive information for tech-stack.md and vision.md files.",
    title = "Setup Project",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SetupProjectTool {
    /// Unique project identifier
    project_name: String,
    /// Complete technology stack information
    tech_stack: TechStackData,
    /// Project vision and strategic information
    vision: VisionData,
    /// Optional path to existing codebase for analysis context (not used for file scanning, but for LLM context)
    #[serde(skip_serializing_if = "Option::is_none")]
    project_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TechStackData {
    /// Programming languages used in the project
    languages: Vec<String>,
    /// Frameworks and libraries used
    frameworks: Vec<String>,
    /// Database systems and storage solutions
    databases: Vec<String>,
    /// Development tools, build systems, and utilities
    tools: Vec<String>,
    /// Deployment platforms and infrastructure tools
    deployment: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct VisionData {
    /// High-level project overview and purpose
    overview: String,
    /// Primary project goals and objectives
    goals: Vec<String>,
    /// Target user groups and personas
    target_users: Vec<String>,
    /// Measurable success criteria and KPIs
    success_criteria: Vec<String>,
}

impl SetupProjectTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        // This will be handled by the handler, not the tool itself
        Ok(CallToolResult::text_content(vec![TextContent::from(
            "Project setup initiated",
        )]))
    }
}

//****************//
//  CreateSpecTool  //
//****************//
#[mcp_tool(
    name = "create_spec",
    description = "Create new specification with task breakdown",
    title = "Create Specification",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateSpecTool {
    /// Target project
    project_name: String,
    /// Snake_case specification name
    spec_name: String,
    /// Feature/specification description
    description: String,
}

impl CreateSpecTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        Ok(CallToolResult::text_content(vec![TextContent::from(
            "Specification creation initiated",
        )]))
    }
}

//****************//
//  LoadSpecTool  //
//****************//
#[mcp_tool(
    name = "load_spec",
    description = "Load specification with full project context",
    title = "Load Specification",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LoadSpecTool {
    /// Target project
    project_name: String,
    /// Timestamped spec identifier
    spec_id: String,
}

impl LoadSpecTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        Ok(CallToolResult::text_content(vec![TextContent::from(
            "Specification loading initiated",
        )]))
    }
}

//****************//
//  UpdateSpecTool  //
//****************//
#[mcp_tool(
    name = "update_spec",
    description = "Update specification, tasks, and notes",
    title = "Update Specification",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UpdateSpecTool {
    /// Target project
    project_name: String,
    /// Specification identifier
    spec_id: String,
    /// Operation to perform: add_task, update_task, remove_task, update_task_status, add_note, reorder_tasks
    operation: String,
    /// Task data (required for add_task, update_task operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<TaskData>,
    /// Task ID (required for remove_task, update_task_status operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    /// New task status (required for update_task_status operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    /// Note content (required for add_note operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Note category (optional for add_note operation)
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TaskData {
    /// Task ID (required for update_task)
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    /// Task title
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// Task description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Task priority
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    /// Task status
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    /// Task dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Vec<String>>,
}

impl UpdateSpecTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        Ok(CallToolResult::text_content(vec![TextContent::from(
            "Specification update initiated",
        )]))
    }
}

//******************//
//  ProjectManagerTools  //
//******************//
// Generates an enum with all our tool variants
tool_box!(
    ProjectManagerTools,
    [
        SetupProjectTool,
        CreateSpecTool,
        LoadSpecTool,
        UpdateSpecTool
    ]
);
