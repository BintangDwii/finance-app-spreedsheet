use axum::{
    extract::{Extension, Path, State},
    Json,
};

use crate::domain::AccountId;
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::account::{CreateAccountRequest, UpdateAccountStatusRequest};
use crate::repository::{account_repo::PgAccountRepository, CrudRepository, Repository};
use crate::utils::{sanitize, validation};

#[tracing::instrument(skip(state, claims))]
pub async fn list(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user=%claims.sub, "list accounts");
    // Canonical read path via Repository (§2) — db stays as low-level executor.
    let repo = PgAccountRepository::new(state.pool.clone());
    let rows = repo.list_all().await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[tracing::instrument(skip(state, claims))]
pub async fn create(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    crate::middleware::auth::require_admin(&claims)?;
    validation::validate_or_400(&payload)?;
    let bank_name = sanitize::sanitize_string(&payload.bank_name);
    let no_rekening = sanitize::sanitize_string(&payload.no_rekening);
    if bank_name.is_empty() || no_rekening.is_empty() {
        return Err(AppError::Validation("bank_name/no_rekening wajib".into()));
    }
    let id = crate::db::account::create_account(
        pool,
        &bank_name,
        &no_rekening,
        payload.currency,
        payload.opening_balance,
        payload.limit_overdraft.unwrap_or(0.0),
        sanitize::sanitize_gl_code(payload.gl_code),
        payload
            .status
            .unwrap_or(crate::domain::AccountStatus::Aktif),
    )
    .await
    .map_err(|e| {
        if crate::db::common::is_unique_violation(&e) {
            AppError::Conflict("Rekening sudah ada".into())
        } else {
            AppError::Database(e)
        }
    })?;
    tracing::info!(id, bank=%bank_name, "account created");
    Ok(Json(serde_json::json!({"id": AccountId(id)})))
}

#[tracing::instrument(skip(state, claims))]
pub async fn update_status(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAccountStatusRequest>,
) -> Result<Json<serde_json::Value>> {
    crate::middleware::auth::require_admin(&claims)?;
    validation::validate_or_400(&payload)?;
    let status = payload.status;
    let aid = AccountId(id);
    let repo = PgAccountRepository::new(state.pool.clone());
    // Canonical path via Repository trait (§2): find + save, no direct db::exists.
    let mut account = repo.find_by_id(aid).await?.ok_or(AppError::NotFound {
        entity: "account",
        id: id.to_string(),
    })?;
    account.status = status;
    repo.save(&account).await?;
    tracing::info!(id, status=?status, "account status updated");
    Ok(Json(serde_json::json!({"id": aid, "status": status})))
}

#[cfg(test)]
mod tests {
    use crate::domain::{AccountStatus, Currency};
    use crate::models::account::CreateAccountRequest;
    use validator::Validate;

    #[test]
    fn create_validate_ok() {
        let req = CreateAccountRequest {
            bank_name: "BCA".into(),
            no_rekening: "12345".into(),
            currency: Currency::IDR,
            opening_balance: 0.0,
            limit_overdraft: None,
            gl_code: None,
            status: Some(AccountStatus::Aktif),
        };
        assert!(req.validate().is_ok());
    }
}
