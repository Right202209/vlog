use serde::Deserialize;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::param::PathParams;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::{category_repo, post_repo, tag_repo};
use crate::services::{admin_guard, admin_post_service};
use crate::templates::{AdminPostEditTemplate, AdminPostsTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;
use crate::utils::slug::slugify;

#[derive(Debug, Deserialize)]
pub struct PostForm {
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub content_md: String,
    #[serde(default)]
    pub cover_image: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub tags: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn list(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminPostsTemplate>, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    let posts = post_repo::list_all(&state.pool).await?;
    Ok(HtmlTemplate(AdminPostsTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        posts,
    }))
}

pub async fn new_form(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminPostEditTemplate>, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    let categories = category_repo::list_all(&state.pool).await?;
    let tags = tag_repo::list_all(&state.pool).await?;
    Ok(HtmlTemplate(AdminPostEditTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        is_edit: false,
        post_id: None,
        title: String::new(),
        slug: String::new(),
        summary: String::new(),
        content_md: String::new(),
        cover_image: String::new(),
        status: "draft".to_string(),
        category_id: 0,
        tags_csv: String::new(),
        categories,
        all_tags: tags,
        error: None,
    }))
}

pub async fn create(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<PostForm>,
) -> Response {
    match create_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn create_inner(
    headers: volo_http::http::header::HeaderMap,
    form: PostForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    admin_guard::verify_csrf(&auth, Some(form.csrf_token.as_str()))?;

    let title = form.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("Title is required.".to_string()));
    }

    let slug = if form.slug.trim().is_empty() {
        slugify(title)
    } else {
        slugify(form.slug.trim())
    };
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Slug cannot be empty after normalization.".to_string(),
        ));
    }
    if post_repo::slug_exists(&state.pool, &slug, None).await? {
        return Err(AppError::Conflict(format!(
            "A post with slug '{slug}' already exists."
        )));
    }

    let status = normalize_status(&form.status);
    let summary_owned =
        admin_post_service::render_summary_or_excerpt(option_str(&form.summary), &form.content_md);
    let content_html = admin_post_service::render_html(&form.content_md);
    let category_id = parse_category_id(&form.category_id);
    validate_category_id(&state.pool, category_id).await?;
    let cover_image = option_str(&form.cover_image);
    let tag_ids = ensure_tag_ids(&state.pool, &form.tags).await?;

    let input = post_repo::PostInput {
        title,
        slug: &slug,
        summary: summary_owned.as_deref(),
        content_md: &form.content_md,
        content_html: &content_html,
        cover_image,
        status: &status,
        category_id,
    };
    post_repo::create_with_tags(&state.pool, &input, &tag_ids).await?;

    Ok(redirect("/admin/posts"))
}

pub async fn edit_form(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
) -> Result<HtmlTemplate<AdminPostEditTemplate>, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    let post = post_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let categories = category_repo::list_all(&state.pool).await?;
    let all_tags = tag_repo::list_all(&state.pool).await?;
    let post_tags = tag_repo::list_for_post(&state.pool, post.id).await?;
    let tags_csv = post_tags
        .iter()
        .map(|t| t.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    Ok(HtmlTemplate(AdminPostEditTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        is_edit: true,
        post_id: Some(post.id),
        title: post.title,
        slug: post.slug,
        summary: post.summary.unwrap_or_default(),
        content_md: post.content_md,
        cover_image: post.cover_image.unwrap_or_default(),
        status: post.status,
        category_id: post.category_id.unwrap_or(0),
        tags_csv,
        categories,
        all_tags,
        error: None,
    }))
}

pub async fn update(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<PostForm>,
) -> Response {
    match update_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn update_inner(
    headers: volo_http::http::header::HeaderMap,
    id: i64,
    form: PostForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    admin_guard::verify_csrf(&auth, Some(form.csrf_token.as_str()))?;

    let _existing = post_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let title = form.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("Title is required.".to_string()));
    }

    let slug = if form.slug.trim().is_empty() {
        slugify(title)
    } else {
        slugify(form.slug.trim())
    };
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Slug cannot be empty after normalization.".to_string(),
        ));
    }
    if post_repo::slug_exists(&state.pool, &slug, Some(id)).await? {
        return Err(AppError::Conflict(format!(
            "A post with slug '{slug}' already exists."
        )));
    }

    let status = normalize_status(&form.status);
    let summary_owned =
        admin_post_service::render_summary_or_excerpt(option_str(&form.summary), &form.content_md);
    let content_html = admin_post_service::render_html(&form.content_md);
    let category_id = parse_category_id(&form.category_id);
    validate_category_id(&state.pool, category_id).await?;
    let cover_image = option_str(&form.cover_image);
    let tag_ids = ensure_tag_ids(&state.pool, &form.tags).await?;

    let input = post_repo::PostInput {
        title,
        slug: &slug,
        summary: summary_owned.as_deref(),
        content_md: &form.content_md,
        content_html: &content_html,
        cover_image,
        status: &status,
        category_id,
    };
    post_repo::update_with_tags(&state.pool, id, &input, &tag_ids).await?;

    Ok(redirect("/admin/posts"))
}

pub async fn publish(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match status_change(headers, id, &form.csrf_token, "published").await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

pub async fn unpublish(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match status_change(headers, id, &form.csrf_token, "draft").await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

pub async fn delete(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match delete_inner(headers, id, &form.csrf_token).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn status_change(
    headers: volo_http::http::header::HeaderMap,
    id: i64,
    csrf: &str,
    status: &str,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    admin_guard::verify_csrf(&auth, Some(csrf))?;
    post_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    post_repo::set_status(&state.pool, id, status).await?;
    Ok(redirect("/admin/posts"))
}

async fn delete_inner(
    headers: volo_http::http::header::HeaderMap,
    id: i64,
    csrf: &str,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    admin_guard::verify_csrf(&auth, Some(csrf))?;
    post_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    post_repo::delete(&state.pool, id).await?;
    Ok(redirect("/admin/posts"))
}

fn normalize_status(raw: &str) -> String {
    match raw {
        "published" | "draft" | "archived" => raw.to_string(),
        _ => "draft".to_string(),
    }
}

fn parse_category_id(raw: &str) -> Option<i64> {
    let value = raw.trim().parse::<i64>().ok()?;
    if value <= 0 {
        None
    } else {
        Some(value)
    }
}

fn option_str(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

async fn validate_category_id(
    pool: &sqlx::SqlitePool,
    category_id: Option<i64>,
) -> Result<(), AppError> {
    if let Some(id) = category_id {
        category_repo::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::BadRequest("Selected category does not exist.".to_string()))?;
    }
    Ok(())
}

async fn ensure_tag_ids(pool: &sqlx::SqlitePool, tags_csv: &str) -> Result<Vec<i64>, AppError> {
    let mut ids = Vec::new();
    for raw in tags_csv.split(',') {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        if slugify(name).is_empty() {
            return Err(AppError::BadRequest(format!(
                "Tag '{name}' does not produce a valid slug."
            )));
        }
        let id = tag_repo::ensure_by_name(pool, name).await?;
        ids.push(id);
    }
    Ok(ids)
}
