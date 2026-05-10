use sqlx::SqlitePool;

use crate::domain::Asset;

pub async fn attach(
    pool: &SqlitePool,
    status_id: i64,
    asset_ids: &[i64],
) -> sqlx::Result<()> {
    for (sort, asset_id) in asset_ids.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO status_assets (status_id, asset_id, sort)
            VALUES (?, ?, ?)
            ON CONFLICT(status_id, asset_id) DO UPDATE SET sort = excluded.sort
            "#,
        )
        .bind(status_id)
        .bind(asset_id)
        .bind(sort as i64)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_for_status(pool: &SqlitePool, status_id: i64) -> sqlx::Result<Vec<Asset>> {
    sqlx::query_as::<_, Asset>(
        r#"
        SELECT a.id, a.original_name, a.stored_path, a.mime, a.byte_size, a.created_at
        FROM assets a
        JOIN status_assets sa ON sa.asset_id = a.id
        WHERE sa.status_id = ?
        ORDER BY sa.sort ASC, a.id ASC
        "#,
    )
    .bind(status_id)
    .fetch_all(pool)
    .await
}

pub async fn list_for_status_ids(
    pool: &SqlitePool,
    status_ids: &[i64],
) -> sqlx::Result<Vec<(i64, Asset)>> {
    if status_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; status_ids.len()].join(",");
    let sql = format!(
        r#"
        SELECT sa.status_id,
               a.id, a.original_name, a.stored_path, a.mime, a.byte_size, a.created_at
        FROM assets a
        JOIN status_assets sa ON sa.asset_id = a.id
        WHERE sa.status_id IN ({placeholders})
        ORDER BY sa.status_id ASC, sa.sort ASC, a.id ASC
        "#
    );
    let mut q = sqlx::query_as::<_, AssetWithStatus>(&sql);
    for id in status_ids {
        q = q.bind(*id);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| (r.status_id, r.into_asset())).collect())
}

#[derive(sqlx::FromRow)]
struct AssetWithStatus {
    status_id: i64,
    id: i64,
    original_name: String,
    stored_path: String,
    mime: String,
    byte_size: i64,
    created_at: String,
}

impl AssetWithStatus {
    fn into_asset(self) -> Asset {
        Asset {
            id: self.id,
            original_name: self.original_name,
            stored_path: self.stored_path,
            mime: self.mime,
            byte_size: self.byte_size,
            created_at: self.created_at,
        }
    }
}
