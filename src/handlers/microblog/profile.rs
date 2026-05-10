use serde::Deserialize;
use volo_http::http::header::HeaderMap;
use volo_http::response::Response;
use volo_http::server::extract::{Form, Query};
use volo_http::server::param::PathParams;
use volo_http::server::IntoResponse;

use crate::app_state;
use crate::handlers::microblog::{total_pages, viewer_from, Pagination};
use crate::repositories::{follow_repo, status_repo, user_repo};
use crate::services::{auth_guard, status_service};
use crate::templates::{
    FollowersTemplate, FollowingTemplate, HtmlTemplate, ProfileTemplate,
};
use crate::utils::error::AppError;
use crate::utils::extract::RequestHeaders;
use crate::utils::response::redirect;

const PER_PAGE: u32 = 20;

#[derive(Debug, Deserialize)]
pub struct CsrfOnly {
    pub csrf_token: String,
}

pub async fn show(
    RequestHeaders(headers): RequestHeaders,
    PathParams(username): PathParams<String>,
    Query(pagination): Query<Pagination>,
) -> Result<HtmlTemplate<ProfileTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let viewer_id = auth.as_ref().map(|a| a.user.id);

    let profile = user_repo::find_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::NotFound)?;

    let page = pagination.page.unwrap_or(1).max(1);
    let statuses =
        status_repo::list_user_timeline(&state.pool, profile.id, page, PER_PAGE).await?;
    let views = status_service::assemble_views(&state.pool, statuses, viewer_id).await?;
    let total = status_repo::count_user_timeline(&state.pool, profile.id).await?;

    let follower_count = follow_repo::count_followers(&state.pool, profile.id).await?;
    let following_count = follow_repo::count_following(&state.pool, profile.id).await?;
    let viewer_following = match viewer_id {
        Some(vid) if vid != profile.id => {
            follow_repo::is_following(&state.pool, vid, profile.id).await?
        }
        _ => false,
    };

    Ok(HtmlTemplate(ProfileTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        profile,
        statuses: views,
        follower_count,
        following_count,
        viewer_following,
        page,
        total_pages: total_pages(total, PER_PAGE),
    }))
}

pub async fn followers(
    RequestHeaders(headers): RequestHeaders,
    PathParams(username): PathParams<String>,
) -> Result<HtmlTemplate<FollowersTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let profile = user_repo::find_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::NotFound)?;
    let users = follow_repo::followers(&state.pool, profile.id).await?;
    Ok(HtmlTemplate(FollowersTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        profile,
        users,
    }))
}

pub async fn following(
    RequestHeaders(headers): RequestHeaders,
    PathParams(username): PathParams<String>,
) -> Result<HtmlTemplate<FollowingTemplate>, AppError> {
    let state = app_state()?;
    let auth = auth_guard::current_user(&state.pool, &headers).await?;
    let profile = user_repo::find_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::NotFound)?;
    let users = follow_repo::following(&state.pool, profile.id).await?;
    Ok(HtmlTemplate(FollowingTemplate {
        site_name: state.settings.site_name.clone(),
        site_description: state.settings.site_description.clone(),
        viewer: viewer_from(auth.as_ref()),
        profile,
        users,
    }))
}

pub async fn follow(
    RequestHeaders(headers): RequestHeaders,
    PathParams(username): PathParams<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match follow_inner(headers, username, &form.csrf_token, true).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

pub async fn unfollow(
    RequestHeaders(headers): RequestHeaders,
    PathParams(username): PathParams<String>,
    Form(form): Form<CsrfOnly>,
) -> Response {
    match follow_inner(headers, username, &form.csrf_token, false).await {
        Ok(resp) => resp,
        Err(error) => error.into_response(),
    }
}

async fn follow_inner(
    headers: HeaderMap,
    username: String,
    csrf: &str,
    follow: bool,
) -> Result<Response, AppError> {
    let state = app_state()?;
    let auth = auth_guard::require_user(&state.pool, &headers).await?;
    auth_guard::verify_csrf(&auth, Some(csrf))?;
    let target = user_repo::find_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::NotFound)?;
    if target.id == auth.user.id {
        return Err(AppError::BadRequest("You can't follow yourself.".into()));
    }
    if follow {
        follow_repo::follow(&state.pool, auth.user.id, target.id).await?;
    } else {
        follow_repo::unfollow(&state.pool, auth.user.id, target.id).await?;
    }
    Ok(redirect(&format!("/u/{username}")))
}
