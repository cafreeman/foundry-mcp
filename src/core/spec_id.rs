use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::path::Path;

/// Build `YYYYMMDD_HHMMSS_<feature>` using the current UTC time.
pub fn timestamp_id(feature: &str) -> String {
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    format!("{ts}_{feature}")
}

/// Parse a spec directory name into `(timestamp, feature)` if it matches the expected shape.
pub fn parse_spec_id(dir_name: &str) -> Option<(String, String)> {
    let (ts, rest) = dir_name.split_once('_')?;
    if ts.len() != 8 || !ts.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (hm, feature) = rest.split_once('_')?;
    if hm.len() != 6 || !hm.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if feature.is_empty() {
        return None;
    }
    Some((format!("{ts}_{hm}"), feature.to_string()))
}

/// Return the single matching spec directory name, or an error (ambiguous / not found).
pub fn resolve_spec_id(specs_root: &Path, partial: &str) -> Result<String> {
    let entries = std::fs::read_dir(specs_root)
        .with_context(|| format!("Failed to read {}", specs_root.display()))?;

    let mut matches = Vec::new();
    for entry in entries {
        let entry =
            entry.with_context(|| format!("Failed to read entry in {}", specs_root.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if spec_dir_matches_partial(&name, partial) {
            matches.push(name);
        }
    }

    match matches.len() {
        0 => bail!("No spec matches {:?}", partial),
        1 => Ok(matches[0].clone()),
        _ => {
            matches.sort();
            bail!(
                "Ambiguous spec id {:?}; matches:\n{}",
                partial,
                matches.join("\n")
            );
        }
    }
}

fn spec_dir_matches_partial(dir_name: &str, partial: &str) -> bool {
    if partial.is_empty() {
        return false;
    }
    if dir_name == partial || dir_name.starts_with(partial) {
        return true;
    }
    parse_spec_id(dir_name)
        .map(|(_, feature)| feature == partial)
        .unwrap_or(false)
}

/// Sort spec directory names ascending by parsed timestamp; unknown shapes sort last by name.
pub fn sort_spec_ids(mut ids: Vec<String>) -> Vec<String> {
    ids.sort_by(|a, b| {
        let ta = parse_spec_id(a)
            .map(|(t, _)| t)
            .unwrap_or_else(|| a.clone());
        let tb = parse_spec_id(b)
            .map(|(t, _)| t)
            .unwrap_or_else(|| b.clone());
        ta.cmp(&tb).then_with(|| a.cmp(b))
    });
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_shape() {
        let id = "20240315_120000_auth-flow";
        let parsed = parse_spec_id(id).expect("parse");
        assert_eq!(parsed.0, "20240315_120000");
        assert_eq!(parsed.1, "auth-flow");
    }
}
