//! Core Foundry operations (storage, symlink bridge, git backup, skill install)

pub mod completed;
pub mod fsutil;
pub mod git;
pub mod link;
pub mod names;
pub mod paths;
pub mod project;
pub mod skill_install;
pub mod spec_id;
pub mod spec_meta;
pub mod spec_store;
pub mod task_parse;
pub mod workflow;

pub use paths::{foundry_dir, project_path, spec_dir_path};
