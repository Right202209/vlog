use sqlx::SqlitePool;

use crate::domain::{Session, User};
use crate::repositories::{session_repo, user_repo};
use crate::services::rate_limit;
use crate::utils::error::AppError;
use crate::utils::{password, token};

pub const SESSION_LIFETIME_SECS: i64 = 60 * 60 * 24 * 7; // 7 days

pub async fn login(
    pool: &SqlitePool,
    username: &str,
    password_input: &str,
) -> Result<Session, AppError> {
    let username = username.trim();
    let key = rate_limit_key(username);
    if let Err(remaining) = rate_limit::check(&key) {
        return Err(AppError::TooManyRequests {
            retry_after_secs: remaining.as_secs().max(1),
        });
    }

    let user = match user_repo::find_by_username(pool, username).await? {
        Some(user) => user,
        None => {
            register_failure(&key);
            return Err(AppError::Unauthorized);
        }
    };

    if !password::verify(password_input, &user.password_hash)? {
        register_failure(&key);
        return Err(AppError::Unauthorized);
    }

    rate_limit::record_success(&key);

    let session_id = token::session_id();
    let csrf = token::csrf_token();
    session_repo::create(pool, &session_id, user.id, &csrf, SESSION_LIFETIME_SECS).await?;
    session_repo::find_active(pool, &session_id)
        .await?
        .ok_or(AppError::Unauthorized)
}

fn rate_limit_key(username: &str) -> String {
    format!("login:{}", username.to_ascii_lowercase())
}

fn register_failure(key: &str) {
    if let Some(lockout) = rate_limit::record_failure(key) {
        tracing::warn!(
            "Login lockout triggered for key '{key}' for {} seconds",
            lockout.as_secs()
        );
    }
}

pub async fn logout(pool: &SqlitePool, session_id: &str) -> Result<(), AppError> {
    session_repo::delete(pool, session_id).await?;
    Ok(())
}

pub async fn current_user(
    pool: &SqlitePool,
    session: &Session,
) -> Result<User, AppError> {
    user_repo::find_by_id(pool, session.user_id)
        .await?
        .ok_or(AppError::Unauthorized)
}

pub async fn ensure_default_admin(pool: &SqlitePool) -> Result<(), AppError> {
    if user_repo::count(pool).await? > 0 {
        return Ok(());
    }

    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let plain = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    if plain.trim().is_empty() {
        return Err(AppError::BadRequest(
            "ADMIN_PASSWORD cannot be empty when bootstrapping the admin user.".to_string(),
        ));
    }

    let hash = password::hash(&plain)?;
    let result = sqlx::query(
        r#"
        INSERT INTO users (username, password_hash, created_at, updated_at)
        SELECT ?, ?, datetime('now'), datetime('now')
        WHERE NOT EXISTS (SELECT 1 FROM users)
        "#,
    )
    .bind(&username)
    .bind(&hash)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(
            "Bootstrapped default admin user '{}' (override with ADMIN_USERNAME / ADMIN_PASSWORD)",
            username
        );
    }
    Ok(())
}
