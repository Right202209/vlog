pub mod hashtag;
pub mod me;
pub mod profile;
pub mod status;
pub mod timeline;

use serde::Deserialize;

use crate::services::auth_guard::AuthContext;
use crate::templates::ViewerContext;

#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub page: Option<u32>,
}

pub fn viewer_from(auth: Option<&AuthContext>) -> Option<ViewerContext> {
    auth.map(|a| ViewerContext::from_user(&a.user, &a.session.csrf_token))
}

pub fn total_pages(total: i64, per_page: u32) -> u32 {
    let total = total.max(0) as u32;
    let per_page = per_page.max(1);
    ((total + per_page - 1) / per_page).max(1)
}
