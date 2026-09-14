use crate::error::Result;

#[async_trait::async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    async fn save(&self, entity: &T) -> Result<()>;
}

/// Extended CRUD for DRY across entities — generic list/exists/update
#[async_trait::async_trait]
pub trait CrudRepository<T, ID>: Repository<T, ID> {
    async fn list_all(&self) -> Result<Vec<T>>;
    async fn exists(&self, id: ID) -> Result<bool>;
}

// Re-exports
pub mod account_repo;
pub mod transaksi_repo;
