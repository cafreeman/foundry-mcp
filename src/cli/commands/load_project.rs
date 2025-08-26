//! Implementation of the load_project command

use crate::cli::args::LoadProjectArgs;
use crate::core::{filesystem, project};
use crate::types::responses::{FoundryResponse, LoadProjectResponse, ProjectContext};

use anyhow::{Context, Result};
use std::fs;

pub async fn execute(args: LoadProjectArgs) -> Result<FoundryResponse<LoadProjectResponse>> {
    // Validate project exists
    validate_project_exists(&args.project_name)?;

    // Load project data
    let project_path = project::get_project_path(&args.project_name)?;
    let project_context = load_project_context(&args.project_name, &project_path)?;
    let specs_available = project_context.specs_available.clone();

    // Build response
    let response_data = LoadProjectResponse {
        project: project_context,
    };

    // Determine validation status based on specs availability
    let validation_status = if specs_available.is_empty() {
        crate::types::responses::ValidationStatus::Incomplete
    } else {
        crate::types::responses::ValidationStatus::Complete
    };

    Ok(crate::types::responses::FoundryResponse {
        data: response_data,
        next_steps: generate_next_steps(&args.project_name, &specs_available),
        validation_status,
        workflow_hints: generate_workflow_hints(&specs_available),
    })
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

/// Load project context from files
fn load_project_context(
    project_name: &str,
    project_path: &std::path::Path,
) -> Result<ProjectContext> {
    // Read project files - handle missing files gracefully
    let vision =
        filesystem::read_file(project_path.join("vision.md")).unwrap_or_else(|_| String::new());
    let tech_stack =
        filesystem::read_file(project_path.join("tech-stack.md")).unwrap_or_else(|_| String::new());
    let summary =
        filesystem::read_file(project_path.join("summary.md")).unwrap_or_else(|_| String::new());

    // Get creation time from directory metadata
    let created_at = fs::metadata(project_path)
        .and_then(|metadata| metadata.created())
        .map_err(anyhow::Error::from)
        .and_then(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .map_err(anyhow::Error::from)
        })
        .map(|duration| {
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    // Scan specs directory for available specifications
    let specs_dir = project_path.join("specs");
    let specs_available = if specs_dir.exists() {
        fs::read_dir(&specs_dir)
            .context("Failed to read specs directory")?
            .filter_map(|entry| {
                entry
                    .ok()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(ProjectContext {
        name: project_name.to_string(),
        vision,
        tech_stack,
        summary,
        specs_available,
        created_at,
    })
}

/// Generate next steps based on available specs
fn generate_next_steps(project_name: &str, specs_available: &[String]) -> Vec<String> {
    if specs_available.is_empty() {
        vec![
            "Project loaded successfully but contains no specifications".to_string(),
            format!(
                "Create your first specification: foundry create-spec {} <feature_name>",
                project_name
            ),
            "Use the loaded project context to guide development".to_string(),
        ]
    } else {
        vec![
            format!(
                "Project loaded with {} specification(s)",
                specs_available.len()
            ),
            format!(
                "Load a specific spec: foundry load-spec {} <spec_name>",
                project_name
            ),
            format!(
                "Create a new spec: foundry create-spec {} <feature_name>",
                project_name
            ),
        ]
    }
}

/// Generate workflow hints based on available specs
fn generate_workflow_hints(specs_available: &[String]) -> Vec<String> {
    let mut hints = vec![
        "Use project summary for quick context in conversations".to_string(),
        "Full vision provides comprehensive background and goals".to_string(),
        "Tech stack details guide implementation decisions".to_string(),
    ];

    if specs_available.is_empty() {
        hints.push("Consider creating specifications to track specific features".to_string());
    } else {
        hints.push(format!("Available specs: {}", specs_available.join(", ")));
        hints.push("Load individual specs to see detailed implementation plans".to_string());
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::filesystem;
    use tempfile::TempDir;

    fn create_test_args(project_name: &str) -> LoadProjectArgs {
        LoadProjectArgs {
            project_name: project_name.to_string(),
        }
    }

    #[test]
    fn test_validate_project_exists_missing_project() {
        let result = validate_project_exists("non-existent-project-12345");

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("not found"));
        assert!(error_message.contains("list-projects"));
    }

    #[test]
    fn test_generate_next_steps_no_specs() {
        let project_name = "test-project";
        let specs_available = Vec::<String>::new();
        let steps = generate_next_steps(project_name, &specs_available);

        assert_eq!(steps.len(), 3);
        assert!(steps[0].contains("contains no specifications"));
        assert!(steps[1].contains("foundry create-spec"));
        assert!(steps[1].contains(project_name));
        assert!(steps[2].contains("guide development"));
    }

    #[test]
    fn test_generate_next_steps_with_specs() {
        let project_name = "test-project";
        let specs_available = vec![
            "20240824_120000_feature1".to_string(),
            "20240824_130000_feature2".to_string(),
        ];
        let steps = generate_next_steps(project_name, &specs_available);

        assert_eq!(steps.len(), 3);
        assert!(steps[0].contains("loaded with 2 specification"));
        assert!(steps[1].contains("foundry load-spec"));
        assert!(steps[1].contains(project_name));
        assert!(steps[2].contains("foundry create-spec"));
        assert!(steps[2].contains(project_name));
    }

    #[test]
    fn test_generate_workflow_hints_no_specs() {
        let specs_available = Vec::<String>::new();
        let hints = generate_workflow_hints(&specs_available);

        assert!(hints.len() >= 4);
        assert!(hints.iter().any(|h| h.contains("project summary")));
        assert!(hints.iter().any(|h| h.contains("Full vision")));
        assert!(hints.iter().any(|h| h.contains("Tech stack")));
        assert!(
            hints
                .iter()
                .any(|h| h.contains("Consider creating specifications"))
        );
        // Should not contain spec-specific hints
        assert!(!hints.iter().any(|h| h.contains("Available specs")));
        assert!(!hints.iter().any(|h| h.contains("Load individual specs")));
    }

    #[test]
    fn test_generate_workflow_hints_with_specs() {
        let specs_available = vec![
            "20240824_120000_feature1".to_string(),
            "20240824_130000_feature2".to_string(),
        ];
        let hints = generate_workflow_hints(&specs_available);

        assert!(hints.len() >= 5);
        assert!(hints.iter().any(|h| h.contains("project summary")));
        assert!(hints.iter().any(|h| h.contains("Full vision")));
        assert!(hints.iter().any(|h| h.contains("Tech stack")));
        assert!(hints.iter().any(|h| h.contains("Available specs")));
        assert!(hints.iter().any(|h| h.contains("feature1")));
        assert!(hints.iter().any(|h| h.contains("feature2")));
        assert!(hints.iter().any(|h| h.contains("Load individual specs")));
        // Should not contain no-specs hints
        assert!(
            !hints
                .iter()
                .any(|h| h.contains("Consider creating specifications"))
        );
    }

    #[tokio::test]
    async fn test_execute_with_missing_project() {
        let args = create_test_args("non-existent-project");
        let result = execute(args).await;

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("not found"));
    }

    #[test]
    fn test_load_project_context_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = "test-project-incomplete";
        let project_path = temp_dir.path();

        // Create minimal project structure without files
        filesystem::create_dir_all(&project_path).unwrap();
        filesystem::create_dir_all(project_path.join("specs")).unwrap();

        let context = load_project_context(project_name, &project_path).unwrap();

        assert_eq!(context.name, project_name);
        // Should handle missing files gracefully with empty strings
        assert!(context.vision.is_empty());
        assert!(context.tech_stack.is_empty());
        assert!(context.summary.is_empty());
        assert!(context.specs_available.is_empty());
        assert!(!context.created_at.is_empty()); // Should still have timestamp
    }

    #[test]
    fn test_load_project_context_with_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_name = "test-project-with-specs";
        let project_path = temp_dir.path();
        let specs_dir = project_path.join("specs");

        // Create project structure with specs
        filesystem::create_dir_all(&specs_dir).unwrap();

        // Create spec directories
        let spec1_dir = specs_dir.join("20240824_120000_feature1");
        let spec2_dir = specs_dir.join("20240824_130000_feature2");
        filesystem::create_dir_all(&spec1_dir).unwrap();
        filesystem::create_dir_all(&spec2_dir).unwrap();

        let context = load_project_context(project_name, &project_path).unwrap();

        assert_eq!(context.name, project_name);
        assert_eq!(context.specs_available.len(), 2);
        assert!(
            context
                .specs_available
                .contains(&"20240824_120000_feature1".to_string())
        );
        assert!(
            context
                .specs_available
                .contains(&"20240824_130000_feature2".to_string())
        );
    }
}
