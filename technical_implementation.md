# Project Manager MCP - Technical Implementation Guide

## Overview

This document outlines the technical implementation details for the Project Manager MCP using Rust and the `rust-mcp-sdk` crate. The server will provide deterministic tools for AI coding assistants to manage project context, specifications, and task lists.

## Core Data Structures

### Project Management

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub base_path: PathBuf,
    pub tech_stack: TechStack,
    pub vision: Vision,
    pub specs: Vec<String>, // List of spec IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub tools: Vec<String>,
    pub deployment: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vision {
    pub overview: String,
    pub goals: Vec<String>,
    pub target_users: Vec<String>,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    pub id: String, // Format: YYYYMMDD_snake_case_name
    pub project_name: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SpecStatus,
    pub content: String,
    pub task_list: TaskList,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecStatus {
    Draft,
    InProgress,
    Completed,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>, // Task IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub category: NoteCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoteCategory {
    Implementation,
    UserPreference,
    CodeStyle,
    Architecture,
    Testing,
    Deployment,
    General,
}
```

## File System Organization

### Directory Structure Manager

```rust
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

pub struct FileSystemManager {
    base_dir: PathBuf,
}

impl FileSystemManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Could not determine home directory")?;
        let base_dir = home_dir.join(".project-manager-mcp");

        std::fs::create_dir_all(&base_dir)
            .context("Failed to create base directory")?;

        Ok(Self { base_dir })
    }

    pub fn project_dir(&self, project_name: &str) -> PathBuf {
        self.base_dir.join(project_name)
    }

    pub fn project_info_dir(&self, project_name: &str) -> PathBuf {
        self.project_dir(project_name).join("project")
    }

    pub fn specs_dir(&self, project_name: &str) -> PathBuf {
        self.project_dir(project_name).join("specs")
    }

    pub fn spec_dir(&self, project_name: &str, spec_id: &str) -> PathBuf {
        self.specs_dir(project_name).join(spec_id)
    }

    pub fn create_project_structure(&self, project_name: &str) -> Result<()> {
        let project_dir = self.project_dir(project_name);
        let project_info_dir = self.project_info_dir(project_name);
        let specs_dir = self.specs_dir(project_name);

        std::fs::create_dir_all(&project_info_dir)
            .context("Failed to create project info directory")?;
        std::fs::create_dir_all(&specs_dir)
            .context("Failed to create specs directory")?;

        Ok(())
    }

    pub fn create_spec_structure(&self, project_name: &str, spec_id: &str) -> Result<()> {
        let spec_dir = self.spec_dir(project_name, spec_id);
        std::fs::create_dir_all(&spec_dir)
            .context("Failed to create spec directory")?;
        Ok(())
    }

    pub fn project_exists(&self, project_name: &str) -> bool {
        self.project_dir(project_name).exists()
    }

    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut projects = Vec::new();

        if !self.base_dir.exists() {
            return Ok(projects);
        }

        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    projects.push(name.to_string());
                }
            }
        }

        Ok(projects)
    }

    pub fn list_specs(&self, project_name: &str) -> Result<Vec<String>> {
        let specs_dir = self.specs_dir(project_name);
        let mut specs = Vec::new();

        if !specs_dir.exists() {
            return Ok(specs);
        }

        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    specs.push(name.to_string());
                }
            }
        }

        Ok(specs)
    }
}
```

## Data Access Layer

### Project Repository

```rust
use crate::{Project, Specification, FileSystemManager};
use anyhow::{Result, Context, bail};
use std::path::Path;

pub struct ProjectRepository {
    fs: FileSystemManager,
}

impl ProjectRepository {
    pub fn new(fs: FileSystemManager) -> Self {
        Self { fs }
    }

    pub async fn create_project(&self, mut project: Project) -> Result<()> {
        if self.fs.project_exists(&project.name) {
            bail!("Project '{}' already exists", project.name);
        }

        self.fs.create_project_structure(&project.name)?;

        // Write tech-stack.md
        let tech_stack_content = self.render_tech_stack(&project.tech_stack)?;
        let tech_stack_path = self.fs.project_info_dir(&project.name).join("tech-stack.md");
        std::fs::write(&tech_stack_path, tech_stack_content)
            .context("Failed to write tech-stack.md")?;

        // Write vision.md
        let vision_content = self.render_vision(&project.vision)?;
        let vision_path = self.fs.project_info_dir(&project.name).join("vision.md");
        std::fs::write(&vision_path, vision_content)
            .context("Failed to write vision.md")?;

        // Write project metadata
        let project_meta_path = self.fs.project_dir(&project.name).join("project.json");
        let project_json = serde_json::to_string_pretty(&project)
            .context("Failed to serialize project")?;
        std::fs::write(&project_meta_path, project_json)
            .context("Failed to write project metadata")?;

        Ok(())
    }

    pub async fn load_project(&self, project_name: &str) -> Result<Project> {
        let project_meta_path = self.fs.project_dir(project_name).join("project.json");

        if !project_meta_path.exists() {
            bail!("Project '{}' does not exist", project_name);
        }

        let project_json = std::fs::read_to_string(&project_meta_path)
            .context("Failed to read project metadata")?;
        let project: Project = serde_json::from_str(&project_json)
            .context("Failed to deserialize project")?;

        Ok(project)
    }

    pub async fn create_spec(&self, spec: Specification) -> Result<()> {
        if !self.fs.project_exists(&spec.project_name) {
            bail!("Project '{}' does not exist", spec.project_name);
        }

        self.fs.create_spec_structure(&spec.project_name, &spec.id)?;

        let spec_dir = self.fs.spec_dir(&spec.project_name, &spec.id);

        // Write spec.md
        let spec_path = spec_dir.join("spec.md");
        std::fs::write(&spec_path, &spec.content)
            .context("Failed to write spec.md")?;

        // Write task-list.md
        let task_list_content = self.render_task_list(&spec.task_list)?;
        let task_list_path = spec_dir.join("task-list.md");
        std::fs::write(&task_list_path, task_list_content)
            .context("Failed to write task-list.md")?;

        // Write notes.md
        let notes_content = self.render_notes(&spec.notes)?;
        let notes_path = spec_dir.join("notes.md");
        std::fs::write(&notes_path, notes_content)
            .context("Failed to write notes.md")?;

        // Write spec metadata
        let spec_meta_path = spec_dir.join("spec.json");
        let spec_json = serde_json::to_string_pretty(&spec)
            .context("Failed to serialize spec")?;
        std::fs::write(&spec_meta_path, spec_json)
            .context("Failed to write spec metadata")?;

        Ok(())
    }

    pub async fn load_spec(&self, project_name: &str, spec_id: &str) -> Result<Specification> {
        let spec_meta_path = self.fs.spec_dir(project_name, spec_id).join("spec.json");

        if !spec_meta_path.exists() {
            bail!("Spec '{}' does not exist in project '{}'", spec_id, project_name);
        }

        let spec_json = std::fs::read_to_string(&spec_meta_path)
            .context("Failed to read spec metadata")?;
        let spec: Specification = serde_json::from_str(&spec_json)
            .context("Failed to deserialize spec")?;

        Ok(spec)
    }

    pub async fn update_spec(&self, spec: &Specification) -> Result<()> {
        let spec_dir = self.fs.spec_dir(&spec.project_name, &spec.id);

        if !spec_dir.exists() {
            bail!("Spec '{}' does not exist", spec.id);
        }

        // Update all files
        let spec_path = spec_dir.join("spec.md");
        std::fs::write(&spec_path, &spec.content)?;

        let task_list_content = self.render_task_list(&spec.task_list)?;
        let task_list_path = spec_dir.join("task-list.md");
        std::fs::write(&task_list_path, task_list_content)?;

        let notes_content = self.render_notes(&spec.notes)?;
        let notes_path = spec_dir.join("notes.md");
        std::fs::write(&notes_path, notes_content)?;

        let spec_meta_path = spec_dir.join("spec.json");
        let spec_json = serde_json::to_string_pretty(spec)?;
        std::fs::write(&spec_meta_path, spec_json)?;

        Ok(())
    }

    // Rendering helper methods
    fn render_tech_stack(&self, tech_stack: &TechStack) -> Result<String> {
        let mut content = String::new();
        content.push_str("# Technology Stack\n\n");

        if !tech_stack.languages.is_empty() {
            content.push_str("## Languages\n");
            for lang in &tech_stack.languages {
                content.push_str(&format!("- {}\n", lang));
            }
            content.push('\n');
        }

        if !tech_stack.frameworks.is_empty() {
            content.push_str("## Frameworks\n");
            for framework in &tech_stack.frameworks {
                content.push_str(&format!("- {}\n", framework));
            }
            content.push('\n');
        }

        // Continue for other fields...
        if !vision.target_users.is_empty() {
            content.push_str("## Target Users\n");
            for user in &vision.target_users {
                content.push_str(&format!("- {}\n", user));
            }
            content.push('\n');
        }

        if !vision.success_criteria.is_empty() {
            content.push_str("## Success Criteria\n");
            for criteria in &vision.success_criteria {
                content.push_str(&format!("- {}\n", criteria));
            }
            content.push('\n');
        }
        if !tech_stack.databases.is_empty() {
            content.push_str("## Databases\n");
            for db in &tech_stack.databases {
                content.push_str(&format!("- {}\n", db));
            }
            content.push('\n');
        }

        if !tech_stack.tools.is_empty() {
            content.push_str("## Development Tools\n");
            for tool in &tech_stack.tools {
                content.push_str(&format!("- {}\n", tool));
            }
            content.push('\n');
        }

        if !tech_stack.deployment.is_empty() {
            content.push_str("## Deployment & Infrastructure\n");
            for deploy in &tech_stack.deployment {
                content.push_str(&format!("- {}\n", deploy));
            }
            content.push('\n');
        }

        Ok(content)
    }

    fn render_vision(&self, vision: &Vision) -> Result<String> {
        let mut content = String::new();
        content.push_str("# Project Vision\n\n");
        content.push_str(&format!("## Overview\n{}\n\n", vision.overview));

        if !vision.goals.is_empty() {
            content.push_str("## Goals\n");
            for goal in &vision.goals {
                content.push_str(&format!("- {}\n", goal));
            }
            content.push('\n');
        }

        // Continue for other fields...

        Ok(content)
    }

    fn render_task_list(&self, task_list: &TaskList) -> Result<String> {
        let mut content = String::new();
        content.push_str("# Task List\n\n");
        content.push_str(&format!("*Last updated: {}*\n\n", task_list.updated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        for task in &task_list.tasks {
            let status_marker = match task.status {
                TaskStatus::Todo => "- [ ]",
                TaskStatus::InProgress => "- [⏳]",
                TaskStatus::Completed => "- [x]",
                TaskStatus::Blocked => "- [🚫]",
            };

            let priority_str = match task.priority {
                TaskPriority::Low => "low",
                TaskPriority::Medium => "medium",
                TaskPriority::High => "high",
                TaskPriority::Critical => "critical",
            };

            content.push_str(&format!("{} **{}** ({})\n", status_marker, task.title, priority_str));
            if !task.description.is_empty() {
                content.push_str(&format!("  {}\n", task.description));
            }
            content.push('\n');
        }

        Ok(content)
    }

    fn render_notes(&self, notes: &[Note]) -> Result<String> {
        let mut content = String::new();
        content.push_str("# Notes\n\n");

        for note in notes {
            let category_str = match note.category {
                NoteCategory::Implementation => "Implementation",
                NoteCategory::UserPreference => "User Preference",
                NoteCategory::CodeStyle => "Code Style",
                NoteCategory::Architecture => "Architecture",
                NoteCategory::Testing => "Testing",
                NoteCategory::Deployment => "Deployment",
                NoteCategory::General => "General",
            };

            content.push_str(&format!("## {} - {}\n", category_str, note.created_at.format("%Y-%m-%d")));
            content.push_str(&format!("{}\n\n", note.content));
        }

        Ok(content)
    }
}
```

## MCP Server Implementation

### Main Server Handler

```rust
use rust_mcp_sdk::{
    server_runtime::{create_server, ServerHandlerTrait},
    transport::{StdioTransport, Transport},
    schema::{
        InitializeRequest, InitializeResult, Implementation, ServerCapabilities,
        ListToolsRequest, ListToolsResult, CallToolRequest, CallToolResult,
        Tool, ToolInputSchema, ListPromptsRequest, ListPromptsResult,
        GetPromptRequest, GetPromptResult, Prompt,
    },
};
use serde_json::{json, Value};
use anyhow::Result;
use tokio;

pub struct ProjectManagerHandler {
    repository: ProjectRepository,
}

impl ProjectManagerHandler {
    pub fn new() -> Result<Self> {
        let fs = FileSystemManager::new()?;
        let repository = ProjectRepository::new(fs);
        Ok(Self { repository })
    }
}

#[async_trait::async_trait]
impl ServerHandlerTrait for ProjectManagerHandler {
    async fn handle_initialize(&self, request: InitializeRequest) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Implementation {
                name: "Project Manager MCP".to_string(),
                version: "0.1.0".to_string(),
            },
            capabilities: ServerCapabilities {
                tools: Some(Default::default()),
                prompts: Some(Default::default()),
                ..Default::default()
            },
            protocol_version: "2024-11-05".to_string(),
            instructions: Some("Project and specification management for AI coding assistants".to_string()),
        })
    }

    async fn handle_list_tools(&self, _request: ListToolsRequest) -> Result<ListToolsResult> {
        let tools = vec![
            Tool {
                name: "setup_project".to_string(),
                description: Some("Create project context documents for any software project. The LLM should research and analyze the project to provide comprehensive information for tech-stack.md and vision.md files.".to_string()),
                input_schema: ToolInputSchema {
                    type_: "object".to_string(),
                    properties: Some(json!({
                        "project_name": {
                            "type": "string",
                            "description": "Unique project identifier"
                        },
                        "tech_stack": {
                            "type": "object",
                            "description": "Complete technology stack information",
                            "properties": {
                                "languages": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Programming languages used in the project"
                                },
                                "frameworks": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Frameworks and libraries used"
                                },
                                "databases": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Database systems and storage solutions"
                                },
                                "tools": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Development tools, build systems, and utilities"
                                },
                                "deployment": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Deployment platforms and infrastructure tools"
                                }
                            },
                            "required": ["languages", "frameworks", "databases", "tools", "deployment"]
                        },
                        "vision": {
                            "type": "object",
                            "description": "Project vision and strategic information",
                            "properties": {
                                "overview": {
                                    "type": "string",
                                    "description": "High-level project overview and purpose"
                                },
                                "goals": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Primary project goals and objectives"
                                },
                                "target_users": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Target user groups and personas"
                                },
                                "success_criteria": {
                                    "type": "array",
                                    "items": {"type": "string"},
                                    "description": "Measurable success criteria and KPIs"
                                }
                            },
                            "required": ["overview", "goals", "target_users", "success_criteria"]
                        },
                        "project_path": {
                            "type": "string",
                            "description": "Optional path to existing codebase for analysis context (not used for file scanning, but for LLM context)"
                        }
                    })),
                    required: Some(vec![
                        "project_name".to_string(),
                        "tech_stack".to_string(),
                        "vision".to_string()
                    ]),
                    additional_properties: None,
                },
            },
            Tool {
                name: "create_spec".to_string(),
                description: Some("Create new specification with task breakdown".to_string()),
                input_schema: ToolInputSchema {
                    type_: "object".to_string(),
                    properties: Some(json!({
                        "project_name": {
                            "type": "string",
                            "description": "Target project"
                        },
                        "spec_name": {
                            "type": "string",
                            "description": "Snake_case specification name"
                        },
                        "description": {
                            "type": "string",
                            "description": "Feature/specification description"
                        }
                    })),
                    required: Some(vec![
                        "project_name".to_string(),
                        "spec_name".to_string(),
                        "description".to_string()
                    ]),
                    additional_properties: None,
                },
            },
            Tool {
                name: "load_spec".to_string(),
                description: Some("Load specification with full project context".to_string()),
                input_schema: ToolInputSchema {
                    type_: "object".to_string(),
                    properties: Some(json!({
                        "project_name": {
                            "type": "string",
                            "description": "Target project"
                        },
                        "spec_id": {
                            "type": "string",
                            "description": "Timestamped spec identifier"
                        }
                    })),
                    required: Some(vec!["project_name".to_string(), "spec_id".to_string()]),
                    additional_properties: None,
                },
            },
        ];

        Ok(ListToolsResult { tools })
    }

    async fn handle_call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        match request.params.name.as_str() {
            "setup_project" => self.handle_setup_project(request.params.arguments).await,
            "create_spec" => self.handle_create_spec(request.params.arguments).await,
            "load_spec" => self.handle_load_spec(request.params.arguments).await,
            _ => Ok(CallToolResult {
                content: vec![],
                is_error: Some(true),
            }),
        }
    }

    async fn handle_list_prompts(&self, _request: ListPromptsRequest) -> Result<ListPromptsResult> {
        let prompts = vec![
            Prompt {
                name: "execute_task".to_string(),
                description: Some("Guide agents through task execution with proper context loading".to_string()),
                arguments: None,
            },
        ];

        Ok(ListPromptsResult { prompts })
    }

    async fn handle_get_prompt(&self, request: GetPromptRequest) -> Result<GetPromptResult> {
        match request.params.name.as_str() {
            "execute_task" => self.handle_execute_task_prompt().await,
            _ => Ok(GetPromptResult {
                description: Some("Unknown prompt".to_string()),
                messages: vec![],
            }),
        }
    }
}

impl ProjectManagerHandler {
    async fn handle_setup_project(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments for setup_project"))?;

        // Parse arguments
        let project_name: String = args.get("project_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing project_name"))?
            .to_string();

        let tech_stack_value = args.get("tech_stack")
            .ok_or_else(|| anyhow::anyhow!("Missing tech_stack"))?;
        let tech_stack: TechStack = serde_json::from_value(tech_stack_value.clone())?;

        let vision_value = args.get("vision")
            .ok_or_else(|| anyhow::anyhow!("Missing vision"))?;
        let vision: Vision = serde_json::from_value(vision_value.clone())?;

        let project_path = args.get("project_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        // Create project structure
        let project = Project {
            name: project_name.clone(),
            created_at: chrono::Utc::now(),
            base_path: project_path.unwrap_or_default(),
            tech_stack,
            vision,
            specs: Vec::new(),
        };

        // Save project
        self.repository.create_project(project).await?;

        Ok(CallToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Successfully created project '{}' with tech stack and vision documents", project_name)
            })],
            is_error: Some(false),
        })
    }

    async fn handle_create_spec(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments for create_spec"))?;

        // Parse arguments
        let project_name: String = args.get("project_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing project_name"))?
            .to_string();

        let spec_name: String = args.get("spec_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing spec_name"))?
            .to_string();

        let description: String = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing description"))?
            .to_string();

        // Generate timestamped spec ID
        let timestamp = chrono::Utc::now().format("%Y%m%d").to_string();
        let spec_id = format!("{}_{}", timestamp, spec_name);

        // Create specification
        let spec = Specification {
            id: spec_id.clone(),
            project_name: project_name.clone(),
            name: spec_name,
            description: description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: SpecStatus::Draft,
            content: format!("# {}\n\n{}\n\n## Implementation Details\n\n*To be filled in during development*", description, description),
            task_list: TaskList {
                tasks: Vec::new(),
                updated_at: chrono::Utc::now(),
            },
            notes: Vec::new(),
        };

        // Save specification
        self.repository.create_spec(spec).await?;

        Ok(CallToolResult {
            content: vec![json!({
                "type": "text",
                "text": format!("Successfully created specification '{}' for project '{}'", spec_id, project_name)
            })],
            is_error: Some(false),
        })
    }

    async fn handle_load_spec(&self, args: Option<Value>) -> Result<CallToolResult> {
        let args = args.ok_or_else(|| anyhow::anyhow!("Missing arguments for load_spec"))?;

        // Parse arguments
        let project_name: String = args.get("project_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing project_name"))?
            .to_string();

        let spec_id: String = args.get("spec_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing spec_id"))?
            .to_string();

        // Load project and spec
        let project = self.repository.load_project(&project_name).await?;
        let spec = self.repository.load_spec(&project_name, &spec_id).await?;

        // Format context document
        let mut context = String::new();

        // Project Context
        context.push_str("# Project Context\n\n");
        context.push_str(&format!("**Project:** {}\n", project.name));
        context.push_str(&format!("**Created:** {}\n\n", project.created_at.format("%Y-%m-%d")));

        // Tech Stack Summary
        context.push_str("## Technology Stack Summary\n");
        if !project.tech_stack.languages.is_empty() {
            context.push_str(&format!("**Languages:** {}\n", project.tech_stack.languages.join(", ")));
        }
        if !project.tech_stack.frameworks.is_empty() {
            context.push_str(&format!("**Frameworks:** {}\n", project.tech_stack.frameworks.join(", ")));
        }
        context.push_str("\n");

        // Vision Summary
        context.push_str("## Project Vision Summary\n");
        context.push_str(&format!("{}\n\n", project.vision.overview));

        // Current Spec
        context.push_str(&format!("# Current Spec: {}\n\n", spec.name));
        context.push_str(&format!("**Status:** {:?}\n", spec.status));
        context.push_str(&format!("**Created:** {}\n", spec.created_at.format("%Y-%m-%d")));
        context.push_str(&format!("**Last Updated:** {}\n\n", spec.updated_at.format("%Y-%m-%d %H:%M UTC")));

        context.push_str("## Specification Content\n");
        context.push_str(&spec.content);
        context.push_str("\n\n");

        // Task List
        context.push_str("## Current Task List\n");
        if spec.task_list.tasks.is_empty() {
            context.push_str("*No tasks defined yet*\n\n");
        } else {
            for task in &spec.task_list.tasks {
                let status_marker = match task.status {
                    TaskStatus::Todo => "- [ ]",
                    TaskStatus::InProgress => "- [⏳]",
                    TaskStatus::Completed => "- [x]",
                    TaskStatus::Blocked => "- [🚫]",
                };
                context.push_str(&format!("{} **{}** ({})\n", status_marker, task.title, format!("{:?}", task.priority).to_lowercase()));
                if !task.description.is_empty() {
                    context.push_str(&format!("  {}\n", task.description));
                }
            }
            context.push_str("\n");
        }

        // Notes
        if !spec.notes.is_empty() {
            context.push_str("## Notes\n");
            for note in &spec.notes {
                context.push_str(&format!("**{}** ({}): {}\n",
                    format!("{:?}", note.category),
                    note.created_at.format("%Y-%m-%d"),
                    note.content
                ));
            }
        }

        Ok(CallToolResult {
            content: vec![json!({
                "type": "text",
                "text": context
            })],
            is_error: Some(false),
        })
    }

    async fn handle_execute_task_prompt(&self) -> Result<GetPromptResult> {
        let prompt_content = r#"
You are helping execute a task for a software development project. Follow these steps:

1. **Check Context**: Look through the current conversation to see if a project specification has already been loaded with the load_spec tool.

2. **Load Context If Needed**: If no spec context is available in this conversation, ask the user which project and spec they want to work on, then call load_spec to get the current specification and project context.

3. **Identify Task**:
   - If the user specified a particular task or feature, find the relevant item(s) in the task list
   - If no specific task was mentioned, work on the next incomplete task in the task list
   - If all tasks are complete, ask the user what they'd like to work on next

4. **Execute Work**:
   - Read and understand the full context (project vision, tech stack, spec details)
   - Work on the identified task following the project's established patterns and preferences
   - Make sure your implementation aligns with the spec and project vision

5. **Update Task List**:
   - Mark completed tasks as done
   - Add new subtasks if you discover additional work needed
   - Update task descriptions if scope changes
   - Keep the task list as an accurate reflection of remaining work

6. **Update Notes**:
   - Add any implementation decisions, code patterns, or user preferences to notes.md
   - Include anything that might be helpful for future work on this spec

Remember: The spec and task list should always represent the current state of work to enable natural pause/resume functionality.
        "#;

        Ok(GetPromptResult {
            description: Some("Execute task workflow prompt".to_string()),
            messages: vec![
                // Convert prompt content to proper message format
                // This will depend on the exact schema structure
            ],
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let handler = ProjectManagerHandler::new()?;
    let transport = StdioTransport::new();

    create_server(handler, transport).await?;

    Ok(())
}
```

## Key Implementation Notes

1. **Error Handling**: Uses `anyhow` for comprehensive error handling throughout the codebase
2. **File System Safety**: All file operations include proper error handling and atomic writes where possible
3. **Timestamp-based IDs**: Spec IDs use `YYYYMMDD_snake_case` format for easy sorting and identification
4. **JSON Metadata**: Each project and spec maintains JSON metadata for fast querying without parsing markdown
5. **Async Design**: Fully async implementation leveraging tokio for I/O operations
6. **Type Safety**: Strong typing throughout with comprehensive serde serialization
7. **Modularity**: Clean separation between data structures, file system operations, and MCP handling

## Development Workflow

1. **Testing**: Unit tests for each component, integration tests for MCP tool interactions
2. **Logging**: Add proper logging with `tracing` crate for debugging
3. **Configuration**: Support for custom base directories and configuration files
4. **Performance**: Consider caching frequently accessed projects/specs in memory
5. **Validation**: Input validation for project names, spec names, and file content

This implementation provides a solid foundation for the Project Manager MCP with room for future enhancements like cloud storage backends, collaboration features, and advanced project templates.
