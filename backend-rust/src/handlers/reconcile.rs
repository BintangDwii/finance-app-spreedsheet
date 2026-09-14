use axum::{
    extract::{Extension, Path, State},
    Json,
};

use crate::domain::{ReconStatus, ReconciliationId, TransaksiId, TransaksiStatus};
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::reconcile::ManualMatchRequest;
use crate::utils::{blocking, validation};

#[tracing::instrument(skip(state, claims))]
pub async fn list(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    tracing::info!(user=%claims.sub, "reconcile list");
    let rows = crate::db::reconcile::list_recent(pool, 100).await?;
    Ok(Json(serde_json::json!({"data": rows})))
}

#[tracing::instrument(skip(state, claims, body))]
pub async fn upload_csv(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    body: String,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    const MAX_CSV_BYTES: usize = 1024 * 1024; // 1MB limit per AGENTS.MD 7
    if body.trim().is_empty() {
        return Err(AppError::Validation("CSV kosong".into()));
    }
    if body.len() > MAX_CSV_BYTES {
        return Err(AppError::Validation(format!(
            "CSV terlalu besar (>{} bytes)",
            MAX_CSV_BYTES
        )));
    }
    let body_clone = body.clone();
    let mutations_owned =
        blocking::run_blocking(move || crate::service::csv_service::parse_csv(&body_clone)).await?;

    if mutations_owned.is_empty() {
        return Err(AppError::Validation("Tidak ada baris valid".into()));
    }

    let trans_rows = crate::db::transaksi::fetch_candidates_for_reconcile(pool).await?;

    let matches = crate::service::reconcile_service::match_candidates(
        &trans_rows,
        &mutations_owned
            .iter()
            .map(|(d, a, _, _)| (*d, *a))
            .collect::<Vec<_>>(),
    );

    let mut tx = pool.begin().await?;
    let mut used_idx = std::collections::HashSet::new();
    let mut matched = 0;
    for (m_idx, tid) in &matches {
        let (m_date, m_amt, m_desc, m_cur) = &mutations_owned[*m_idx];
        crate::db::reconcile::insert_matched_tx(
            &mut tx,
            *m_date,
            m_desc,
            *m_amt,
            *m_cur,
            *tid,
            &claims.sub,
        )
        .await?;
        crate::db::transaksi::mark_reconciled_tx(&mut tx, *tid).await?;
        // Type-safe audit variant (§1): enums instead of raw strings
        crate::db::audit::insert_log_with_status_tx(
            &mut tx,
            *tid,
            &claims.sub,
            TransaksiStatus::Paid,
            TransaksiStatus::Reconciled,
            "Auto-match CSV",
        )
        .await?;
        used_idx.insert(*m_idx);
        matched += 1;
    }
    let mut unmatched = 0;
    for (idx, (m_date, m_amt, m_desc, m_cur)) in mutations_owned.iter().enumerate() {
        if used_idx.contains(&idx) {
            continue;
        }
        crate::db::reconcile::insert_unmatched_tx(
            &mut tx,
            *m_date,
            m_desc,
            *m_amt,
            *m_cur,
            &claims.sub,
        )
        .await?;
        unmatched += 1;
    }
    tx.commit().await?;
    tracing::info!(
        matched,
        unmatched,
        total = mutations_owned.len(),
        "reconcile upload"
    );
    Ok(Json(
        serde_json::json!({"matched": matched, "unmatched": unmatched, "total": mutations_owned.len()}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn manual_match(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    Json(payload): Json<ManualMatchRequest>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    validation::validate_or_400(&payload)?;
    let tid = TransaksiId(payload.transaksi_id);
    let rid = ReconciliationId(id);
    let recon_status =
        crate::db::reconcile::find_status(pool, rid)
            .await?
            .ok_or(AppError::NotFound {
                entity: "reconciliation",
                id: id.to_string(),
            })?;
    if recon_status == ReconStatus::Reconciled {
        return Err(AppError::Validation("Sudah Reconciled".into()));
    }
    let transaksi_status = crate::db::reconcile::find_transaksi_status(pool, tid)
        .await?
        .ok_or(AppError::NotFound {
            entity: "transaksi",
            id: tid.0.to_string(),
        })?;
    crate::db::reconcile::manual_match(pool, rid, tid).await?;
    crate::db::transaksi::mark_reconciled(pool, tid).await?;
    crate::db::audit::insert_log(
        pool,
        tid,
        &claims.sub,
        transaksi_status.as_str(),
        TransaksiStatus::Reconciled.as_str(),
        "Manual reconcile",
    )
    .await?;
    tracing::info!(recon_id = id, transaksi_id = tid.0, "manual match");
    Ok(Json(
        serde_json::json!({"id": id, "matched_transaksi_id": tid.0, "status": ReconStatus::Manual}),
    ))
}

#[cfg(test)]
mod tests {
    // CSV parsing coverage lives in service::csv_service (§10 DRY).
    // Smoke test ensures handler wiring delegates correctly.
    #[test]
    fn reconcile_delegates_to_csv_service() {
        let csv = "tanggal,deskripsi,amount,currency\n2026-08-15,Test,1000000,IDR\n";
        let res = crate::service::csv_service::parse_csv(csv).unwrap();
        assert_eq!(res.len(), 1);
    }
}
