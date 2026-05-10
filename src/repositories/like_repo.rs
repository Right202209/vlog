use sqlx::SqlitePool;

pub async fn like(pool: &SqlitePool, user_id: i64, status_id: i64) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO likes (user_id, status_id, created_at)
        VALUES (?, ?, datetime('now'))
        ON CONFLICT(user_id, status_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(status_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unlike(pool: &SqlitePool, user_id: i64, status_id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM likes WHERE user_id = ? AND status_id = ?")
        .bind(user_id)
        .bind(status_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn has_liked(
    pool: &SqlitePool,
    user_id: i64,
    status_id: i64,
) -> sqlx::Result<bool> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM likes WHERE user_id = ? AND status_id = ?",
    )
    .bind(user_id)
    .bind(status_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn liked_status_ids_for(
    pool: &SqlitePool,
    viewer_id: i64,
    status_ids: &[i64],
) -> sqlx::Result<Vec<i64>> {
    if status_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; status_ids.len()].join(",");
    let sql = format!(
        "SELECT status_id FROM likes WHERE user_id = ? AND status_id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, (i64,)>(&sql);
    q = q.bind(viewer_id);
    for id in status_ids {
        q = q.bind(*id);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
