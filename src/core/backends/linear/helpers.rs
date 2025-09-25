/// Humanize a raw feature or title string into a Linear-friendly title.
pub fn humanize_title(raw: &str) -> String {
    let s = raw.trim().replace('_', " ");
    // Collapse whitespace
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    // Simple sentence case preserving common acronyms
    let acronyms = ["API", "HTTP", "TCP", "UI", "UX", "CLI", "SDK", "ID", "URL"];
    let mut chars = out.chars();
    let first = chars
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or_default();
    let mut rest: String = chars.collect();
    rest = rest.to_ascii_lowercase();
    let mut title = String::new();
    title.push(first);
    title.push_str(&rest);
    for a in acronyms.iter() {
        let lower = a.to_ascii_lowercase();
        // Use word boundary replacement to avoid replacing substrings within words
        let words: Vec<&str> = title.split_whitespace().collect();
        let new_words: Vec<String> = words
            .iter()
            .map(|word| {
                if word.to_ascii_lowercase() == lower {
                    a.to_string()
                } else {
                    word.to_string()
                }
            })
            .collect();
        title = new_words.join(" ");
    }
    title.trim().to_string()
}

#[cfg(test)]
pub fn identity_marker() -> &'static str {
    "linear"
}

#[cfg(test)]
pub fn fqid(id: impl AsRef<str>) -> String {
    format!("linear:{}", id.as_ref())
}

/// Extract `taskKey` from a Foundry hidden marker within a Linear body/description.
/// Expected marker format examples:
/// <!-- foundry:specId=20250921_192611_linear_backend; type=task; v=1; taskKey=add-login-flow -->
/// Returns None if no valid task marker is present.
pub fn parse_foundry_task_key_marker(body: &str) -> Option<String> {
    // Fast-path: ensure marker present
    let marker_start = body.find("<!--").and_then(|idx| {
        if body[idx..].contains("foundry:") || body[idx..].contains("taskKey=") {
            Some(idx)
        } else {
            None
        }
    })?;

    // Extract the comment segment (up to -->) to avoid scanning full body
    let tail = &body[marker_start..];
    let end_idx = tail.find("-->")?;
    let comment = &tail[4..end_idx]; // Skip "<!--" (4 chars)

    // Ensure this is a task marker
    if !comment.contains("type=task") {
        return None;
    }

    // Tokenize segments after the initial prefix up to the closing
    // Format uses semicolon delimiters: foundry:specId=...; type=task; v=1; taskKey=...
    // Normalize whitespace around tokens.
    for part in comment.split(';') {
        let p = part.trim();
        if let Some(rest) = p.strip_prefix("taskKey=") {
            let key = rest.trim();
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

/// Extract `specId` from a Foundry hidden marker within a Linear body/description.
/// Expected marker format examples:
/// <!-- foundry:specId=20250921_192611_linear_backend; type=spec; v=1 -->
/// Returns None if no valid spec marker is present.
pub fn parse_foundry_spec_marker(body: &str) -> Result<Option<String>, anyhow::Error> {
    // Fast-path: ensure marker present
    let marker_start = body.find("<!--").and_then(|idx| {
        if body[idx..].contains("foundry:") || body[idx..].contains("specId=") {
            Some(idx)
        } else {
            None
        }
    });

    let marker_start = match marker_start {
        Some(start) => start,
        None => return Ok(None),
    };

    // Extract the comment segment (up to -->) to avoid scanning full body
    let tail = &body[marker_start..];
    let end_idx = match tail.find("-->") {
        Some(idx) => idx,
        None => return Ok(None),
    };
    let comment = &tail[4..end_idx]; // Skip "<!--" (4 chars)

    // Ensure this is a spec marker
    if !comment.contains("type=spec") {
        return Ok(None);
    }

    // Tokenize segments after the initial prefix up to the closing
    // Format uses semicolon delimiters: foundry:specId=...; type=spec; v=1
    // Normalize whitespace around tokens.
    for part in comment.split(';') {
        let p = part.trim();
        if let Some(rest) = p.strip_prefix("foundry:specId=") {
            let spec_id = rest.trim();
            if !spec_id.is_empty() {
                return Ok(Some(spec_id.to_string()));
            }
        } else if let Some(rest) = p.strip_prefix("specId=") {
            let spec_id = rest.trim();
            if !spec_id.is_empty() {
                return Ok(Some(spec_id.to_string()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{
        fqid, humanize_title, identity_marker, parse_foundry_spec_marker,
        parse_foundry_task_key_marker,
    };

    #[test]
    fn basic_humanize() {
        assert_eq!(humanize_title("linear_backend"), "Linear backend");
        assert_eq!(humanize_title("  build__API_client  "), "Build API client");
        assert_eq!(humanize_title("setup HTTP retries"), "Setup HTTP retries");
    }

    #[test]
    fn markers() {
        assert_eq!(identity_marker(), "linear");
        assert_eq!(fqid("abc123"), "linear:abc123");
    }

    #[test]
    fn parses_task_key_from_marker() {
        let body = "<!-- foundry:specId=20250921_192611_linear_backend; type=task; v=1; taskKey=add-login-flow -->\nTitle text";
        assert_eq!(
            parse_foundry_task_key_marker(body),
            Some("add-login-flow".into())
        );
    }

    #[test]
    fn returns_none_when_not_task_marker() {
        let body = "<!-- foundry:specId=abc; type=spec; v=1 -->\n...";
        assert_eq!(parse_foundry_task_key_marker(body), None);
    }

    #[test]
    fn returns_none_when_no_marker() {
        let body = "No markers here";
        assert_eq!(parse_foundry_task_key_marker(body), None);
    }

    #[test]
    fn parses_spec_id_from_marker() {
        let body = "<!-- foundry:specId=20250921_192611_linear_backend; type=spec; v=1 -->\nSpec content here";
        assert_eq!(
            parse_foundry_spec_marker(body).unwrap(),
            Some("20250921_192611_linear_backend".into())
        );

        // Test alternative format
        let body2 = "<!-- specId=20250922_120000_another_spec; type=spec; v=1 -->\nContent";
        assert_eq!(
            parse_foundry_spec_marker(body2).unwrap(),
            Some("20250922_120000_another_spec".into())
        );
    }

    #[test]
    fn returns_none_when_not_spec_marker() {
        let body = "<!-- foundry:specId=abc; type=task; v=1 -->\n...";
        assert_eq!(parse_foundry_spec_marker(body).unwrap(), None);
    }

    #[test]
    fn returns_none_when_no_spec_marker() {
        let body = "No markers here";
        assert_eq!(parse_foundry_spec_marker(body).unwrap(), None);
    }
}
