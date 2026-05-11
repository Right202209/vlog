use serde::Deserialize;
use volo_http::server::extract::Query;

use crate::app_state;
use crate::repositories::{category_repo, post_repo, tag_repo};
use crate::services::post_service;
use crate::templates::{HtmlTemplate, SearchSuggestTemplate, SearchTemplate};
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub async fn search(Query(params): Query<SearchParams>) -> Result<HtmlTemplate<SearchTemplate>, AppError> {
    let state = app_state()?;
    let query = params.q.unwrap_or_default();
    let posts = if query.trim().is_empty() {
        Vec::new()
    } else {
        post_repo::search(&state.pool, &query).await?
    };

    Ok(HtmlTemplate(SearchTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        query,
        posts: post_service::enrich_posts(&state.pool, posts).await?,
        categories: category_repo::list_all(&state.pool).await?,
        tags: tag_repo::list_all(&state.pool).await?,
    }))
}

pub async fn suggest(
    Query(params): Query<SearchParams>,
) -> Result<HtmlTemplate<SearchSuggestTemplate>, AppError> {
    let state = app_state()?;
    let query = params.q.unwrap_or_default();
    let trimmed = query.trim();
    let posts = if trimmed.is_empty() {
        Vec::new()
    } else {
        let mut found = post_repo::search(&state.pool, trimmed).await?;
        found.truncate(8);
        post_service::enrich_posts(&state.pool, found).await?
    };

    Ok(HtmlTemplate(SearchSuggestTemplate { query, posts }))
}
