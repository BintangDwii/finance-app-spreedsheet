use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};

use crate::domain::{Currency, ReconStatus, ReconciliationId, TransaksiId, TransaksiStatus};
use crate::models::reconcile::Reconciliation;

pub async fn list_recent(pool: &PgPool, limit: i64) -> Result<Vec<Reconciliation>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM reconciliation ORDER BY created_at DESC LIMIT $1",
        crate::db::common::RECON_COLS
    );
    sqlx::query_as::<_, Reconciliation>(&sql)
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn find_status(
    pool: &PgPool,
    id: ReconciliationId,
) -> Result<Option<ReconStatus>, sqlx::Error> {
    let row = sqlx::query_as::<_, (ReconStatus,)>("SELECT status FROM reconciliation WHERE id=$1")
        .bind(id.0)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn find_transaksi_status(
    pool: &PgPool,
    transaksi_id: TransaksiId,
) -> Result<Option<TransaksiStatus>, sqlx::Error> {
    let row = sqlx::query_as::<_, (TransaksiStatus,)>(
        "SELECT status FROM transaksi WHERE id=$1 AND is_deleted=false",
    )
    .bind(transaksi_id.0)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn insert_matched_tx(
    tx: &mut Transaction<'_, Postgres>,
    statement_date: NaiveDate,
    description: &str,
    amount: f64,
    currency: Currency,
    matched_transaksi_id: TransaksiId,
    uploaded_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO reconciliation (statement_date, description, amount, currency, matched_transaksi_id, status, uploaded_by) VALUES ($1,$2,$3,$4,$5,$6,$7)")
        .bind(statement_date).bind(description).bind(amount).bind(currency).bind(matched_transaksi_id.0).bind(ReconStatus::Reconciled).bind(uploaded_by)
        .execute(&mut **tx).await?;
    Ok(())
}

pub async fn insert_unmatched_tx(
    tx: &mut Transaction<'_, Postgres>,
    statement_date: NaiveDate,
    description: &str,
    amount: f64,
    currency: Currency,
    uploaded_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO reconciliation (statement_date, description, amount, currency, status, uploaded_by) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(statement_date).bind(description).bind(amount).bind(currency).bind(ReconStatus::Unreconciled).bind(uploaded_by)
        .execute(&mut **tx).await?;
    Ok(())
}

pub async fn manual_match(
    pool: &PgPool,
    recon_id: ReconciliationId,
    transaksi_id: TransaksiId,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE reconciliation SET matched_transaksi_id=$1, status=$2 WHERE id=$3")
        .bind(transaksi_id.0)
        .bind(ReconStatus::Manual)
        .bind(recon_id.0)
        .execute(pool)
        .await?;
    Ok(())
}
