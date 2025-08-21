//! MCP server handler implementation

use crate::cache::ProjectManagerCache;
use crate::errors::Result;
use crate::filesystem::FileSystemManager;
use crate::handlers::{
    create_spec::CreateSpecHandler, load_spec::LoadSpecHandler, setup_project::SetupProjectHandler,
    update_spec::UpdateSpecHandler,
};
use crate::repository::{ProjectRepository, SpecificationRepository};
use crate::security::RateLimiter;
use crate::tools::ProjectManagerTools;
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
use std::time::Duration;

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
    rate_limiter: RateLimiter,
}

impl ProjectManagerHandler {
    /// Create a new ProjectManagerHandler instance
    pub fn new() -> Result<Self> {
        let fs_manager = FileSystemManager::new()?;
        let cache = ProjectManagerCache::new();

        let project_repo = ProjectRepository::with_cache(fs_manager.clone(), cache.clone());
        let spec_repo = SpecificationRepository::with_cache(fs_manager.clone(), cache.clone());

        let setup_project_handler = SetupProjectHandler::new(project_repo.clone());
        let create_spec_handler = CreateSpecHandler::new(project_repo.clone(), spec_repo.clone());
        let load_spec_handler = LoadSpecHandler::new(project_repo.clone(), spec_repo.clone());
        let update_spec_handler = UpdateSpecHandler::new(project_repo.clone(), spec_repo.clone());

        // Configure rate limiter: 100 requests per minute
        let rate_limiter = RateLimiter::new(100, Duration::from_secs(60));

        Ok(Self {
            fs_manager,
            project_repo,
            spec_repo,
            setup_project_handler,
            create_spec_handler,
            load_spec_handler,
            update_spec_handler,
            rate_limiter,
        })
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
        // Rate limiting check
        let client_id = "default"; // In a real implementation, you'd extract this from the request
        if !self.rate_limiter.is_allowed(client_id) {
            return Err(CallToolError::new(
                RpcError::invalid_request().with_message(
                    "Rate limit exceeded. Please wait before making more requests.".to_string(),
                ),
            ));
        }

        // Convert request parameters into ProjectManagerTools enum
        let tool_params: ProjectManagerTools = ProjectManagerTools::try_from(request.params)
            .map_err(|e| {
                CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
            })?;

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
            "execute_task" => self.handle_execute_task_prompt().await,
            _ => Err(RpcError::invalid_request()
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
            "name": tool_json["name"],
            "description": tool_json["description"],
            "overview": tool_json["overview"],
            "languages": tool_json["languages"],
            "frameworks": tool_json["frameworks"],
            "databases": tool_json["databases"],
            "tools": tool_json["tools"],
            "deployment": tool_json["deployment"],
            "goals": tool_json["goals"],
            "target_users": tool_json["target_users"],
            "success_criteria": tool_json["success_criteria"]
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
        // Convert the tool to JSON and then extract the fields
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "spec_name": tool_json["spec_name"],
            "description": tool_json["description"],
            "content": tool_json["content"]
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
        // Convert the tool to JSON and then extract the fields
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
        // Convert the tool to JSON and then extract the fields
        let tool_json = serde_json::to_value(tool).map_err(|e| {
            CallToolError::new(RpcError::internal_error().with_message(e.to_string()))
        })?;

        let arguments = serde_json::json!({
            "project_name": tool_json["project_name"],
            "spec_id": tool_json["spec_id"],
            "operation": tool_json["operation"],
            "title": tool_json["title"],
            "description": tool_json["description"],
            "priority": tool_json["priority"],
            "status": tool_json["status"],
            "content": tool_json["content"],
            "category": tool_json["category"],
            "task_id": tool_json["task_id"]
        });

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

    /// Handle execute_task prompt
    async fn handle_execute_task_prompt(&self) -> std::result::Result<GetPromptResult, RpcError> {
        let prompt_content = self
            .generate_execute_task_prompt()
            .await
            .map_err(|e| RpcError::internal_error().with_message(e.to_string()))?;

        Ok(GetPromptResult {
            description: Some(
                "Guide agents through task execution with proper context loading".to_string(),
            ),
            messages: vec![rust_mcp_sdk::schema::PromptMessage {
                content: rust_mcp_sdk::schema::ContentBlock::text_content(prompt_content),
                role: rust_mcp_sdk::schema::Role::User,
            }],
            meta: None,
        })
    }

    /// Generate the execute_task prompt content
    async fn generate_execute_task_prompt(&self) -> Result<String> {
        let projects = self.project_repo.list_projects().await?;

        if projects.is_empty() {
            return Ok(
                "No projects found. Please create a project first using the setup_project tool."
                    .to_string(),
            );
        }

        let mut prompt = String::new();
        prompt.push_str("Available projects:\n");

        for project in &projects {
            prompt.push_str(&format!("- {}: {}\n", project.name, project.description));

            let specs = self.spec_repo.list_specs(&project.name).await?;
            if !specs.is_empty() {
                prompt.push_str("  Specifications:\n");
                for spec in &specs {
                    prompt.push_str(&format!("    - {}: {}\n", spec.id, spec.name));
                }
            }
        }

        prompt.push_str("\nTo execute a task, please:\n");
        prompt.push_str("1. Load the relevant specification using load_spec\n");
        prompt.push_str("2. Review the current tasks and context\n");
        prompt.push_str("3. Update task status or add notes as needed using update_spec\n");
        prompt.push_str("4. Provide clear, actionable guidance for the next steps\n");

        Ok(prompt)
    }
}
