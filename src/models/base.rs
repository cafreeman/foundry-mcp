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
