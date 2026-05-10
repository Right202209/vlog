use sqlx::SqlitePool;
use volo_http::http::header::HeaderMap;

use crate::domain::{Session, User};
use crate::repositories::{session_repo, user_repo};
use crate::utils::cookie::{parse_cookies, SESSION_COOKIE};
use crate::utils::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: User,
    pub session: Session,
}

impl AuthContext {
    pub fn csrf_token(&self) -> &str {
        &self.session.csrf_token
    }
}

pub async fn require_admin(pool: &SqlitePool, headers: &HeaderMap) -> Result<AuthContext, AppError> {
    let cookies = parse_cookies(headers);
    let session_id = cookies.get(SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
    let session = session_repo::find_active(pool, session_id)
        .await?
        .ok_or(AppError::InvalidSession)?;
    let user = user_repo::find_by_id(pool, session.user_id)
        .await?
        .ok_or(AppError::InvalidSession)?;
    Ok(AuthContext { user, session })
}

pub fn verify_csrf(auth: &AuthContext, submitted: Option<&str>) -> Result<(), AppError> {
    match submitted {
        Some(token) if constant_time_eq(token.as_bytes(), auth.session.csrf_token.as_bytes()) => {
            Ok(())
        }
        _ => Err(AppError::Forbidden),
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right.iter())
        .fold(0u8, |diff, (left, right)| diff | (left ^ right))
        == 0
}
