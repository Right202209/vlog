use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::Form;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::domain::SiteSettings;
use crate::repositories::settings_repo;
use crate::services::admin_guard;
use crate::templates::{AdminSettingsTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;

#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    #[serde(default)]
    pub site_name: String,
    #[serde(default)]
    pub site_subtitle: String,
    #[serde(default)]
    pub site_description: String,
    #[serde(default)]
    pub footer_copyright: String,
    #[serde(default)]
    pub about_content: String,
    #[serde(default)]
    pub posts_per_page: String,
    #[serde(default)]
    pub seo_title_template: String,
    pub csrf_token: String,
}

pub async fn show(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminSettingsTemplate>, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    let map = settings_repo::load_all(&state.pool).await?;
    let settings = SiteSettings::from_map(&map);
    Ok(HtmlTemplate(AdminSettingsTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        settings,
        message: None,
    }))
}

pub async fn save(
    RequestHeaders(headers): RequestHeaders,
    Form(form): Form<SettingsForm>,
) -> Response {
    match save_inner(headers, form).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn save_inner(headers: HeaderMap, form: SettingsForm) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = admin_guard::require_admin(&state.pool, &headers).await?;
    admin_guard::verify_csrf(&auth, Some(&form.csrf_token))?;

    let posts_per_page: u32 = form.posts_per_page.trim().parse().unwrap_or(10).clamp(1, 100);

    let pairs = [
        ("site_name", form.site_name.trim().to_string()),
        ("site_subtitle", form.site_subtitle.trim().to_string()),
        ("site_description", form.site_description.trim().to_string()),
        ("footer_copyright", form.footer_copyright.trim().to_string()),
        ("about_content", form.about_content.to_string()),
        ("posts_per_page", posts_per_page.to_string()),
        ("seo_title_template", form.seo_title_template.trim().to_string()),
    ];
    for (key, value) in &pairs {
        settings_repo::upsert(&state.pool, key, value).await?;
    }
    Ok(redirect("/admin/settings"))
}
