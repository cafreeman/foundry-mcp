//! Validation utilities to eliminate code duplication

/// Create a single validation error message
pub fn single_error<T: ToString>(message: T) -> Vec<String> {
    vec![message.to_string()]
}

/// Create a single validation suggestion message
pub fn single_suggestion<T: ToString>(message: T) -> Vec<String> {
    vec![message.to_string()]
}

/// Create validation error messages from a condition and message
pub fn conditional_error<T: ToString>(condition: bool, message: T) -> Vec<String> {
    if condition {
        single_error(message)
    } else {
        Vec::new()
    }
}

/// Create validation suggestion messages from a condition and message
pub fn conditional_suggestion<T: ToString>(condition: bool, message: T) -> Vec<String> {
    if condition {
        single_suggestion(message)
    } else {
        Vec::new()
    }
}

/// Create multiple validation suggestions from conditions and messages
pub fn conditional_suggestions(conditions_and_messages: &[(bool, &str)]) -> Vec<String> {
    conditions_and_messages
        .iter()
        .filter_map(|(condition, message)| {
            if *condition {
                Some(message.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Format validation error with content type prefix
pub fn format_validation_error(content_type: &str, error: &str) -> String {
    format!("{}: {}", content_type, error)
}

/// Format validation suggestion with content type prefix
pub fn format_validation_suggestion(content_type: &str, suggestion: &str) -> String {
    format!("{}: {}", content_type, suggestion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conditional_error_true() {
        let errors = conditional_error(true, "This should appear");
        assert_eq!(errors, vec!["This should appear".to_string()]);
    }

    #[test]
    fn test_conditional_error_false() {
        let errors = conditional_error(false, "This should not appear");
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn test_conditional_suggestions_mixed() {
        let conditions = &[
            (true, "Should appear"),
            (false, "Should not appear"),
            (true, "Should also appear"),
        ];

        let suggestions = conditional_suggestions(conditions);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"Should appear".to_string()));
        assert!(suggestions.contains(&"Should also appear".to_string()));
    }

    #[test]
    fn test_format_validation_error() {
        let error = format_validation_error("Vision", "Content too short");
        assert_eq!(error, "Vision: Content too short");
    }
}
