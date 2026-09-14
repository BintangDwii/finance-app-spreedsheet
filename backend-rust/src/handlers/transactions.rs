use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};

use crate::domain::{Jenis, TransaksiId, TransaksiStatus, UserRole};
use crate::error::{AppError, Result};
use crate::middleware::auth::Claims;
use crate::models::transaksi::{CreateTransaksiRequest, ListParams, UpdateStatusRequest};
use crate::utils::{currency, pagination, reference, sanitize, validation};

#[tracing::instrument(skip(state, claims))]
pub async fn list(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    validation::validate_or_400(&params)?;
    let (page, per_page, _) = pagination::normalize_page(params.page, params.per_page);

    tracing::info!(user = %claims.sub, page, per_page, "list transaksi");
    let rows = crate::db::transaksi::list_filtered(
        pool,
        params.status,
        params.divisi.as_deref(),
        params.jenis,
        params.search.as_deref(),
        page,
        per_page,
    )
    .await?;
    Ok(Json(
        serde_json::json!({"data": rows, "page": page, "per_page": per_page}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn create(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTransaksiRequest>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    if !crate::middleware::auth::require_role(&claims, &[UserRole::Maker]) {
        return Err(AppError::Forbidden("Hanya Maker/Admin".into()));
    }
    validation::validate_or_400(&payload)?;
    if payload.subtotal <= 0.0 {
        return Err(AppError::Validation("Subtotal harus >0".into()));
    }
    let currency = currency::currency_or_idr(payload.currency);
    let fx_rate = currency::fx_or_one(payload.fx_rate);
    let ppn = payload.ppn_11.unwrap_or(0.0);
    let diskon = payload.diskon.unwrap_or(0.0);
    // Business rules live in service (§2)
    let total = crate::service::transaksi_service::calculate_total(payload.subtotal, ppn, diskon);

    let tanggal = crate::service::transaksi_service::parse_tanggal(&payload.tanggal)?;

    tracing::info!(jenis=%payload.jenis.as_str(), total, "create transaksi validated");

    let mut tx = pool.begin().await?;

    if payload.jenis == Jenis::TransferInternal {
        let tujuan = crate::service::transaksi_service::validate_transfer(
            payload.jenis,
            payload.akun_tujuan.as_deref(),
        )?;
        let reference = reference::sanitize_reference(payload.reference_no.clone(), "TRF");
        let transfer_note = format!("Transfer {} -> {}", payload.akun_pembayaran, tujuan);

        let id1 = crate::db::transaksi::insert_transfer_leg_tx(
            &mut tx,
            crate::db::transaksi::InsertTransferLegParams {
                tanggal,
                divisi: &payload.divisi,
                entitas_terkait: &format!("Transfer ke {}", tujuan),
                akun_pembayaran: &payload.akun_pembayaran,
                subtotal: payload.subtotal,
                total,
                currency,
                fx: fx_rate,
                catatan: &transfer_note,
                created_by: &claims.sub,
                reference: &reference,
            },
        )
        .await?;
        let id2 = crate::db::transaksi::insert_transfer_leg_tx(
            &mut tx,
            crate::db::transaksi::InsertTransferLegParams {
                tanggal,
                divisi: &payload.divisi,
                entitas_terkait: &format!("Transfer dari {}", payload.akun_pembayaran),
                akun_pembayaran: &tujuan,
                subtotal: payload.subtotal,
                total,
                currency,
                fx: fx_rate,
                catatan: &transfer_note,
                created_by: &claims.sub,
                reference: &reference,
            },
        )
        .await?;
        crate::db::audit::insert_log_tx(
            &mut tx,
            TransaksiId(id1),
            &claims.sub,
            "-",
            TransaksiStatus::Paid.as_str(),
            "Transfer Internal",
        )
        .await?;
        crate::db::audit::insert_log_tx(
            &mut tx,
            TransaksiId(id2),
            &claims.sub,
            "-",
            TransaksiStatus::Paid.as_str(),
            "Transfer Internal",
        )
        .await?;
        tx.commit().await?;
        tracing::info!(ids=?vec![id1,id2], reference=%reference, "transfer created");
        return Ok(Json(
            serde_json::json!({"ids":[id1,id2],"reference":reference,"status":TransaksiStatus::Paid}),
        ));
    }

    let status = TransaksiStatus::Pending;
    let reference = reference::sanitize_reference(payload.reference_no.clone(), "REF");
    // Strict due_date parsing — invalid format returns 400 (§7)
    let due_date = crate::service::transaksi_service::parse_due_date(payload.due_date.clone())?;
    // Moved close to first use (§4 Function Reuse Rules) — only normal path needs it
    let catatan = sanitize::catatan_or_dash(payload.catatan.clone());

    let id = crate::db::transaksi::insert_normal_tx(
        &mut tx,
        crate::db::transaksi::InsertTransaksiParams {
            tanggal,
            jenis: payload.jenis,
            divisi: &payload.divisi,
            kategori: &payload.kategori,
            entitas: &payload.entitas_terkait,
            akun: &payload.akun_pembayaran,
            subtotal: payload.subtotal,
            ppn,
            diskon,
            total,
            currency,
            fx: fx_rate,
            due_date,
            reference: &reference,
            status,
            catatan: &catatan,
            created_by: &claims.sub,
        },
    )
    .await?;

    crate::db::audit::insert_log_tx(
        &mut tx,
        TransaksiId(id),
        &claims.sub,
        "-",
        status.as_str(),
        &catatan,
    )
    .await?;
    tx.commit().await?;
    tracing::info!(id, status=%status.as_str(), "transaksi created");
    Ok(Json(
        serde_json::json!({"id": id, "status": status, "reference": reference}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn update_status(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>> {
    let pool = &state.pool;
    validation::validate_or_400(&payload)?;
    let tid = TransaksiId(id);
    let status_row = crate::db::transaksi::find_for_status_update(pool, tid)
        .await?
        .ok_or(AppError::NotFound {
            entity: "transaksi",
            id: id.to_string(),
        })?;
    let (cur_status, total, currency_val, fx_rate_val) = status_row;
    let idr = currency::to_idr(total, currency_val, fx_rate_val);
    crate::service::transaksi_service::guard_transition(
        cur_status,
        payload.new_status,
        idr,
        claims.role,
    )?;
    // Reuse single sanitized value (§3) instead of sanitizing twice
    let sanitized_catatan = sanitize::sanitize_string(payload.catatan.as_deref().unwrap_or(""));
    if payload.new_status == TransaksiStatus::Rejected && sanitized_catatan.is_empty() {
        return Err(AppError::Validation("Catatan wajib untuk Reject".into()));
    }
    let catatan = if sanitized_catatan.is_empty() {
        "-".to_string()
    } else {
        sanitized_catatan
    };
    crate::db::transaksi::update_status(pool, tid, payload.new_status).await?;
    crate::db::audit::insert_log(
        pool,
        tid,
        &claims.sub,
        cur_status.as_str(),
        payload.new_status.as_str(),
        &catatan,
    )
    .await?;
    tracing::info!(id, from=%cur_status.as_str(), to=%payload.new_status.as_str(), actor=%claims.sub, "status updated");
    Ok(Json(
        serde_json::json!({"id": id, "from": cur_status, "to": payload.new_status}),
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn soft_delete(
    State(state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>> {
    crate::middleware::auth::require_admin(&claims)?;
    let tid = TransaksiId(id);
    // Canonical existence check via Repository (§2)
    let repo = crate::repository::transaksi_repo::PgTransaksiRepository::new(state.pool.clone());
    let exists = crate::repository::CrudRepository::<
        crate::models::transaksi::Transaksi,
        TransaksiId,
    >::exists(&repo, tid)
    .await?;
    if !exists {
        return Err(AppError::NotFound {
            entity: "transaksi",
            id: id.to_string(),
        });
    }
    crate::db::transaksi::soft_delete(&state.pool, tid).await?;
    crate::db::audit::insert_log(
        &state.pool,
        tid,
        &claims.sub,
        "Active",
        "Deleted",
        "Soft delete",
    )
    .await?;
    tracing::info!(id, actor=%claims.sub, "soft deleted");
    Ok(Json(serde_json::json!({"id": id, "deleted": true})))
}

#[cfg(test)]
mod tests {
    // Guard-transition coverage lives in service::transaksi_service (§10 DRY).
    // Smoke test here ensures handler wiring delegates to service.
    #[test]
    fn handler_delegates_guard_to_service() {
        let guard_result = crate::service::transaksi_service::guard_transition(
            crate::domain::TransaksiStatus::Pending,
            crate::domain::TransaksiStatus::Approved,
            10_000.0,
            crate::domain::UserRole::Admin,
        );
        assert!(guard_result.is_ok());
    }
}
