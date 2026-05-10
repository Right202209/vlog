use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::param::PathParams;
use volo_http::server::utils::multipart::Multipart;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::handlers::microblog::viewer_from;
use crate::repositories::{like_repo, status_repo};
use crate::services::status_service::ComposeInput;
use crate::services::upload_service::UploadedAsset;
use crate::services::{auth_guard, status_service, upload_service};
use crate::templates::{HtmlTemplate, StatusDetailTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;

const MAX_TEXT_FIELD_BYTES: usize = 16 * 1024;
const MAX_FILES_PER_STATUS: usize = 4;
const COMPOSE_PLACEHOLDER: &str = "Write a reply… 💬";

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn detail(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
) -> Result<HtmlTemplate<StatusDetailTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let viewer_id = auth.as_ref().map(|a| a.user.id);

    let status = status_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let root = status_service::assemble_view(&state.pool, status, viewer_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let replies = status_repo::list_replies(&state.pool, id).await?;
    let replies = status_service::assemble_views(&state.pool, replies, viewer_id).await?;

    Ok(HtmlTemplate(StatusDetailTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        root,
        replies,
        compose_action: "/compose".to_string(),
        composer_placeholder: COMPOSE_PLACEHOLDER.to_string(),
        parent_id: Some(id),
    }))
}

pub async fn compose(
    RequestHeaders(headers): RequestHeaders,
    multipart: Multipart,
) -> Response {
    match compose_inner(headers, multipart).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn compose_inner(headers: HeaderMap, mut multipart: Multipart) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;

    let mut csrf_verified = false;
    let mut content_md = String::new();
    let mut parent_id: Option<i64> = None;
    let mut repost_of_id: Option<i64> = None;
    let mut asset_ids: Vec<i64> = Vec::new();
    let mut uploaded: Vec<UploadedAsset> = Vec::new();

    while let Some(mut field) = match multipart.next_field().await {
        Ok(field) => field,
        Err(error) => {
            upload_service::cleanup_all(&uploaded).await;
            return Err(AppError::BadRequest(format!("multipart error: {error}")));
        }
    } {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "csrf_token" => {
                let token = read_text(&mut field).await?;
                if let Err(error) = auth_guard::verify_csrf(&auth, Some(&token)) {
                    upload_service::cleanup_all(&uploaded).await;
                    return Err(error);
                }
                csrf_verified = true;
            }
            "parent_id" => {
                let value = read_text(&mut field).await?;
                if !value.trim().is_empty() {
                    parent_id = value.trim().parse().ok();
                }
            }
            "repost_of_id" => {
                let value = read_text(&mut field).await?;
                if !value.trim().is_empty() {
                    repost_of_id = value.trim().parse().ok();
                }
            }
            "content_md" => {
                content_md = read_text(&mut field).await?;
            }
            "files" => {
                if !csrf_verified {
                    upload_service::cleanup_all(&uploaded).await;
                    return Err(AppError::Forbidden);
                }
                if field.file_name().map_or(true, |n| n.is_empty()) {
                    continue;
                }
                if uploaded.len() >= MAX_FILES_PER_STATUS {
                    upload_service::cleanup_all(&uploaded).await;
                    return Err(AppError::BadRequest(format!(
                        "Up to {MAX_FILES_PER_STATUS} attachments per status."
                    )));
                }
                let upload = match upload_service::store_image_field(
                    &mut field,
                    &state.settings.upload_dir,
                    &state.pool,
                )
                .await
                {
                    Ok(u) => u,
                    Err(error) => {
                        upload_service::cleanup_all(&uploaded).await;
                        return Err(error);
                    }
                };
                asset_ids.push(upload.asset_id);
                uploaded.push(upload);
            }
            _ => {
                let _ = read_text(&mut field).await;
            }
        }
    }

    if !csrf_verified {
        upload_service::cleanup_all(&uploaded).await;
        return Err(AppError::Forbidden);
    }

    if let Some(parent) = parent_id {
        status_repo::find_by_id(&state.pool, parent)
            .await?
            .ok_or(AppError::NotFound)?;
    }
    if let Some(original) = repost_of_id {
        status_repo::find_by_id(&state.pool, original)
            .await?
            .ok_or(AppError::NotFound)?;
    }

    let create = status_service::create_status(
        &state.pool,
        &ComposeInput {
            user_id: auth.user.id,
            content_md: &content_md,
            parent_id,
            repost_of_id,
            asset_ids: &asset_ids,
        },
    )
    .await;

    let new_id = match create {
        Ok(id) => id,
        Err(error) => {
            upload_service::cleanup_all(&uploaded).await;
            return Err(error);
        }
    };

    let target = match parent_id {
        Some(pid) => format!("/s/{pid}"),
        None => format!("/s/{new_id}"),
    };
    Ok(redirect(&target))
}

pub async fn like(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match like_inner(headers, id, &form.csrf_token, true).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

pub async fn unlike(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match like_inner(headers, id, &form.csrf_token, false).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn like_inner(
    headers: HeaderMap,
    id: i64,
    csrf: &str,
    like: bool,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(csrf))?;
    status_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if like {
        like_repo::like(&state.pool, auth.user.id, id).await?;
    } else {
        like_repo::unlike(&state.pool, auth.user.id, id).await?;
    }
    Ok(redirect_back(&headers, &format!("/s/{id}")))
}

pub async fn repost(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match repost_inner(headers, id, &form.csrf_token).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn repost_inner(
    headers: HeaderMap,
    id: i64,
    csrf: &str,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(csrf))?;
    let original = status_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if original.user_id == auth.user.id {
        return Err(AppError::BadRequest(
            "You can't repost your own status.".to_string(),
        ));
    }
    if status_repo::user_has_reposted(&state.pool, auth.user.id, id).await? {
        return Ok(redirect_back(&headers, &format!("/s/{id}")));
    }
    status_service::create_status(
        &state.pool,
        &ComposeInput {
            user_id: auth.user.id,
            content_md: "",
            parent_id: None,
            repost_of_id: Some(id),
            asset_ids: &[],
        },
    )
    .await?;
    Ok(redirect_back(&headers, &format!("/s/{id}")))
}

pub async fn unrepost(
    RequestHeaders(headers): RequestHeaders,
    PathParams(id): PathParams<i64>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match unrepost_inner(headers, id, &form.csrf_token).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn unrepost_inner(
    headers: HeaderMap,
    id: i64,
    csrf: &str,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(csrf))?;
    status_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    status_repo::delete_repost(&state.pool, auth.user.id, id).await?;
    Ok(redirect_back(&headers, &format!("/s/{id}")))
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

async fn delete_inner(
    headers: HeaderMap,
    id: i64,
    csrf: &str,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(csrf))?;
    let status = status_repo::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    if status.user_id != auth.user.id && !auth.is_admin() {
        return Err(AppError::Forbidden);
    }
    status_repo::delete(&state.pool, id).await?;
    Ok(redirect("/"))
}

async fn read_text<'r>(field: &mut multer::Field<'r>) -> Result<String, AppError> {
    let mut bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        if bytes.len() + chunk.len() > MAX_TEXT_FIELD_BYTES {
            return Err(AppError::BadRequest("Field is too long.".to_string()));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes)
        .map_err(|_| AppError::BadRequest("Invalid UTF-8 in field.".to_string()))
}

fn redirect_back(headers: &HeaderMap, fallback: &str) -> Response {
    let referer = headers
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with('/') || v.contains("://"));
    redirect(referer.unwrap_or(fallback))
}
