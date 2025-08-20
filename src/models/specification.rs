//! Specification models for feature specifications and requirements
//!
//! This module provides data structures for managing project specifications. Specifications
//! represent detailed requirements, features, or implementation plans for specific aspects
//! of a project. Each specification tracks its progress through defined status states.
//!
//! # Examples
//!
//! ```rust
//! use project_manager_mcp::models::specification::{Specification, SpecStatus};
//! use chrono::Utc;
//!
//! // Create a new specification
//! let spec = Specification {
//!     id: "20240115_user_authentication".to_string(),
//!     name: "User Authentication System".to_string(),
//!     description: "OAuth2 and JWT-based authentication".to_string(),
//!     status: SpecStatus::Draft,
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//!     content: "# User Authentication\n\n## Requirements\n- OAuth2 support\n- JWT tokens".to_string(),
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a specification throughout its lifecycle.
///
/// Specifications progress through different states as they are developed and implemented.
/// This enum tracks the current state of a specification to help teams understand
/// progress and coordinate work.
///
/// # States
///
/// * `Draft` - Initial state for new specifications under development
/// * `InProgress` - Specification is actively being implemented or worked on
/// * `Completed` - Implementation is finished and the specification is fulfilled
/// * `OnHold` - Work is paused temporarily due to dependencies or priorities
///
/// # Examples
///
/// ```rust
/// use project_manager_mcp::models::specification::SpecStatus;
///
/// // New specification starts as draft
/// let status = SpecStatus::Draft;
///
/// // Progress through implementation
/// let status = SpecStatus::InProgress;
///
/// // Mark as completed when done
/// let status = SpecStatus::Completed;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpecStatus {
    /// Initial state for new specifications under development
    Draft,
    /// Specification is actively being implemented or worked on
    InProgress,
    /// Implementation is finished and the specification is fulfilled
    Completed,
    /// Work is paused temporarily due to dependencies or priorities
    OnHold,
}

/// Represents a project specification with detailed requirements and implementation details.
///
/// A `Specification` captures the detailed requirements, design decisions, and implementation
/// plans for a specific feature or component of a project. Specifications are stored as
/// structured documents with markdown content and are tracked through various status states.
///
/// # ID Format
///
/// Specification IDs follow the format `YYYYMMDD_snake_case_name` where:
/// - `YYYYMMDD` is the creation date (e.g., "20240115")
/// - `snake_case_name` describes the specification (e.g., "user_authentication")
/// - Example: "20240115_user_authentication"
///
/// # Fields
///
/// * `id` - Unique identifier in YYYYMMDD_name format
/// * `name` - Human-readable title for the specification
/// * `description` - Brief summary of what the specification covers
/// * `status` - Current progress state (Draft, InProgress, Completed, OnHold)
/// * `created_at` - Timestamp when the specification was created
/// * `updated_at` - Timestamp of the last modification
/// * `content` - Full specification content in Markdown format
///
/// # Examples
///
/// ```rust
/// use project_manager_mcp::models::specification::{Specification, SpecStatus};
/// use chrono::Utc;
///
/// // Create a new API specification
/// let api_spec = Specification {
///     id: "20240115_rest_api".to_string(),
///     name: "REST API Design".to_string(),
///     description: "RESTful API endpoints for user management".to_string(),
///     status: SpecStatus::Draft,
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
///     content: "# REST API Specification\n\n## Overview\nThis specification defines the REST API endpoints for user management.\n\n## Endpoints\n\n### GET /users\n- Returns list of all users\n- Supports pagination via query parameters\n\n### POST /users\n- Creates a new user\n- Requires JSON body with user data\n\n## Authentication\nAll endpoints require JWT token in Authorization header.".to_string(),
/// };
///
/// // Create a database schema specification
/// let db_spec = Specification {
///     id: "20240116_database_schema".to_string(),
///     name: "Database Schema Design".to_string(),
///     description: "PostgreSQL schema for user and product data".to_string(),
///     status: SpecStatus::InProgress,
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
///     content: "# Database Schema\n\n## User Table\n```sql\nCREATE TABLE users (\n    id SERIAL PRIMARY KEY,\n    email VARCHAR(255) UNIQUE NOT NULL,\n    password_hash VARCHAR(255) NOT NULL,\n    created_at TIMESTAMP DEFAULT NOW()\n);\n```".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    /// Unique identifier in YYYYMMDD_name format (e.g., "20240115_user_auth")
    pub id: String,
    /// Human-readable title for the specification
    pub name: String,
    /// Brief summary of what the specification covers
    pub description: String,
    /// Current progress state of the specification
    pub status: SpecStatus,
    /// Timestamp when the specification was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last modification
    pub updated_at: DateTime<Utc>,
    /// Full specification content in Markdown format
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_sample_specification() -> Specification {
        Specification {
            id: "20240101_test_spec".to_string(),
            name: "Test Specification".to_string(),
            description: "A test specification for validation".to_string(),
            status: SpecStatus::Draft,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
            content: "# Test Specification\n\nThis is a test specification.".to_string(),
        }
    }

    #[test]
    fn test_spec_status_serialization() {
        let statuses = [
            SpecStatus::Draft,
            SpecStatus::InProgress,
            SpecStatus::Completed,
            SpecStatus::OnHold,
        ];

        for status in &statuses {
            let serialized = serde_json::to_string(status).expect("Failed to serialize SpecStatus");
            let deserialized: SpecStatus = serde_json::from_str(&serialized).expect("Failed to deserialize SpecStatus");
            assert_eq!(*status, deserialized);
        }
    }

    #[test]
    fn test_spec_status_json_values() {
        assert_eq!(serde_json::to_string(&SpecStatus::Draft).unwrap(), "\"Draft\"");
        assert_eq!(serde_json::to_string(&SpecStatus::InProgress).unwrap(), "\"InProgress\"");
        assert_eq!(serde_json::to_string(&SpecStatus::Completed).unwrap(), "\"Completed\"");
        assert_eq!(serde_json::to_string(&SpecStatus::OnHold).unwrap(), "\"OnHold\"");
    }

    #[test]
    fn test_specification_serialization() {
        let spec = create_sample_specification();
        let serialized = serde_json::to_string(&spec).expect("Failed to serialize Specification");
        let deserialized: Specification = serde_json::from_str(&serialized).expect("Failed to deserialize Specification");

        assert_eq!(spec.id, deserialized.id);
        assert_eq!(spec.name, deserialized.name);
        assert_eq!(spec.description, deserialized.description);
        assert_eq!(spec.status, deserialized.status);
        assert_eq!(spec.created_at, deserialized.created_at);
        assert_eq!(spec.updated_at, deserialized.updated_at);
        assert_eq!(spec.content, deserialized.content);
    }

    #[test]
    fn test_specification_empty_content() {
        let spec = Specification {
            id: "20240101_empty_spec".to_string(),
            name: "Empty Spec".to_string(),
            description: "".to_string(),
            status: SpecStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: "".to_string(),
        };

        let serialized = serde_json::to_string(&spec).expect("Failed to serialize empty Specification");
        let deserialized: Specification = serde_json::from_str(&serialized).expect("Failed to deserialize empty Specification");

        assert_eq!(spec.description, deserialized.description);
        assert_eq!(spec.content, deserialized.content);
        assert!(deserialized.description.is_empty());
        assert!(deserialized.content.is_empty());
    }

    #[test]
    fn test_specification_markdown_content() {
        let spec = Specification {
            id: "20240101_markdown_spec".to_string(),
            name: "Markdown Spec".to_string(),
            description: "Spec with markdown content".to_string(),
            status: SpecStatus::InProgress,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: r#"# Main Title

## Section 1
- Item 1
- Item 2

## Section 2
```rust
fn hello() {
    println!("Hello, world!");
}
```

### Subsection
**Bold text** and *italic text*.

> Blockquote example

[Link](https://example.com)"#.to_string(),
        };

        let serialized = serde_json::to_string(&spec).expect("Failed to serialize Markdown Specification");
        let deserialized: Specification = serde_json::from_str(&serialized).expect("Failed to deserialize Markdown Specification");

        assert_eq!(spec.content, deserialized.content);
        assert!(deserialized.content.contains("# Main Title"));
        assert!(deserialized.content.contains("```rust"));
        assert!(deserialized.content.contains("println!"));
    }

    #[test]
    fn test_specification_special_characters() {
        let spec = Specification {
            id: "20240101_special_chars".to_string(),
            name: "Special Characters 特殊字符 🎯".to_string(),
            description: "Testing Unicode: αβγ, 中文, 日本語, Emoji: 🚀🦀".to_string(),
            status: SpecStatus::Completed,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: "Content with quotes \"double\" and 'single', newlines\nand tabs\t.".to_string(),
        };

        let serialized = serde_json::to_string(&spec).expect("Failed to serialize Specification with special characters");
        let deserialized: Specification = serde_json::from_str(&serialized).expect("Failed to deserialize Specification with special characters");

        assert_eq!(spec.name, deserialized.name);
        assert_eq!(spec.description, deserialized.description);
        assert_eq!(spec.content, deserialized.content);
    }

    #[test]
    fn test_specification_id_format_validation() {
        let spec = create_sample_specification();
        
        // Test valid ID format: YYYYMMDD_snake_case
        assert!(spec.id.starts_with("20240101_"));
        assert!(spec.id.contains("_"));
        
        let parts: Vec<&str> = spec.id.split('_').collect();
        assert!(parts.len() >= 2);
        assert_eq!(parts[0].len(), 8); // YYYYMMDD format
    }

    #[test]
    fn test_specification_status_transitions() {
        let mut spec = create_sample_specification();
        
        // Test status transitions
        assert_eq!(spec.status, SpecStatus::Draft);
        
        spec.status = SpecStatus::InProgress;
        let serialized = serde_json::to_string(&spec).unwrap();
        let deserialized: Specification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.status, SpecStatus::InProgress);
        
        spec.status = SpecStatus::Completed;
        let serialized = serde_json::to_string(&spec).unwrap();
        let deserialized: Specification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.status, SpecStatus::Completed);
        
        spec.status = SpecStatus::OnHold;
        let serialized = serde_json::to_string(&spec).unwrap();
        let deserialized: Specification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.status, SpecStatus::OnHold);
    }
}
