use sqlx::SqlitePool;

use crate::domain::Status;

#[derive(Debug, Clone)]
pub struct StatusInput<'a> {
    pub user_id: i64,
    pub content_md: &'a str,
    pub content_html: &'a str,
    pub parent_id: Option<i64>,
    pub repost_of_id: Option<i64>,
}

const STATUS_COLUMNS: &str = "id, user_id, content_md, content_html, parent_id, repost_of_id, \
                              reply_count, like_count, repost_count, created_at";

pub async fn create(pool: &SqlitePool, input: &StatusInput<'_>) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO statuses (
            user_id, content_md, content_html, parent_id, repost_of_id, created_at
        )
        VALUES (?, ?, ?, ?, ?, datetime('now'))
        RETURNING id
        "#,
    )
    .bind(input.user_id)
    .bind(input.content_md)
    .bind(input.content_html)
    .bind(input.parent_id)
    .bind(input.repost_of_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Status>> {
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS} FROM statuses WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_by_ids(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<Vec<Status>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT {STATUS_COLUMNS} FROM statuses WHERE id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, Status>(&sql);
    for id in ids {
        q = q.bind(*id);
    }
    q.fetch_all(pool).await
}

pub async fn list_global_timeline(
    pool: &SqlitePool,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Status>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS}
         FROM statuses
         WHERE parent_id IS NULL
         ORDER BY created_at DESC, id DESC
         LIMIT ? OFFSET ?"
    ))
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_global_timeline(pool: &SqlitePool) -> sqlx::Result<i64> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM statuses WHERE parent_id IS NULL")
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn list_user_timeline(
    pool: &SqlitePool,
    user_id: i64,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Status>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS}
         FROM statuses
         WHERE user_id = ? AND parent_id IS NULL
         ORDER BY created_at DESC, id DESC
         LIMIT ? OFFSET ?"
    ))
    .bind(user_id)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_user_timeline(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM statuses WHERE user_id = ? AND parent_id IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn list_home_timeline(
    pool: &SqlitePool,
    viewer_id: i64,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Status>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS}
         FROM statuses
         WHERE parent_id IS NULL
           AND (
               user_id = ?
               OR user_id IN (SELECT followee_id FROM follows WHERE follower_id = ?)
           )
         ORDER BY created_at DESC, id DESC
         LIMIT ? OFFSET ?"
    ))
    .bind(viewer_id)
    .bind(viewer_id)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_home_timeline(pool: &SqlitePool, viewer_id: i64) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM statuses
         WHERE parent_id IS NULL
           AND (
               user_id = ?
               OR user_id IN (SELECT followee_id FROM follows WHERE follower_id = ?)
           )",
    )
    .bind(viewer_id)
    .bind(viewer_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn list_replies(pool: &SqlitePool, parent_id: i64) -> sqlx::Result<Vec<Status>> {
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS}
         FROM statuses
         WHERE parent_id = ?
         ORDER BY created_at ASC, id ASC"
    ))
    .bind(parent_id)
    .fetch_all(pool)
    .await
}

pub async fn search_by_hashtag(
    pool: &SqlitePool,
    tag: &str,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Status>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    let pattern = format!("%#{tag}%");
    sqlx::query_as::<_, Status>(&format!(
        "SELECT {STATUS_COLUMNS}
         FROM statuses
         WHERE parent_id IS NULL
           AND content_md LIKE ?
         ORDER BY created_at DESC, id DESC
         LIMIT ? OFFSET ?"
    ))
    .bind(&pattern)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_by_hashtag(pool: &SqlitePool, tag: &str) -> sqlx::Result<i64> {
    let pattern = format!("%#{tag}%");
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM statuses
         WHERE parent_id IS NULL AND content_md LIKE ?",
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM statuses WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn user_has_reposted(
    pool: &SqlitePool,
    user_id: i64,
    original_id: i64,
) -> sqlx::Result<bool> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM statuses
         WHERE user_id = ? AND repost_of_id = ?",
    )
    .bind(user_id)
    .bind(original_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn user_repost_targets(
    pool: &SqlitePool,
    user_id: i64,
    original_ids: &[i64],
) -> sqlx::Result<Vec<i64>> {
    if original_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; original_ids.len()].join(",");
    let sql = format!(
        "SELECT DISTINCT repost_of_id FROM statuses
         WHERE user_id = ?
           AND TRIM(content_md) = ''
           AND repost_of_id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, (Option<i64>,)>(&sql);
    q = q.bind(user_id);
    for id in original_ids {
        q = q.bind(*id);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().filter_map(|(id,)| id).collect())
}

pub async fn delete_repost(
    pool: &SqlitePool,
    user_id: i64,
    original_id: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "DELETE FROM statuses
         WHERE user_id = ?
           AND repost_of_id = ?
           AND TRIM(content_md) = ''",
    )
    .bind(user_id)
    .bind(original_id)
    .execute(pool)
    .await?;
    Ok(())
}
