use crate::models::budget::{Budget, VarianceRow};
use sqlx::PgPool;

pub async fn list_budgets(pool: &PgPool) -> Result<Vec<Budget>, sqlx::Error> {
    sqlx::query_as::<_, Budget>("SELECT id, bulan, divisi, kategori, planned::float8 AS planned, currency FROM budgets ORDER BY bulan, divisi, kategori")
        .fetch_all(pool)
        .await
}

pub async fn variance(pool: &PgPool) -> Result<Vec<VarianceRow>, sqlx::Error> {
    let sql = format!(
        r#"
        SELECT b.bulan, b.divisi, b.kategori, b.planned::float8 AS planned,
               COALESCE(a.actual,0)::float8 as actual,
               (b.planned - COALESCE(a.actual,0))::float8 as sisa,
               CASE WHEN b.planned=0 THEN 0 ELSE ROUND(COALESCE(a.actual,0)/b.planned*100,1) END::float8 as burn
        FROM budgets b
        LEFT JOIN (
            SELECT to_char(tanggal::date,'YYYY-MM') as bulan, divisi, kategori, SUM(total_akhir * fx_rate) as actual
            FROM transaksi WHERE is_deleted=false AND status IN ({paid}) AND jenis='Pengeluaran'
            GROUP BY 1,2,3
        ) a USING (bulan, divisi, kategori)
        ORDER BY b.bulan, b.divisi, b.kategori
        "#,
        paid = crate::utils::currency::PAID_STATUSES_SQL
    );
    sqlx::query_as::<_, VarianceRow>(&sql).fetch_all(pool).await
}

pub async fn upsert_budget(
    pool: &PgPool,
    bulan: &str,
    divisi: &str,
    kategori: &str,
    planned: f64,
    currency: crate::domain::Currency,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "INSERT INTO budgets (bulan, divisi, kategori, planned, currency) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (bulan, divisi, kategori) DO UPDATE SET planned=$4, currency=$5"
    )
    .bind(bulan)
    .bind(divisi)
    .bind(kategori)
    .bind(planned)
    .bind(currency)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}
