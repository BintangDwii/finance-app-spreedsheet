use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::domain::UserRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // username
    pub role: UserRole,
    pub divisi: Option<String>,
    pub exp: usize,
}

#[tracing::instrument(skip(_state, headers))]
pub async fn jwt_auth(
    State(_state): State<crate::AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok());
    // Call once, reuse (§Function Reuse Rules): Bearer preferred, cookie fallback.
    let token = crate::utils::cookie::extract_token(auth, cookie).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header or session cookie"})),
        )
    })?;
    let secret = crate::utils::jwt::jwt_secret().map_err(|e| {
        tracing::error!(error=%e, "JWT_SECRET missing");
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Server misconfigured: JWT_SECRET not set"})),
        )
    })?;
    let token_data = decode::<Claims>(
        token.as_str(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        tracing::warn!(error=%e, "invalid token");
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid token"})),
        )
    })?;

    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}

pub fn require_role(claims: &Claims, allowed: &[UserRole]) -> bool {
    if claims.role == UserRole::Admin {
        return true;
    }
    allowed.contains(&claims.role)
}

/// DRY helper (§10): single place for Admin-only checks.
/// Returns `Ok(())` for Admin, `Err(Forbidden)` otherwise.
pub fn require_admin(claims: &Claims) -> Result<(), crate::error::AppError> {
    if claims.role == UserRole::Admin {
        Ok(())
    } else {
        Err(crate::error::AppError::Forbidden("Hanya Admin".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UserRole;

    #[test]
    fn require_role_admin_bypass() {
        let c = Claims {
            sub: "admin".into(),
            role: UserRole::Admin,
            divisi: None,
            exp: 0,
        };
        assert!(require_role(&c, &[UserRole::Maker]));
        assert!(require_admin(&c).is_ok());
    }

    #[test]
    fn require_role_checker() {
        let c = Claims {
            sub: "checker1".into(),
            role: UserRole::Checker,
            divisi: None,
            exp: 0,
        };
        assert!(require_role(&c, &[UserRole::Checker, UserRole::Approver]));
        assert!(!require_role(&c, &[UserRole::Maker]));
        assert!(require_admin(&c).is_err());
    }
}
