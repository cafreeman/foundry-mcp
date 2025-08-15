//! Load specification tool handler

use crate::repository::{ProjectRepository, SpecificationRepository};
use anyhow::Result;
use serde_json::Value;

/// Handler for loading specifications with full context
pub struct LoadSpecHandler {
    project_repo: ProjectRepository,
    spec_repo: SpecificationRepository,
}

impl LoadSpecHandler {
    /// Create a new LoadSpecHandler instance
    pub fn new(project_repo: ProjectRepository, spec_repo: SpecificationRepository) -> Self {
        Self {
            project_repo,
            spec_repo,
        }
    }

    /// Handle the load_spec tool call
    pub async fn handle_load_spec(&self, arguments: &Value) -> Result<String> {
        // Parse and validate arguments
        let project_name = arguments["project_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: project_name"))?;

        let spec_id = arguments["spec_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: spec_id"))?;

        // Validate project exists
        if !self.project_repo.project_exists(project_name).await {
            return Err(anyhow::anyhow!(
                "Project '{}' does not exist. Please create the project first using setup_project.",
                project_name
            ));
        }

        // Validate spec exists
        if !self.spec_repo.spec_exists(project_name, spec_id).await {
            return Err(anyhow::anyhow!(
                "Specification '{}' does not exist in project '{}'.",
                spec_id,
                project_name
            ));
        }

        // Load project context
        let project = self.project_repo.load_project(project_name).await?;

        // Load specification
        let spec = self.spec_repo.load_spec(project_name, spec_id).await?;

        // Load task list
        let task_list = self.spec_repo.load_task_list(project_name, spec_id).await?;

        // Load notes
        let notes = self.spec_repo.load_notes(project_name, spec_id).await?;

        // Format unified context document
        let context = self.format_context(&project, &spec, &task_list, &notes);

        Ok(context)
    }

    /// Format the complete context for AI consumption
    fn format_context(
        &self,
        project: &crate::models::Project,
        spec: &crate::models::Specification,
        task_list: &crate::models::TaskList,
        notes: &[crate::models::Note],
    ) -> String {
        let mut context = String::new();

        // Project Overview
        context.push_str("# Project Context\n\n");
        context.push_str(&format!("**Project**: {}\n", project.name));
        context.push_str(&format!("**Description**: {}\n\n", project.description));

        // Technology Stack
        context.push_str("## Technology Stack\n\n");
        if !project.tech_stack.languages.is_empty() {
            context.push_str("**Languages**: ");
            context.push_str(&project.tech_stack.languages.join(", "));
            context.push_str("\n\n");
        }
        if !project.tech_stack.frameworks.is_empty() {
            context.push_str("**Frameworks**: ");
            context.push_str(&project.tech_stack.frameworks.join(", "));
            context.push_str("\n\n");
        }
        if !project.tech_stack.databases.is_empty() {
            context.push_str("**Databases**: ");
            context.push_str(&project.tech_stack.databases.join(", "));
            context.push_str("\n\n");
        }
        if !project.tech_stack.tools.is_empty() {
            context.push_str("**Tools**: ");
            context.push_str(&project.tech_stack.tools.join(", "));
            context.push_str("\n\n");
        }
        if !project.tech_stack.deployment.is_empty() {
            context.push_str("**Deployment**: ");
            context.push_str(&project.tech_stack.deployment.join(", "));
            context.push_str("\n\n");
        }

        // Project Vision
        context.push_str("## Project Vision\n\n");
        context.push_str(&project.vision.overview);
        context.push_str("\n\n");

        if !project.vision.goals.is_empty() {
            context.push_str("**Goals**:\n");
            for goal in &project.vision.goals {
                context.push_str(&format!("- {}\n", goal));
            }
            context.push_str("\n");
        }

        if !project.vision.target_users.is_empty() {
            context.push_str("**Target Users**:\n");
            for user in &project.vision.target_users {
                context.push_str(&format!("- {}\n", user));
            }
            context.push_str("\n");
        }

        if !project.vision.success_criteria.is_empty() {
            context.push_str("**Success Criteria**:\n");
            for criterion in &project.vision.success_criteria {
                context.push_str(&format!("- {}\n", criterion));
            }
            context.push_str("\n");
        }

        // Specification Details
        context.push_str("## Specification\n\n");
        context.push_str(&format!("**Name**: {}\n", spec.name));
        context.push_str(&format!("**ID**: {}\n", spec.id));
        context.push_str(&format!("**Status**: {:?}\n", spec.status));
        context.push_str(&format!("**Description**: {}\n\n", spec.description));
        context.push_str(&format!("**Content**:\n{}\n\n", spec.content));

        // Task List
        context.push_str("## Current Tasks\n\n");
        if task_list.tasks.is_empty() {
            context.push_str("No tasks defined yet.\n\n");
        } else {
            context.push_str(&format!(
                "**Last Updated**: {}\n\n",
                task_list.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
            ));

            // Group tasks by status
            use std::collections::HashMap;
            let mut tasks_by_status: HashMap<crate::models::TaskStatus, Vec<&crate::models::Task>> =
                HashMap::new();
            for task in &task_list.tasks {
                tasks_by_status
                    .entry(task.status.clone())
                    .or_default()
                    .push(task);
            }

            let status_order = [
                crate::models::TaskStatus::Todo,
                crate::models::TaskStatus::InProgress,
                crate::models::TaskStatus::Blocked,
                crate::models::TaskStatus::Completed,
            ];

            for status in status_order.iter() {
                if let Some(tasks) = tasks_by_status.get(status) {
                    if !tasks.is_empty() {
                        context.push_str(&format!("### {:?}\n", status));
                        for task in tasks {
                            context.push_str(&format!(
                                "- **{}** (Priority: {:?})\n",
                                task.title, task.priority
                            ));
                            if !task.description.is_empty() {
                                context.push_str(&format!("  {}\n", task.description));
                            }
                            if !task.dependencies.is_empty() {
                                context.push_str(&format!(
                                    "  Dependencies: {}\n",
                                    task.dependencies.join(", ")
                                ));
                            }
                            context.push_str("\n");
                        }
                    }
                }
            }
        }

        // Notes
        context.push_str("## Notes\n\n");
        if notes.is_empty() {
            context.push_str("No notes yet.\n\n");
        } else {
            use std::collections::HashMap;
            let mut notes_by_category: HashMap<
                crate::models::NoteCategory,
                Vec<&crate::models::Note>,
            > = HashMap::new();
            for note in notes {
                notes_by_category
                    .entry(note.category.clone())
                    .or_default()
                    .push(note);
            }

            for (category, category_notes) in notes_by_category.iter() {
                context.push_str(&format!("### {:?}\n", category));
                for note in category_notes {
                    context.push_str(&format!("- {}\n", note.content));
                    context.push_str(&format!(
                        "  *Added: {}*\n\n",
                        note.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                    ));
                }
            }
        }

        context
    }
}
