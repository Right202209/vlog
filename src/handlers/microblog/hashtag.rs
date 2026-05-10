use volo_http::server::extract::Query;
use volo_http::server::param::PathParams;

use crate::app_state;
use crate::handlers::microblog::{total_pages, viewer_from, Pagination};
use crate::repositories::status_repo;
use crate::services::{auth_guard, status_service};
use crate::templates::{HashtagTemplate, HtmlTemplate};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;

const PER_PAGE: u32 = 20;

pub async fn show(
    RequestHeaders(headers): RequestHeaders,
    PathParams(tag): PathParams<String>,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<HashtagTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let viewer_id = auth.as_ref().map(|a| a.user.id);

    let normalized = normalize_tag(&tag);
    if normalized.is_empty() {
        return Err(AppError::NotFound);
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let statuses = status_repo::search_by_hashtag(&state.pool, &normalized, page, PER_PAGE).await?;
    let views = status_service::assemble_views(&state.pool, statuses, viewer_id).await?;
    let total = status_repo::count_by_hashtag(&state.pool, &normalized).await?;

    Ok(HtmlTemplate(HashtagTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        tag: normalized,
        statuses: views,
        total,
        page,
        total_pages: total_pages(total, PER_PAGE),
    }))
}

fn normalize_tag(tag: &str) -> String {
    tag.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}
