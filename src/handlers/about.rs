use crate::app_state;
use crate::templates::{AboutTemplate, HtmlTemplate};
use crate::utils::error::AppError;

pub async fn about() -> Result<HtmlTemplate<AboutTemplate>, AppError> {
    let state = app_state()?;
    Ok(HtmlTemplate(AboutTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
    }))
}

