/// Central SQL fragments — single source of truth
pub const TRANSAKSI_COLS_TEXT: &str = "id, timestamp, tanggal::text, jenis, divisi, kategori, entitas_terkait, akun_pembayaran, subtotal, ppn_11, diskon, total_akhir, currency, fx_rate, due_date::text, reference_no, status, file_bukti, catatan, created_by, updated_at::text, is_deleted, (total_akhir * fx_rate) as idr_amount";
pub const ACCOUNT_COLS: &str =
    "id, bank_name, no_rekening, currency, opening_balance, limit_overdraft, gl_code, status";
pub const RECON_COLS: &str = "id, statement_date::text, description, amount, currency, matched_transaksi_id, status, uploaded_by, created_at::text";

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
