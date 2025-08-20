//! Base models for Project, TechStack, and Vision

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a software project with its context and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tech_stack: TechStack,
    pub vision: Vision,
}

/// Represents the technology stack used in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub tools: Vec<String>,
    pub deployment: Vec<String>,
}

/// Represents the vision and goals for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vision {
    pub overview: String,
    pub goals: Vec<String>,
    pub target_users: Vec<String>,
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
            overview: "A comprehensive project management tool for AI coding assistants".to_string(),
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
        let deserialized: TechStack = serde_json::from_str(&serialized).expect("Failed to deserialize TechStack");
        
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
        
        let serialized = serde_json::to_string(&empty_tech_stack).expect("Failed to serialize empty TechStack");
        let deserialized: TechStack = serde_json::from_str(&serialized).expect("Failed to deserialize empty TechStack");
        
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
        let deserialized: Vision = serde_json::from_str(&serialized).expect("Failed to deserialize Vision");
        
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
        
        let serialized = serde_json::to_string(&vision).expect("Failed to serialize Vision with empty overview");
        let deserialized: Vision = serde_json::from_str(&serialized).expect("Failed to deserialize Vision with empty overview");
        
        assert_eq!(vision.overview, deserialized.overview);
        assert!(deserialized.overview.is_empty());
    }

    #[test]
    fn test_project_serialization() {
        let project = create_sample_project();
        let serialized = serde_json::to_string(&project).expect("Failed to serialize Project");
        let deserialized: Project = serde_json::from_str(&serialized).expect("Failed to deserialize Project");
        
        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(project.created_at, deserialized.created_at);
        assert_eq!(project.updated_at, deserialized.updated_at);
        assert_eq!(project.tech_stack.languages, deserialized.tech_stack.languages);
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
        
        let serialized = serde_json::to_string(&project).expect("Failed to serialize Project with Unicode");
        let deserialized: Project = serde_json::from_str(&serialized).expect("Failed to deserialize Project with Unicode");
        
        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        assert_eq!(project.tech_stack.languages[0], deserialized.tech_stack.languages[0]);
        assert_eq!(project.vision.overview, deserialized.vision.overview);
    }

    #[test]
    fn test_project_json_format() {
        let project = create_sample_project();
        let serialized = serde_json::to_string_pretty(&project).expect("Failed to serialize Project as pretty JSON");
        
        assert!(serialized.contains("\"name\""));
        assert!(serialized.contains("\"test-project\""));
        assert!(serialized.contains("\"tech_stack\""));
        assert!(serialized.contains("\"vision\""));
        assert!(serialized.contains("\"languages\""));
        assert!(serialized.contains("\"Rust\""));
    }
}
