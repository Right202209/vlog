use sqlx::SqlitePool;

use crate::domain::User;

pub async fn count(pool: &SqlitePool) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_username(pool: &SqlitePool, username: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO users (username, password_hash, created_at, updated_at)
        VALUES (?, ?, datetime('now'), datetime('now'))
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn update_password(
    pool: &SqlitePool,
    id: i64,
    password_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = ?, updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(password_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
