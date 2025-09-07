//! Implementation of the list_specs command

use crate::cli::args::ListSpecsArgs;
use crate::core::{project, spec};
use crate::types::responses::{FoundryResponse, ListSpecsResponse, SpecInfo};
use crate::utils::response::{build_incomplete_response, build_success_response};
use anyhow::{Context, Result};

pub async fn execute(args: ListSpecsArgs) -> Result<FoundryResponse<ListSpecsResponse>> {
    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Get specs for the project
    let specs = spec::list_specs(&args.project_name)
        .with_context(|| format!("Failed to list specs for project '{}'", args.project_name))?;

    // Convert to response format
    let spec_infos: Vec<SpecInfo> = specs
        .into_iter()
        .map(|spec_meta| SpecInfo {
            name: spec_meta.name,
            feature_name: spec_meta.feature_name,
            created_at: spec_meta.created_at,
        })
        .collect();

    let response_data = ListSpecsResponse {
        project_name: args.project_name.clone(),
        specs: spec_infos.clone(),
        total_count: spec_infos.len(),
    };

    // Generate appropriate response based on spec count
    if response_data.specs.is_empty() {
        let next_steps = vec![
            "No specifications found for this project - ready for specification creation"
                .to_string(),
            format!(
                "You can create your first specification: foundry create-spec {} <feature_name>",
                args.project_name
            ),
            "You can use 'foundry load-project' to see full project context".to_string(),
        ];

        let workflow_hints = vec![
            "You can start by creating specifications to track development features".to_string(),
            "Each spec includes implementation notes and task lists for comprehensive planning"
                .to_string(),
        ];

        Ok(build_incomplete_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    } else {
        let spec_count = response_data.specs.len();
        let mut next_steps = vec![
            format!("Found {} specification(s) in project", spec_count),
            format!(
                "You can load a specific spec: foundry load-spec {} <spec_name>",
                args.project_name
            ),
        ];

        if spec_count <= 5 {
            next_steps.push("Available specs:".to_string());
            for spec in &response_data.specs {
                next_steps.push(format!("  - {} ({})", spec.name, spec.feature_name));
            }
        }

        next_steps.push(format!(
            "You can create a new spec: foundry create-spec {} <feature_name>",
            args.project_name
        ));

        let workflow_hints = vec![
            "Specifications are timestamped and organized by feature for easy navigation"
                .to_string(),
            format!("Total specs: {}", spec_count),
            "You can load individual specs to see detailed implementation plans".to_string(),
            "Specs include specification content, notes, and task lists for complete context"
                .to_string(),
        ];

        Ok(build_success_response(
            response_data,
            next_steps,
            workflow_hints,
        ))
    }
}

/// Validate that project exists
fn validate_project_exists(project_name: &str) -> Result<()> {
    if !project::project_exists(project_name)? {
        return Err(anyhow::anyhow!(
            "Project '{}' not found. Use 'foundry list-projects' to see available projects.",
            project_name
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestEnvironment;
    use crate::types::responses::ValidationStatus;
    use crate::types::spec::{SpecConfig, SpecContentData};

    #[tokio::test]
    async fn test_execute_with_existing_project() {
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test_list_specs_project";

        // Create project first
        let project_config = crate::types::project::ProjectConfig {
            name: project_name.to_string(),
            vision: "Test vision content".to_string(),
            tech_stack: "Test tech stack".to_string(),
            summary: "Test summary".to_string(),
        };
        crate::core::project::create_project(project_config).unwrap();

        // Create a test spec
        let spec_config = SpecConfig {
            project_name: project_name.to_string(),
            feature_name: "test_feature".to_string(),
            content: SpecContentData {
                spec: "Test specification".to_string(),
                notes: "Test notes".to_string(),
                tasks: "- Test task".to_string(),
            },
        };
        crate::core::spec::create_spec(spec_config).unwrap();

        // Test list_specs command
        let args = ListSpecsArgs {
            project_name: project_name.to_string(),
        };

        let result = execute(args).await.unwrap();
        assert_eq!(result.data.project_name, project_name);
        assert_eq!(result.data.total_count, 1);
        assert_eq!(result.data.specs.len(), 1);
        assert_eq!(result.data.specs[0].feature_name, "test_feature");
        assert_eq!(result.validation_status, ValidationStatus::Complete);
    }

    #[tokio::test]
    async fn test_execute_with_empty_project() {
        let _env = TestEnvironment::new().unwrap();
        let project_name = "test_empty_project";

        // Create project first
        let project_config = crate::types::project::ProjectConfig {
            name: project_name.to_string(),
            vision: "Test vision content".to_string(),
            tech_stack: "Test tech stack".to_string(),
            summary: "Test summary".to_string(),
        };
        crate::core::project::create_project(project_config).unwrap();

        // Test list_specs command
        let args = ListSpecsArgs {
            project_name: project_name.to_string(),
        };

        let result = execute(args).await.unwrap();
        assert_eq!(result.data.project_name, project_name);
        assert_eq!(result.data.total_count, 0);
        assert_eq!(result.data.specs.len(), 0);
        assert_eq!(result.validation_status, ValidationStatus::Incomplete);
    }

    #[tokio::test]
    async fn test_execute_with_nonexistent_project() {
        let _env = TestEnvironment::new().unwrap();
        let project_name = "nonexistent_project";

        let args = ListSpecsArgs {
            project_name: project_name.to_string(),
        };

        let result = execute(args).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Project 'nonexistent_project' not found")
        );
    }
}
