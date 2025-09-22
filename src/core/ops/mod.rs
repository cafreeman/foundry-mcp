//! Operation layer for tool-agnostic business actions

pub mod analyze_project;
pub mod create_project;
pub mod create_spec;
pub mod delete_spec;
pub mod get_foundry_help;
pub mod list_projects;
pub mod list_specs;
pub mod load_project;
pub mod load_spec;
pub mod update_spec;
pub mod validate_content;
// Additional ops will be added incrementally and wired in
