use crate::domain::AccountId;
use crate::error::Result;
use crate::models::account::Account;
use crate::repository::{CrudRepository, Repository};
use sqlx::PgPool;

pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl Repository<Account, AccountId> for PgAccountRepository {
    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: AccountId) -> Result<Option<Account>> {
        // Delegate to db layer — single SQL source (§10 DRY)
        crate::db::account::find_by_id(&self.pool, id)
            .await
            .map_err(crate::error::AppError::Database)
    }

    #[tracing::instrument(skip(self, entity))]
    async fn save(&self, entity: &Account) -> Result<()> {
        sqlx::query("UPDATE accounts SET status=$1 WHERE id=$2")
            .bind(entity.status)
            .bind(entity.id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl CrudRepository<Account, AccountId> for PgAccountRepository {
    #[tracing::instrument(skip(self))]
    async fn list_all(&self) -> Result<Vec<Account>> {
        crate::db::account::list_accounts(&self.pool)
            .await
            .map_err(crate::error::AppError::Database)
    }

    #[tracing::instrument(skip(self))]
    async fn exists(&self, id: AccountId) -> Result<bool> {
        crate::db::account::exists(&self.pool, id)
            .await
            .map_err(crate::error::AppError::Database)
    }
}
