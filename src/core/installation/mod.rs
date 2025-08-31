//! Core installation infrastructure for MCP server setup

pub mod claude_code;
pub mod cursor;
pub mod json_config;
pub mod paths;
pub mod utils;

// Selective reexports from submodules
pub use claude_code::{
    get_claude_code_status, install_for_claude_code, uninstall_from_claude_code,
};

pub use cursor::{get_cursor_status, install_for_cursor, uninstall_from_cursor};

pub use json_config::{
    McpConfig, add_server_to_config, create_cursor_server_config, create_server_config,
    get_server_config, has_server_config, read_config_file, remove_server_from_config,
    validate_config, write_config_file,
};

pub use paths::{
    get_all_config_paths, get_claude_code_config_dir, get_cursor_config_dir,
    get_cursor_mcp_config_path, validate_config_dir_writable,
};

pub use utils::{
    InstallationResult, UninstallationResult, check_binary_accessible, create_installation_result,
    create_uninstallation_result, detect_binary_path, ensure_directory_exists, get_home_dir,
    validate_binary_path,
};

// Re-export types for convenience
pub use crate::types::responses::EnvironmentStatus;

/// Install Foundry MCP server for the specified target environment
pub async fn install_for_target(target: &str, force: bool) -> anyhow::Result<InstallationResult> {
    match target {
        "claude-code" => install_for_claude_code(force).await,
        "cursor" => install_for_cursor(force).await,
        _ => Err(anyhow::anyhow!(
            "Unsupported installation target: {}",
            target
        )),
    }
}

/// Uninstall Foundry MCP server from the specified target environment
pub async fn uninstall_from_target(
    target: &str,
    remove_config: bool,
    force: bool,
) -> anyhow::Result<UninstallationResult> {
    match target {
        "claude-code" => uninstall_from_claude_code(force).await,
        "cursor" => uninstall_from_cursor(remove_config, force).await,
        _ => Err(anyhow::anyhow!(
            "Unsupported uninstallation target: {}",
            target
        )),
    }
}

/// Get status for all supported environments
pub async fn get_all_environment_statuses(
    detailed: bool,
) -> anyhow::Result<Vec<EnvironmentStatus>> {
    let mut statuses = Vec::new();

    // Get status for Claude Code
    match get_claude_code_status(detailed).await {
        Ok(status) => statuses.push(status),
        Err(e) => {
            statuses.push(EnvironmentStatus {
                name: "claude-code".to_string(),
                installed: false,
                config_path: "".to_string(),
                config_exists: false,
                binary_path: "claude".to_string(),
                binary_accessible: false,
                config_content: None,
                issues: vec![format!("Failed to get status: {}", e)],
            });
        }
    }

    // Get status for Cursor
    match get_cursor_status(detailed).await {
        Ok(status) => statuses.push(status),
        Err(e) => {
            statuses.push(EnvironmentStatus {
                name: "cursor".to_string(),
                installed: false,
                config_path: "".to_string(),
                config_exists: false,
                binary_path: "unknown".to_string(),
                binary_accessible: false,
                config_content: None,
                issues: vec![format!("Failed to get status: {}", e)],
            });
        }
    }

    Ok(statuses)
}

/// Get status for a specific environment
pub async fn get_environment_status(
    target: &str,
    detailed: bool,
) -> anyhow::Result<EnvironmentStatus> {
    match target {
        "claude-code" => get_claude_code_status(detailed).await,
        "cursor" => get_cursor_status(detailed).await,
        _ => Err(anyhow::anyhow!("Unsupported status target: {}", target)),
    }
}
