use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::utils::multipart::Multipart;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::handlers::microblog::viewer_from;
use crate::repositories::user_repo;
use crate::services::{auth_guard, upload_service};
use crate::templates::{HtmlTemplate, MeEditTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::password;
use crate::utils::response::redirect;

const MAX_TEXT_FIELD_BYTES: usize = 4 * 1024;

#[derive(Debug, Deserialize)]
pub struct ProfileForm {
    pub csrf_token: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub bio: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordForm {
    pub csrf_token: String,
    pub new_password: String,
}

pub async fn edit_form(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<MeEditTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    Ok(HtmlTemplate(MeEditTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(Some(&auth)),
        csrf_token: auth.session.csrf_token.clone(),
        display_name: auth.user.display_name.clone().unwrap_or_default(),
        bio: auth.user.bio.clone().unwrap_or_default(),
        avatar_url: auth.user.avatar_url.clone(),
        message: None,
    }))
}

pub async fn save(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<ProfileForm>,
) -> Response {
    match save_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn save_inner(headers: HeaderMap, form: ProfileForm) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    let display = form.display_name.trim();
    let bio = form.bio.trim();
    user_repo::update_profile(
        &state.pool,
        auth.user.id,
        if display.is_empty() { None } else { Some(display) },
        if bio.is_empty() { None } else { Some(bio) },
        None,
    )
    .await?;
    Ok(redirect("/me/edit"))
}

pub async fn change_password(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<PasswordForm>,
) -> Response {
    match change_password_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn change_password_inner(
    headers: HeaderMap,
    form: PasswordForm,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    if form.new_password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters.".to_string(),
        ));
    }
    let hash = password::hash(&form.new_password)?;
    user_repo::update_password(&state.pool, auth.user.id, &hash).await?;
    Ok(redirect("/me/edit"))
}

pub async fn upload_avatar(
    RequestHeaders(headers): RequestHeaders,
    multipart: Multipart,
) -> Response {
    match upload_avatar_inner(headers, multipart).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn upload_avatar_inner(
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;

    let mut csrf_verified = false;
    let mut new_avatar: Option<String> = None;
    let mut uploaded: Vec<upload_service::UploadedAsset> = Vec::new();

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
                let mut bytes = Vec::new();
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
                {
                    if bytes.len() + chunk.len() > MAX_TEXT_FIELD_BYTES {
                        return Err(AppError::BadRequest("CSRF token too large.".to_string()));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                let token = String::from_utf8(bytes)
                    .map_err(|_| AppError::BadRequest("Invalid CSRF token.".to_string()))?;
                if let Err(error) = auth_guard::verify_csrf(&auth, Some(&token)) {
                    upload_service::cleanup_all(&uploaded).await;
                    return Err(error);
                }
                csrf_verified = true;
            }
            "file" => {
                if !csrf_verified {
                    upload_service::cleanup_all(&uploaded).await;
                    return Err(AppError::Forbidden);
                }
                if field.file_name().map_or(true, |n| n.is_empty()) {
                    continue;
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
                new_avatar = Some(upload.public_path.clone());
                uploaded.push(upload);
            }
            _ => continue,
        }
    }

    if !csrf_verified {
        upload_service::cleanup_all(&uploaded).await;
        return Err(AppError::Forbidden);
    }

    if let Some(path) = new_avatar {
        user_repo::set_avatar(&state.pool, auth.user.id, &path).await?;
    }
    Ok(redirect("/me/edit"))
}
