//! Specification models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecStatus {
    Draft,
    InProgress,
    Completed,
    OnHold,
}

/// Represents a project specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: SpecStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String,
}
