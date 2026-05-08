use sqlx::SqlitePool;

use crate::domain::{Post, PostListItem};
use crate::repositories::{category_repo, tag_repo};

pub async fn enrich_posts(
    pool: &SqlitePool,
    posts: Vec<Post>,
) -> sqlx::Result<Vec<PostListItem>> {
    let mut items = Vec::with_capacity(posts.len());
    for post in posts {
        let category = category_repo::find_by_post_id(pool, post.id).await?;
        let tags = tag_repo::list_for_post(pool, post.id).await?;
        items.push(PostListItem {
            post,
            category,
            tags,
        });
    }
    Ok(items)
}

