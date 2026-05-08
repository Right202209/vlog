use sqlx::{FromRow, SqlitePool};

use crate::domain::{ArchiveMonth, Post};

#[derive(Debug, FromRow)]
struct ArchiveRow {
    year: String,
    month: String,
}

pub async fn list_published(
    pool: &SqlitePool,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Post>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Post>(base_select(
        "WHERE status = 'published'
         ORDER BY published_at DESC, id DESC
         LIMIT ? OFFSET ?",
    ))
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_published(pool: &SqlitePool) -> sqlx::Result<i64> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 'published'")
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn find_by_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Post>> {
    sqlx::query_as::<_, Post>(base_select(
        "WHERE status = 'published' AND slug = ?
         LIMIT 1",
    ))
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_by_category_slug(
    pool: &SqlitePool,
    slug: &str,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Post>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Post>(
        r#"
        SELECT p.id, p.title, p.slug, p.summary, p.content_md, p.content_html,
               p.cover_image, p.status, p.category_id, p.created_at, p.updated_at,
               p.published_at
        FROM posts p
        JOIN categories c ON c.id = p.category_id
        WHERE p.status = 'published' AND c.slug = ?
        ORDER BY p.published_at DESC, p.id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(slug)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_by_category_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM posts p
        JOIN categories c ON c.id = p.category_id
        WHERE p.status = 'published' AND c.slug = ?
        "#,
    )
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn list_by_tag_slug(
    pool: &SqlitePool,
    slug: &str,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Post>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Post>(
        r#"
        SELECT p.id, p.title, p.slug, p.summary, p.content_md, p.content_html,
               p.cover_image, p.status, p.category_id, p.created_at, p.updated_at,
               p.published_at
        FROM posts p
        JOIN post_tags pt ON pt.post_id = p.id
        JOIN tags t ON t.id = pt.tag_id
        WHERE p.status = 'published' AND t.slug = ?
        ORDER BY p.published_at DESC, p.id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(slug)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_by_tag_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM posts p
        JOIN post_tags pt ON pt.post_id = p.id
        JOIN tags t ON t.id = pt.tag_id
        WHERE p.status = 'published' AND t.slug = ?
        "#,
    )
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn archive_grouped_by_year_month(pool: &SqlitePool) -> sqlx::Result<Vec<ArchiveMonth>> {
    let months = sqlx::query_as::<_, ArchiveRow>(
        r#"
        SELECT strftime('%Y', published_at) AS year,
               strftime('%m', published_at) AS month
        FROM posts
        WHERE status = 'published' AND published_at IS NOT NULL
        GROUP BY year, month
        ORDER BY year DESC, month DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut archive = Vec::with_capacity(months.len());
    for row in months {
        let posts = sqlx::query_as::<_, Post>(
            r#"
            SELECT id, title, slug, summary, content_md, content_html, cover_image,
                   status, category_id, created_at, updated_at, published_at
            FROM posts
            WHERE status = 'published'
              AND strftime('%Y', published_at) = ?
              AND strftime('%m', published_at) = ?
            ORDER BY published_at DESC, id DESC
            "#,
        )
        .bind(&row.year)
        .bind(&row.month)
        .fetch_all(pool)
        .await?;

        archive.push(ArchiveMonth {
            year: row.year,
            month: row.month,
            posts,
        });
    }

    Ok(archive)
}

pub async fn search(pool: &SqlitePool, q: &str) -> sqlx::Result<Vec<Post>> {
    let term = format!("%{}%", q.trim());
    sqlx::query_as::<_, Post>(
        r#"
        SELECT DISTINCT p.id, p.title, p.slug, p.summary, p.content_md, p.content_html,
               p.cover_image, p.status, p.category_id, p.created_at, p.updated_at,
               p.published_at
        FROM posts p
        LEFT JOIN post_tags pt ON pt.post_id = p.id
        LEFT JOIN tags t ON t.id = pt.tag_id
        WHERE p.status = 'published'
          AND (
              p.title LIKE ?
              OR COALESCE(p.summary, '') LIKE ?
              OR COALESCE(t.name, '') LIKE ?
          )
        ORDER BY p.published_at DESC, p.id DESC
        LIMIT 50
        "#,
    )
    .bind(&term)
    .bind(&term)
    .bind(&term)
    .fetch_all(pool)
    .await
}

fn base_select(tail: &str) -> String {
    format!(
        r#"
        SELECT id, title, slug, summary, content_md, content_html, cover_image,
               status, category_id, created_at, updated_at, published_at
        FROM posts
        {tail}
        "#
    )
}

