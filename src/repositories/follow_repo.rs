use sqlx::SqlitePool;

use crate::domain::User;

const USER_COLUMNS: &str = "u.id, u.username, u.password_hash, u.created_at, u.updated_at, \
                            u.display_name, u.bio, u.avatar_url, u.role";

pub async fn follow(
    pool: &SqlitePool,
    follower_id: i64,
    followee_id: i64,
) -> sqlx::Result<()> {
    if follower_id == followee_id {
        return Ok(());
    }
    sqlx::query(
        r#"
        INSERT INTO follows (follower_id, followee_id, created_at)
        VALUES (?, ?, datetime('now'))
        ON CONFLICT(follower_id, followee_id) DO NOTHING
        "#,
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unfollow(
    pool: &SqlitePool,
    follower_id: i64,
    followee_id: i64,
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM follows WHERE follower_id = ? AND followee_id = ?")
        .bind(follower_id)
        .bind(followee_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn is_following(
    pool: &SqlitePool,
    follower_id: i64,
    followee_id: i64,
) -> sqlx::Result<bool> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM follows WHERE follower_id = ? AND followee_id = ?",
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn followers(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS}
         FROM users u
         JOIN follows f ON f.follower_id = u.id
         WHERE f.followee_id = ?
         ORDER BY f.created_at DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn following(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS}
         FROM users u
         JOIN follows f ON f.followee_id = u.id
         WHERE f.follower_id = ?
         ORDER BY f.created_at DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn count_followers(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM follows WHERE followee_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn count_following(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM follows WHERE follower_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn following_ids(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<i64>> {
    let rows: Vec<(i64,)> =
        sqlx::query_as("SELECT followee_id FROM follows WHERE follower_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
