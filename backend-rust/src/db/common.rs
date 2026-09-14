/// Central SQL fragments — single source of truth
/// NOTE: date/timestamptz columns are selected NATIVELY (§5: map directly to
/// chrono types). Do NOT cast them to ::text — sqlx cannot decode TEXT into
/// NaiveDate/DateTime<Utc> and login/list endpoints will 500.
pub const TRANSAKSI_COLS_TEXT: &str = "id, timestamp, tanggal, jenis, divisi, kategori, entitas_terkait, akun_pembayaran, subtotal::float8 AS subtotal, ppn_11::float8 AS ppn_11, diskon::float8 AS diskon, total_akhir::float8 AS total_akhir, currency, fx_rate::float8 AS fx_rate, due_date, reference_no, status, file_bukti, catatan, created_by, updated_at, is_deleted, (total_akhir * fx_rate)::float8 as idr_amount";
pub const ACCOUNT_COLS: &str = "id, bank_name, no_rekening, currency, opening_balance::float8 AS opening_balance, limit_overdraft::float8 AS limit_overdraft, gl_code, status";
pub const RECON_COLS: &str = "id, statement_date, description, amount::float8 AS amount, currency, matched_transaksi_id, status, uploaded_by, created_at";

/// Check unique violation via SQLSTATE 23505 (more robust than string contains)
pub fn is_unique_violation(e: &sqlx::Error) -> bool {
    if let Some(db_err) = e.as_database_error() {
        if db_err.code().as_deref() == Some("23505") {
            return true;
        }
    }
    let s = e.to_string().to_lowercase();
    s.contains("duplicate") || s.contains("unique")
}
