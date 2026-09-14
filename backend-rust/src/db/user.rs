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

/// Persist an upgraded password hash (legacy sha256 → argon2 transparent upgrade).
pub async fn update_password_hash(
    pool: &PgPool,
    username: &str,
    new_hash: &str,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("UPDATE users SET password_hash=$1 WHERE username=$2")
        .bind(new_hash)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}
