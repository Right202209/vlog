pub mod config;
pub mod domain;
pub mod handlers;
pub mod middleware;
pub mod repositories;
pub mod services;
pub mod templates;
pub mod utils;

use std::sync::Arc;

use once_cell::sync::OnceCell;
use sqlx::SqlitePool;
use volo_http::server::route::{get, Router};
use volo_http::server::utils::ServeDir;

use crate::config::Settings;
use crate::utils::error::AppError;

static APP_STATE: OnceCell<Arc<AppState>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Settings,
    pub pool: SqlitePool,
}

pub fn app_state() -> Result<Arc<AppState>, AppError> {
    APP_STATE
        .get()
        .cloned()
        .ok_or(AppError::StateNotInitialized)
}

pub fn build_router(state: AppState) -> Router {
    let static_dir = state.settings.static_dir.clone();
    let _ = APP_STATE.get_or_init(|| Arc::new(state));

    Router::new()
        .route("/", get(handlers::post::index))
        .route("/posts", get(handlers::post::index))
        .route("/posts/{slug}", get(handlers::post::post_detail))
        .route("/categories/{slug}", get(handlers::post::category_page))
        .route("/tags/{slug}", get(handlers::post::tag_page))
        .route("/archive", get(handlers::post::archive))
        .route("/search", get(handlers::search::search))
        .route("/about", get(handlers::about::about))
        .nest_service("/static/", ServeDir::new(static_dir))
        .fallback(handlers::not_found::not_found)
}

