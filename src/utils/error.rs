use thiserror::Error;
use volo_http::http::StatusCode;
use volo_http::response::ServerResponse;
use volo_http::server::IntoResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("state has not been initialized")]
    StateNotInitialized,
    #[error("resource not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("template error: {0}")]
    Template(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> ServerResponse {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            Self::StateNotInitialized => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Application state is not initialized")
                    .into_response()
            }
            Self::Database(_) | Self::Template(_) => {
                tracing::error!(error = %self, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

