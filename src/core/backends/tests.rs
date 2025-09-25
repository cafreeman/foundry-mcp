//! Backend testing infrastructure using modern testing patterns
//!
//! This module provides backend contract tests and utilities following
//! the established modern testing approach with assert_fs + temp-env.

#[cfg(test)]
mod test_utils {
    use anyhow::Result;
    use assert_fs::TempDir;
    use std::future::Future;

    use crate::types::{
        project::ProjectConfig,
        spec::{SpecConfig, SpecContentData},
    };

    /// Modern test environment using assert_fs + temp-env
    /// Follows the established testing patterns without global mutexes
    pub struct TestEnvironment {
        pub temp_dir: TempDir,
    }

    impl TestEnvironment {
        /// Create a new test environment with isolated directory
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            Ok(TestEnvironment { temp_dir })
        }

        /// Get the foundry directory path within the test environment
        pub fn foundry_dir(&self) -> std::path::PathBuf {
            self.temp_dir.path().join(".foundry")
        }

        /// Execute async test logic with proper environment isolation
        /// Uses temp-env for scoped environment variable management
        pub fn with_env_async<F, Fut, T>(&self, f: F) -> T
        where
            F: FnOnce() -> Fut,
            Fut: Future<Output = T>,
        {
            let home_dir = self.temp_dir.path().to_string_lossy().to_string();

            // Use temp-env for scoped environment variables
            temp_env::with_var("HOME", Some(&home_dir), || {
                // Create a new single-threaded runtime for simplicity and isolation
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime for test");
                rt.block_on(f())
            })
        }

        /// Create valid test config for create_project
        pub fn create_project_config(&self, project_name: &str) -> ProjectConfig {
            ProjectConfig {
                name: project_name.to_string(),
                vision: "This is a comprehensive test vision that meets all validation requirements. It describes a revolutionary software project that aims to solve complex problems in the development workflow. The project targets developers and teams who need better tooling for managing project contexts and specifications. Our unique value proposition includes seamless AI integration and deterministic project management.".to_string(),
                tech_stack: "This project leverages Rust as the primary programming language for its performance and safety guarantees. We use clap for CLI argument parsing, serde for JSON serialization, anyhow for error handling, and chrono for timestamp management. The architecture follows modular design principles with clear separation between CLI interfaces, core business logic, and utility functions.".to_string(),
                summary: "A comprehensive Rust-based project management CLI tool that creates structured contexts for AI-assisted software development with atomic file operations and rich JSON responses.".to_string(),
            }
        }

        /// Create valid test config for create_spec
        pub fn create_spec_config(&self, project_name: &str, feature_name: &str) -> SpecConfig {
            SpecConfig {
                project_name: project_name.to_string(),
                feature_name: feature_name.to_string(),
                content: SpecContentData {
                    spec: "# Feature Name\n\n## Overview\nThis specification defines a comprehensive feature implementation that includes detailed requirements, functional specifications, and behavioral expectations.\n\n## Requirements\nThe feature should integrate seamlessly with existing system architecture while providing robust error handling and user-friendly interfaces. Implementation should follow established patterns and include proper testing coverage.".to_string(),
                    notes: "# Implementation Notes\n\n## Security Considerations\nImplementation notes include important considerations for security, performance, and maintainability.\n\n## Error Handling\nSpecial attention should be paid to error handling and edge cases.\n\n## Dependencies\nConsider using established libraries where appropriate and ensure compatibility with existing system components.".to_string(),
                    tasks: "## Tasks\n- [ ] Create feature scaffolding and basic structure\n- [ ] Implement core functionality with proper error handling\n- [ ] Add comprehensive test coverage for all scenarios\n- [ ] Update documentation and user guides\n- [ ] Perform integration testing with existing features\n- [ ] Conduct code review and optimization".to_string(),
                },
            }
        }
    }
}

#[cfg(test)]
mod contract_tests {
    use super::test_utils::TestEnvironment;
    use crate::core::backends::{
        FoundryBackend, filesystem::FilesystemBackend, memory::InMemoryBackend,
    };
    use crate::types::spec::SpecFileType;
    use anyhow::Result;

    /// Contract test that verifies FoundryBackend trait conformance
    /// This test runs against multiple backend implementations to ensure consistency
    async fn test_backend_contract<B: FoundryBackend>(backend: B) -> Result<()> {
        // Test project operations
        let project_config = crate::types::project::ProjectConfig {
            name: "contract-test".to_string(),
            vision:
                "Test vision that meets validation requirements for comprehensive testing purposes"
                    .to_string(),
            tech_stack: "Test tech stack with detailed description for validation requirements"
                .to_string(),
            summary:
                "Test summary that meets the minimum length requirements for project validation"
                    .to_string(),
        };

        // Create project
        let project = backend.create_project(project_config.clone()).await?;
        assert_eq!(project.name, "contract-test");
        assert!(project.vision.is_some());
        assert!(project.tech_stack.is_some());
        assert!(project.summary.is_some());

        // Test project exists
        assert!(backend.project_exists("contract-test").await?);
        assert!(!backend.project_exists("nonexistent").await?);

        // Test list projects
        let projects = backend.list_projects().await?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "contract-test");

        // Test load project
        let loaded_project = backend.load_project("contract-test").await?;
        assert_eq!(loaded_project.name, project.name);
        // Note: We don't compare created_at exactly due to potential precision differences between backends
        assert!(!loaded_project.created_at.is_empty());

        // Test spec operations
        let spec_config = crate::types::spec::SpecConfig {
            project_name: "contract-test".to_string(),
            feature_name: "test_feature".to_string(),
            content: crate::types::spec::SpecContentData {
                spec: "# Test Feature\n\n## Requirements\nTest spec content for validation"
                    .to_string(),
                notes: "Test notes content that meets validation requirements".to_string(),
                tasks: "## Tasks\n- [ ] Test task for validation purposes".to_string(),
            },
        };

        // Create spec
        let spec = backend.create_spec(spec_config).await?;
        assert_eq!(spec.project_name, "contract-test");
        assert!(spec.name.contains("test_feature"));

        // Test list specs
        let specs = backend.list_specs("contract-test").await?;
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].project_name, "contract-test");

        // Test load spec
        let loaded_spec = backend.load_spec("contract-test", &spec.name).await?;
        assert_eq!(loaded_spec.name, spec.name);
        assert_eq!(loaded_spec.content.spec, spec.content.spec);

        // Test update spec content
        backend
            .update_spec_content(
                "contract-test",
                &spec.name,
                SpecFileType::Spec,
                "Updated spec content for testing purposes",
            )
            .await?;

        // Verify update
        let updated_spec = backend.load_spec("contract-test", &spec.name).await?;
        assert_eq!(
            updated_spec.content.spec,
            "Updated spec content for testing purposes"
        );

        // Test helper operations
        let latest_spec = backend.get_latest_spec("contract-test").await?;
        assert!(latest_spec.is_some());
        assert_eq!(latest_spec.unwrap().name, spec.name);

        let spec_count = backend.count_specs("contract-test").await?;
        assert_eq!(spec_count, 1);

        // Test delete spec
        backend.delete_spec("contract-test", &spec.name).await?;
        let specs_after_delete = backend.list_specs("contract-test").await?;
        assert_eq!(specs_after_delete.len(), 0);

        // Test capabilities
        let capabilities = backend.capabilities();
        assert!(capabilities.supports_documents);
        assert!(capabilities.atomic_replace);
        assert!(capabilities.strong_consistency);

        Ok(())
    }

    #[test]
    fn test_memory_backend_contract() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();
            test_backend_contract(backend).await.unwrap();
        });
    }

    #[test]
    fn test_filesystem_backend_contract() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = FilesystemBackend::new();
            test_backend_contract(backend).await.unwrap();
        });
    }

    #[test]
    fn test_backend_error_handling() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();

            // Test loading nonexistent project
            let result = backend.load_project("nonexistent").await;
            assert!(result.is_err());

            // Test creating spec in nonexistent project
            let spec_config = crate::types::spec::SpecConfig {
                project_name: "nonexistent".to_string(),
                feature_name: "test".to_string(),
                content: crate::types::spec::SpecContentData {
                    spec: "Test".to_string(),
                    notes: "Test".to_string(),
                    tasks: "Test".to_string(),
                },
            };
            let result = backend.create_spec(spec_config).await;
            assert!(result.is_err());

            // Test duplicate project creation
            let project_config = crate::types::project::ProjectConfig {
                name: "test".to_string(),
                vision: "Test vision".to_string(),
                tech_stack: "Test tech".to_string(),
                summary: "Test summary".to_string(),
            };
            backend
                .create_project(project_config.clone())
                .await
                .unwrap();
            let result = backend.create_project(project_config).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_backend_sorting_and_ordering() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();

            // Create multiple projects with slight time differences
            for i in 0..3 {
                let config = crate::types::project::ProjectConfig {
                    name: format!("project-{}", i),
                    vision: "Test vision for ordering validation".to_string(),
                    tech_stack: "Test tech stack for validation".to_string(),
                    summary: "Test summary for validation".to_string(),
                };
                backend.create_project(config).await.unwrap();

                // Small delay to ensure different timestamps
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            // Test that projects are sorted newest first
            let projects = backend.list_projects().await.unwrap();
            assert_eq!(projects.len(), 3);

            // The last created project should be first (newest first)
            assert_eq!(projects[0].name, "project-2");
            assert_eq!(projects[1].name, "project-1");
            assert_eq!(projects[2].name, "project-0");

            // Verify created_at timestamps are in descending order
            for i in 0..projects.len() - 1 {
                assert!(projects[i].created_at >= projects[i + 1].created_at);
            }
        });
    }
}

#[cfg(test)]
mod filesystem_backend_tests {
    use super::test_utils::TestEnvironment;
    use crate::core::backends::{FoundryBackend, filesystem::FilesystemBackend};
    use crate::types::spec::SpecFileType;

    #[test]
    fn test_filesystem_backend_creates_directories() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = FilesystemBackend::new();

            let config = env.create_project_config("fs-test");
            let _project = backend.create_project(config).await.unwrap();

            // Verify directory structure exists
            let foundry_dir = env.foundry_dir();
            let project_dir = foundry_dir.join("fs-test");
            let specs_dir = project_dir.join("specs");

            assert!(foundry_dir.exists());
            assert!(project_dir.exists());
            assert!(specs_dir.exists());

            // Verify project files exist
            assert!(project_dir.join("vision.md").exists());
            assert!(project_dir.join("tech-stack.md").exists());
            assert!(project_dir.join("summary.md").exists());
        });
    }

    #[test]
    fn test_filesystem_backend_atomic_writes() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = FilesystemBackend::new();

            // Create project and spec
            let project_config = env.create_project_config("atomic-test");
            backend.create_project(project_config).await.unwrap();

            let spec_config = env.create_spec_config("atomic-test", "atomic_feature");
            let spec = backend.create_spec(spec_config).await.unwrap();

            // Update spec content multiple times rapidly
            for i in 0..10 {
                let content = format!("Updated content iteration {}", i);
                backend
                    .update_spec_content("atomic-test", &spec.name, SpecFileType::Spec, &content)
                    .await
                    .unwrap();
            }

            // Verify final state is consistent
            let final_spec = backend.load_spec("atomic-test", &spec.name).await.unwrap();
            assert!(
                final_spec
                    .content
                    .spec
                    .contains("Updated content iteration")
            );
        });
    }

    #[test]
    fn test_filesystem_backend_file_validation() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = FilesystemBackend::new();

            let config = env.create_project_config("validation-test");
            let _project = backend.create_project(config).await.unwrap();

            let foundry_dir = env.foundry_dir();
            let project_dir = foundry_dir.join("validation-test");

            // Verify file content directly
            let vision_file = project_dir.join("vision.md");
            assert!(vision_file.exists());

            let vision_content = std::fs::read_to_string(&vision_file).unwrap();
            assert!(vision_content.contains("revolutionary software project"));
        });
    }
}

#[cfg(test)]
mod facade_integration_tests {
    use super::test_utils::TestEnvironment;
    use crate::core::backends::memory::InMemoryBackend;
    use crate::core::foundry::Foundry;
    use crate::types::spec::SpecFileType;

    #[test]
    fn test_facade_with_memory_backend() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();
            let foundry = Foundry::new(backend);

            // Test project operations through facade
            let config = env.create_project_config("facade-test");
            let project = foundry.create_project(config).await.unwrap();

            assert_eq!(project.name, "facade-test");
            assert!(foundry.project_exists("facade-test").await.unwrap());

            // Test spec operations through facade
            let spec_config = env.create_spec_config("facade-test", "facade_feature");
            let spec = foundry.create_spec(spec_config).await.unwrap();

            assert!(spec.name.contains("facade_feature"));

            // Test domain logic methods
            let spec_name = Foundry::<InMemoryBackend>::generate_spec_name("test_feature");
            assert!(spec_name.contains("test_feature"));

            let validation_result = Foundry::<InMemoryBackend>::validate_spec_name(&spec.name);
            assert!(validation_result.is_ok());

            // Test fuzzy matching
            let match_result = foundry
                .find_spec_match("facade-test", "facade")
                .await
                .unwrap();
            use crate::core::spec::SpecMatchStrategy;
            match match_result {
                SpecMatchStrategy::FeatureFuzzy(matched_name) => {
                    assert_eq!(matched_name, spec.name);
                }
                _ => panic!("Expected fuzzy match for 'facade'"),
            }
        });
    }

    #[test]
    fn test_facade_spec_content_store() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();
            let foundry = Foundry::new(backend);

            // Create project and spec
            let project_config = env.create_project_config("store-test");
            foundry.create_project(project_config).await.unwrap();

            let spec_config = env.create_spec_config("store-test", "store_feature");
            let spec = foundry.create_spec(spec_config).await.unwrap();

            // Test SpecContentStore implementation
            use crate::core::backends::SpecContentStore;
            let original_content = foundry
                .read_spec_file("store-test", &spec.name, SpecFileType::Spec)
                .await
                .unwrap();

            let new_content = "Updated content via SpecContentStore";
            foundry
                .write_spec_file("store-test", &spec.name, SpecFileType::Spec, new_content)
                .await
                .unwrap();

            let updated_content = foundry
                .read_spec_file("store-test", &spec.name, SpecFileType::Spec)
                .await
                .unwrap();
            assert_eq!(updated_content, new_content);

            // Test is_file_modified
            let is_modified = foundry
                .is_file_modified(
                    "store-test",
                    &spec.name,
                    SpecFileType::Spec,
                    &original_content,
                )
                .await
                .unwrap();
            assert!(is_modified);

            let is_not_modified = foundry
                .is_file_modified("store-test", &spec.name, SpecFileType::Spec, new_content)
                .await
                .unwrap();
            assert!(!is_not_modified);
        });
    }
}

#[cfg(test)]
mod edit_engine_integration_tests {
    use super::test_utils::TestEnvironment;
    use crate::core::backends::memory::InMemoryBackend;
    use crate::core::foundry::Foundry;
    use crate::types::edit_commands::{
        EditCommand, EditCommandName, EditCommandTarget, EditSelector, TaskStatus,
    };

    #[test]
    fn test_edit_engine_with_facade_and_backend() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();
            let foundry = Foundry::new(backend);

            // Setup project and spec
            let project_config = env.create_project_config("edit-test");
            foundry.create_project(project_config).await.unwrap();

            let spec_config = env.create_spec_config("edit-test", "edit_feature");
            let spec = foundry.create_spec(spec_config).await.unwrap();

            // Test edit commands through facade
            let commands = vec![EditCommand {
                target: EditCommandTarget::Tasks,
                command: EditCommandName::SetTaskStatus,
                selector: EditSelector::TaskText {
                    value: "Create feature scaffolding and basic structure".to_string(),
                    section_context: None,
                },
                status: Some(TaskStatus::Done),
                content: None,
            }];

            let result = foundry
                .apply_edit_commands("edit-test", &spec.name, &commands)
                .await
                .unwrap();

            assert_eq!(result.applied_count, 1);
            assert_eq!(result.errors.len(), 0);

            // Verify the change was applied
            let updated_spec = foundry.load_spec("edit-test", &spec.name).await.unwrap();
            assert!(
                updated_spec
                    .content
                    .tasks
                    .contains("- [x] Create feature scaffolding and basic structure")
            );
        });
    }

    #[test]
    fn test_edit_engine_idempotency() {
        let env = TestEnvironment::new().unwrap();
        env.with_env_async(|| async {
            let backend = InMemoryBackend::new();
            let foundry = Foundry::new(backend);

            // Setup
            let project_config = env.create_project_config("idempotent-test");
            foundry.create_project(project_config).await.unwrap();

            let spec_config = env.create_spec_config("idempotent-test", "idempotent_feature");
            let spec = foundry.create_spec(spec_config).await.unwrap();

            // Apply same command twice
            let commands = vec![EditCommand {
                target: EditCommandTarget::Tasks,
                command: EditCommandName::UpsertTask,
                selector: EditSelector::TaskText {
                    value: "New test task".to_string(),
                    section_context: None,
                },
                status: None,
                content: Some("- [ ] New test task".to_string()),
            }];

            // First application
            let result1 = foundry
                .apply_edit_commands("idempotent-test", &spec.name, &commands)
                .await
                .unwrap();
            assert_eq!(result1.applied_count, 1);
            assert_eq!(result1.skipped_idempotent_count, 0);

            // Second application should be idempotent
            let result2 = foundry
                .apply_edit_commands("idempotent-test", &spec.name, &commands)
                .await
                .unwrap();
            assert_eq!(result2.applied_count, 0);
            assert_eq!(result2.skipped_idempotent_count, 1);

            // Verify content wasn't duplicated
            let final_spec = foundry
                .load_spec("idempotent-test", &spec.name)
                .await
                .unwrap();
            let task_count = final_spec.content.tasks.matches("New test task").count();
            assert_eq!(task_count, 1);
        });
    }
}
