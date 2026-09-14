use chrono::Utc;

pub fn generate(prefix: &str) -> String {
    format!("{}-{}", prefix, Utc::now().format("%Y%m%d%H%M%S"))
}

pub fn sanitize_reference(raw: Option<String>, prefix: &str) -> String {
    let sanitized = crate::utils::sanitize::sanitize_string(&raw.unwrap_or_default());
    if sanitized.is_empty() {
        generate(prefix)
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_prefix() {
        let generated = generate("TRF");
        assert!(generated.starts_with("TRF-"));
    }

    #[test]
    fn sanitize_fallback() {
        assert!(sanitize_reference(Some("  ".into()), "REF").starts_with("REF-"));
        assert_eq!(
            sanitize_reference(Some(" INV-123 ".into()), "REF"),
            "INV-123"
        );
    }
}
