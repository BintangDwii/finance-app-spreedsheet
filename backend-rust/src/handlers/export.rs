use axum::{
    extract::{Extension, State},
    http::header,
    response::IntoResponse,
};

use crate::error::Result;
use crate::middleware::auth::Claims;
use crate::utils::blocking;

#[tracing::instrument(skip(state, claims))]
pub async fn export_transaksi(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "export transaksi");
    let rows = crate::db::transaksi::fetch_export_rows(pool).await?;

    let buf = blocking::run_blocking(move || crate::utils::excel::build_excel(rows)).await?;

    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"transaksi_export.xlsx\"",
            ),
        ],
        buf,
    ))
}
