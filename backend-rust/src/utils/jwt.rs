use crate::error::{AppError, Result};

/// Canonical JWT secret loader — fail hard if missing (AGENTS.MD §7).
/// Single implementation (§5: no premature alias wrappers).
pub fn jwt_secret() -> Result<String> {
    let secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;
    if secret == "change-this-super-secret-jwt-key-min-32-chars" {
        tracing::warn!("JWT_SECRET is default insecure value");
    }
    Ok(secret)
}

pub fn jwt_exp_hours() -> i64 {
    std::env::var("JWT_EXP_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24)
}
