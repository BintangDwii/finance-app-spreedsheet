use crate::domain::{AccountId, AccountStatus, Currency};
use crate::models::account::{Account, SaldoPerRekening};
use sqlx::PgPool;

pub async fn list_accounts(pool: &PgPool) -> Result<Vec<Account>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM accounts ORDER BY currency, bank_name",
        crate::db::common::ACCOUNT_COLS
    );
    sqlx::query_as::<_, Account>(&sql).fetch_all(pool).await
}

pub async fn find_by_id(pool: &PgPool, id: AccountId) -> Result<Option<Account>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM accounts WHERE id=$1",
        crate::db::common::ACCOUNT_COLS
    );
    sqlx::query_as::<_, Account>(&sql)
        .bind(id.0)
        .fetch_optional(pool)
        .await
}

pub async fn exists(pool: &PgPool, id: AccountId) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i32>("SELECT id FROM accounts WHERE id=$1")
        .bind(id.0)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_account(
    pool: &PgPool,
    bank_name: &str,
    no_rekening: &str,
    currency: Currency,
    opening_balance: f64,
    limit_overdraft: f64,
    gl_code: Option<String>,
    status: AccountStatus,
) -> Result<i32, sqlx::Error> {
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO accounts (bank_name, no_rekening, currency, opening_balance, limit_overdraft, gl_code, status) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id"
    )
    .bind(bank_name)
    .bind(no_rekening)
    .bind(currency)
    .bind(opening_balance)
    .bind(limit_overdraft)
    .bind(gl_code)
    .bind(status)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn update_status(
    pool: &PgPool,
    id: AccountId,
    status: AccountStatus,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE accounts SET status=$1 WHERE id=$2")
        .bind(status)
        .bind(id.0)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn saldo_per_rekening(pool: &PgPool) -> Result<Vec<SaldoPerRekening>, sqlx::Error> {
    let sql = format!(
        r#"
        SELECT a.bank_name, a.currency, a.opening_balance::float8 AS opening_balance,
               COALESCE(SUM(CASE WHEN t.jenis='Pemasukan' AND t.status IN ({paid}) THEN t.total_akhir END)::float8,0.0) as total_masuk,
               COALESCE(SUM(CASE WHEN t.jenis='Pengeluaran' AND t.status IN ({paid}) THEN t.total_akhir END)::float8,0.0) as total_keluar,
               (a.opening_balance + COALESCE(SUM(CASE WHEN t.jenis='Pemasukan' THEN t.total_akhir WHEN t.jenis='Pengeluaran' THEN -t.total_akhir ELSE 0 END),0))::float8 as saldo
        FROM accounts a
        LEFT JOIN transaksi t ON t.akun_pembayaran=a.bank_name AND t.is_deleted=false AND t.status IN ({paid})
        WHERE a.status='Aktif'
        GROUP BY a.bank_name, a.currency, a.opening_balance
        ORDER BY a.currency, a.bank_name
        "#,
        paid = crate::utils::currency::PAID_STATUSES_SQL
    );
    sqlx::query_as::<_, SaldoPerRekening>(&sql)
        .fetch_all(pool)
        .await
}

pub async fn saldo_summary(pool: &PgPool) -> Result<(f64, f64, f64), sqlx::Error> {
    let sql = format!(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN jenis='Pemasukan' AND status IN ({paid}) THEN total_akhir * fx_rate END)::float8,0.0) as masuk,
            COALESCE(SUM(CASE WHEN jenis='Pengeluaran' AND status IN ({paid}) THEN total_akhir * fx_rate END)::float8,0.0) as keluar,
            COALESCE(SUM(CASE WHEN jenis='Pemasukan' THEN total_akhir*fx_rate WHEN jenis='Pengeluaran' THEN -total_akhir*fx_rate ELSE 0 END)::float8,0.0) as saldo
        FROM transaksi WHERE is_deleted=false AND status IN ({paid})
        "#,
        paid = crate::utils::currency::PAID_STATUSES_SQL
    );
    let row = sqlx::query_as::<_, (f64, f64, f64)>(&sql)
        .fetch_one(pool)
        .await?;
    Ok(row)
}
