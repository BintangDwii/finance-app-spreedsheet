use axum::{extract::State, Extension, Json};

use crate::db::user as db_user;
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::user::{LoginRequest, LoginResponse, UserPublic};
use crate::utils::{blocking, jwt, validation};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{encode, EncodingKey, Header};

#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<crate::AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let pool = &state.pool;
    validation::validate_or_400(&payload)?;

    let user = db_user::find_by_username(pool, &payload.username)
        .await?
        .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

    let hash_clone = user.password_hash.clone();
    let password_clone = payload.password.clone();
    let valid = blocking::run_blocking(move || {
        let parsed = PasswordHash::new(&hash_clone)
            .map_err(|_| AppError::Unauthorized("Invalid hash".into()))?;
        Ok::<bool, AppError>(
            Argon2::default()
                .verify_password(password_clone.as_bytes(), &parsed)
                .is_ok(),
        )
    })
    .await?;

    if !valid {
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
    Ok(Json(LoginResponse {
        token,
        user: UserPublic {
            username: user.username,
            role: user.role,
            divisi: user.divisi,
        },
    }))
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
