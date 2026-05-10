use sqlx::SqlitePool;

use crate::domain::User;
use crate::repositories::user_repo;
use crate::utils::error::AppError;

pub async fn lookup(pool: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    Ok(user_repo::find_by_username(pool, username).await?)
}
