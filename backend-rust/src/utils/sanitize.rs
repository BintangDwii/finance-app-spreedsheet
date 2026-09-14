/// Sanitize string: trim, remove control chars, strip dangerous HTML/SQL fragments
pub fn sanitize_string(s: &str) -> String {
    let trimmed = s.trim();
    // Remove control chars and strip < > ; -- for basic XSS/SQL hardening
    let filtered: String = trimmed.chars().filter(|c| !c.is_control()).collect();
    // Basic strip of script tags and SQL comment
    filtered
        .replace(['<', '>', ';'], "")
        .replace("--", "")
        .trim()
        .to_string()
}

pub fn sanitize_required(s: &str) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

pub fn catatan_or_dash(opt: Option<String>) -> String {
    let s = opt.unwrap_or_else(|| "-".to_string());
    let t = s.trim().to_string();
    if t.is_empty() {
        "-".to_string()
    } else {
        t
    }
}

pub fn sanitize_optional(opt: Option<String>) -> Option<String> {
    opt.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

pub fn sanitize_gl_code(opt: Option<String>) -> Option<String> {
    opt.map(|s| {
        s.trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .collect::<String>()
    })
    .filter(|s| !s.is_empty())
}

/// Escape `%`, `_`, `\` for ILIKE patterns (§7) — prevents wildcard injection.
pub fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '%' || c == '_' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catatan_empty_becomes_dash() {
        assert_eq!(catatan_or_dash(None), "-");
        assert_eq!(catatan_or_dash(Some("  ".into())), "-");
        assert_eq!(catatan_or_dash(Some(" hello ".into())), "hello");
    }

    #[test]
    fn escape_like_escapes_wildcards() {
        assert_eq!(escape_like("100%_x\\y"), "100\\%\\_x\\\\y");
        assert_eq!(escape_like("normal"), "normal");
    }

    #[test]
    fn gl_code_filters_dangerous_chars() {
        assert_eq!(
            sanitize_gl_code(Some("GL-100_A.1<script>".into())),
            Some("GL-100_A.1script".into())
        );
        assert_eq!(sanitize_gl_code(Some("   ".into())), None);
    }
}
