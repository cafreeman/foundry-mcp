//! Core business logic modules

pub mod filesystem;
pub mod installation;
pub mod project;
pub mod spec;
pub mod templates;
pub mod validation;

// Selective reexports from filesystem module
pub use filesystem::{
    create_dir_all, ensure_foundry_dir, file_exists, foundry_dir, read_file, write_file_atomic,
};

// Selective reexports from installation module
pub use installation::{
    InstallationResult, UninstallationResult, check_binary_accessible, create_installation_result,
    create_server_config, create_uninstallation_result, detect_binary_path,
    get_all_environment_statuses, get_claude_code_status, get_cursor_status,
    get_environment_status, install_for_claude_code, install_for_cursor, install_for_target,
    read_config_file, uninstall_from_claude_code, uninstall_from_cursor, uninstall_from_target,
    write_config_file,
};

// Selective reexports from other modules
pub use project::*;
pub use spec::*;
pub use templates::*;
pub use validation::*;
