use crate::app_state;
use crate::repositories::{category_repo, post_repo, tag_repo};
use crate::services::auth_guard;
use crate::templates::{AdminDashboardTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;

pub async fn dashboard(
    RequestHeaders(headers): RequestHeaders,
) -> Result<HtmlTemplate<AdminDashboardTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_admin(&state.pool, &headers).await?;

    let posts = post_repo::list_all(&state.pool).await?;
    let total_posts = posts.len();
    let published = posts.iter().filter(|p| p.status == "published").count();
    let drafts = posts.iter().filter(|p| p.status == "draft").count();
    let archived = posts.iter().filter(|p| p.status == "archived").count();

    let category_count = category_repo::list_all(&state.pool).await?.len();
    let tag_count = tag_repo::list_all(&state.pool).await?.len();

    Ok(HtmlTemplate(AdminDashboardTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        username: auth.user.username,
        csrf_token: auth.session.csrf_token,
        total_posts,
        published,
        drafts,
        archived,
        category_count,
        tag_count,
    }))
}
