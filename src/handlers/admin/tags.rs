use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::param::PathParams;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::tag_repo;
use crate::services::auth_guard;
use crate::templates::{AdminTagsTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;
use crate::utils::slug::slugify;

#[derive(Debug, Deserialize)]
pub struct TagForm {
    pub name: String,
    #[serde(default)]
    pub slug: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn list(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminTagsTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    let tags = tag_repo::list_all(&state.pool).await?;
    Ok(HtmlTemplate(AdminTagsTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        tags,
    }))
}

pub async fn create(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<TagForm>,
) -> Response {
    match create_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn create_inner(headers: HeaderMap, form: TagForm) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    let name = form.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required.".to_string()));
    }
    let slug = if form.slug.trim().is_empty() {
        slugify(name)
    } else {
        slugify(form.slug.trim())
    };
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Slug cannot be empty after normalization.".to_string(),
        ));
    }
    if tag_repo::slug_exists(&state.pool, &slug, None).await? {
        return Err(AppError::Conflict(format!(
            "Tag slug '{slug}' is already taken."
        )));
    }
    tag_repo::create(&state.pool, name, &slug).await?;
    Ok(redirect("/admin/tags"))
}

pub async fn update(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<TagForm>,
) -> Response {
    match update_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn update_inner(headers: HeaderMap, id: i64, form: TagForm) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    tag_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let name = form.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required.".to_string()));
    }
    let slug = if form.slug.trim().is_empty() {
        slugify(name)
    } else {
        slugify(form.slug.trim())
    };
    if slug.is_empty() {
        return Err(AppError::BadRequest(
            "Slug cannot be empty after normalization.".to_string(),
        ));
    }
    if tag_repo::slug_exists(&state.pool, &slug, Some(id)).await? {
        return Err(AppError::Conflict(format!(
            "Tag slug '{slug}' is already taken."
        )));
    }
    tag_repo::update(&state.pool, id, name, &slug).await?;
    Ok(redirect("/admin/tags"))
}

pub async fn delete(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match delete_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn delete_inner(headers: HeaderMap, id: i64, form: CsrfOnly) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;
    tag_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    tag_repo::delete(&state.pool, id).await?;
    Ok(redirect("/admin/tags"))
}
