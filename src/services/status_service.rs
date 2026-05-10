use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::domain::{Status, StatusView};
use crate::repositories::{like_repo, status_asset_repo, status_repo, user_repo};
use crate::utils::error::AppError;
use crate::utils::markdown;

#[derive(Debug, Clone)]
pub struct ComposeInput<'a> {
    pub user_id: i64,
    pub content_md: &'a str,
    pub parent_id: Option<i64>,
    pub repost_of_id: Option<i64>,
    pub asset_ids: &'a [i64],
}

pub async fn create_status(
    pool: &SqlitePool,
    input: &ComposeInput<'_>,
) -> Result<i64, AppError> {
    let body = input.content_md.trim();
    if body.is_empty() && input.repost_of_id.is_none() {
        return Err(AppError::BadRequest("Status body cannot be empty.".into()));
    }
    if body.len() > 8 * 1024 {
        return Err(AppError::BadRequest("Status is too long (max 8 KiB).".into()));
    }

    let content_html = render_status_html(body);

    let id = status_repo::create(
        pool,
        &status_repo::StatusInput {
            user_id: input.user_id,
            content_md: body,
            content_html: &content_html,
            parent_id: input.parent_id,
            repost_of_id: input.repost_of_id,
        },
    )
    .await?;

    if !input.asset_ids.is_empty() {
        status_asset_repo::attach(pool, id, input.asset_ids).await?;
    }
    Ok(id)
}

pub fn render_status_html(content_md: &str) -> String {
    let rendered = markdown::render(content_md);
    auto_link(&rendered)
}

/// Batches author/asset/like/repost lookups for a list of statuses and turns them
/// into render-ready `StatusView`s. For repost rows the `original` is loaded too.
pub async fn assemble_views(
    pool: &SqlitePool,
    statuses: Vec<Status>,
    viewer_id: Option<i64>,
) -> Result<Vec<StatusView>, AppError> {
    if statuses.is_empty() {
        return Ok(Vec::new());
    }

    let mut user_ids: HashSet<i64> = HashSet::new();
    let mut all_status_ids: Vec<i64> = Vec::with_capacity(statuses.len() * 2);
    let mut original_ids: Vec<i64> = Vec::new();

    for status in &statuses {
        user_ids.insert(status.user_id);
        all_status_ids.push(status.id);
        if let Some(original_id) = status.repost_of_id {
            original_ids.push(original_id);
        }
    }

    let originals = if original_ids.is_empty() {
        Vec::new()
    } else {
        status_repo::list_by_ids(pool, &original_ids).await?
    };
    for original in &originals {
        user_ids.insert(original.user_id);
        all_status_ids.push(original.id);
    }

    let user_ids_vec: Vec<i64> = user_ids.into_iter().collect();
    let users = user_repo::list_by_ids(pool, &user_ids_vec).await?;
    let users_by_id: HashMap<i64, _> = users.into_iter().map(|u| (u.id, u)).collect();

    let asset_pairs = status_asset_repo::list_for_status_ids(pool, &all_status_ids).await?;
    let mut assets_by_status: HashMap<i64, Vec<_>> = HashMap::new();
    for (status_id, asset) in asset_pairs {
        assets_by_status.entry(status_id).or_default().push(asset);
    }

    let (liked_ids, reposted_targets) = match viewer_id {
        Some(viewer_id) => {
            let liked = like_repo::liked_status_ids_for(pool, viewer_id, &all_status_ids).await?;
            let reposted =
                status_repo::user_repost_targets(pool, viewer_id, &all_status_ids).await?;
            (
                liked.into_iter().collect::<HashSet<_>>(),
                reposted.into_iter().collect::<HashSet<_>>(),
            )
        }
        None => (HashSet::new(), HashSet::new()),
    };

    let originals_by_id: HashMap<i64, Status> =
        originals.into_iter().map(|s| (s.id, s)).collect();

    let mut views = Vec::with_capacity(statuses.len());
    for status in statuses {
        let original_view = match status.repost_of_id {
            Some(original_id) => build_original_view(
                original_id,
                &originals_by_id,
                &users_by_id,
                &assets_by_status,
                &liked_ids,
                &reposted_targets,
            ),
            None => None,
        };

        let status_id = status.id;
        let Some(author) = users_by_id.get(&status.user_id).cloned() else {
            continue;
        };
        let assets = assets_by_status.get(&status_id).cloned().unwrap_or_default();
        views.push(StatusView {
            author,
            viewer_liked: liked_ids.contains(&status_id),
            viewer_reposted: reposted_targets.contains(&status_id),
            assets,
            status,
            original: original_view,
        });
    }
    Ok(views)
}

pub async fn assemble_view(
    pool: &SqlitePool,
    status: Status,
    viewer_id: Option<i64>,
) -> Result<Option<StatusView>, AppError> {
    let mut views = assemble_views(pool, vec![status], viewer_id).await?;
    Ok(views.pop())
}

fn build_original_view(
    original_id: i64,
    originals_by_id: &HashMap<i64, Status>,
    users_by_id: &HashMap<i64, crate::domain::User>,
    assets_by_status: &HashMap<i64, Vec<crate::domain::Asset>>,
    liked_ids: &HashSet<i64>,
    reposted_targets: &HashSet<i64>,
) -> Option<Box<StatusView>> {
    let original = originals_by_id.get(&original_id)?.clone();
    let author = users_by_id.get(&original.user_id)?.clone();
    let assets = assets_by_status
        .get(&original_id)
        .cloned()
        .unwrap_or_default();
    Some(Box::new(StatusView {
        author,
        viewer_liked: liked_ids.contains(&original_id),
        viewer_reposted: reposted_targets.contains(&original_id),
        assets,
        status: original,
        original: None,
    }))
}

/// Walks rendered HTML and wraps `@user` / `#tag` tokens in `<a>` links,
/// skipping any text inside `<a>`, `<code>`, or `<pre>` regions.
pub fn auto_link(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 32);
    let mut skip_depth: i32 = 0;
    let mut last_char: Option<char> = None;
    let mut idx = 0usize;
    let len = html.len();

    while idx < len {
        let rest = &html[idx..];
        let ch = match rest.chars().next() {
            Some(c) => c,
            None => break,
        };

        if ch == '<' {
            let tag_end_rel = rest.find('>').unwrap_or(rest.len() - 1);
            let tag = &rest[..=tag_end_rel];
            let name = parse_tag_name(tag);
            let is_close = tag.starts_with("</");
            let is_self_close = tag.ends_with("/>");
            if matches!(name.as_str(), "a" | "code" | "pre") {
                if is_close {
                    skip_depth = (skip_depth - 1).max(0);
                } else if !is_self_close {
                    skip_depth += 1;
                }
            }
            out.push_str(tag);
            last_char = tag.chars().last();
            idx += tag.len();
            continue;
        }

        if skip_depth > 0 {
            out.push(ch);
            last_char = Some(ch);
            idx += ch.len_utf8();
            continue;
        }

        if (ch == '@' || ch == '#')
            && !matches!(last_char, Some(c) if c.is_alphanumeric() || c == '_')
        {
            let id_start = idx + ch.len_utf8();
            let mut id_end = id_start;
            for c in html[id_start..].chars() {
                if c.is_alphanumeric() || c == '_' {
                    id_end += c.len_utf8();
                } else {
                    break;
                }
            }
            if id_end > id_start {
                let id = &html[id_start..id_end];
                let (path, class, prefix) = if ch == '@' {
                    ("/u/", "mention", '@')
                } else {
                    ("/h/", "hashtag", '#')
                };
                out.push_str("<a class=\"");
                out.push_str(class);
                out.push_str("\" href=\"");
                out.push_str(path);
                out.push_str(&html_escape(id));
                out.push_str("\">");
                out.push(prefix);
                out.push_str(&html_escape(id));
                out.push_str("</a>");
                last_char = Some(';');
                idx = id_end;
                continue;
            }
        }

        out.push(ch);
        last_char = Some(ch);
        idx += ch.len_utf8();
    }

    out
}

fn parse_tag_name(tag: &str) -> String {
    let inner = tag
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim_end_matches('/');
    let inner = inner.strip_prefix('/').unwrap_or(inner);
    inner
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_link_mention_and_hashtag() {
        let out = auto_link("<p>hi @bob, see #rust!</p>");
        assert!(out.contains("<a class=\"mention\" href=\"/u/bob\">@bob</a>"));
        assert!(out.contains("<a class=\"hashtag\" href=\"/h/rust\">#rust</a>"));
    }

    #[test]
    fn auto_link_skips_inside_code() {
        let out = auto_link("<p>before <code>@nope #nope</code> after @yes</p>");
        assert!(out.contains("<code>@nope #nope</code>"));
        assert!(out.contains("<a class=\"mention\" href=\"/u/yes\">@yes</a>"));
    }

    #[test]
    fn auto_link_email_not_linked() {
        let out = auto_link("<p>foo@bar</p>");
        assert!(!out.contains("class=\"mention\""));
    }
}
