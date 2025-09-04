//! String formatting utilities to eliminate code duplication

/// Format a message with a single placeholder
pub fn format_message<T: std::fmt::Display>(template: &str, value: T) -> String {
    template.replace("{}", &value.to_string())
}

/// Format a message with two placeholders
pub fn format_message_2<T1: std::fmt::Display, T2: std::fmt::Display>(
    template: &str,
    value1: T1,
    value2: T2,
) -> String {
    template
        .replace("{}", &value1.to_string())
        .replace("{}", &value2.to_string())
}

/// Format a count with proper pluralization
pub fn format_count(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {}", singular)
    } else {
        format!("{} {}", count, plural)
    }
}

/// Format a list of items with a separator and optional prefix
pub fn format_list_with_details(items: &[String], separator: &str, prefix: Option<&str>) -> String {
    if items.is_empty() {
        String::new()
    } else {
        let list_part = items.join(separator);
        if let Some(p) = prefix {
            format!("{}{}", p, list_part)
        } else {
            list_part
        }
    }
}

/// Format install command output for human-readable display
pub fn format_install_output(
    target: &str,
    binary_path: &str,
    config_path: &str,
    success: bool,
    actions_taken: &[String],
) -> String {
    let status_icon = if success { "✅" } else { "⚠️" };
    let status_text = if success {
        "Successfully installed"
    } else {
        "Partially installed"
    };

    let mut output = format!(
        "{} {} Foundry MCP for {}\n",
        status_icon, status_text, target
    );

    output.push_str(&format!("📁 Config: {}\n", config_path));
    output.push_str(&format!("🔧 Binary: {}\n", binary_path));

    if !actions_taken.is_empty() {
        output.push_str("\n📋 Actions taken:\n");
        for action in actions_taken {
            output.push_str(&format!("  • {}\n", action));
        }
    }

    if success {
        output.push_str(
            "\n🎉 Installation complete! You can now use Foundry MCP in your environment.\n",
        );
    } else {
        output.push_str(
            "\n⚠️  Installation completed with warnings. Check the actions above for details.\n",
        );
    }

    output
}

/// Format uninstall command output for human-readable display
pub fn format_uninstall_output(
    target: &str,
    config_path: &str,
    success: bool,
    actions_taken: &[String],
    files_removed: &[String],
) -> String {
    let status_icon = if success { "✅" } else { "⚠️" };
    let status_text = if success {
        "Successfully uninstalled"
    } else {
        "Partially uninstalled"
    };

    let mut output = format!(
        "{} {} Foundry MCP from {}\n",
        status_icon, status_text, target
    );

    output.push_str(&format!("📁 Config: {}\n", config_path));

    if !actions_taken.is_empty() {
        output.push_str("\n📋 Actions taken:\n");
        for action in actions_taken {
            output.push_str(&format!("  • {}\n", action));
        }
    }

    if !files_removed.is_empty() {
        output.push_str("\n🗑️  Files removed:\n");
        for file in files_removed {
            output.push_str(&format!("  • {}\n", file));
        }
    }

    if success {
        output.push_str(
            "\n🎉 Uninstallation complete! Foundry MCP has been removed from your environment.\n",
        );
    } else {
        output.push_str(
            "\n⚠️  Uninstallation completed with warnings. Check the actions above for details.\n",
        );
    }

    output
}
