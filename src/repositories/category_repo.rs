use sqlx::SqlitePool;

use crate::domain::Category;

pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name, slug, description
        FROM categories
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name, slug, description
        FROM categories
        WHERE slug = ?
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_post_id(pool: &SqlitePool, post_id: i64) -> sqlx::Result<Option<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT c.id, c.name, c.slug, c.description
        FROM categories c
        JOIN posts p ON p.category_id = c.id
        WHERE p.id = ?
        "#,
    )
    .bind(post_id)
    .fetch_optional(pool)
    .await
}

