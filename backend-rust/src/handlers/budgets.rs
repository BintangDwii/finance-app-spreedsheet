use axum::{
    extract::{Extension, State},
    Json,
};

use crate::domain::UserRole;
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::budget::UpsertBudgetRequest;
use crate::utils::{sanitize, validation};

#[tracing::instrument(skip(state, claims))]
pub async fn list(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "list budgets");
    let rows = crate::db::budget::list_budgets(pool).await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[tracing::instrument(skip(state, claims))]
pub async fn upsert(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpsertBudgetRequest>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    if !crate::middleware::auth::require_role(&claims, &[UserRole::Approver]) {
        return Err(AppError::Forbidden("Hanya Admin/Approver".into()));
    }
    validation::validate_or_400(&payload)?;
    let budget_currency = crate::utils::currency::currency_or_idr(payload.currency);
    crate::db::budget::upsert_budget(
        pool,
        &payload.bulan,
        &sanitize::sanitize_string(&payload.divisi),
        &sanitize::sanitize_string(&payload.kategori),
        payload.planned,
        budget_currency,
    )
    .await?;
    tracing::info!(bulan=%payload.bulan, divisi=%payload.divisi, "budget upsert");
    Ok(Json(
        serde_json::json!({"bulan": payload.bulan, "divisi": payload.divisi, "kategori": payload.kategori, "planned": payload.planned}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn variance(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "budget variance");
    let rows = crate::db::budget::variance(pool).await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[cfg(test)]
mod tests {
    use crate::models::budget::UpsertBudgetRequest;
    use validator::Validate;

    #[test]
    fn upsert_validate_ok() {
        let req = UpsertBudgetRequest {
            bulan: "2026-08".into(),
            divisi: "IT & Engineering".into(),
            kategori: "Lainnya".into(),
            planned: 1000.0,
            currency: None,
        };
        assert!(req.validate().is_ok());
    }
}
