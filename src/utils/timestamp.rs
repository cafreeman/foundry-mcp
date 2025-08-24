//! Timestamp generation and parsing utilities

use anyhow::Result;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

/// Generate ISO timestamp string for general use
pub fn iso_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Generate timestamp for spec names (YYYYMMDD_HHMMSS format)
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

/// Validate timestamp format (YYYYMMDD_HHMMSS)
pub fn validate_timestamp_format(timestamp: &str) -> bool {
    if timestamp.len() != 15 {
        return false;
    }

    // Check format: 8 digits, underscore, 6 digits
    let parts: Vec<&str> = timestamp.split('_').collect();
    if parts.len() != 2 || parts[0].len() != 8 || parts[1].len() != 6 {
        return false;
    }

    // Validate all characters are digits
    parts[0].chars().all(|c| c.is_ascii_digit()) && parts[1].chars().all(|c| c.is_ascii_digit())
}

/// Parse timestamp from spec name with validation
pub fn parse_spec_timestamp(spec_name: &str) -> Option<String> {
    if let Some(underscore_pos) = spec_name.find('_') {
        if underscore_pos == 8 && spec_name.len() > 15 {
            let timestamp_part = &spec_name[0..15];
            if validate_timestamp_format(timestamp_part) {
                return Some(timestamp_part.to_string());
            }
        }
    }
    None
}

/// Convert spec timestamp to ISO format
pub fn spec_timestamp_to_iso(spec_timestamp: &str) -> Result<String> {
    if !validate_timestamp_format(spec_timestamp) {
        return Err(anyhow::anyhow!(
            "Invalid timestamp format: {}",
            spec_timestamp
        ));
    }

    let date_part = &spec_timestamp[0..8];
    let time_part = &spec_timestamp[9..15];

    let year: i32 = date_part[0..4].parse()?;
    let month: u32 = date_part[4..6].parse()?;
    let day: u32 = date_part[6..8].parse()?;
    let hour: u32 = time_part[0..2].parse()?;
    let minute: u32 = time_part[2..4].parse()?;
    let second: u32 = time_part[4..6].parse()?;

    let naive_date = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| anyhow::anyhow!("Invalid date values in timestamp"))?;
    let naive_datetime = naive_date
        .and_hms_opt(hour, minute, second)
        .ok_or_else(|| anyhow::anyhow!("Invalid time values in timestamp"))?;

    let datetime = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
    Ok(datetime.to_rfc3339())
}

/// Convert ISO timestamp to spec format
pub fn iso_to_spec_timestamp(iso_timestamp: &str) -> Result<String> {
    let datetime = DateTime::parse_from_rfc3339(iso_timestamp)?;
    let utc_datetime = datetime.with_timezone(&Utc);

    Ok(format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        utc_datetime.year(),
        utc_datetime.month(),
        utc_datetime.day(),
        utc_datetime.hour(),
        utc_datetime.minute(),
        utc_datetime.second()
    ))
}

/// Format timestamp for human-readable display
pub fn format_timestamp_for_display(timestamp: &str) -> String {
    if validate_timestamp_format(timestamp) {
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

/// Extract feature name from spec name
pub fn extract_feature_name(spec_name: &str) -> Option<String> {
    if let Some(timestamp) = parse_spec_timestamp(spec_name) {
        if spec_name.len() > timestamp.len() + 1 {
            return Some(spec_name[timestamp.len() + 1..].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_timestamp_format() {
        assert!(validate_timestamp_format("20240824_120000"));
        assert!(!validate_timestamp_format("2024824_120000")); // Too short date
        assert!(!validate_timestamp_format("20240824_12000")); // Too short time
        assert!(!validate_timestamp_format("20240824120000")); // Missing underscore
        assert!(!validate_timestamp_format("2024082a_120000")); // Invalid character
    }

    #[test]
    fn test_parse_spec_timestamp() {
        assert_eq!(
            parse_spec_timestamp("20240824_120000_user_auth"),
            Some("20240824_120000".to_string())
        );
        assert_eq!(parse_spec_timestamp("invalid_name"), None);
        assert_eq!(parse_spec_timestamp("20240824_user_auth"), None); // Invalid format
    }

    #[test]
    fn test_spec_timestamp_to_iso() {
        let result = spec_timestamp_to_iso("20240824_120000").unwrap();
        assert!(result.starts_with("2024-08-24T12:00:00"));
    }

    #[test]
    fn test_extract_feature_name() {
        assert_eq!(
            extract_feature_name("20240824_120000_user_auth"),
            Some("user_auth".to_string())
        );
        assert_eq!(
            extract_feature_name("20240824_120000_multi_part_feature"),
            Some("multi_part_feature".to_string())
        );
        assert_eq!(extract_feature_name("invalid_name"), None);
    }

    #[test]
    fn test_format_timestamp_for_display() {
        assert_eq!(
            format_timestamp_for_display("20240824_120000"),
            "2024-08-24 12:00:00"
        );
        assert_eq!(format_timestamp_for_display("invalid"), "invalid");
    }
}
