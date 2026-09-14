use crate::models::user::User;
use sqlx::PgPool;

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, role, divisi, created_at FROM users WHERE username=$1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}
