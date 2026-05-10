use sqlx::SqlitePool;

use crate::domain::User;

const USER_COLUMNS: &str = "id, username, password_hash, created_at, updated_at, \
                            display_name, bio, avatar_url, role";

pub async fn count(pool: &SqlitePool) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_username(pool: &SqlitePool, username: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users WHERE username = ?"
    ))
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users ORDER BY id ASC"
    ))
    .fetch_all(pool)
    .await
}

pub async fn list_by_ids(pool: &SqlitePool, ids: &[i64]) -> sqlx::Result<Vec<User>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT {USER_COLUMNS} FROM users WHERE id IN ({placeholders}) ORDER BY id ASC"
    );
    let mut q = sqlx::query_as::<_, User>(&sql);
    for id in ids {
        q = q.bind(*id);
    }
    q.fetch_all(pool).await
}

pub async fn create_with_password(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
    role: &str,
) -> sqlx::Result<i64> {
    let role = normalize_role(role);
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO users (
            username, password_hash, created_at, updated_at,
            display_name, role
        )
        VALUES (?, ?, datetime('now'), datetime('now'), ?, ?)
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(username)
    .bind(role)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn update_password(
    pool: &SqlitePool,
    id: i64,
    password_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = ?, updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(password_hash)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_profile(
    pool: &SqlitePool,
    id: i64,
    display_name: Option<&str>,
    bio: Option<&str>,
    avatar_url: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET display_name = ?,
            bio = ?,
            avatar_url = COALESCE(?, avatar_url),
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(display_name)
    .bind(bio)
    .bind(avatar_url)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_avatar(pool: &SqlitePool, id: i64, avatar_url: &str) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET avatar_url = ?, updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(avatar_url)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_role(pool: &SqlitePool, id: i64, role: &str) -> sqlx::Result<()> {
    let role = normalize_role(role);
    sqlx::query(
        r#"
        UPDATE users
        SET role = ?, updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(role)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

fn normalize_role(raw: &str) -> &str {
    if raw == "admin" {
        "admin"
    } else {
        "user"
    }
}
