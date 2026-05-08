use sqlx::FromRow;

use super::{Category, Tag};

#[derive(Debug, Clone, FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub content_md: String,
    pub content_html: String,
    pub cover_image: Option<String>,
    pub status: String,
    pub category_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PostListItem {
    pub post: Post,
    pub category: Option<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct ArchiveMonth {
    pub year: String,
    pub month: String,
    pub posts: Vec<Post>,
}

