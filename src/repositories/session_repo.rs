use sqlx::SqlitePool;

use crate::domain::Session;

pub async fn create(
    pool: &SqlitePool,
    id: &str,
    user_id: i64,
    csrf_token: &str,
    lifetime_secs: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, csrf_token, created_at, expires_at)
        VALUES (?, ?, ?, datetime('now'), datetime('now', ? || ' seconds'))
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(csrf_token)
    .bind(format!("+{}", lifetime_secs))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_active(pool: &SqlitePool, id: &str) -> sqlx::Result<Option<Session>> {
    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, user_id, csrf_token, created_at, expires_at
        FROM sessions
        WHERE id = ? AND expires_at > datetime('now')
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn purge_expired(pool: &SqlitePool) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
