//! Human-readable status formatting utilities

use crate::types::responses::{EnvironmentStatus, StatusResponse};
use console::{Style, style};

/// Format status response for human-readable CLI output
pub fn format_status_output(response: &StatusResponse, detailed: bool) -> String {
    let mut output = Vec::new();

    // Header
    output.push(format!(
        "{}",
        style("Foundry MCP Server Status").bold().cyan()
    ));
    output.push(String::new());

    // Binary status
    let binary_status = if response.binary_found {
        style("✓").green().bold()
    } else {
        style("✗").red().bold()
    };

    output.push(format!(
        "{} Binary: {} {}",
        binary_status,
        style(&response.binary_path).dim(),
        if response.binary_found {
            style("(accessible)").green()
        } else {
            style("(not found)").red()
        }
    ));
    output.push(String::new());

    // Environment statuses
    output.push(format!("{}", style("Environment Status:").bold()));

    for env in &response.environments {
        output.push(format_environment_status(env, detailed));
    }

    // Summary
    let installed_count = response.environments.iter().filter(|e| e.installed).count();
    let total_count = response.environments.len();

    output.push(String::new());
    let summary_style = if installed_count == total_count {
        Style::new().green().bold()
    } else if installed_count > 0 {
        Style::new().yellow().bold()
    } else {
        Style::new().red().bold()
    };

    output.push(format!(
        "{}",
        summary_style.apply_to(format!(
            "Summary: {}/{} environments installed",
            installed_count, total_count
        ))
    ));

    output.join("\n")
}

/// Format individual environment status
fn format_environment_status(env: &EnvironmentStatus, detailed: bool) -> String {
    let mut lines = Vec::new();

    // Main status line
    let status_icon = if env.installed {
        style("✓").green().bold()
    } else {
        style("✗").red().bold()
    };

    let env_name = style(&env.name).bold();
    let status_text = if env.installed {
        style("installed").green()
    } else {
        style("not installed").red()
    };

    lines.push(format!("  {} {} ({})", status_icon, env_name, status_text));

    // Issues (always show if present)
    if !env.issues.is_empty() {
        for issue in &env.issues {
            lines.push(format!(
                "    {} {}",
                style("!").yellow().bold(),
                style(issue).yellow()
            ));
        }
    }

    // Detailed information
    if detailed {
        lines.push(format!("    Config: {}", style(&env.config_path).dim()));

        let config_status = if env.config_exists {
            style("exists").green()
        } else {
            style("missing").red()
        };
        lines.push(format!("    Config Status: {}", config_status));

        let binary_status = if env.binary_accessible {
            style("accessible").green()
        } else {
            style("not accessible").red()
        };
        lines.push(format!(
            "    Binary: {} ({})",
            style(&env.binary_path).dim(),
            binary_status
        ));

        // Show config content if available and not too verbose
        if let Some(config) = &env.config_content
            && config.len() < 500
        {
            // Only show short configs in detailed mode
            lines.push("    Config Content:".to_string());
            for line in config.lines().take(10) {
                // Limit to 10 lines
                lines.push(format!("      {}", style(line).dim()));
            }
        }

        lines.push(String::new()); // Add spacing between detailed entries
    }

    lines.join("\n")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_status_output_basic() {
        let response = StatusResponse {
            binary_path: "/usr/local/bin/foundry".to_string(),
            binary_found: true,
            environments: vec![EnvironmentStatus {
                name: "claude-code".to_string(),
                installed: true,
                config_path: "/home/user/.config/claude/config.json".to_string(),
                config_exists: true,
                binary_path: "/usr/local/bin/foundry".to_string(),
                binary_accessible: true,
                config_content: None,
                issues: vec![],
            }],
        };

        let output = format_status_output(&response, false);

        // Basic structure tests
        assert!(output.contains("Foundry MCP Server Status"));
        assert!(output.contains("claude-code"));
        assert!(output.contains("Summary: 1/1 environments installed"));
    }

    #[test]
    fn test_format_environment_with_issues() {
        let env = EnvironmentStatus {
            name: "problematic-env".to_string(),
            installed: false,
            config_path: "/missing/config.json".to_string(),
            config_exists: false,
            binary_path: "missing-binary".to_string(),
            binary_accessible: false,
            config_content: None,
            issues: vec![
                "Binary not found".to_string(),
                "Config file missing".to_string(),
            ],
        };

        let output = format_environment_status(&env, false);

        assert!(output.contains("problematic-env"));
        assert!(output.contains("Binary not found"));
        assert!(output.contains("Config file missing"));
    }

}
