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
use volo_http::server::route::{get, post, Router};
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
    let upload_dir = state.settings.upload_dir.clone();
    let _ = APP_STATE.get_or_init(|| Arc::new(state));

    Router::new()
        // Public read-only routes
        .route("/", get(handlers::post::index))
        .route("/posts", get(handlers::post::index))
        .route("/posts/{slug}", get(handlers::post::post_detail))
        .route("/categories/{slug}", get(handlers::post::category_page))
        .route("/tags/{slug}", get(handlers::post::tag_page))
        .route("/archive", get(handlers::post::archive))
        .route("/search", get(handlers::search::search))
        .route("/about", get(handlers::about::about))
        // Feeds & SEO
        .route("/rss.xml", get(handlers::feed::rss))
        .route("/sitemap.xml", get(handlers::feed::sitemap))
        .route("/robots.txt", get(handlers::feed::robots))
        // Admin auth
        .route(
            "/admin/login",
            get(handlers::admin::auth::login_form).post(handlers::admin::auth::login_submit),
        )
        .route("/admin/logout", post(handlers::admin::auth::logout))
        // Admin dashboard
        .route("/admin", get(handlers::admin::dashboard::dashboard))
        // Admin posts
        .route(
            "/admin/posts",
            get(handlers::admin::posts::list).post(handlers::admin::posts::create),
        )
        .route("/admin/posts/new", get(handlers::admin::posts::new_form))
        .route(
            "/admin/posts/{id}/edit",
            get(handlers::admin::posts::edit_form),
        )
        .route("/admin/posts/{id}", post(handlers::admin::posts::update))
        .route(
            "/admin/posts/{id}/publish",
            post(handlers::admin::posts::publish),
        )
        .route(
            "/admin/posts/{id}/unpublish",
            post(handlers::admin::posts::unpublish),
        )
        .route(
            "/admin/posts/{id}/delete",
            post(handlers::admin::posts::delete),
        )
        // Admin categories
        .route(
            "/admin/categories",
            get(handlers::admin::categories::list).post(handlers::admin::categories::create),
        )
        .route(
            "/admin/categories/{id}",
            post(handlers::admin::categories::update),
        )
        .route(
            "/admin/categories/{id}/delete",
            post(handlers::admin::categories::delete),
        )
        // Admin tags
        .route(
            "/admin/tags",
            get(handlers::admin::tags::list).post(handlers::admin::tags::create),
        )
        .route("/admin/tags/{id}", post(handlers::admin::tags::update))
        .route(
            "/admin/tags/{id}/delete",
            post(handlers::admin::tags::delete),
        )
        // Admin settings & uploads
        .route(
            "/admin/settings",
            get(handlers::admin::settings::show).post(handlers::admin::settings::save),
        )
        .route("/admin/upload", post(handlers::admin::upload::upload))
        // Static assets and uploads
        .nest_service("/static/uploads/", ServeDir::new(upload_dir))
        .nest_service("/static/", ServeDir::new(static_dir))
        .fallback(handlers::not_found::not_found)
}
