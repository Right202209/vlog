use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::param::PathParams;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::user_repo;
use crate::services::auth_guard;
use crate::templates::{AdminUsersTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::password;
use crate::utils::response::redirect;

#[derive(Debug, Deserialize)]
pub struct CreateForm {
    pub csrf_token: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetForm {
    pub csrf_token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RoleForm {
    pub csrf_token: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn list(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminUsersTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    let users = user_repo::list_all(&state.pool).await?;
    Ok(HtmlTemplate(AdminUsersTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        users,
        message: None,
        current_user_id: auth.user.id,
    }))
}

pub async fn create(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<CreateForm>,
) -> Response {
    match create_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn create_inner(headers: HeaderMap, form: CreateForm) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    let username = form.username.trim();
    validate_username(username)?;
    if form.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters.".to_string(),
        ));
    }
    if user_repo::find_by_username(&state.pool, username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "User '{username}' already exists."
        )));
    }
    let hash = password::hash(&form.password)?;
    let role = if form.role.trim() == "admin" {
        "admin"
    } else {
        "user"
    };
    user_repo::create_with_password(&state.pool, username, &hash, role).await?;
    Ok(redirect("/admin/users"))
}

pub async fn reset_password(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<ResetForm>,
) -> Response {
    match reset_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn reset_inner(
    headers: HeaderMap,
    id: i64,
    form: ResetForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;
    user_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if form.new_password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters.".to_string(),
        ));
    }
    let hash = password::hash(&form.new_password)?;
    user_repo::update_password(&state.pool, id, &hash).await?;
    Ok(redirect("/admin/users"))
}

pub async fn set_role(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<RoleForm>,
) -> Response {
    match set_role_inner(headers, id, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn set_role_inner(
    headers: HeaderMap,
    id: i64,
    form: RoleForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;
    user_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if id == auth.user.id && form.role != "admin" {
        return Err(AppError::BadRequest(
            "You can't demote yourself.".to_string(),
        ));
    }
    let role = if form.role.trim() == "admin" {
        "admin"
    } else {
        "user"
    };
    user_repo::set_role(&state.pool, id, role).await?;
    Ok(redirect("/admin/users"))
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
    headers: HeaderMap,
    id: i64,
    form: CsrfOnly,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;
    if id == auth.user.id {
        return Err(AppError::BadRequest(
            "You can't delete yourself.".to_string(),
        ));
    }
    user_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    user_repo::delete(&state.pool, id).await?;
    Ok(redirect("/admin/users"))
}

fn validate_username(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 40 {
        return Err(AppError::BadRequest(
            "Username must be 1–40 characters.".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest(
            "Username may only contain letters, digits, underscores, and dashes.".to_string(),
        ));
    }
    Ok(())
}
