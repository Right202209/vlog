use thiserror::Error;
use volo_http::http::header::{HeaderName, HeaderValue, LOCATION, SET_COOKIE};
use volo_http::http::StatusCode;
use volo_http::response::Response;
use volo_http::server::IntoResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("state has not been initialized")]
    StateNotInitialized,
    #[error("resource not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid session")]
    InvalidSession,
    #[error("forbidden")]
    Forbidden,
    #[error("too many requests: try again in {retry_after_secs}s")]
    TooManyRequests { retry_after_secs: u64 },
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(sqlx::Error),
    #[error("template error: {0}")]
    Template(#[from] askama::Error),
    #[error("password error: {0}")]
    Password(#[from] crate::utils::password::PasswordError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if error
            .as_database_error()
            .is_some_and(|database_error| database_error.is_unique_violation())
        {
            Self::Conflict("A record with that unique value already exists.".to_string())
        } else {
            Self::Database(error)
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            Self::Unauthorized => redirect("/admin/login"),
            Self::InvalidSession => redirect_with_cookie_clear("/admin/login"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            Self::TooManyRequests { retry_after_secs } => too_many_requests(retry_after_secs),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg).into_response(),
            Self::StateNotInitialized => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Application state is not initialized",
            )
                .into_response(),
            Self::Database(_)
            | Self::Template(_)
            | Self::Password(_)
            | Self::Io(_) => {
                tracing::error!(error = %self, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

fn redirect_with_cookie_clear(target: &'static str) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::SEE_OTHER;
    resp.headers_mut()
        .insert(LOCATION, HeaderValue::from_static(target));
    if let Ok(cookie) = HeaderValue::from_str(&crate::utils::cookie::clear_session_cookie()) {
        resp.headers_mut().insert(SET_COOKIE, cookie);
    }
    resp
}

fn redirect(target: &'static str) -> Response {
    let mut resp = Response::default();
    *resp.status_mut() = StatusCode::SEE_OTHER;
    resp.headers_mut()
        .insert(LOCATION, HeaderValue::from_static(target));
    resp
}

fn too_many_requests(retry_after_secs: u64) -> Response {
    let body = format!("Too many requests. Try again in {retry_after_secs}s.");
    let mut resp = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
    if let Ok(value) = HeaderValue::from_str(&retry_after_secs.to_string()) {
        resp.headers_mut()
            .insert(HeaderName::from_static("retry-after"), value);
    }
    resp
}
