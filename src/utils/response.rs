//! Response building utilities to eliminate code duplication

use crate::types::responses::{FoundryResponse, ValidationStatus};

/// Build a standard success response with common patterns
pub fn build_success_response<T>(
    data: T,
    next_steps: Vec<String>,
    workflow_hints: Vec<String>,
) -> FoundryResponse<T> {
    FoundryResponse {
        data,
        next_steps,
        validation_status: ValidationStatus::Complete,
        workflow_hints,
    }
}

/// Build a response with incomplete validation status
pub fn build_incomplete_response<T>(
    data: T,
    next_steps: Vec<String>,
    workflow_hints: Vec<String>,
) -> FoundryResponse<T> {
    FoundryResponse {
        data,
        next_steps,
        validation_status: ValidationStatus::Incomplete,
        workflow_hints,
    }
}

/// Create a single-item vector with a formatted message
pub fn single_message<T: ToString>(message: T) -> Vec<String> {
    vec![message.to_string()]
}

/// Create a vector with multiple formatted messages
pub fn multiple_messages<T: ToString>(messages: &[T]) -> Vec<String> {
    messages.iter().map(|m| m.to_string()).collect()
}

/// Format a list of items with a separator
pub fn format_list(items: &[String], separator: &str) -> String {
    items.join(separator)
}

/// Format a list with a custom prefix
pub fn format_list_with_prefix(items: &[String], prefix: &str, separator: &str) -> String {
    if items.is_empty() {
        String::new()
    } else {
        format!("{}{}", prefix, items.join(separator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::responses::ValidationStatus;

    #[test]
    fn test_build_success_response() {
        let data = "test data";
        let next_steps = vec!["step 1".to_string(), "step 2".to_string()];
        let workflow_hints = vec!["hint 1".to_string()];

        let response = build_success_response(data, next_steps.clone(), workflow_hints.clone());

        assert_eq!(response.data, "test data");
        assert_eq!(response.next_steps, next_steps);
        assert_eq!(response.workflow_hints, workflow_hints);
        assert!(matches!(
            response.validation_status,
            ValidationStatus::Complete
        ));
    }

    #[test]
    fn test_build_incomplete_response() {
        let data = 42;
        let next_steps = vec!["complete this".to_string()];
        let workflow_hints = vec!["missing info".to_string()];

        let response = build_incomplete_response(data, next_steps.clone(), workflow_hints.clone());

        assert_eq!(response.data, 42);
        assert_eq!(response.next_steps, next_steps);
        assert_eq!(response.workflow_hints, workflow_hints);
        assert!(matches!(
            response.validation_status,
            ValidationStatus::Incomplete
        ));
    }

    #[test]
    fn test_single_message() {
        let message = "test message";
        let result = single_message(message);

        assert_eq!(result, vec!["test message".to_string()]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_multiple_messages() {
        let messages = &["msg1", "msg2", "msg3"];
        let result = multiple_messages(messages);

        assert_eq!(
            result,
            vec!["msg1".to_string(), "msg2".to_string(), "msg3".to_string()]
        );
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_format_list() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        assert_eq!(format_list(&items, ", "), "a, b, c");
        assert_eq!(format_list(&items, "|"), "a|b|c");
        assert_eq!(format_list(&items, " -> "), "a -> b -> c");
    }
}
