//! Test environment utilities for isolated test runs
//!
//! This module provides a common TestEnvironment interface that is available
//! in unit tests. Integration tests use the same shared implementation.
//!
//! The shared base implementation is in test_support/test_environment_base.rs

/// Re-export shared TestEnvironment implementation for unit tests
#[cfg(test)]
mod test_impl {
    use anyhow::Result;
    use assert_fs::TempDir;
    use std::ffi::OsString;
    use std::fs;
    use std::future::Future;
    use std::path::{Path, PathBuf};

    // Unit test version - imports from crate::
    use crate::core::ops::create_project;
    use crate::core::ops::create_spec;

    // Include shared implementation but with local imports
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_support/test_environment_base.rs"
    ));

    impl TestEnvironment {
        /// Create a test project with minimal valid content (unit test version)
        pub async fn create_test_project(&self, project_name: &str) -> Result<()> {
            let input = create_project::Input {
                project_name: project_name.to_string(),
                vision: format!(
                    "## Problem Statement\n\n{} solves testing isolation and environment management for foundry-mcp development.\n\n## Target Users\n\nDevelopers working on foundry-mcp who need isolated test environments that don't interfere with each other or with production data.\n\n## Value Proposition\n\nProvides reliable, reproducible test execution with complete environment isolation using modern Rust testing patterns and assert_fs temporary directory management.",
                    project_name
                ),
                tech_stack: "## Core Technologies\n\n- **Language**: Rust for performance and safety\n- **Testing**: Integration tests with assert_fs isolation\n- **Architecture**: Component-based design for maintainability".to_string(),
                summary: format!(
                    "Test project {} created for foundry-mcp integration testing with complete environment isolation, modern testing patterns using assert_fs, and reliable reproducible test execution that prevents interference between test runs.",
                    project_name
                ),
            };

            create_project::run(input).await.map(|_| ())
        }

        /// Create a test spec with minimal valid content (unit test version)
        pub async fn create_test_spec(
            &self,
            project_name: &str,
            feature_name: &str,
            spec_content: &str,
        ) -> Result<()> {
            let input = create_spec::Input {
                project_name: project_name.to_string(),
                feature_name: feature_name.to_string(),
                spec: format!(
                    "# {}\n\n## Overview\n\n{}\n\n## Requirements\n\n- Requirement 1: Basic functionality\n- Requirement 2: Error handling\n\n## Implementation\n\nImplementation details here.\n\n## Testing\n\n- Unit tests for core functionality\n- Integration tests for API",
                    feature_name, spec_content
                ),
                tasks: format!(
                    "## Setup Phase\n\n- [ ] Create base structure for {}\n- [ ] Initialize configuration\n\n## Development Phase\n\n- [ ] Implement core functionality\n- [ ] Add error handling\n\n## Testing Phase\n\n- [ ] Write unit tests\n- [ ] Run integration tests\n- [ ] Validate implementation",
                    feature_name
                ),
                notes: format!(
                    "## Design Decisions\n\n- **Architecture**: Component-based design for {}\n- **Testing**: Comprehensive test coverage with isolation\n\n## Implementation Context\n\nThis feature provides {} functionality with proper error handling and validation.",
                    feature_name,
                    spec_content.to_lowercase()
                ),
            };

            create_spec::run(input).await.map(|_| ())
        }
    }
}

/// Export TestEnvironment for unit tests
#[cfg(test)]
pub use test_impl::TestEnvironment;
