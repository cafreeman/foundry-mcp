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
        title = title.replace(&lower, a);
    }
    title.trim().to_string()
}

pub fn identity_marker() -> &'static str {
    "linear"
}

pub fn fqid(id: impl AsRef<str>) -> String {
    format!("linear:{}", id.as_ref())
}

#[cfg(test)]
mod tests {
    use super::{fqid, humanize_title, identity_marker};

    #[test]
    fn basic_humanize() {
        assert_eq!(humanize_title("linear_backend"), "Linear backend");
        assert_eq!(humanize_title("  build__API_client  "), "Build api client");
        assert_eq!(humanize_title("setup HTTP retries"), "Setup HTTP retries");
    }

    #[test]
    fn markers() {
        assert_eq!(identity_marker(), "linear");
        assert_eq!(fqid("abc123"), "linear:abc123");
    }
}

/// Extract `taskKey` from a Foundry hidden marker within a Linear body/description.
/// Expected marker format examples:
/// <!-- foundry:specId=20250921_192611_linear_backend; type=task; v=1; taskKey=add-login-flow -->
/// Returns None if no valid task marker is present.
pub fn parse_foundry_task_key_marker(body: &str) -> Option<String> {
    // Fast-path: ensure marker present
    let marker_start = body.find("<!--").and_then(|idx| {
        if body[idx..].contains("foundry:") {
            Some(idx)
        } else {
            None
        }
    })?;

    // Extract the comment segment (up to -->) to avoid scanning full body
    let tail = &body[marker_start..];
    let end_idx = tail.find("-->")?;
    let comment = &tail[..end_idx];

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

#[cfg(test)]
mod tests {
    use super::parse_foundry_task_key_marker;

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
}
