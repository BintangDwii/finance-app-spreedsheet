use crate::domain::TransaksiId;
use crate::error::Result;
use crate::models::transaksi::Transaksi;
use crate::repository::{CrudRepository, Repository};
use sqlx::PgPool;

pub struct PgTransaksiRepository {
    pool: PgPool,
}

impl PgTransaksiRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl Repository<Transaksi, TransaksiId> for PgTransaksiRepository {
    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: TransaksiId) -> Result<Option<Transaksi>> {
        // Reuse centralized column list (§10) — no duplicated SELECT strings
        let sql = format!(
            "SELECT {} FROM transaksi WHERE id=$1 AND is_deleted=false",
            crate::db::common::TRANSAKSI_COLS_TEXT
        );
        let row = sqlx::query_as::<_, Transaksi>(&sql)
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    #[tracing::instrument(skip(self, entity))]
    async fn save(&self, entity: &Transaksi) -> Result<()> {
        sqlx::query("UPDATE transaksi SET status=$1, updated_at=now() WHERE id=$2")
            .bind(entity.status)
            .bind(entity.id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl CrudRepository<Transaksi, TransaksiId> for PgTransaksiRepository {
    #[tracing::instrument(skip(self))]
    async fn list_all(&self) -> Result<Vec<Transaksi>> {
        // Default paginated list via db layer — single delegation point (§10)
        crate::db::transaksi::list_filtered(&self.pool, None, None, None, None, 1, 50)
            .await
            .map_err(crate::error::AppError::Database)
    }

    #[tracing::instrument(skip(self))]
    async fn exists(&self, id: TransaksiId) -> Result<bool> {
        crate::db::transaksi::exists_active(&self.pool, id)
            .await
            .map_err(crate::error::AppError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_newtype() {
        let id = TransaksiId(1001);
        assert_eq!(id.0, 1001);
    }
}
