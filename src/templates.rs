use askama::Template;
use volo_http::http::{header, HeaderValue, StatusCode};
use volo_http::response::Response;
use volo_http::server::IntoResponse;

use crate::domain::{ArchiveMonth, Category, Post, PostListItem, Tag};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub site_name: String,
    pub site_description: String,
    pub posts: Vec<PostListItem>,
    pub page: u32,
    pub total_pages: u32,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "post_detail.html")]
pub struct PostDetailTemplate {
    pub site_name: String,
    pub site_description: String,
    pub post: Post,
    pub category: Option<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "category.html")]
pub struct CategoryTemplate {
    pub site_name: String,
    pub site_description: String,
    pub category: Category,
    pub posts: Vec<PostListItem>,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Template)]
#[template(path = "tag.html")]
pub struct TagTemplate {
    pub site_name: String,
    pub site_description: String,
    pub tag: Tag,
    pub posts: Vec<PostListItem>,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Template)]
#[template(path = "archive.html")]
pub struct ArchiveTemplate {
    pub site_name: String,
    pub site_description: String,
    pub months: Vec<ArchiveMonth>,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchTemplate {
    pub site_name: String,
    pub site_description: String,
    pub query: String,
    pub posts: Vec<PostListItem>,
}

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub site_name: String,
    pub site_description: String,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub site_name: String,
    pub site_description: String,
}

pub struct HtmlTemplate<T: Template>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(body) => {
                let mut response = body.into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                response
            }
            Err(error) => {
                tracing::error!(%error, "failed to render template");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

