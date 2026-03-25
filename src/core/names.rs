use anyhow::{Result, bail};
use std::path::{Component, Path};

/// One filesystem path segment for spec ids and similar (no `.`, `..`, or separators).
pub fn validate_single_path_segment(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("Name cannot be empty");
    }
    if id.contains('/') || id.contains('\\') {
        bail!("Name cannot contain path separators");
    }
    let mut components = Path::new(id).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => bail!("Name must be a single path segment (not `.` or `..`)"),
    }
}

/// Validate a kebab-case identifier (project names, symlink-detected names, spec features).
///
/// Rules: lowercase letters, digits, hyphens; no leading/trailing hyphens; no `--`.
pub fn validate_kebab_case(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Name cannot be empty");
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("Name must be kebab-case (lowercase letters, digits, and hyphens only)");
    }

    if name.starts_with('-') || name.ends_with('-') {
        bail!("Name cannot start or end with a hyphen");
    }

    if name.contains("--") {
        bail!("Name cannot contain consecutive hyphens");
    }

    Ok(())
}

/// Normalize arbitrary input into kebab-case, or `None` if nothing usable remains.
pub fn normalize_to_kebab_case(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut prev_hyphen = true;

    for ch in raw.trim().chars() {
        let ch = match ch {
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            _ => '-',
        };

        if ch == '-' {
            if !prev_hyphen {
                out.push('-');
            }
            prev_hyphen = true;
        } else {
            out.push(ch);
            prev_hyphen = false;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        return None;
    }

    validate_kebab_case(&out).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_mixed_case_and_underscores() {
        assert_eq!(normalize_to_kebab_case("My_App").as_deref(), Some("my-app"));
    }

    #[test]
    fn validate_rejects_uppercase() {
        assert!(validate_kebab_case("My-app").is_err());
    }

    #[test]
    fn validate_rejects_path_segments() {
        assert!(validate_kebab_case("..").is_err());
        assert!(validate_kebab_case("foo/bar").is_err());
    }

    #[test]
    fn single_segment_accepts_spec_style_ids() {
        validate_single_path_segment("20240315_120000_auth-flow").unwrap();
    }

    #[test]
    fn single_segment_rejects_dot_dot_and_slash() {
        assert!(validate_single_path_segment("..").is_err());
        assert!(validate_single_path_segment("a/b").is_err());
        assert!(validate_single_path_segment(".").is_err());
    }
}
