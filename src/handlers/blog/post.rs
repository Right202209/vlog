use serde::Deserialize;
use volo_http::server::extract::Query;
use volo_http::server::param::PathParams;

use crate::app_state;
use crate::repositories::{category_repo, post_repo, tag_repo};
use crate::services::post_service;
use crate::templates::{
    ArchiveTemplate, CategoryTemplate, HtmlTemplate, IndexTemplate, PostDetailTemplate, TagTemplate,
};
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
}

pub async fn index(Query(pagination): Query<Pagination>) -> Result<HtmlTemplate<IndexTemplate>, AppError> {
    let state = app_state()?;
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = state.settings.posts_per_page.max(1);
    let posts = post_repo::list_published(&state.pool, page, per_page).await?;
    let posts = post_service::enrich_posts(&state.pool, posts).await?;
    let total = post_repo::count_published(&state.pool).await?;

    Ok(HtmlTemplate(IndexTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        posts,
        page,
        total_pages: total_pages(total, per_page),
        categories: category_repo::list_all(&state.pool).await?,
        tags: tag_repo::list_all(&state.pool).await?,
    }))
}

pub async fn post_detail(
    PathParams(slug): PathParams<String>,
) -> Result<HtmlTemplate<PostDetailTemplate>, AppError> {
    let state = app_state()?;
    let post = post_repo::find_by_slug(&state.pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let category = category_repo::find_by_post_id(&state.pool, post.id).await?;
    let tags = tag_repo::list_for_post(&state.pool, post.id).await?;

    Ok(HtmlTemplate(PostDetailTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        site_url: state.settings.site_url.clone(),
        post,
        category,
        tags,
    }))
}

pub async fn category_page(
    PathParams(slug): PathParams<String>,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<CategoryTemplate>, AppError> {
    let state = app_state()?;
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = state.settings.posts_per_page.max(1);
    let category = category_repo::find_by_slug(&state.pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let posts = post_repo::list_by_category_slug(&state.pool, &slug, page, per_page).await?;
    let posts = post_service::enrich_posts(&state.pool, posts).await?;
    let total = post_repo::count_by_category_slug(&state.pool, &slug).await?;

    Ok(HtmlTemplate(CategoryTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        category,
        posts,
        page,
        total_pages: total_pages(total, per_page),
    }))
}

pub async fn tag_page(
    PathParams(slug): PathParams<String>,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<TagTemplate>, AppError> {
    let state = app_state()?;
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = state.settings.posts_per_page.max(1);
    let tag = tag_repo::find_by_slug(&state.pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let posts = post_repo::list_by_tag_slug(&state.pool, &slug, page, per_page).await?;
    let posts = post_service::enrich_posts(&state.pool, posts).await?;
    let total = post_repo::count_by_tag_slug(&state.pool, &slug).await?;

    Ok(HtmlTemplate(TagTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        tag,
        posts,
        page,
        total_pages: total_pages(total, per_page),
    }))
}

pub async fn archive() -> Result<HtmlTemplate<ArchiveTemplate>, AppError> {
    let state = app_state()?;
    Ok(HtmlTemplate(ArchiveTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        months: post_repo::archive_grouped_by_year_month(&state.pool).await?,
    }))
}

fn total_pages(total: i64, per_page: u32) -> u32 {
    let total = total.max(0) as u32;
    ((total + per_page - 1) / per_page).max(1)
}

