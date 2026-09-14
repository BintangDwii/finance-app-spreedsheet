use axum::{
    extract::{Extension, State},
    Json,
};

use crate::error::Result;
use crate::middleware::auth::Claims;

#[tracing::instrument(skip(state, claims))]
pub async fn get_saldo(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "get saldo");
    let (masuk, keluar, saldo) = crate::db::account::saldo_summary(pool).await?;
    Ok(Json(
        serde_json::json!({"masuk": masuk, "keluar": keluar, "saldo": saldo}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn per_rekening(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "per_rekening");
    let rows = crate::db::account::saldo_per_rekening(pool).await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[cfg(test)]
mod tests {
    #[test]
    fn saldo_dummy() {
        assert_eq!(1 + 1, 2);
    }
}
