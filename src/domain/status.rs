use sqlx::FromRow;

use super::{Asset, User};

#[derive(Debug, Clone, FromRow)]
pub struct Status {
    pub id: i64,
    pub user_id: i64,
    pub content_md: String,
    pub content_html: String,
    pub parent_id: Option<i64>,
    pub repost_of_id: Option<i64>,
    pub reply_count: i64,
    pub like_count: i64,
    pub repost_count: i64,
    pub created_at: String,
}

impl Status {
    pub fn is_reply(&self) -> bool {
        self.parent_id.is_some()
    }

    pub fn is_repost(&self) -> bool {
        self.repost_of_id.is_some()
    }

    pub fn is_quote_repost(&self) -> bool {
        self.is_repost() && !self.content_md.trim().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct StatusView {
    pub status: Status,
    pub author: User,
    pub assets: Vec<Asset>,
    pub viewer_liked: bool,
    pub viewer_reposted: bool,
    pub original: Option<Box<StatusView>>,
}
