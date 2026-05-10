use volo_http::server::extract::Query;

use crate::app_state;
use crate::handlers::microblog::{total_pages, viewer_from, Pagination};
use crate::repositories::status_repo;
use crate::services::{auth_guard, status_service};
use crate::templates::{HomeTemplate, HtmlTemplate, TimelineTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;

const COMPOSE_PLACEHOLDER: &str = "Write a note… ✨";
const PER_PAGE: u32 = 20;

pub async fn global(
    RequestHeaders(headers): RequestHeaders,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<TimelineTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let viewer_id = auth.as_ref().map(|a| a.user.id);

    let page = pagination.page.unwrap_or(1).max(1);
    let statuses = status_repo::list_global_timeline(&state.pool, page, PER_PAGE).await?;
    let views = status_service::assemble_views(&state.pool, statuses, viewer_id).await?;
    let total = status_repo::count_global_timeline(&state.pool).await?;

    Ok(HtmlTemplate(TimelineTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        statuses: views,
        page,
        total_pages: total_pages(total, PER_PAGE),
        compose_action: "/compose".to_string(),
        composer_placeholder: COMPOSE_PLACEHOLDER.to_string(),
        parent_id: None,
    }))
}

pub async fn home(
    RequestHeaders(headers): RequestHeaders,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<HomeTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    let viewer_id = auth.user.id;

    let page = pagination.page.unwrap_or(1).max(1);
    let statuses =
        status_repo::list_home_timeline(&state.pool, viewer_id, page, PER_PAGE).await?;
    let views = status_service::assemble_views(&state.pool, statuses, Some(viewer_id)).await?;
    let total = status_repo::count_home_timeline(&state.pool, viewer_id).await?;

    Ok(HtmlTemplate(HomeTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(Some(&auth)),
        statuses: views,
        page,
        total_pages: total_pages(total, PER_PAGE),
        compose_action: "/compose".to_string(),
        composer_placeholder: COMPOSE_PLACEHOLDER.to_string(),
        parent_id: None,
    }))
}
