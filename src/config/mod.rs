use std::path::PathBuf;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub site_name: String,
    pub site_description: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub static_dir: PathBuf,
    pub upload_dir: PathBuf,
    pub posts_per_page: u32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config/default.toml: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse config/default.toml: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid PORT value: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

pub fn load() -> Result<Settings, ConfigError> {
    let raw = std::fs::read_to_string("config/default.toml")?;
    let mut settings: Settings = toml::from_str(&raw)?;

    if let Ok(value) = std::env::var("SITE_NAME") {
        settings.site_name = value;
    }
    if let Ok(value) = std::env::var("SITE_DESCRIPTION") {
        settings.site_description = value;
    }
    if let Ok(value) = std::env::var("HOST") {
        settings.host = value;
    }
    if let Ok(value) = std::env::var("PORT") {
        settings.port = value.parse()?;
    }
    if let Ok(value) = std::env::var("DATABASE_URL") {
        settings.database_url = value;
    }
    if let Ok(value) = std::env::var("POSTS_PER_PAGE") {
        settings.posts_per_page = value.parse()?;
    }

    Ok(settings)
}

