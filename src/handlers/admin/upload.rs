use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::utils::multipart::Multipart;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::asset_repo;
use crate::services::admin_guard;
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;
use crate::utils::token;

const ALLOWED_MIME_TYPES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
const MAX_BYTES: usize = 5 * 1024 * 1024;
const MAX_CSRF_BYTES: usize = 1024;

struct StoredUpload {
    original: String,
    public_path: String,
    mime: String,
    bytes: i64,
    path: PathBuf,
}

pub async fn upload(
    RequestHeaders(headers): RequestHeaders,
    multipart: Multipart,
) -> Response {
    match upload_inner(headers, multipart).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn upload_inner(headers: HeaderMap, mut multipart: Multipart) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;

    let mut csrf_verified = false;
    let mut uploads = Vec::new();
    let upload_dir: PathBuf = state.settings.upload_dir.clone();
    tokio::fs::create_dir_all(&upload_dir).await?;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "csrf_token" {
            if csrf_verified {
                continue;
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = field
                .chunk()
                .await
                .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
            {
                if bytes.len() + chunk.len() > MAX_CSRF_BYTES {
                    return Err(AppError::BadRequest("CSRF token is too large.".to_string()));
                }
                bytes.extend_from_slice(&chunk);
            }
            let token = String::from_utf8(bytes)
                .map_err(|_| AppError::BadRequest("Invalid CSRF token.".to_string()))?;
            admin_guard::verify_csrf(&auth, Some(&token))?;
            csrf_verified = true;
            continue;
        }

        if field_name != "file" {
            continue;
        }
        if !csrf_verified {
            return Err(AppError::Forbidden);
        }

        let original = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "upload.bin".to_string());
        let mime = field
            .content_type()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        if !ALLOWED_MIME_TYPES.contains(&mime.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Unsupported mime type: {mime}"
            )));
        }

        let extension = guess_extension(&mime);
        let filename = format!("{}.{}", token::random_token(8), sanitize_ext(extension));
        let stored_path = upload_dir.join(&filename);
        let temp_path = upload_dir.join(format!("{filename}.part"));

        let mut file = tokio::fs::File::create(&temp_path).await?;
        let mut bytes = 0usize;
        while let Some(chunk) = (match field.chunk().await {
            Ok(chunk) => chunk,
            Err(error) => {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(AppError::BadRequest(format!("multipart error: {error}")));
            }
        }) {
            bytes = match bytes.checked_add(chunk.len()) {
                Some(bytes) => bytes,
                None => {
                    drop(file);
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return Err(AppError::BadRequest("File too large.".to_string()));
                }
            };
            if bytes > MAX_BYTES {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(AppError::BadRequest("File too large.".to_string()));
            }
            if let Err(error) = file.write_all(&chunk).await {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(error.into());
            }
        }
        if let Err(error) = file.flush().await {
            drop(file);
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(error.into());
        }
        drop(file);

        if let Err(error) = tokio::fs::rename(&temp_path, &stored_path).await {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(error.into());
        }

        let public_path = format!("/static/uploads/{}", filename);
        uploads.push(StoredUpload {
            original,
            public_path,
            mime,
            bytes: bytes as i64,
            path: stored_path,
        });
    }

    if !csrf_verified {
        cleanup_uploads(&uploads).await;
        return Err(AppError::Forbidden);
    }

    for upload in &uploads {
        if let Err(error) = asset_repo::create(
            &state.pool,
            &upload.original,
            &upload.public_path,
            &upload.mime,
            upload.bytes,
        )
        .await
        {
            cleanup_uploads(&uploads).await;
            return Err(error.into());
        }
    }

    Ok(redirect("/admin/settings"))
}

fn guess_extension(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    }
}

fn sanitize_ext(ext: &str) -> String {
    ext.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase()
}

async fn cleanup_uploads(uploads: &[StoredUpload]) {
    for upload in uploads {
        let _ = tokio::fs::remove_file(&upload.path).await;
    }
}
