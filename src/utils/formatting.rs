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
