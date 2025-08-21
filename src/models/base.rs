//! Base models for Project, TechStack, and Vision
//!
//! This module provides the fundamental data structures used throughout the Project Manager MCP server.
//! These models represent the core entities: projects, their technology stacks, and business vision.
//!
//! # Examples
//!
//! ```rust
//! use foundry_mcp::models::base::{Project, TechStack, Vision};
//! use chrono::Utc;
//!
//! // Create a technology stack
//! let tech_stack = TechStack {
//!     languages: vec!["Rust".to_string(), "TypeScript".to_string()],
//!     frameworks: vec!["Actix-Web".to_string(), "React".to_string()],
//!     databases: vec!["PostgreSQL".to_string()],
//!     tools: vec!["Cargo".to_string(), "npm".to_string()],
//!     deployment: vec!["Docker".to_string(), "AWS".to_string()],
//! };
//!
//! // Create a project vision
//! let vision = Vision {
//!     overview: "A modern web application".to_string(),
//!     goals: vec!["Fast performance".to_string(), "High reliability".to_string()],
//!     target_users: vec!["Web developers".to_string()],
//!     success_criteria: vec!["< 100ms response time".to_string()],
//! };
//!
//! // Create a complete project
//! let project = Project {
//!     name: "my-web-app".to_string(),
//!     description: "A full-stack web application".to_string(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//!     tech_stack,
//!     vision,
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a software project with its context and metadata.
///
/// A `Project` is the central entity in the Project Manager MCP system. It contains
/// all the high-level information about a software project including its technical
/// context (technology stack) and business context (vision and goals).
///
/// # Fields
///
/// * `name` - A unique identifier for the project (alphanumeric with dashes/underscores)
/// * `description` - A human-readable description of what the project does
/// * `created_at` - Timestamp when the project was first created
/// * `updated_at` - Timestamp of the last modification to project metadata
/// * `tech_stack` - The technologies, frameworks, and tools used in the project
/// * `vision` - The business goals, target users, and success criteria
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::base::{Project, TechStack, Vision};
/// use chrono::Utc;
///
/// let project = Project {
///     name: "e-commerce-api".to_string(),
///     description: "REST API for an e-commerce platform".to_string(),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
///     tech_stack: TechStack {
///         languages: vec!["Rust".to_string()],
///         frameworks: vec!["Actix-Web".to_string()],
///         databases: vec!["PostgreSQL".to_string()],
///         tools: vec!["Cargo".to_string()],
///         deployment: vec!["Docker".to_string()],
///     },
///     vision: Vision {
///         overview: "Fast and reliable e-commerce backend".to_string(),
///         goals: vec!["Handle 10k concurrent users".to_string()],
///         target_users: vec!["Online shoppers".to_string()],
///         success_criteria: vec!["99.9% uptime".to_string()],
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier (alphanumeric with dashes/underscores only)
    pub name: String,
    /// Human-readable description of the project
    pub description: String,
    /// Timestamp when the project was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update to project metadata
    pub updated_at: DateTime<Utc>,
    /// Technology stack and tools used in the project
    pub tech_stack: TechStack,
    /// Business vision, goals, and success criteria
    pub vision: Vision,
}

/// Represents the technology stack used in a project.
///
/// The `TechStack` captures all the technical components and tools that make up
/// a software project. This information is crucial for AI coding assistants to
/// understand the project context and make appropriate technology choices.
///
/// # Fields
///
/// * `languages` - Programming languages used (e.g., "Rust", "TypeScript", "Python")
/// * `frameworks` - Web frameworks, libraries, or major dependencies (e.g., "React", "Actix-Web")
/// * `databases` - Database systems and storage solutions (e.g., "PostgreSQL", "Redis")
/// * `tools` - Development tools, build systems, and utilities (e.g., "Cargo", "npm", "Docker")
/// * `deployment` - Deployment platforms and infrastructure (e.g., "AWS", "Vercel", "Kubernetes")
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::base::TechStack;
///
/// // Full-stack web application
/// let web_stack = TechStack {
///     languages: vec!["TypeScript".to_string(), "Rust".to_string()],
///     frameworks: vec!["Next.js".to_string(), "Actix-Web".to_string()],
///     databases: vec!["PostgreSQL".to_string(), "Redis".to_string()],
///     tools: vec!["npm".to_string(), "Cargo".to_string(), "Docker".to_string()],
///     deployment: vec!["Vercel".to_string(), "AWS ECS".to_string()],
/// };
///
/// // Simple CLI tool
/// let cli_stack = TechStack {
///     languages: vec!["Rust".to_string()],
///     frameworks: vec!["clap".to_string()],
///     databases: vec![],
///     tools: vec!["Cargo".to_string()],
///     deployment: vec!["GitHub Releases".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    /// Programming languages used in the project
    pub languages: Vec<String>,
    /// Frameworks, libraries, and major dependencies
    pub frameworks: Vec<String>,
    /// Database systems and storage solutions
    pub databases: Vec<String>,
    /// Development tools, build systems, and utilities
    pub tools: Vec<String>,
    /// Deployment platforms and infrastructure
    pub deployment: Vec<String>,
}

/// Represents the vision and goals for a project.
///
/// The `Vision` captures the business context and objectives of a project. This helps
/// AI coding assistants understand the project's purpose and make decisions that align
/// with the intended goals and user needs.
///
/// # Fields
///
/// * `overview` - A high-level summary of what the project aims to achieve
/// * `goals` - Specific, measurable objectives for the project
/// * `target_users` - The intended audience or user base for the project
/// * `success_criteria` - Metrics or conditions that define project success
///
/// # Examples
///
/// ```rust
/// use foundry_mcp::models::base::Vision;
///
/// // E-commerce platform vision
/// let ecommerce_vision = Vision {
///     overview: "A modern, fast e-commerce platform for small businesses".to_string(),
///     goals: vec![
///         "Support 10,000 concurrent users".to_string(),
///         "Process payments securely".to_string(),
///         "Provide real-time inventory tracking".to_string(),
///     ],
///     target_users: vec![
///         "Small business owners".to_string(),
///         "Online shoppers".to_string(),
///         "Store administrators".to_string(),
///     ],
///     success_criteria: vec![
///         "99.9% uptime".to_string(),
///         "< 2 second page load times".to_string(),
///         "PCI DSS compliance".to_string(),
///     ],
/// };
///
/// // Developer tool vision
/// let dev_tool_vision = Vision {
///     overview: "A CLI tool to streamline developer workflows".to_string(),
///     goals: vec![
///         "Reduce setup time for new projects".to_string(),
///         "Standardize development practices".to_string(),
///     ],
///     target_users: vec!["Software developers".to_string()],
///     success_criteria: vec![
///         "Adopted by 100+ developers".to_string(),
///         "Reduces project setup from hours to minutes".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vision {
    /// High-level summary of what the project aims to achieve
    pub overview: String,
    /// Specific, measurable objectives for the project
    pub goals: Vec<String>,
    /// The intended audience or user base for the project
    pub target_users: Vec<String>,
    /// Metrics or conditions that define project success
    pub success_criteria: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_sample_tech_stack() -> TechStack {
        TechStack {
            languages: vec!["Rust".to_string(), "TypeScript".to_string()],
            frameworks: vec!["Actix-Web".to_string(), "React".to_string()],
            databases: vec!["PostgreSQL".to_string(), "Redis".to_string()],
            tools: vec!["Cargo".to_string(), "npm".to_string()],
            deployment: vec!["Docker".to_string(), "AWS".to_string()],
        }
    }

    fn create_sample_vision() -> Vision {
        Vision {
            overview: "A comprehensive project management tool for AI coding assistants"
                .to_string(),
            goals: vec![
                "Provide deterministic context management".to_string(),
                "Enable seamless project collaboration".to_string(),
            ],
            target_users: vec![
                "AI coding assistants".to_string(),
                "Development teams".to_string(),
            ],
            success_criteria: vec![
                "Reduces context switching time by 50%".to_string(),
                "Increases development velocity".to_string(),
            ],
        }
    }

    fn create_sample_project() -> Project {
        Project {
            name: "test-project".to_string(),
            description: "A test project for validation".to_string(),
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
            tech_stack: create_sample_tech_stack(),
            vision: create_sample_vision(),
        }
    }

    #[test]
    fn test_tech_stack_serialization() {
        let tech_stack = create_sample_tech_stack();
        let serialized = serde_json::to_string(&tech_stack).expect("Failed to serialize TechStack");
        let deserialized: TechStack =
            serde_json::from_str(&serialized).expect("Failed to deserialize TechStack");

        assert_eq!(tech_stack.languages, deserialized.languages);
        assert_eq!(tech_stack.frameworks, deserialized.frameworks);
        assert_eq!(tech_stack.databases, deserialized.databases);
        assert_eq!(tech_stack.tools, deserialized.tools);
        assert_eq!(tech_stack.deployment, deserialized.deployment);
    }

    #[test]
    fn test_tech_stack_empty_fields() {
        let empty_tech_stack = TechStack {
            languages: vec![],
            frameworks: vec![],
            databases: vec![],
            tools: vec![],
            deployment: vec![],
        };

        let serialized =
            serde_json::to_string(&empty_tech_stack).expect("Failed to serialize empty TechStack");
        let deserialized: TechStack =
            serde_json::from_str(&serialized).expect("Failed to deserialize empty TechStack");

        assert!(deserialized.languages.is_empty());
        assert!(deserialized.frameworks.is_empty());
        assert!(deserialized.databases.is_empty());
        assert!(deserialized.tools.is_empty());
        assert!(deserialized.deployment.is_empty());
    }

    #[test]
    fn test_vision_serialization() {
        let vision = create_sample_vision();
        let serialized = serde_json::to_string(&vision).expect("Failed to serialize Vision");
        let deserialized: Vision =
            serde_json::from_str(&serialized).expect("Failed to deserialize Vision");

        assert_eq!(vision.overview, deserialized.overview);
        assert_eq!(vision.goals, deserialized.goals);
        assert_eq!(vision.target_users, deserialized.target_users);
        assert_eq!(vision.success_criteria, deserialized.success_criteria);
    }

    #[test]
    fn test_vision_empty_overview() {
        let vision = Vision {
            overview: "".to_string(),
            goals: vec!["Test goal".to_string()],
            target_users: vec!["Test user".to_string()],
            success_criteria: vec!["Test criteria".to_string()],
        };

        let serialized =
            serde_json::to_string(&vision).expect("Failed to serialize Vision with empty overview");
        let deserialized: Vision = serde_json::from_str(&serialized)
            .expect("Failed to deserialize Vision with empty overview");

        assert_eq!(vision.overview, deserialized.overview);
        assert!(deserialized.overview.is_empty());
    }

    #[test]
    fn test_project_serialization() {
        let project = create_sample_project();
        let serialized = serde_json::to_string(&project).expect("Failed to serialize Project");
        let deserialized: Project =
            serde_json::from_str(&serialized).expect("Failed to deserialize Project");

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(project.created_at, deserialized.created_at);
        assert_eq!(project.updated_at, deserialized.updated_at);
        assert_eq!(
            project.tech_stack.languages,
            deserialized.tech_stack.languages
        );
        assert_eq!(project.vision.overview, deserialized.vision.overview);
    }

    #[test]
    fn test_project_with_unicode() {
        let project = Project {
            name: "test-项目".to_string(),
            description: "测试项目 with émojis 🚀".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tech_stack: TechStack {
                languages: vec!["Rust 🦀".to_string()],
                frameworks: vec![],
                databases: vec![],
                tools: vec![],
                deployment: vec![],
            },
            vision: Vision {
                overview: "Multi-language support 多语言支持".to_string(),
                goals: vec![],
                target_users: vec![],
                success_criteria: vec![],
            },
        };

        let serialized =
            serde_json::to_string(&project).expect("Failed to serialize Project with Unicode");
        let deserialized: Project =
            serde_json::from_str(&serialized).expect("Failed to deserialize Project with Unicode");

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(
            project.tech_stack.languages[0],
            deserialized.tech_stack.languages[0]
        );
        assert_eq!(project.vision.overview, deserialized.vision.overview);
    }

    #[test]
    fn test_project_json_format() {
        let project = create_sample_project();
        let serialized = serde_json::to_string_pretty(&project)
            .expect("Failed to serialize Project as pretty JSON");

        assert!(serialized.contains("\"name\""));
        assert!(serialized.contains("\"test-project\""));
        assert!(serialized.contains("\"tech_stack\""));
        assert!(serialized.contains("\"vision\""));
        assert!(serialized.contains("\"languages\""));
        assert!(serialized.contains("\"Rust\""));
    }
}
