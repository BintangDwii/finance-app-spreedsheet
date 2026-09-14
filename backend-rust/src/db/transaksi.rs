use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};

use crate::db::common::TRANSAKSI_COLS_TEXT;
use crate::domain::{Currency, Jenis, TransaksiId, TransaksiStatus};
use crate::models::transaksi::Transaksi;
use crate::utils::sanitize::escape_like;

pub async fn list_filtered(
    pool: &PgPool,
    status: Option<TransaksiStatus>,
    divisi: Option<&str>,
    jenis: Option<Jenis>,
    search: Option<&str>,
    page: i64,
    per_page: i64,
) -> Result<Vec<Transaksi>, sqlx::Error> {
    let offset = (page - 1) * per_page;
    // Escape ILIKE wildcards (§7) — search is sanitized before binding.
    let search_escaped = search.map(escape_like);
    let sql = format!(
        r#"
        SELECT {cols}
        FROM transaksi
        WHERE is_deleted=false
          AND ($1::status_t IS NULL OR status=$1)
          AND ($2::text IS NULL OR divisi=$2)
          AND ($3::jenis_t IS NULL OR jenis=$3)
          AND ($4::text IS NULL OR (entitas_terkait ILIKE '%' || $4 || '%' ESCAPE '\' OR catatan ILIKE '%' || $4 || '%' ESCAPE '\' OR reference_no ILIKE '%' || $4 || '%' ESCAPE '\'))
        ORDER BY tanggal DESC, id DESC
        LIMIT $5 OFFSET $6
        "#,
        cols = TRANSAKSI_COLS_TEXT
    );
    let rows = sqlx::query_as::<_, Transaksi>(&sql)
        .bind(status)
        .bind(divisi)
        .bind(jenis)
        .bind(search_escaped.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn find_for_status_update(
    pool: &PgPool,
    id: TransaksiId,
) -> Result<Option<(TransaksiStatus, f64, Currency, f64)>, sqlx::Error> {
    sqlx::query_as::<_, (TransaksiStatus, f64, Currency, f64)>(
        "SELECT status, total_akhir, currency, fx_rate FROM transaksi WHERE id=$1 AND is_deleted=false",
    )
    .bind(id.0)
    .fetch_optional(pool)
    .await
}

pub async fn exists_active(pool: &PgPool, id: TransaksiId) -> Result<bool, sqlx::Error> {
    let row =
        sqlx::query_scalar::<_, i64>("SELECT id FROM transaksi WHERE id=$1 AND is_deleted=false")
            .bind(id.0)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

pub async fn update_status(
    pool: &PgPool,
    id: TransaksiId,
    new_status: TransaksiStatus,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE transaksi SET status=$1, updated_at=now() WHERE id=$2")
        .bind(new_status)
        .bind(id.0)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn soft_delete(pool: &PgPool, id: TransaksiId) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE transaksi SET is_deleted=true, updated_at=now() WHERE id=$1")
        .bind(id.0)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn mark_reconciled(pool: &PgPool, id: TransaksiId) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE transaksi SET status=$1, updated_at=now() WHERE id=$2")
        .bind(TransaksiStatus::Reconciled)
        .bind(id.0)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn mark_reconciled_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: TransaksiId,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE transaksi SET status=$1, updated_at=now() WHERE id=$2")
        .bind(TransaksiStatus::Reconciled)
        .bind(id.0)
        .execute(&mut **tx)
        .await?;
    Ok(res.rows_affected())
}

pub async fn fetch_candidates_for_reconcile(
    pool: &PgPool,
) -> Result<Vec<(TransaksiId, NaiveDate, f64)>, sqlx::Error> {
    let sql = format!(
        "SELECT id, tanggal::date, total_akhir * fx_rate as idr FROM transaksi WHERE is_deleted=false AND status IN ({})",
        crate::utils::currency::PAID_STATUSES_SQL
    );
    sqlx::query_as::<_, (TransaksiId, NaiveDate, f64)>(&sql)
        .fetch_all(pool)
        .await
}

pub async fn fetch_export_rows(
    pool: &PgPool,
) -> Result<
    Vec<(
        i64,
        String,
        String,
        String,
        String,
        String,
        String,
        f64,
        String,
    )>,
    sqlx::Error,
> {
    sqlx::query_as::<_, (i64, String, String, String, String, String, String, f64, String)>(
        "SELECT id, tanggal::text, jenis::text, divisi, kategori, entitas_terkait, akun_pembayaran, total_akhir, status::text FROM transaksi WHERE is_deleted=false ORDER BY tanggal DESC LIMIT 5000"
    )
    .fetch_all(pool)
    .await
}

pub struct InsertTransaksiParams<'a> {
    pub tanggal: NaiveDate,
    pub jenis: Jenis,
    pub divisi: &'a str,
    pub kategori: &'a str,
    pub entitas: &'a str,
    pub akun: &'a str,
    pub subtotal: f64,
    pub ppn: f64,
    pub diskon: f64,
    pub total: f64,
    pub currency: Currency,
    pub fx: f64,
    pub due_date: Option<NaiveDate>,
    pub reference: &'a str,
    pub status: TransaksiStatus,
    pub catatan: &'a str,
    pub created_by: &'a str,
}

pub struct InsertTransferLegParams<'a> {
    pub tanggal: NaiveDate,
    pub divisi: &'a str,
    pub entitas_terkait: &'a str,
    pub akun_pembayaran: &'a str,
    pub subtotal: f64,
    pub total: f64,
    pub currency: Currency,
    pub fx: f64,
    pub catatan: &'a str,
    pub created_by: &'a str,
    pub reference: &'a str,
}

pub async fn insert_normal_tx(
    tx: &mut Transaction<'_, Postgres>,
    p: InsertTransaksiParams<'_>,
) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO transaksi (timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran, subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date, reference_no, status, catatan, created_by) VALUES (now()::text, $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) RETURNING id"
    )
    .bind(p.tanggal)
    .bind(p.jenis)
    .bind(p.divisi)
    .bind(p.kategori)
    .bind(p.entitas)
    .bind(p.akun)
    .bind(p.subtotal)
    .bind(p.ppn)
    .bind(p.diskon)
    .bind(p.total)
    .bind(p.currency)
    .bind(p.fx)
    .bind(p.due_date)
    .bind(p.reference)
    .bind(p.status)
    .bind(p.catatan)
    .bind(p.created_by)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

pub async fn insert_transfer_leg_tx(
    tx: &mut Transaction<'_, Postgres>,
    p: InsertTransferLegParams<'_>,
) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO transaksi (timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran, subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, status, catatan, created_by, reference_no) VALUES (now()::text, $1, $2, $3, 'Transfer Antar Rekening', $4, $5, $6, 0,0,$7,$8,$9,$10,$11,$12,$13) RETURNING id"
    )
    .bind(p.tanggal)
    .bind(Jenis::TransferInternal)
    .bind(p.divisi)
    .bind(p.entitas_terkait)
    .bind(p.akun_pembayaran)
    .bind(p.subtotal)
    .bind(p.total)
    .bind(p.currency)
    .bind(p.fx)
    .bind(TransaksiStatus::Paid)
    .bind(p.catatan)
    .bind(p.created_by)
    .bind(p.reference)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}
