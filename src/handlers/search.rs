use serde::Deserialize;
use volo_http::server::extract::Query;

use crate::app_state;
use crate::repositories::post_repo;
use crate::services::post_service;
use crate::templates::{HtmlTemplate, SearchTemplate};
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
    }))
}

