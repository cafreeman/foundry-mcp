//! ID generation utilities

use chrono::Utc;

/// Generate a timestamped spec ID
pub fn generate_spec_id(name: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d");
    format!("{}_{}", timestamp, name)
}
