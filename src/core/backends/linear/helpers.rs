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
    let first = chars.next().map(|c| c.to_ascii_uppercase()).unwrap_or_default();
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
    use super::{humanize_title, fqid, identity_marker};

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
