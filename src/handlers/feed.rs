use volo_http::response::Response;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::repositories::post_repo;
use crate::templates::{FeedItem, RssTemplate, SitemapItem, SitemapTemplate, XmlTemplate};
use crate::utils::datetime;
use crate::utils::error::AppError;

const RSS_LIMIT: i64 = 20;

pub async fn rss() -> Result<XmlTemplate<RssTemplate>, AppError> {
    let state = app_state()?;
    let posts = post_repo::list_published_recent(&state.pool, RSS_LIMIT).await?;
    let items = posts
        .into_iter()
        .map(|post| {
            let pub_date_rfc2822 = match post.published_at.as_deref() {
                Some(value) => datetime::rfc2822(value),
                None => datetime::rfc2822(&post.updated_at),
            };
            let url = format!("{}/blog/posts/{}", state.settings.site_url, post.slug);
            FeedItem {
                title: post.title,
                url,
                pub_date_rfc2822,
                summary: post.summary.unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    Ok(XmlTemplate(RssTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        site_url: state.settings.site_url.clone(),
        last_build_rfc2822: datetime::now_rfc2822(),
        items,
    }))
}

pub async fn sitemap() -> Result<XmlTemplate<SitemapTemplate>, AppError> {
    let state = app_state()?;
    let posts = post_repo::list_published_for_sitemap(&state.pool).await?;
    let items = posts
        .into_iter()
        .map(|post| {
            let lastmod = match post.published_at.as_deref() {
                Some(value) => datetime::iso_date(value),
                None => datetime::iso_date(&post.updated_at),
            };
            SitemapItem {
                url: format!("{}/blog/posts/{}", state.settings.site_url, post.slug),
                lastmod,
            }
        })
        .collect::<Vec<_>>();

    Ok(XmlTemplate(SitemapTemplate {
        site_url: state.settings.site_url.clone(),
        items,
    }))
}

pub async fn robots() -> Result<Response, AppError> {
    let state = app_state()?;
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /admin\nSitemap: {}/sitemap.xml\n",
        state.settings.site_url,
    );
    let mut response = body.into_response();
    response.headers_mut().insert(
        volo_http::http::header::CONTENT_TYPE,
        volo_http::http::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    Ok(response)
}
