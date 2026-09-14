use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use validator::Validate;

use crate::error::Result;
use crate::middleware::auth::Claims;
use crate::utils::{pagination, validation};

#[derive(Debug, Deserialize, Validate)]
pub struct AuditParams {
    #[validate(range(min = 1, max = 500))]
    pub limit: Option<i64>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AuditParams>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    validation::validate_or_400(&params)?;
    let limit = pagination::normalize_limit(params.limit, 100, 500);
    tracing::info!(user=%claims.sub, limit, "audit list");
    let rows = crate::db::audit::list_recent(pool, limit).await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn audit_params_validate() {
        let p = AuditParams { limit: Some(100) };
        assert!(p.validate().is_ok());
        let bad = AuditParams { limit: Some(1000) };
        assert!(bad.validate().is_err());
    }
}
