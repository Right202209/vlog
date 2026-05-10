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

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name, slug, description
        FROM categories
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
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

pub async fn create(
    pool: &SqlitePool,
    name: &str,
    slug: &str,
    description: Option<&str>,
) -> sqlx::Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO categories (name, slug, description)
        VALUES (?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    slug: &str,
    description: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE categories
        SET name = ?, slug = ?, description = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn slug_exists(pool: &SqlitePool, slug: &str, exclude_id: Option<i64>) -> sqlx::Result<bool> {
    let (count,): (i64,) = match exclude_id {
        Some(id) => sqlx::query_as("SELECT COUNT(*) FROM categories WHERE slug = ? AND id <> ?")
            .bind(slug)
            .bind(id)
            .fetch_one(pool)
            .await?,
        None => sqlx::query_as("SELECT COUNT(*) FROM categories WHERE slug = ?")
            .bind(slug)
            .fetch_one(pool)
            .await?,
    };
    Ok(count > 0)
}

