use std::collections::HashMap;

use sqlx::SqlitePool;

pub async fn load_all(pool: &SqlitePool) -> sqlx::Result<HashMap<String, String>> {
    // TODO(M2): public pages still read boot-time config; wire this into request rendering.
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM site_settings")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn upsert(pool: &SqlitePool, key: &str, value: &str) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO site_settings (key, value, updated_at)
        VALUES (?, ?, datetime('now'))
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
