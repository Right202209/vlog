use sqlx::SqlitePool;

use crate::domain::Tag;

pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<Tag>> {
    sqlx::query_as::<_, Tag>(
        r#"
        SELECT id, name, slug
        FROM tags
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Tag>> {
    sqlx::query_as::<_, Tag>(
        r#"
        SELECT id, name, slug
        FROM tags
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Tag>> {
    sqlx::query_as::<_, Tag>(
        r#"
        SELECT id, name, slug
        FROM tags
        WHERE slug = ?
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_for_post(pool: &SqlitePool, post_id: i64) -> sqlx::Result<Vec<Tag>> {
    sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name, t.slug
        FROM tags t
        JOIN post_tags pt ON pt.tag_id = t.id
        WHERE pt.post_id = ?
        ORDER BY t.name ASC
        "#,
    )
    .bind(post_id)
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &SqlitePool, name: &str, slug: &str) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO tags (name, slug)
        VALUES (?, ?)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn update(pool: &SqlitePool, id: i64, name: &str, slug: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE tags SET name = ?, slug = ? WHERE id = ?")
        .bind(name)
        .bind(slug)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn ensure_by_name(pool: &SqlitePool, name: &str) -> sqlx::Result<i64> {
    let trimmed = name.trim();
    let slug = crate::utils::slug::slugify(trimmed);
    if let Some(existing) = find_by_slug(pool, &slug).await? {
        return Ok(existing.id);
    }
    create(pool, trimmed, &slug).await
}

pub async fn slug_exists(pool: &SqlitePool, slug: &str, exclude_id: Option<i64>) -> sqlx::Result<bool> {
    let (count,): (i64,) = match exclude_id {
        Some(id) => sqlx::query_as("SELECT COUNT(*) FROM tags WHERE slug = ? AND id <> ?")
            .bind(slug)
            .bind(id)
            .fetch_one(pool)
            .await?,
        None => sqlx::query_as("SELECT COUNT(*) FROM tags WHERE slug = ?")
            .bind(slug)
            .fetch_one(pool)
            .await?,
    };
    Ok(count > 0)
}

