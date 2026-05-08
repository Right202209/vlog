use volo_http::http::StatusCode;

use crate::app_state;
use crate::templates::{HtmlTemplate, NotFoundTemplate};

pub async fn not_found() -> (StatusCode, HtmlTemplate<NotFoundTemplate>) {
    let (site_name, site_description) = match app_state() {
        Ok(state) => (
            state.settings.site_name.clone(),
            state.settings.site_description.clone(),
        ),
        Err(_) => ("Volo Blog".to_string(), "Page not found".to_string()),
    };

    (
        StatusCode::NOT_FOUND,
        HtmlTemplate(NotFoundTemplate {
            site_name,
            site_description,
        }),
    )
}

