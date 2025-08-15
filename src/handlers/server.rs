//! MCP server handler implementation

use crate::filesystem::FileSystemManager;
use crate::handlers::{
    create_spec::CreateSpecHandler, load_spec::LoadSpecHandler, setup_project::SetupProjectHandler,
    update_spec::UpdateSpecHandler,
};
use crate::repository::{ProjectRepository, SpecificationRepository};
use crate::tools::ProjectManagerTools;
use anyhow::Result;
use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{
        CallToolRequest, CallToolResult, GetPromptRequest, GetPromptResult, ListPromptsRequest,
        ListPromptsResult, ListToolsRequest, ListToolsResult, Prompt, RpcError,
        schema_utils::CallToolError,
    },
};

/// Main MCP server handler for Project Manager
#[allow(dead_code)]
pub struct ProjectManagerHandler {
    fs_manager: FileSystemManager,
    project_repo: ProjectRepository,
    spec_repo: SpecificationRepository,
    setup_project_handler: SetupProjectHandler,
    create_spec_handler: CreateSpecHandler,
    load_spec_handler: LoadSpecHandler,
    update_spec_handler: UpdateSpecHandler,
}

impl ProjectManagerHandler {
    /// Create a new ProjectManagerHandler instance
    pub fn new() -> Result<Self> {
        let fs_manager = FileSystemManager::new()?;
        let project_repo = ProjectRepository::new(fs_manager.clone());
        let spec_repo = SpecificationRepository::new(fs_manager.clone());
        let setup_project_handler = SetupProjectHandler::new(project_repo.clone());
        let create_spec_handler = CreateSpecHandler::new(project_repo.clone(), spec_repo.clone());
        let load_spec_handler = LoadSpecHandler::new(project_repo.clone(), spec_repo.clone());
        let update_spec_handler = UpdateSpecHandler::new(project_repo.clone(), spec_repo.clone());

        Ok(Self {
            fs_manager,
            project_repo,
            spec_repo,
            setup_project_handler,
            create_spec_handler,
            load_spec_handler,
            update_spec_handler,
        })
    }

    /// Generate the execute_task prompt content
    pub async fn generate_execute_task_prompt(
        &self,
        arguments: &serde_json::Value,
    ) -> Result<String> {
        let project_name = arguments["project_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing project_name"))?;
        let spec_id = arguments["spec_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing spec_id"))?;
        let task_description = arguments["task_description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing task_description"))?;

        // Load the specification context
        let context = self
            .load_spec_handler
            .handle_load_spec(&serde_json::json!({
                "project_name": project_name,
                "spec_id": spec_id
            }))
            .await?;

        let prompt = format!(
            r#"# Task Execution Context

## Project: {}
## Specification: {}
## Task: {}

## Full Context
{}

## Instructions
1. **Context Check**: Review the provided context to understand the project, technology stack, vision, and current specification state.
2. **Task Analysis**: Analyze the task description and identify what needs to be accomplished.
3. **Implementation Plan**: Provide a clear, step-by-step plan to complete the task.
4. **Code/Content**: If applicable, provide the actual implementation, code, or content needed.
5. **Next Steps**: Suggest what should be done next or any follow-up tasks.

## Task Description
{}

Please provide a comprehensive response that addresses the task while leveraging the full context provided."#,
            project_name, spec_id, task_description, context, task_description
        );

        Ok(prompt)
    }
}

impl Default for ProjectManagerHandler {
    fn default() -> Self {
        Self::new().expect("Failed to create ProjectManagerHandler")
    }
}

#[async_trait]
impl ServerHandler for ProjectManagerHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: ProjectManagerTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Convert request parameters into ProjectManagerTools enum
        let tool_params: ProjectManagerTools =
            ProjectManagerTools::try_from(request.params).map_err(CallToolError::new)?;

        // Match the tool variant and execute its corresponding logic
        match tool_params {
            ProjectManagerTools::SetupProjectTool(tool) => self.handle_setup_project(&tool).await,
            ProjectManagerTools::CreateSpecTool(tool) => self.handle_create_spec(&tool).await,
            ProjectManagerTools::LoadSpecTool(tool) => self.handle_load_spec(&tool).await,
            ProjectManagerTools::UpdateSpecTool(tool) => self.handle_update_spec(&tool).await,
        }
    }

    async fn handle_list_prompts_request(
        &self,
        _request: ListPromptsRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListPromptsResult, RpcError> {
        let prompts = vec![Prompt {
            name: "execute_task".to_string(),
            description: Some(
                "Guide agents through task execution with proper context loading".to_string(),
            ),
            arguments: vec![],
            meta: None,
            title: Some("Execute Task".to_string()),
        }];

        Ok(ListPromptsResult {
            prompts,
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_get_prompt_request(
        &self,
        request: GetPromptRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<GetPromptResult, RpcError> {
        match request.params.name.as_str() {
            "execute_task" => {
                let arguments = request
                    .params
                    .arguments
                    .map(|args| serde_json::to_value(args).unwrap_or_default())
                    .unwrap_or_default();
                let prompt_content = self
                    .generate_execute_task_prompt(&arguments)
                    .await
                    .map_err(|e| RpcError::internal_error().with_message(e.to_string()))?;

                Ok(GetPromptResult {
                    description: Some(
                        "Guide agents through task execution with proper context loading"
                            .to_string(),
                    ),
                    messages: vec![rust_mcp_sdk::schema::PromptMessage {
                        content: rust_mcp_sdk::schema::ContentBlock::text_content(prompt_content),
                        role: rust_mcp_sdk::schema::Role::User,
                    }],
                    meta: None,
                })
            }
            _ => Err(RpcError::method_not_found()
                .with_message(format!("Unknown prompt: {}", request.params.name))),
        }
    }
}

impl ProjectManagerHandler {
    /// Handle setup_project tool calls
    async fn handle_setup_project(
        &self,
        tool: &crate::tools::SetupProjectTool,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Convert the tool to JSON and then extract the fields
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "tech_stack": tool_json["tech_stack"],
            "vision": tool_json["vision"],
            "project_path": tool_json["project_path"]
        });

        let result = self
            .setup_project_handler
            .handle_setup_project(&arguments)
            .await
            .map_err(|e| {
                CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
            })?;

        Ok(CallToolResult::text_content(vec![
            rust_mcp_sdk::schema::TextContent::from(result),
        ]))
    }

    /// Handle create_spec tool calls
    async fn handle_create_spec(
        &self,
        tool: &crate::tools::CreateSpecTool,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "spec_name": tool_json["spec_name"],
            "description": tool_json["description"]
        });

        let result = self
            .create_spec_handler
            .handle_create_spec(&arguments)
            .await
            .map_err(|e| {
                CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
            })?;

        Ok(CallToolResult::text_content(vec![
            rust_mcp_sdk::schema::TextContent::from(result),
        ]))
    }

    /// Handle load_spec tool calls
    async fn handle_load_spec(
        &self,
        tool: &crate::tools::LoadSpecTool,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "spec_id": tool_json["spec_id"]
        });

        let result = self
            .load_spec_handler
            .handle_load_spec(&arguments)
            .await
            .map_err(|e| {
                CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
            })?;

        Ok(CallToolResult::text_content(vec![
            rust_mcp_sdk::schema::TextContent::from(result),
        ]))
    }

    /// Handle update_spec tool calls
    async fn handle_update_spec(
        &self,
        tool: &crate::tools::UpdateSpecTool,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let mut arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "spec_id": tool_json["spec_id"],
            "operation": tool_json["operation"]
        });

        // Add optional fields if they exist
        if let Some(task) = tool_json.get("task") {
            arguments["task"] = task.clone();
        }

        if let Some(task_id) = tool_json.get("task_id") {
            arguments["task_id"] = task_id.clone();
        }

        if let Some(status) = tool_json.get("status") {
            arguments["status"] = status.clone();
        }

        if let Some(content) = tool_json.get("content") {
            arguments["content"] = content.clone();
        }

        if let Some(category) = tool_json.get("category") {
            arguments["category"] = category.clone();
        }

        let result = self
            .update_spec_handler
            .handle_update_spec(&arguments)
            .await
            .map_err(|e| {
                CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
            })?;

        Ok(CallToolResult::text_content(vec![
            rust_mcp_sdk::schema::TextContent::from(result),
        ]))
    }
}
