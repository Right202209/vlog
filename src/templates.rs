use askama::Template;
use volo_http::http::{header, header::HeaderName, HeaderValue, StatusCode};
use volo_http::response::Response;
use volo_http::server::IntoResponse;

use crate::domain::{ArchiveMonth, Category, Post, PostListItem, SiteSettings, StatusView, Tag, User};

#[derive(Debug, Clone)]
pub struct ViewerContext {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub csrf_token: String,
    pub is_admin: bool,
}

impl ViewerContext {
    pub fn from_user(user: &User, csrf_token: &str) -> Self {
        Self {
            user_id: user.id,
            username: user.username.clone(),
            display_name: user.display_label().to_string(),
            avatar_url: user.avatar_url.clone(),
            csrf_token: csrf_token.to_string(),
            is_admin: user.is_admin(),
        }
    }
}

// ===== Microblog =====

#[derive(Template)]
#[template(path = "timeline.html")]
pub struct TimelineTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub statuses: Vec<StatusView>,
    pub page: u32,
    pub total_pages: u32,
    pub compose_action: String,
    pub composer_placeholder: String,
    pub parent_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub statuses: Vec<StatusView>,
    pub page: u32,
    pub total_pages: u32,
    pub compose_action: String,
    pub composer_placeholder: String,
    pub parent_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "status_detail.html")]
pub struct StatusDetailTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub root: StatusView,
    pub replies: Vec<StatusView>,
    pub compose_action: String,
    pub composer_placeholder: String,
    pub parent_id: Option<i64>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub profile: User,
    pub statuses: Vec<StatusView>,
    pub follower_count: i64,
    pub following_count: i64,
    pub viewer_following: bool,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Template)]
#[template(path = "followers.html")]
pub struct FollowersTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub profile: User,
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "following.html")]
pub struct FollowingTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub profile: User,
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "hashtag.html")]
pub struct HashtagTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub tag: String,
    pub statuses: Vec<StatusView>,
    pub total: i64,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Template)]
#[template(path = "me_edit.html")]
pub struct MeEditTemplate {
    pub site_name: String,
    pub site_description: String,
    pub viewer: Option<ViewerContext>,
    pub csrf_token: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub message: Option<String>,
}

// ===== Blog (moved under templates/blog/) =====

#[derive(Template)]
#[template(path = "blog/index.html")]
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
#[template(path = "blog/post_detail.html")]
pub struct PostDetailTemplate {
    pub site_name: String,
    pub site_description: String,
    pub site_url: String,
    pub post: Post,
    pub category: Option<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "blog/category.html")]
pub struct CategoryTemplate {
    pub site_name: String,
    pub site_description: String,
    pub category: Category,
    pub posts: Vec<PostListItem>,
    pub page: u32,
    pub total_pages: u32,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "blog/tag.html")]
pub struct TagTemplate {
    pub site_name: String,
    pub site_description: String,
    pub tag: Tag,
    pub posts: Vec<PostListItem>,
    pub page: u32,
    pub total_pages: u32,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "blog/archive.html")]
pub struct ArchiveTemplate {
    pub site_name: String,
    pub site_description: String,
    pub months: Vec<ArchiveMonth>,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "blog/search.html")]
pub struct SearchTemplate {
    pub site_name: String,
    pub site_description: String,
    pub query: String,
    pub posts: Vec<PostListItem>,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "blog/_search_suggest.html")]
pub struct SearchSuggestTemplate {
    pub query: String,
    pub posts: Vec<PostListItem>,
}

#[derive(Template)]
#[template(path = "blog/about.html")]
pub struct AboutTemplate {
    pub site_name: String,
    pub site_description: String,
}

#[derive(Template)]
#[template(path = "blog/404.html")]
pub struct NotFoundTemplate {
    pub site_name: String,
    pub site_description: String,
}

// ===== Feeds =====

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub url: String,
    pub pub_date_rfc2822: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct SitemapItem {
    pub url: String,
    pub lastmod: String,
}

#[derive(Template)]
#[template(path = "rss.xml")]
pub struct RssTemplate {
    pub site_name: String,
    pub site_description: String,
    pub site_url: String,
    pub last_build_rfc2822: String,
    pub items: Vec<FeedItem>,
}

#[derive(Template)]
#[template(path = "sitemap.xml")]
pub struct SitemapTemplate {
    pub site_url: String,
    pub items: Vec<SitemapItem>,
}

// ===== Admin =====

#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct AdminLoginTemplate {
    pub site_name: String,
    pub site_description: String,
    pub error: Option<String>,
    pub username: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub total_posts: usize,
    pub published: usize,
    pub drafts: usize,
    pub archived: usize,
    pub category_count: usize,
    pub tag_count: usize,
}

#[derive(Template)]
#[template(path = "admin/posts.html")]
pub struct AdminPostsTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "admin/post_edit.html")]
pub struct AdminPostEditTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub is_edit: bool,
    pub post_id: Option<i64>,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub content_md: String,
    pub cover_image: String,
    pub status: String,
    pub category_id: i64,
    pub tags_csv: String,
    pub categories: Vec<Category>,
    pub all_tags: Vec<Tag>,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/categories.html")]
pub struct AdminCategoriesTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub categories: Vec<Category>,
}

#[derive(Template)]
#[template(path = "admin/tags.html")]
pub struct AdminTagsTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub tags: Vec<Tag>,
}

#[derive(Template)]
#[template(path = "admin/settings.html")]
pub struct AdminSettingsTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub settings: SiteSettings,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/users.html")]
pub struct AdminUsersTemplate {
    pub site_name: String,
    pub site_description: String,
    pub username: String,
    pub csrf_token: String,
    pub users: Vec<User>,
    pub message: Option<String>,
    pub current_user_id: i64,
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
                response.headers_mut().insert(
                    HeaderName::from_static("x-content-type-options"),
                    HeaderValue::from_static("nosniff"),
                );
                response.headers_mut().insert(
                    HeaderName::from_static("referrer-policy"),
                    HeaderValue::from_static("same-origin"),
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

pub struct XmlTemplate<T: Template>(pub T);

impl<T: Template> IntoResponse for XmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(body) => {
                let mut response = body.into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/xml; charset=utf-8"),
                );
                response.headers_mut().insert(
                    HeaderName::from_static("x-content-type-options"),
                    HeaderValue::from_static("nosniff"),
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
