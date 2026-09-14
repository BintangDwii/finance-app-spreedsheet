use crate::domain::TransaksiId;
use crate::models::audit::AuditLog;
use sqlx::{PgPool, Postgres, Transaction};

/// Single source of truth for audit INSERT — §10 DRY
const INSERT_AUDIT_SQL: &str =
    "INSERT INTO audit_log (transaksi_id, actor, from_status, to_status, catatan) VALUES ($1,$2,$3,$4,$5)";

pub async fn list_recent(pool: &PgPool, limit: i64) -> Result<Vec<AuditLog>, sqlx::Error> {
    sqlx::query_as::<_, AuditLog>(
        "SELECT id, transaksi_id, actor, from_status, to_status, catatan, timestamp::text FROM audit_log ORDER BY timestamp DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn insert_log(
    pool: &PgPool,
    transaksi_id: TransaksiId,
    actor: &str,
    from_status: &str,
    to_status: &str,
    catatan: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(INSERT_AUDIT_SQL)
        .bind(transaksi_id.0)
        .bind(actor)
        .bind(from_status)
        .bind(to_status)
        .bind(catatan)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_log_tx(
    tx: &mut Transaction<'_, Postgres>,
    transaksi_id: TransaksiId,
    actor: &str,
    from_status: &str,
    to_status: &str,
    catatan: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(INSERT_AUDIT_SQL)
        .bind(transaksi_id.0)
        .bind(actor)
        .bind(from_status)
        .bind(to_status)
        .bind(catatan)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn insert_log_with_status_tx(
    tx: &mut Transaction<'_, Postgres>,
    transaksi_id: TransaksiId,
    actor: &str,
    from: crate::domain::TransaksiStatus,
    to: crate::domain::TransaksiStatus,
    catatan: &str,
) -> Result<(), sqlx::Error> {
    // Delegate to canonical insert — avoids SQL duplication (§10)
    insert_log_tx(tx, transaksi_id, actor, from.as_str(), to.as_str(), catatan).await
}
