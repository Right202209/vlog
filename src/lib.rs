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
use volo_http::response::Response;
use volo_http::server::extract::Query;
use volo_http::server::param::PathParams;
use volo_http::server::route::{get, post, Router};
use volo_http::server::utils::ServeDir;

use crate::config::Settings;
use crate::utils::error::AppError;
use crate::utils::response::redirect_permanent;

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
        // Microblog (front door)
        .route("/", get(handlers::microblog::timeline::global))
        .route("/home", get(handlers::microblog::timeline::home))
        .route("/s/{id}", get(handlers::microblog::status::detail))
        .route("/compose", post(handlers::microblog::status::compose))
        .route("/s/{id}/like", post(handlers::microblog::status::like))
        .route("/s/{id}/unlike", post(handlers::microblog::status::unlike))
        .route("/s/{id}/repost", post(handlers::microblog::status::repost))
        .route("/s/{id}/unrepost", post(handlers::microblog::status::unrepost))
        .route("/s/{id}/delete", post(handlers::microblog::status::delete))
        .route("/u/{username}", get(handlers::microblog::profile::show))
        .route(
            "/u/{username}/followers",
            get(handlers::microblog::profile::followers),
        )
        .route(
            "/u/{username}/following",
            get(handlers::microblog::profile::following),
        )
        .route(
            "/u/{username}/follow",
            post(handlers::microblog::profile::follow),
        )
        .route(
            "/u/{username}/unfollow",
            post(handlers::microblog::profile::unfollow),
        )
        .route("/h/{tag}", get(handlers::microblog::hashtag::show))
        .route(
            "/me/edit",
            get(handlers::microblog::me::edit_form).post(handlers::microblog::me::save),
        )
        .route("/me/avatar", post(handlers::microblog::me::upload_avatar))
        .route("/me/password", post(handlers::microblog::me::change_password))
        // Blog (moved under /blog)
        .route("/blog", get(handlers::blog::post::index))
        .route("/blog/posts/{slug}", get(handlers::blog::post::post_detail))
        .route(
            "/blog/categories/{slug}",
            get(handlers::blog::post::category_page),
        )
        .route("/blog/tags/{slug}", get(handlers::blog::post::tag_page))
        .route("/blog/archive", get(handlers::blog::post::archive))
        .route("/blog/search", get(handlers::blog::search::search))
        .route("/blog/search/suggest", get(handlers::blog::search::suggest))
        .route("/blog/about", get(handlers::blog::about::about))
        // Backwards-compat 301 redirects from the M1/M2 blog paths
        .route("/posts", get(legacy_redirect_blog))
        .route("/posts/{slug}", get(legacy_redirect_post))
        .route("/categories/{slug}", get(legacy_redirect_category))
        .route("/tags/{slug}", get(legacy_redirect_tag))
        .route("/archive", get(legacy_redirect_archive))
        .route("/search", get(legacy_redirect_search))
        .route("/about", get(legacy_redirect_about))
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
        // Admin users
        .route(
            "/admin/users",
            get(handlers::admin::users::list).post(handlers::admin::users::create),
        )
        .route(
            "/admin/users/{id}/reset",
            post(handlers::admin::users::reset_password),
        )
        .route(
            "/admin/users/{id}/role",
            post(handlers::admin::users::set_role),
        )
        .route(
            "/admin/users/{id}/delete",
            post(handlers::admin::users::delete),
        )
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

async fn legacy_redirect_blog() -> Response {
    redirect_permanent("/blog")
}

async fn legacy_redirect_post(PathParams(slug): PathParams<String>) -> Response {
    redirect_permanent(&format!("/blog/posts/{slug}"))
}

async fn legacy_redirect_category(PathParams(slug): PathParams<String>) -> Response {
    redirect_permanent(&format!("/blog/categories/{slug}"))
}

async fn legacy_redirect_tag(PathParams(slug): PathParams<String>) -> Response {
    redirect_permanent(&format!("/blog/tags/{slug}"))
}

async fn legacy_redirect_archive() -> Response {
    redirect_permanent("/blog/archive")
}

async fn legacy_redirect_about() -> Response {
    redirect_permanent("/blog/about")
}

async fn legacy_redirect_search(Query(params): Query<LegacySearch>) -> Response {
    match params.q.as_deref() {
        Some(q) if !q.is_empty() => {
            redirect_permanent(&format!("/blog/search?q={}", percent_encode_query(q)))
        }
        _ => redirect_permanent("/blog/search"),
    }
}

#[derive(Debug, serde::Deserialize)]
struct LegacySearch {
    q: Option<String>,
}

fn percent_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        if matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
