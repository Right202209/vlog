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

