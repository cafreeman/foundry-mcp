//! Timestamp generation utilities

use chrono::{Datelike, Timelike, Utc};

/// Generate ISO timestamp string
pub fn iso_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Generate timestamp for spec names (YYYYMMDD_HHMMSS)
pub fn spec_timestamp() -> String {
    let now = Utc::now();
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// Generate human-readable timestamp
pub fn human_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Parse timestamp from spec name
pub fn parse_spec_timestamp(spec_name: &str) -> Option<String> {
    if let Some((timestamp_str, _)) = spec_name.split_once('_') {
        if timestamp_str.len() >= 15 {
            // YYYYMMDD_HHMMSS is 15 chars
            Some(timestamp_str.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Format timestamp for display
pub fn format_timestamp_for_display(timestamp: &str) -> String {
    if timestamp.len() >= 15 {
        format!(
            "{}-{}-{} {}:{}:{}",
            &timestamp[0..4],
            &timestamp[4..6],
            &timestamp[6..8],
            &timestamp[9..11],
            &timestamp[11..13],
            &timestamp[13..15]
        )
    } else {
        timestamp.to_string()
    }
}
