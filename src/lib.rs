//! # Foundry MCP
//!
//! A comprehensive CLI tool and MCP server for deterministic project management
//! and AI coding assistant integration.
//!
//! This library provides the core functionality for managing project specifications
//! and maintaining context for LLM-based coding assistants.

pub mod cli;
pub mod core;
pub mod types;
pub mod utils;

pub use crate::core::filesystem::*;
pub use crate::core::project::*;
pub use crate::core::spec::*;
pub use crate::core::validation::*;
pub use crate::types::project::*;
pub use crate::types::responses::*;
pub use crate::types::spec::*;
pub use crate::utils::paths::*;
pub use crate::utils::timestamp::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the foundry directory path (~/.foundry)
pub fn foundry_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".foundry")
}
