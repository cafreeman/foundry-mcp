//! # MCP Request Handlers
//!
//! This module implements the ServerHandler trait to route MCP tool requests
//! to core operations, maintaining identical functionality and response formats.

use anyhow::Result;
use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError, TextContent,
        schema_utils::CallToolError,
    },
};
use serde_json::Value;

use crate::cli;
use crate::mcp::{error::FoundryMcpError, tools::FoundryTools, traits::McpToolDefinition};

/// Main server handler that routes MCP requests to core operations
pub struct FoundryServerHandler;

impl FoundryServerHandler {
    /// Create a new server handler
    pub fn new() -> Self {
        Self
    }

    /// Convert MCP parameters to typed inputs and execute operation
    async fn route_to_cli_command(
        &self,
        tool_name: &str,
        params: &Value,
    ) -> Result<Value, FoundryMcpError> {
        match tool_name {
            "create_project" => {
                let args =
                    <cli::args::CreateProjectArgs as McpToolDefinition>::from_mcp_params(params)
                        .map_err(|e| {
                            FoundryMcpError::invalid_params(format!(
                                "Invalid parameters for create_project: {}",
                                e
                            ))
                        })?;

                let result = crate::core::ops::create_project::run(
                    crate::core::ops::create_project::Input {
                        project_name: args.project_name,
                        vision: args.vision,
                        tech_stack: args.tech_stack,
                        summary: args.summary,
                    },
                )
                .await?;

                Ok(serde_json::to_value(result)?)
            }
            "analyze_project" => {
                let args = cli::args::AnalyzeProjectArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for analyze_project: {}",
                        e
                    ))
                })?;

                let result = crate::core::ops::analyze_project::run(
                    crate::core::ops::analyze_project::Input {
                        project_name: args.project_name,
                        vision: args.vision,
                        tech_stack: args.tech_stack,
                        summary: args.summary,
                    },
                )
                .await?;

                Ok(serde_json::to_value(result)?)
            }
            "load_project" => {
                let args = cli::args::LoadProjectArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for load_project: {}",
                        e
                    ))
                })?;

                let result =
                    crate::core::ops::load_project::run(crate::core::ops::load_project::Input {
                        project_name: args.project_name,
                    })
                    .await?;

                Ok(serde_json::to_value(result)?)
            }
            "create_spec" => {
                let args = cli::args::CreateSpecArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for create_spec: {}",
                        e
                    ))
                })?;

                let result =
                    crate::core::ops::create_spec::run(crate::core::ops::create_spec::Input {
                        project_name: args.project_name,
                        feature_name: args.feature_name,
                        spec: args.spec,
                        notes: args.notes,
                        tasks: args.tasks,
                    })
                    .await?;

                Ok(serde_json::to_value(result)?)
            }
            "load_spec" => {
                let args = cli::args::LoadSpecArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for load_spec: {}",
                        e
                    ))
                })?;

                let result = crate::core::ops::load_spec::run(crate::core::ops::load_spec::Input {
                    project_name: args.project_name,
                    spec_name: args.spec_name,
                })
                .await?;

                Ok(serde_json::to_value(result)?)
            }
            "list_projects" => {
                let args = cli::args::ListProjectsArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for list_projects: {}",
                        e
                    ))
                })?;

                let _ = args; // unit struct
                let result =
                    crate::core::ops::list_projects::run(crate::core::ops::list_projects::Input)
                        .await?;

                Ok(serde_json::to_value(result)?)
            }
            "list_specs" => {
                let args = cli::args::ListSpecsArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for list_specs: {}",
                        e
                    ))
                })?;

                let result =
                    crate::core::ops::list_specs::run(crate::core::ops::list_specs::Input {
                        project_name: args.project_name,
                    })
                    .await?;

                Ok(serde_json::to_value(result)?)
            }
            "validate_content" => {
                let args =
                    cli::args::ValidateContentArgs::from_mcp_params(params).map_err(|e| {
                        FoundryMcpError::invalid_params(format!(
                            "Invalid parameters for validate_content: {}",
                            e
                        ))
                    })?;

                let result = crate::core::ops::validate_content::run(
                    crate::core::ops::validate_content::Input {
                        content_type: args.content_type,
                        content: args.content,
                    },
                )
                .await?;

                Ok(serde_json::to_value(result)?)
            }
            "get_foundry_help" => {
                let args = cli::args::GetFoundryHelpArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for get_foundry_help: {}",
                        e
                    ))
                })?;

                let result = crate::core::ops::get_foundry_help::run(
                    crate::core::ops::get_foundry_help::Input { topic: args.topic },
                )
                .await?;

                Ok(serde_json::to_value(result)?)
            }
            "update_spec" => {
                let args = cli::args::UpdateSpecArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for update_spec: {}",
                        e
                    ))
                })?;

                let result =
                    crate::core::ops::update_spec::run(crate::core::ops::update_spec::Input {
                        project_name: args.project_name,
                        spec_name: args.spec_name,
                        commands_json: args.commands,
                    })
                    .await?;

                Ok(serde_json::to_value(result)?)
            }
            "delete_spec" => {
                let args = cli::args::DeleteSpecArgs::from_mcp_params(params).map_err(|e| {
                    FoundryMcpError::invalid_params(format!(
                        "Invalid parameters for delete_spec: {}",
                        e
                    ))
                })?;

                let result =
                    crate::core::ops::delete_spec::run(crate::core::ops::delete_spec::Input {
                        project_name: args.project_name,
                        spec_name: args.spec_name,
                        confirm: args.confirm,
                    })
                    .await?;

                Ok(serde_json::to_value(result)?)
            }
            _ => Err(FoundryMcpError::invalid_params(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }
}

impl Default for FoundryServerHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServerHandler for FoundryServerHandler {
    /// Handle tool listing requests
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> Result<ListToolsResult, RpcError> {
        tracing::debug!("Handling list_tools request");

        Ok(ListToolsResult {
            tools: FoundryTools::all_tools(),
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle tool call requests by routing to CLI command functions
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> Result<CallToolResult, CallToolError> {
        let tool_name = request.tool_name();
        let default_map = serde_json::Map::new();
        let params = request.params.arguments.as_ref().unwrap_or(&default_map);
        let params_value = serde_json::Value::Object(params.clone());

        tracing::debug!("Handling call_tool request for: {}", tool_name);

        // Route to CLI command and get JSON result
        let result = self.route_to_cli_command(tool_name, &params_value).await?;

        // Convert JSON result to MCP tool result
        // The CLI commands return structured JSON, so we return it as-is
        let content_text = serde_json::to_string_pretty(&result).map_err(FoundryMcpError::from)?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            content_text,
        )]))
    }
}
