//! # Foundry MCP
//!
//! A comprehensive CLI tool and MCP server for deterministic project management
//! and AI coding assistant integration.
//!
//! This library provides the core functionality for managing project specifications
//! and maintaining context for LLM-based coding assistants.

pub mod cli;
pub mod core;
pub mod mcp;
pub mod types;
pub mod utils;

// Test environment support
#[cfg(test)]
pub mod test_environment;

// Test utilities for unit tests only
// Integration tests should use tests/common/test_utils.rs instead
#[cfg(test)]
pub mod test_utils {
    pub use crate::test_environment::TestEnvironment;
}

// Selective reexports from core modules (only what's needed for CLI functionality)
pub use crate::core::filesystem::{create_dir_all, file_exists, read_file, write_file_atomic};
pub use crate::core::project::{create_project, list_projects, load_project, project_exists};
pub use crate::core::spec::{
    create_spec, delete_spec, list_specs, load_spec, update_spec_content, validate_spec_name,
};
pub use crate::core::validation::{ContentType, parse_content_type, validate_content};
pub use crate::types::project::*;
pub use crate::types::responses::*;
pub use crate::types::spec::*;
pub use crate::utils::paths::*;
pub use crate::utils::timestamp::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the foundry directory path (~/.foundry)
pub fn foundry_dir() -> anyhow::Result<std::path::PathBuf> {
    crate::core::filesystem::foundry_dir()
}

// Compile Linear Phase D reconciliation logic and its unit tests without enabling the full backend
#[cfg(any(test, feature = "linear_backend"))]
mod linear_reconcile {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/core/backends/linear/reconcile.rs"
    ));
}

// Compile task parser and unit tests in test builds
#[cfg(any(test, feature = "linear_backend"))]
mod linear_task_parser {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/core/backends/linear/task_parser.rs"
    ));
}

// Compile Phase D orchestrator tests
#[cfg(any(test, feature = "linear_backend"))]
mod linear_phase_d {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/core/backends/linear/phase_d.rs"
    ));
}

// Compile reconciliation executor tests
#[cfg(any(test, feature = "linear_backend"))]
mod linear_executor {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/core/backends/linear/executor.rs"
    ));
}
