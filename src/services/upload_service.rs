use std::path::{Path, PathBuf};

use multer::Field;
use sqlx::SqlitePool;
use tokio::io::AsyncWriteExt;

use crate::repositories::asset_repo;
use crate::utils::error::AppError;
use crate::utils::token;

pub const ALLOWED_IMAGE_MIME: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
];
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct UploadedAsset {
    pub asset_id: i64,
    pub public_path: String,
    pub disk_path: PathBuf,
    pub mime: String,
    pub bytes: i64,
    pub original: String,
}

/// Stream a multipart image `Field` into `upload_dir`, validating mime and
/// size, then record an `assets` row. Cleans up the temp file on error.
pub async fn store_image_field<'r>(
    field: &mut Field<'r>,
    upload_dir: &Path,
    pool: &SqlitePool,
) -> Result<UploadedAsset, AppError> {
    let original = field
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "upload.bin".to_string());
    let mime = field
        .content_type()
        .map(|m| m.essence_str().to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    validate_image_mime(&mime)?;

    tokio::fs::create_dir_all(upload_dir).await?;
    let extension = guess_image_extension(&mime);
    let filename = format!("{}.{}", token::random_token(8), sanitize_extension(extension));
    let stored_path = upload_dir.join(&filename);
    let temp_path = upload_dir.join(format!("{filename}.part"));

    let mut file = tokio::fs::File::create(&temp_path).await?;
    let mut bytes = 0usize;
    loop {
        let chunk = match field.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(AppError::BadRequest(format!("multipart error: {error}")));
            }
        };
        bytes = match bytes.checked_add(chunk.len()) {
            Some(b) if b <= MAX_IMAGE_BYTES => b,
            _ => {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(AppError::BadRequest("File too large.".to_string()));
            }
        };
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

    let public_path = format!("/static/uploads/{filename}");
    match asset_repo::create(pool, &original, &public_path, &mime, bytes as i64).await {
        Ok(asset_id) => Ok(UploadedAsset {
            asset_id,
            public_path,
            disk_path: stored_path,
            mime,
            bytes: bytes as i64,
            original,
        }),
        Err(error) => {
            let _ = tokio::fs::remove_file(&stored_path).await;
            Err(error.into())
        }
    }
}

pub async fn cleanup(asset: &UploadedAsset) {
    let _ = tokio::fs::remove_file(&asset.disk_path).await;
}

pub async fn cleanup_all(assets: &[UploadedAsset]) {
    for a in assets {
        cleanup(a).await;
    }
}

pub fn validate_image_mime(mime: &str) -> Result<(), AppError> {
    if ALLOWED_IMAGE_MIME.contains(&mime) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!("Unsupported mime type: {mime}")))
    }
}

pub fn guess_image_extension(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "bin",
    }
}

pub fn sanitize_extension(ext: &str) -> String {
    ext.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(8)
        .collect::<String>()
        .to_ascii_lowercase()
}
