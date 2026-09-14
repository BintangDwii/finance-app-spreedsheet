//! Cookie session helpers — single source of truth (§10 DRY).
//!
//! Auth strategy: `POST /api/auth/login` issues a JWT both as JSON `token`
//! (legacy Bearer clients, e.g. Streamlit) AND as `HttpOnly` cookie `jwt`
//! (Leptos SPA same-origin). `jwt_auth` middleware accepts either, Bearer
//! first then cookie fallback. No new dependency: raw `Cookie`/`Set-Cookie`
//! header parsing only.

/// Canonical session cookie name.
pub const SESSION_COOKIE: &str = "jwt";

/// Max-age seconds for the session cookie (mirrors `JWT_EXP_HOURS`).
pub fn cookie_max_age_secs() -> i64 {
    let hours: i64 = std::env::var("JWT_EXP_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);
    hours.max(1) * 3600
}

/// Extract Bearer token, falling back to `jwt=` cookie (§7: never panic).
pub fn extract_token(authorization: Option<&str>, cookie_header: Option<&str>) -> Option<String> {
    if let Some(auth) = authorization {
        let t = auth.strip_prefix("Bearer ").unwrap_or(auth).trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let cookies = cookie_header?;
    for part in cookies.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            let v = v.trim().trim_matches('"');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Build a `Set-Cookie` value for login. `Secure` only when `COOKIE_SECURE=1`.
pub fn build_set_cookie(token: &str, max_age_secs: i64) -> String {
    let secure = std::env::var("COOKIE_SECURE").as_deref() == Ok("1");
    let mut c =
        format!("{SESSION_COOKIE}={token}; HttpOnly; Path=/; Max-Age={max_age_secs}; SameSite=Lax");
    if secure {
        c.push_str("; Secure");
    }
    c
}

/// Build a clearing `Set-Cookie` value for logout.
pub fn build_clear_cookie() -> String {
    format!("{SESSION_COOKIE}=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax")
}

/// Validate a `file_bukti` reference: allow only safe filenames/paths with
/// pdf/jpg/jpeg/png extension (§7). Returns the sanitized filename.
pub fn validate_bukti_filename(raw: &str) -> Result<String, crate::error::AppError> {
    let name = raw.trim().replace('\\', "/");
    let base = name.rsplit('/').next().unwrap_or("").trim();
    if base.is_empty() || base.len() > 255 {
        return Err(crate::error::AppError::Validation(
            "Nama file bukti tidak valid".into(),
        ));
    }
    if base.contains("..") {
        return Err(crate::error::AppError::Validation(
            "Nama file bukti tidak valid".into(),
        ));
    }
    let lower = base.to_lowercase();
    let ok = ["pdf", "jpg", "jpeg", "png"]
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")));
    if !ok {
        return Err(crate::error::AppError::Validation(
            "Bukti harus pdf/jpg/jpeg/png".into(),
        ));
    }
    let safe: String = base
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    if safe.is_empty() {
        return Err(crate::error::AppError::Validation(
            "Nama file bukti tidak valid".into(),
        ));
    }
    Ok(safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_preferred_over_cookie() {
        let t = extract_token(Some("Bearer abc"), Some("jwt=xyz"));
        assert_eq!(t.as_deref(), Some("abc"));
    }

    #[test]
    fn cookie_fallback() {
        let t = extract_token(None, Some("a=1; jwt=tok123; b=2"));
        assert_eq!(t.as_deref(), Some("tok123"));
    }

    #[test]
    fn missing_everything_is_none() {
        assert_eq!(extract_token(None, None), None);
        assert_eq!(extract_token(Some(""), Some("a=1")), None);
    }

    #[test]
    fn set_cookie_flags() {
        let c = build_set_cookie("t", 3600);
        assert!(c.contains("HttpOnly"));
        assert!(c.contains("SameSite=Lax"));
        assert!(c.contains("Max-Age=3600"));
    }

    #[test]
    fn bukti_validation() {
        assert!(validate_bukti_filename("bukti-invoice.pdf").is_ok());
        assert!(validate_bukti_filename("../etc/passwd").is_err());
        assert!(validate_bukti_filename("nota.exe").is_err());
        assert!(validate_bukti_filename("").is_err());
    }
}
