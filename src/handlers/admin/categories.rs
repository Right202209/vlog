use serde::Deserialize;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::param::PathParams;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::category_repo;
use crate::services::auth_guard;
use crate::templates::{AdminCategoriesTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;
use crate::utils::slug::slugify;

#[derive(Debug, Deserialize)]
pub struct CategoryForm {
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub description: String,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn list(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminCategoriesTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    let categories = category_repo::list_all(&state.pool).await?;
    Ok(HtmlTemplate(AdminCategoriesTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        categories,
    }))
}

pub async fn create(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<CategoryForm>,
) -> Response {
    match create_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn create_inner(
    headers: volo_http::http::header::HeaderMap,
    form: CategoryForm,
) -> Result<Response, AppError> {
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
    if category_repo::slug_exists(&state.pool, &slug, None).await? {
        return Err(AppError::Conflict(format!(
            "Category slug '{slug}' is already taken."
        )));
    }
    let description = if form.description.trim().is_empty() {
        None
    } else {
        Some(form.description.trim())
    };
    category_repo::create(&state.pool, name, &slug, description).await?;
    Ok(redirect("/admin/categories"))
}

pub async fn update(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CategoryForm>,
) -> Response {
    match update_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn update_inner(
    headers: volo_http::http::header::HeaderMap,
    id: i64,
    form: CategoryForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    category_repo::find_by_id(&state.pool, id)
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
    if category_repo::slug_exists(&state.pool, &slug, Some(id)).await? {
        return Err(AppError::Conflict(format!(
            "Category slug '{slug}' is already taken."
        )));
    }
    let description = if form.description.trim().is_empty() {
        None
    } else {
        Some(form.description.trim())
    };
    category_repo::update(&state.pool, id, name, &slug, description).await?;
    Ok(redirect("/admin/categories"))
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

async fn delete_inner(
    headers: volo_http::http::header::HeaderMap,
    id: i64,
    form: CsrfOnly,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;
    category_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    category_repo::delete(&state.pool, id).await?;
    Ok(redirect("/admin/categories"))
}
