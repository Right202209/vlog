use sqlx::{FromRow, SqlitePool};

use crate::domain::{ArchiveMonth, Post};

#[derive(Debug, FromRow)]
struct ArchiveRow {
    year: String,
    month: String,
}

#[derive(Debug, Clone)]
pub struct PostInput<'a> {
    pub title: &'a str,
    pub slug: &'a str,
    pub summary: Option<&'a str>,
    pub content_md: &'a str,
    pub content_html: &'a str,
    pub cover_image: Option<&'a str>,
    pub status: &'a str,
    pub category_id: Option<i64>,
}

pub async fn list_published(
    pool: &SqlitePool,
    page: u32,
    per_page: u32,
) -> sqlx::Result<Vec<Post>> {
    let offset = ((page.max(1) - 1) * per_page) as i64;
    sqlx::query_as::<_, Post>(&base_select(
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

pub async fn list_published_recent(pool: &SqlitePool, limit: i64) -> sqlx::Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(&base_select(
        "WHERE status = 'published'
         ORDER BY published_at DESC, id DESC
         LIMIT ?",
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_published_for_sitemap(pool: &SqlitePool) -> sqlx::Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(&base_select(
        "WHERE status = 'published'
         ORDER BY COALESCE(updated_at, published_at) DESC, id DESC",
    ))
    .fetch_all(pool)
    .await
}

pub async fn find_by_slug(pool: &SqlitePool, slug: &str) -> sqlx::Result<Option<Post>> {
    sqlx::query_as::<_, Post>(&base_select(
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

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Post>> {
    sqlx::query_as::<_, Post>(&base_select("WHERE id = ? LIMIT 1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(&base_select(
        "ORDER BY COALESCE(published_at, updated_at) DESC, id DESC",
    ))
    .fetch_all(pool)
    .await
}

pub async fn slug_exists(pool: &SqlitePool, slug: &str, exclude_id: Option<i64>) -> sqlx::Result<bool> {
    let (count,): (i64,) = match exclude_id {
        Some(id) => sqlx::query_as("SELECT COUNT(*) FROM posts WHERE slug = ? AND id <> ?")
            .bind(slug)
            .bind(id)
            .fetch_one(pool)
            .await?,
        None => sqlx::query_as("SELECT COUNT(*) FROM posts WHERE slug = ?")
            .bind(slug)
            .fetch_one(pool)
            .await?,
    };
    Ok(count > 0)
}

pub async fn create_with_tags(
    pool: &SqlitePool,
    input: &PostInput<'_>,
    tag_ids: &[i64],
) -> sqlx::Result<i64> {
    let mut tx = pool.begin().await?;
    let published_at_sql = if input.status == "published" {
        Some(())
    } else {
        None
    };

    let row: (i64,) = if published_at_sql.is_some() {
        sqlx::query_as(
            r#"
            INSERT INTO posts (
                title, slug, summary, content_md, content_html, cover_image,
                status, category_id, created_at, updated_at, published_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?,
                    datetime('now'), datetime('now'), datetime('now'))
            RETURNING id
            "#,
        )
    } else {
        sqlx::query_as(
            r#"
            INSERT INTO posts (
                title, slug, summary, content_md, content_html, cover_image,
                status, category_id, created_at, updated_at, published_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?,
                    datetime('now'), datetime('now'), NULL)
            RETURNING id
            "#,
        )
    }
    .bind(input.title)
    .bind(input.slug)
    .bind(input.summary)
    .bind(input.content_md)
    .bind(input.content_html)
    .bind(input.cover_image)
    .bind(input.status)
    .bind(input.category_id)
    .fetch_one(&mut *tx)
    .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT OR IGNORE INTO post_tags (post_id, tag_id) VALUES (?, ?)")
            .bind(row.0)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(row.0)
}

pub async fn update_with_tags(
    pool: &SqlitePool,
    id: i64,
    input: &PostInput<'_>,
    tag_ids: &[i64],
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE posts
        SET title = ?,
            slug = ?,
            summary = ?,
            content_md = ?,
            content_html = ?,
            cover_image = ?,
            status = ?,
            category_id = ?,
            updated_at = datetime('now'),
            published_at = CASE
                WHEN ? = 'published' AND published_at IS NULL THEN datetime('now')
                WHEN ? = 'published' THEN published_at
                ELSE published_at
            END
        WHERE id = ?
        "#,
    )
    .bind(input.title)
    .bind(input.slug)
    .bind(input.summary)
    .bind(input.content_md)
    .bind(input.content_html)
    .bind(input.cover_image)
    .bind(input.status)
    .bind(input.category_id)
    .bind(input.status)
    .bind(input.status)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM post_tags WHERE post_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for tag_id in tag_ids {
        sqlx::query("INSERT OR IGNORE INTO post_tags (post_id, tag_id) VALUES (?, ?)")
            .bind(id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE posts
        SET status = ?,
            updated_at = datetime('now'),
            published_at = CASE
                WHEN ? = 'published' AND published_at IS NULL THEN datetime('now')
                WHEN ? = 'published' THEN published_at
                ELSE published_at
            END
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(status)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM posts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
