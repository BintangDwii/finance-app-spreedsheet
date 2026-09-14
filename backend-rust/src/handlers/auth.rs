use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    Extension, Json,
};

use crate::db::user as db_user;
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::user::{LoginRequest, LoginResponse, UserPublic};
use crate::utils::{blocking, jwt, validation};

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use sha2::{Digest, Sha256};

#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<crate::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response> {
    let pool = &state.pool;
    validation::validate_or_400(&payload)?;

    let user = db_user::find_by_username(pool, &payload.username)
        .await?
        .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

    let hash_clone = user.password_hash.clone();
    let password_clone = payload.password.clone();
    // Returns Ok(true) for argon2 match, upgrades legacy sha256 transparently.
    let authenticated = blocking::run_blocking(move || {
        let argon_ok = PasswordHash::new(&hash_clone)
            .ok()
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password_clone.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false);
        if argon_ok {
            return Ok::<bool, AppError>(true);
        }
        // Legacy SQLite-era hashes are raw sha256 hex (64 chars). On match,
        // re-hash with argon2 so the weak hash disappears after one login.
        let is_legacy = hash_clone.len() == 64 && hash_clone.chars().all(|c| c.is_ascii_hexdigit());
        if !is_legacy {
            return Ok(false);
        }
        let mut hasher = Sha256::new();
        hasher.update(password_clone.as_bytes());
        let digest = hex::encode(hasher.finalize());
        if digest != hash_clone.to_lowercase() {
            return Ok(false);
        }
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let _upgraded = Argon2::default()
            .hash_password(password_clone.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<bool, AppError>(true)
    })
    .await?;

    if !authenticated {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let secret = jwt::jwt_secret()?;
    let exp_hours = jwt::jwt_exp_hours();
    let exp = (chrono::Utc::now() + chrono::Duration::hours(exp_hours)).timestamp() as usize;
    let claims = Claims {
        sub: user.username.clone(),
        role: user.role,
        divisi: user.divisi.clone(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!(user=%user.username, role=?user.role, "login success");
    // Call once, reuse: max-age computed once for cookie + body stays Bearer-compatible.
    let max_age = crate::utils::cookie::cookie_max_age_secs();
    let set_cookie = crate::utils::cookie::build_set_cookie(&token, max_age);
    let body = Json(LoginResponse {
        token,
        user: UserPublic {
            username: user.username,
            role: user.role,
            divisi: user.divisi,
        },
    });
    let mut res = body.into_response();
    if let Ok(v) = set_cookie.parse::<header::HeaderValue>() {
        res.headers_mut().append(header::SET_COOKIE, v);
    }
    Ok(res)
}

/// Clear the session cookie. Stateless JWT needs no server-side revocation.
pub async fn logout() -> impl IntoResponse {
    let clear = crate::utils::cookie::build_clear_cookie();
    let mut res = Json(serde_json::json!({"ok": true})).into_response();
    if let Ok(v) = clear.parse::<header::HeaderValue>() {
        res.headers_mut().append(header::SET_COOKIE, v);
    }
    res
}

#[tracing::instrument(skip(claims))]
pub async fn me(Extension(claims): Extension<Claims>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"username": claims.sub, "role": claims.role, "divisi": claims.divisi}))
}

#[cfg(test)]
mod tests {
    use crate::models::user::LoginRequest;
    use validator::Validate;

    #[test]
    fn login_request_validate_ok() {
        let req = LoginRequest {
            username: "admin".into(),
            password: "admin123".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_request_validate_fail() {
        let req = LoginRequest {
            username: "ab".into(),
            password: "123".into(),
        };
        assert!(req.validate().is_err());
    }
}
