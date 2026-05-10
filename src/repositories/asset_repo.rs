use sqlx::SqlitePool;

use crate::domain::Asset;

pub async fn create(
    pool: &SqlitePool,
    original_name: &str,
    stored_path: &str,
    mime: &str,
    byte_size: i64,
) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO assets (original_name, stored_path, mime, byte_size, created_at)
        VALUES (?, ?, ?, ?, datetime('now'))
        RETURNING id
        "#,
    )
    .bind(original_name)
    .bind(stored_path)
    .bind(mime)
    .bind(byte_size)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn list_recent(pool: &SqlitePool, limit: i64) -> sqlx::Result<Vec<Asset>> {
    sqlx::query_as::<_, Asset>(
        r#"
        SELECT id, original_name, stored_path, mime, byte_size, created_at
        FROM assets
        ORDER BY id DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}
